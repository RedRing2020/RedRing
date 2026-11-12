use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ContourLevelPath, CuttingDirection, PathSegment, SegmentType,
    ToolPath, write_toolpath_payload_v1,
};
use geo_algorithms::Point3D;

use super::*;
use crate::machining_sequence::{
    MachiningStock, ScanlineParams, StockMargins, Tool, scanline_finish_operation_content,
};

/// 線分 1 本のツールパスの成果物のバイト列（`x_end` で中身を変える）
pub(crate) fn toolpath_artifact(x_end: f64) -> Vec<u8> {
    let toolpath = ToolPath::new(
        "t".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            vec![PathSegment::new_line(
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(x_end, 0.0, 0.0),
                SegmentType::Cutting { feed_rate: 1.0 },
            )],
        )],
        vec![],
    );
    let mut payload = Vec::new();
    write_toolpath_payload_v1(&mut payload, &toolpath).unwrap();
    let mut bytes = Vec::new();
    ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64)
        .write_to(&mut bytes)
        .unwrap();
    bytes.extend_from_slice(&payload);
    bytes
}

/// 加工工程を 1 つ持つ加工手順の保管先と、その手順・工程
fn one_operation() -> (
    MachiningSequenceStore,
    MachiningSequenceId,
    MachiningOperationId,
) {
    let mut sequences = MachiningSequenceStore::new();
    let stock = MachiningStock {
        margins: StockMargins::default(),
        precision: 0.05,
    };
    let sequence = sequences.create_sequence("手順".to_string(), Vec::new(), stock);
    let content = scanline_finish_operation_content(
        "工程".to_string(),
        Tool::ball_end_mill("B".to_string(), 6.0, 30.0),
        ScanlineParams {
            stepover: 1.0,
            feed_rate: 1000.0,
            tool_range: Default::default(),
            cut_direction: Default::default(),
            angle_degrees: 0.0,
        },
    );
    let operation = sequences.add_operation(sequence, content).unwrap();
    (sequences, sequence, operation)
}

fn record_of(
    sequences: &MachiningSequenceStore,
    sequence: MachiningSequenceId,
    operation: MachiningOperationId,
) -> ImportedToolpathRecord {
    ImportedToolpathRecord {
        operation,
        revision: NonZeroU32::new(1).unwrap(),
        revision_key: sequences
            .operation_revision_key(sequence, operation)
            .unwrap(),
        edit_number: 0,
        artifact: toolpath_artifact(1.0),
    }
}

#[test]
fn imported_toolpaths_are_current_until_the_conditions_change() {
    let (mut sequences, sequence, operation) = one_operation();
    let empty = ImportedToolpathStore::new();
    assert_eq!(
        empty.state(&sequences, sequence, operation).unwrap(),
        ImportedToolpathState::None
    );

    let store =
        ImportedToolpathStore::from_records(vec![record_of(&sequences, sequence, operation)])
            .unwrap();
    assert_eq!(
        store.state(&sequences, sequence, operation).unwrap(),
        ImportedToolpathState::Current
    );
    sequences
        .set_stock(
            sequence,
            MachiningStock {
                margins: StockMargins::default(),
                precision: 0.2,
            },
        )
        .unwrap();
    // 前の条件の CL は、除く前でも使わない
    assert_eq!(
        store.state(&sequences, sequence, operation).unwrap(),
        ImportedToolpathState::None
    );
}

#[test]
fn the_cl_of_previous_conditions_is_removed() {
    let (mut sequences, sequence, operation) = one_operation();
    let mut store =
        ImportedToolpathStore::from_records(vec![record_of(&sequences, sequence, operation)])
            .unwrap();
    // 今の条件の CL は除かない
    assert_eq!(
        store.remove_outdated(&sequences, sequence).unwrap(),
        Vec::new()
    );
    assert_eq!(store.records().len(), 1);
    sequences
        .set_stock(
            sequence,
            MachiningStock {
                margins: StockMargins::default(),
                precision: 0.2,
            },
        )
        .unwrap();
    assert_eq!(
        store.remove_outdated(&sequences, sequence).unwrap(),
        vec![operation]
    );
    assert!(store.records().is_empty());
}

#[test]
fn records_that_break_the_rules_are_not_restored() {
    let (sequences, sequence, operation) = one_operation();
    let record = record_of(&sequences, sequence, operation);
    assert_eq!(
        ImportedToolpathStore::from_records(vec![record.clone(), record.clone()]).unwrap_err(),
        ImportedToolpathRestoreError::DuplicateOperation(operation)
    );
    let broken = ImportedToolpathRecord {
        artifact: b"not a toolpath".to_vec(),
        ..record.clone()
    };
    assert_eq!(
        ImportedToolpathStore::from_records(vec![broken]).unwrap_err(),
        ImportedToolpathRestoreError::InvalidArtifact(operation)
    );
    // もとにした工程がない CL も、そのまま戻す
    let orphan = ImportedToolpathRecord {
        operation: MachiningOperationId::from_uuid(uuid::Uuid::from_u128(7)),
        ..record
    };
    assert_eq!(
        ImportedToolpathStore::from_records(vec![orphan.clone()])
            .unwrap()
            .records(),
        &[orphan]
    );
}

#[test]
fn removed_toolpaths_are_no_longer_held() {
    let (sequences, sequence, operation) = one_operation();
    let mut store =
        ImportedToolpathStore::from_records(vec![record_of(&sequences, sequence, operation)])
            .unwrap();
    store.remove(operation);
    assert!(store.records().is_empty());
    assert_eq!(
        store.state(&sequences, sequence, operation).unwrap(),
        ImportedToolpathState::None
    );
}

#[test]
fn the_toolpath_of_an_operation_is_read_from_its_record() {
    let (sequences, sequence, operation) = one_operation();
    let empty = ImportedToolpathStore::new();
    assert!(empty.toolpath(operation).is_none());
    let store =
        ImportedToolpathStore::from_records(vec![record_of(&sequences, sequence, operation)])
            .unwrap();
    assert!(store.toolpath(operation).is_some());
}
