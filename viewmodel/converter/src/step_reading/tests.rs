use std::time::Instant;

use application::product_shape::{StepImportStage, read_step_file};

use super::*;
use crate::space_model_document::tests::step_fixture;

/// 読み込み `reading` が終わるのを待って、結果を受け取る
fn wait_for(reading: &mut StepReading) -> StepReadingOutcome {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(outcome) = reading.take_outcome() {
            return outcome;
        }
        assert!(Instant::now() < deadline, "the reading does not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// 取り消しを頼んでから処理 `work` を別のスレッドで始め、終わるのを待った結果
fn run_cancelled<T: Send + 'static>(
    work: impl FnOnce(&TaskContext<StepImportProgress>) -> Result<T, TaskFailure<ProductShapeError>>
    + Send
    + 'static,
) -> TaskOutcome<T, ProductShapeError> {
    let (start, started) = std::sync::mpsc::channel::<()>();
    let mut task = BackgroundTask::spawn("step-reading-test", move |context| {
        started.recv().unwrap();
        work(context)
    })
    .unwrap();
    task.request_cancel();
    start.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(outcome) = task.take_outcome() {
            return outcome;
        }
        assert!(Instant::now() < deadline, "the task does not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn a_step_file_is_read_on_another_thread_up_to_the_display_meshes() {
    let display = ProductShapeDisplaySettings::default();
    let mut reading = StepReading::start(step_fixture("block_with_void.step"), display);
    assert!(!reading.is_cancel_requested());

    let StepReadingOutcome::Read(shapes) = wait_for(&mut reading) else {
        panic!("the file is read");
    };
    // 結果は 1 回だけ受け取る
    assert!(reading.take_outcome().is_none());
    // 最後の進み具合は、表示用の三角形の段の終わり（すべての B-rep の三角形を作り終えている）
    let progress = reading.progress().unwrap();
    assert_eq!(progress.stage, StepImportStage::DisplayMeshes);
    assert_eq!(progress.done, progress.total);
    assert_eq!(
        shapes
            .display_mesh_work(display.chord_tolerance)
            .brep_count(),
        1
    );
}

#[test]
fn a_file_that_cannot_be_read_fails() {
    let mut reading = StepReading::start(
        step_fixture("missing.step"),
        ProductShapeDisplaySettings::default(),
    );

    let StepReadingOutcome::Failed(error) = wait_for(&mut reading) else {
        panic!("the reading fails");
    };
    assert!(matches!(error, ProductShapeError::StepFile { .. }));
}

#[test]
fn cancellation_is_requested_without_waiting_for_the_reading_to_stop() {
    let mut reading = StepReading::start(
        step_fixture("block.step"),
        ProductShapeDisplaySettings::default(),
    );

    reading.request_cancel();

    assert!(reading.is_cancel_requested());
    // 小さなファイルは、取り消しを確かめる前に読み終えることがある（そのときは、読めた結果のまま）
    assert!(matches!(
        wait_for(&mut reading),
        StepReadingOutcome::Cancelled | StepReadingOutcome::Read(_)
    ));
}

#[test]
fn reading_a_step_file_stops_when_cancellation_is_requested() {
    let path = step_fixture("block.step");
    let display = ProductShapeDisplaySettings::default();

    let outcome =
        run_cancelled(move |context| read_step_shapes(&path, display, context).map(|_| ()));

    assert_eq!(outcome, TaskOutcome::Cancelled);
}

#[test]
fn building_the_display_meshes_stops_when_cancellation_is_requested() {
    let path = step_fixture("block.step");
    let mut file_reading =
        BackgroundTask::run_on_current_thread(|context| read_step_file(&path, context));
    let Some(TaskOutcome::Succeeded(shapes)) = file_reading.take_outcome() else {
        panic!("the file is read");
    };
    let work =
        Arc::new(shapes.display_mesh_work(ProductShapeDisplaySettings::default().chord_tolerance));

    let outcome = run_cancelled(move |context| triangulate_display_meshes(&work, context));

    assert_eq!(outcome, TaskOutcome::Cancelled);
}
