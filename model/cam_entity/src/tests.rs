use cam_core::{
    MachiningBoundary, MachiningStage, OperationSpec, RectangleBoundary, ScanlineCutDirection,
    ScanlineParams, ScanlineToolRange, StockMargins, ToleranceProfile, Tool,
};
use geo_entity::FaceId;

use crate::{
    MachiningOperation, MachiningOperationAttributes, MachiningOperationConditions,
    MachiningOperationContent, MachiningOperationId, MachiningRevisionKey, MachiningSequence,
    MachiningSequenceId, MachiningStock,
};

fn content() -> MachiningOperationContent {
    MachiningOperationContent {
        attributes: MachiningOperationAttributes {
            name: "仕上げ".to_string(),
            stage: MachiningStage::Finish,
        },
        conditions: MachiningOperationConditions {
            spec: OperationSpec::Scanline(ScanlineParams {
                stepover: 1.0,
                feed_rate: 1000.0,
                tool_range: ScanlineToolRange::default(),
                cut_direction: ScanlineCutDirection::default(),
                angle_degrees: 0.0,
            }),
            tool: Tool::ball_end_mill("T1".to_string(), 10.0, 20.0),
            boundary: None,
            tolerance_profile: ToleranceProfile::PressRough,
        },
    }
}

fn stock() -> MachiningStock {
    MachiningStock {
        margins: StockMargins::default(),
        precision: 0.05,
    }
}

#[test]
fn changing_the_content_increments_the_revision() {
    let mut operation = MachiningOperation::new(content());
    let id = operation.id();
    let first_key = operation.revision_key();
    assert_eq!(operation.revision(), 1);

    // 同じ中身では増えない（改訂の鍵も変わらない）
    operation.set_content(content());
    assert_eq!(operation.revision(), 1);
    assert_eq!(operation.revision_key(), first_key);

    // 属性（名前・加工の段階）だけの変更では増えない（属性は計算に使わない）。属性は替わる
    let mut renamed = content();
    renamed.attributes.name = "仕上げ 2".to_string();
    renamed.attributes.stage = MachiningStage::Rough;
    operation.set_content(renamed);
    assert_eq!(operation.revision(), 1);
    assert_eq!(operation.revision_key(), first_key);
    assert_eq!(operation.content().attributes.name, "仕上げ 2");
    assert_eq!(operation.content().attributes.stage, MachiningStage::Rough);

    // 計算の条件を変えると増え、改訂の鍵を作り直す
    let mut changed = operation.content().clone();
    changed.conditions.tolerance_profile = ToleranceProfile::MoldFinish;
    operation.set_content(changed);
    assert_eq!(operation.revision(), 2);
    assert_ne!(operation.revision_key(), first_key);
    assert_eq!(
        operation.content().conditions.tolerance_profile,
        ToleranceProfile::MoldFinish
    );
    assert_eq!(operation.id(), id);
}

#[test]
fn changing_any_of_the_conditions_increments_the_revision() {
    let changes: [fn(&mut MachiningOperationConditions); 4] = [
        |conditions| {
            let OperationSpec::Scanline(scanline) = &mut conditions.spec;
            scanline.stepover = 2.0;
        },
        |conditions| conditions.tool = Tool::ball_end_mill("T1".to_string(), 8.0, 20.0),
        |conditions| {
            conditions.boundary = Some(MachiningBoundary::Rectangle(RectangleBoundary {
                rect_min: [0.0, 0.0],
                rect_max: [10.0, 10.0],
            }))
        },
        |conditions| conditions.tolerance_profile = ToleranceProfile::MoldFinish,
    ];
    for change in changes {
        let mut operation = MachiningOperation::new(content());
        let mut changed = content();
        change(&mut changed.conditions);
        operation.set_content(changed);
        assert_eq!(operation.revision(), 2);
    }
}

#[test]
fn changing_the_target_or_the_stock_increments_the_revisions_of_all_operations() {
    let face = FaceId::new_random();
    let mut sequence = MachiningSequence::new("手順".to_string(), vec![face], stock());
    sequence.push_operation(MachiningOperation::new(content()));
    sequence.push_operation(MachiningOperation::new(content()));
    let revisions = |sequence: &MachiningSequence| {
        sequence
            .operations()
            .iter()
            .map(MachiningOperation::revision)
            .collect::<Vec<_>>()
    };

    sequence.set_target_faces(vec![face]);
    sequence.set_stock(stock());
    assert_eq!(revisions(&sequence), [1, 1]);

    let other = FaceId::new_random();
    sequence.set_target_faces(vec![face, other]);
    assert_eq!(revisions(&sequence), [2, 2]);

    // 同じ面の集合は、順と重なりが違っても変えたことにしない
    sequence.set_target_faces(vec![other, face, other]);
    assert_eq!(revisions(&sequence), [2, 2]);
    assert_eq!(sequence.target_faces(), [other, face]);

    let mut thicker = stock();
    thicker.margins.top = 1.0;
    sequence.set_stock(thicker);
    assert_eq!(revisions(&sequence), [3, 3]);
}

#[test]
fn an_operation_is_changed_through_its_sequence() {
    let mut sequence = MachiningSequence::new("手順".to_string(), Vec::new(), stock());
    let first = MachiningOperation::new(content());
    let second = MachiningOperation::new(content());
    let second_id = second.id();
    sequence.push_operation(first);
    sequence.push_operation(second);

    let mut changed = content();
    changed.conditions.tolerance_profile = ToleranceProfile::MoldFinish;
    sequence
        .operation_mut(second_id)
        .expect("the operation belongs to the sequence")
        .set_content(changed);

    let operations = sequence.operations();
    assert_eq!(operations[0].revision(), 1);
    assert_eq!(operations[1].id(), second_id);
    assert_eq!(operations[1].revision(), 2);
    assert_eq!(
        operations[1].content().conditions.tolerance_profile,
        ToleranceProfile::MoldFinish
    );
}

#[test]
fn ids_and_revision_keys_are_restored_from_their_uuids() {
    let operation = MachiningOperation::new(content());
    let id = MachiningOperationId::from_uuid(operation.id().as_uuid());
    let key = MachiningRevisionKey::from_uuid(operation.revision_key().as_uuid());
    assert_eq!(id, operation.id());
    assert_eq!(key, operation.revision_key());
    let sequence = MachiningSequence::new("手順".to_string(), Vec::new(), stock());
    assert_eq!(
        MachiningSequenceId::from_uuid(sequence.id().as_uuid()),
        sequence.id()
    );
}

#[test]
fn the_revision_stops_at_its_limit_and_the_revision_key_still_changes() {
    let saved = MachiningOperation::new(content());
    let mut operation = MachiningOperation::restore(
        saved.id(),
        content(),
        std::num::NonZeroU32::MAX,
        saved.revision_key(),
    );
    let mut changed = content();
    changed.conditions.tolerance_profile = ToleranceProfile::MoldFinish;
    operation.set_content(changed);
    assert_eq!(operation.revision(), u32::MAX);
    assert_ne!(operation.revision_key(), saved.revision_key());
}
