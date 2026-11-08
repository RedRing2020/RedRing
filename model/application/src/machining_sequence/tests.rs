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
            angle_degrees: 0.0,
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

/// 加工工程 2 つ（2 つ目は改訂 2）を持つ加工手順と、もう 1 つの加工手順を持つ保管先
fn store_with_two_sequences() -> MachiningSequenceStore {
    let mut store = MachiningSequenceStore::new();
    let first = store.create_sequence(
        "手順 1".to_string(),
        vec![FaceId::new_random(), FaceId::new_random()],
        stock(),
    );
    store.add_operation(first, content(1.0)).unwrap();
    let second = store.add_operation(first, content(1.0)).unwrap();
    store
        .set_operation_content(first, second, content(2.0))
        .unwrap();
    let other = store.create_sequence("手順 2".to_string(), Vec::new(), stock());
    store.add_operation(other, content(0.5)).unwrap();
    store
}

#[test]
fn records_are_restored_with_the_same_ids_revisions_and_contents() {
    let records = store_with_two_sequences().records();
    assert_eq!(records[0].operations[1].revision.get(), 2);

    let restored = MachiningSequenceStore::from_records(records.clone()).unwrap();
    assert_eq!(restored.records(), records);

    // 戻した工程を変えると、続きの改訂の番号と新しい改訂の鍵になる
    let mut restored = restored;
    let sequence = records[0].id;
    let operation = records[0].operations[1].id;
    restored
        .set_operation_content(sequence, operation, content(3.0))
        .unwrap();
    let changed = restored.operation(sequence, operation).unwrap();
    assert_eq!(changed.revision(), 3);
    assert_ne!(
        changed.revision_key(),
        records[0].operations[1].revision_key
    );
}

#[test]
fn records_that_break_the_rules_are_not_restored() {
    let records = store_with_two_sequences().records();
    let restore = |records: Vec<MachiningSequenceRecord>| {
        MachiningSequenceStore::from_records(records).unwrap_err()
    };

    let mut duplicate_sequence = records.clone();
    duplicate_sequence[1].id = records[0].id;
    assert_eq!(
        restore(duplicate_sequence),
        MachiningSequenceRestoreError::DuplicateSequence(records[0].id)
    );

    // ほかの加工手順の工程とも重ならない
    let mut duplicate_operation = records.clone();
    duplicate_operation[1].operations[0].id = records[0].operations[0].id;
    assert_eq!(
        restore(duplicate_operation),
        MachiningSequenceRestoreError::DuplicateOperation(records[0].operations[0].id)
    );

    let mut duplicate_face = records.clone();
    let face = records[0].target_faces[0];
    duplicate_face[0].target_faces.push(face);
    assert_eq!(
        restore(duplicate_face),
        MachiningSequenceRestoreError::DuplicateTargetFace {
            sequence: records[0].id,
            face,
        }
    );
}

#[test]
fn operations_are_inserted_duplicated_moved_and_removed() {
    let mut store = MachiningSequenceStore::new();
    let sequence = store.create_sequence("手順".to_string(), Vec::new(), stock());
    let rough = store.add_operation(sequence, content(4.0)).unwrap();
    let finish = store.add_operation(sequence, content(1.0)).unwrap();
    // 位置に挿む（工程の数より大きい位置は最後）
    let middle = store.insert_operation(sequence, 1, content(2.0)).unwrap();
    let last = store.insert_operation(sequence, 99, content(0.5)).unwrap();
    assert_eq!(
        store.operation_ids(sequence).unwrap(),
        vec![rough, middle, finish, last]
    );
    // 同じ中身で挿んでも、新しい工程（改訂の番号は 1、改訂の鍵は別）
    store
        .set_operation_content(sequence, rough, content(5.0))
        .unwrap();
    let copy = store.insert_operation(sequence, 1, content(5.0)).unwrap();
    assert_eq!(store.operation_position(sequence, copy).unwrap(), 1);
    assert_eq!(revision(&store, sequence, copy), 1);
    assert_ne!(
        store.operation_revision_key(sequence, copy).unwrap(),
        store.operation_revision_key(sequence, rough).unwrap()
    );
    // 並べ替えは、改訂の番号・改訂の鍵を変えない
    let key = store.operation_revision_key(sequence, finish).unwrap();
    store.move_operation(sequence, finish, 0).unwrap();
    assert_eq!(store.operation_position(sequence, finish).unwrap(), 0);
    assert_eq!(store.operation_revision_key(sequence, finish).unwrap(), key);
    store.move_operation(sequence, finish, 99).unwrap();
    assert_eq!(store.operation_ids(sequence).unwrap().last(), Some(&finish));
    // 除く
    store.remove_operation(sequence, middle).unwrap();
    assert_eq!(
        store.operation_ids(sequence).unwrap(),
        vec![rough, copy, last, finish]
    );
    assert_eq!(
        store.remove_operation(sequence, middle),
        Err(MachiningSequenceError::UnknownOperation(middle))
    );
    assert_eq!(
        store.move_operation(sequence, middle, 0),
        Err(MachiningSequenceError::UnknownOperation(middle))
    );
}
