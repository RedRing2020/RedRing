//! `MachiningSequenceStore` のテスト

use cam_core::{MachiningStage, ToleranceProfile};

use super::*;

fn stock() -> MachiningStock {
    MachiningStock {
        margins: StockMargins::default(),
        precision: 0.05,
    }
}

fn content(stepover: f64) -> MachiningOperationContent {
    MachiningOperationContent {
        name: "仕上げ".to_string(),
        stage: MachiningStage::Finish,
        spec: OperationSpec::Scanline(ScanlineParams {
            stepover,
            feed_rate: 1200.0,
            tool_range: cam_core::ScanlineToolRange::default(),
            cut_direction: cam_core::ScanlineCutDirection::default(),
        }),
        tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
        boundary: None,
        tolerance_profile: ToleranceProfile::PressRough,
    }
}

/// 保管先 `store` の加工手順 `sequence` の加工工程 `operation` の改訂の番号
fn revision(
    store: &MachiningSequenceStore,
    sequence: MachiningSequenceId,
    operation: MachiningOperationId,
) -> u32 {
    store.operation(sequence, operation).unwrap().revision()
}

#[test]
fn a_sequence_is_found_and_changed_by_its_id() {
    let mut store = MachiningSequenceStore::new();
    let first = store.create_sequence("手順".to_string(), Vec::new(), stock());
    let second = store.create_sequence("手順".to_string(), Vec::new(), stock());
    let face = FaceId::new_random();

    store.set_target_faces(second, vec![face]).unwrap();

    assert!(store.sequence(first).unwrap().target_faces().is_empty());
    assert_eq!(store.sequence(second).unwrap().target_faces(), [face]);
}

#[test]
fn operations_are_added_and_edited_through_the_store() {
    let mut store = MachiningSequenceStore::new();
    let sequence = store.create_sequence("手順".to_string(), Vec::new(), stock());
    let operation = store.add_operation(sequence, content(2.0)).unwrap();
    assert_eq!(
        store.operation(sequence, operation).unwrap().content(),
        &content(2.0)
    );
    assert_eq!(revision(&store, sequence, operation), 1);

    // 中身を変えると改訂の番号が増え、同じ中身では増えない
    store
        .set_operation_content(sequence, operation, content(3.0))
        .unwrap();
    store
        .set_operation_content(sequence, operation, content(3.0))
        .unwrap();
    assert_eq!(revision(&store, sequence, operation), 2);

    // 手順の素材・加工の対象を変えても、工程の改訂の番号が増える
    store
        .set_stock(
            sequence,
            MachiningStock {
                precision: 0.1,
                ..stock()
            },
        )
        .unwrap();
    store
        .set_target_faces(sequence, vec![FaceId::new_random()])
        .unwrap();
    assert_eq!(revision(&store, sequence, operation), 4);
}

#[test]
fn unknown_sequences_and_operations_are_reported() {
    let mut store = MachiningSequenceStore::new();
    let sequence = store.create_sequence("手順".to_string(), Vec::new(), stock());
    let operation = store.add_operation(sequence, content(2.0)).unwrap();
    let mut other = MachiningSequenceStore::new();
    let unknown_sequence = other.create_sequence("手順".to_string(), Vec::new(), stock());
    let unknown_operation = other.add_operation(unknown_sequence, content(2.0)).unwrap();

    assert_eq!(
        store.add_operation(unknown_sequence, content(2.0)),
        Err(MachiningSequenceError::UnknownSequence(unknown_sequence))
    );
    assert_eq!(
        store.set_operation_content(sequence, unknown_operation, content(3.0)),
        Err(MachiningSequenceError::UnknownOperation(unknown_operation))
    );
    assert_eq!(
        store.set_stock(unknown_sequence, stock()),
        Err(MachiningSequenceError::UnknownSequence(unknown_sequence))
    );
    // 失敗した編集は、保管している工程を変えない
    assert_eq!(revision(&store, sequence, operation), 1);
}

#[test]
fn scanline_operations_are_made_with_the_default_tolerance_profile() {
    let tool = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    let OperationSpec::Scanline(scanline) = content(2.0).spec;
    let made = scanline_finish_operation_content("仕上げ".to_string(), tool, scanline);
    assert_eq!(made, content(2.0));

    let mut store = MachiningSequenceStore::new();
    let sequence = store.create_sequence("手順".to_string(), Vec::new(), stock());
    let operation = store.add_operation(sequence, made).unwrap();
    // 精度プロファイル press_rough のトレランス
    assert_eq!(store.operation_tolerance(sequence, operation), Ok(0.001));
}

#[test]
fn the_held_content_and_stock_are_read_back() {
    let mut store = MachiningSequenceStore::new();
    let sequence = store.create_sequence("手順".to_string(), Vec::new(), stock());
    let operation = store.add_operation(sequence, content(2.0)).unwrap();
    assert_eq!(
        store.operation_content(sequence, operation),
        Ok(&content(2.0))
    );
    assert_eq!(store.stock(sequence), Ok(&stock()));
}

#[test]
fn cusp_heights_on_flat_surfaces_follow_the_tool_and_the_stepover() {
    let with = |tool: Tool<f64>, stepover: f64| {
        flat_surface_cusp_height(&MachiningOperationContent {
            tool,
            ..content(stepover)
        })
    };
    let ball = || Tool::ball_end_mill("BEM10".to_string(), 10.0, 30.0);
    let flat = || Tool::flat_end_mill("EM10".to_string(), 10.0, 30.0);

    // r = 5 mm、s = 2 mm: 5 − √24
    assert!((with(ball(), 2.0).unwrap() - (5.0 - 24.0_f64.sqrt())).abs() < 1e-12);
    // ピックフィードが工具径と同じなら、山の高さは工具の半径
    assert_eq!(with(ball(), 10.0), Some(5.0));
    assert_eq!(with(flat(), 2.0), Some(0.0));
    // ピックフィードが工具径を超える場合と、ラジアスエンドミルは値を持たない
    assert_eq!(with(ball(), 10.5), None);
    assert_eq!(with(flat(), 10.5), None);
    let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 30.0);
    assert_eq!(with(radius, 2.0), None);
}
