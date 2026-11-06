use cam_core::{
    MachiningStage, OperationSpec, ScanlineCutDirection, ScanlineParams, ScanlineToolRange,
    StockMargins, ToleranceProfile, Tool,
};
use geo_entity::FaceId;

use crate::{MachiningOperation, MachiningOperationContent, MachiningSequence, MachiningStock};

fn content() -> MachiningOperationContent {
    MachiningOperationContent {
        name: "仕上げ".to_string(),
        stage: MachiningStage::Finish,
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
    assert_eq!(operation.revision(), 1);

    // 同じ中身では増えない
    operation.set_content(content());
    assert_eq!(operation.revision(), 1);

    // 名前だけの変更でも増える
    let mut renamed = content();
    renamed.name = "仕上げ 2".to_string();
    operation.set_content(renamed);
    assert_eq!(operation.revision(), 2);

    let mut changed = operation.content().clone();
    changed.stage = MachiningStage::Rough;
    operation.set_content(changed);
    assert_eq!(operation.revision(), 3);
    assert_eq!(operation.content().stage, MachiningStage::Rough);
    assert_eq!(operation.id(), id);
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
    changed.stage = MachiningStage::SemiFinish;
    sequence
        .operation_mut(second_id)
        .expect("the operation belongs to the sequence")
        .set_content(changed);

    let operations = sequence.operations();
    assert_eq!(operations[0].revision(), 1);
    assert_eq!(operations[1].id(), second_id);
    assert_eq!(operations[1].revision(), 2);
    assert_eq!(operations[1].content().stage, MachiningStage::SemiFinish);
}
