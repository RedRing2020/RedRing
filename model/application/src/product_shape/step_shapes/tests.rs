use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};

use geo_contracts::FaceSetEntity3DProperties;
use geo_entity::Metadata;

use super::*;
use crate::background_task::{BackgroundTask, TaskOutcome};
use crate::product_shape::step_import::tests::{ASSEMBLY, PART, step};
use crate::product_shape::{ProductShapeStore, ShellSource, SimilarityTransform3D, test_fixtures};

/// 表示用の三角形の許容弦誤差（mm）
const CHORD_TOLERANCE: f64 = 0.05;

/// STEP のバイト列 `bytes` を書いた、試験ごとの一時のファイル（並べて走る試験どうしで分ける）
fn temporary_step(bytes: &[u8]) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "redring_step_shapes_{}_{}.step",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, bytes).unwrap();
    path
}

/// STEP のバイト列 `bytes` を、今のスレッドで終わりまで読んだ形状と、最後に知らせた進み具合
fn read(bytes: &[u8]) -> (StepShapes, Option<StepImportProgress>) {
    let path = temporary_step(bytes);
    let mut task = BackgroundTask::run_on_current_thread(|context| read_step_file(&path, context));
    std::fs::remove_file(&path).unwrap();
    let progress = task.progress();
    match task.take_outcome() {
        Some(TaskOutcome::Succeeded(shapes)) => (shapes, progress),
        _ => panic!("the file is read"),
    }
}

/// 処理 `task` が終わるのを待って、結果を受け取る
fn wait_for<T, E, P>(task: &mut BackgroundTask<T, E, P>) -> TaskOutcome<T, E> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(outcome) = task.take_outcome() {
            return outcome;
        }
        assert!(Instant::now() < deadline, "the task does not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// B-rep `brep` を配置なしで置く、読んだシェル
fn imported(brep: &Arc<Brep<f64>>) -> ImportedShell {
    ImportedShell {
        brep: Arc::clone(brep),
        placement: SimilarityTransform3D::identity(),
        source: ShellSource {
            file_name: "shapes.step".to_string(),
            occurrences: Vec::new(),
            faces: vec![0; brep.face_count()],
        },
        metadata: Metadata::default(),
    }
}

#[test]
fn read_shapes_are_added_as_shells_that_share_the_brep_of_their_part() {
    let (shapes, progress) = read(&step(&(PART.to_string() + ASSEMBLY)));
    // 読む処理は、B-rep を作る段まで知らせる
    assert_eq!(
        progress.map(|progress| progress.stage),
        Some(StepImportStage::Topology)
    );

    let mut store = ProductShapeStore::new();
    let report = store.add_step_shapes(shapes);

    assert_eq!(report.shells, 2);
    assert_eq!(report.skipped_occurrences, 1);
    let [first, second] = store.shells() else {
        panic!("two shells are added");
    };
    assert!(Arc::ptr_eq(&first.shell().brep, &second.shell().brep));
    assert!(Arc::ptr_eq(&first.shell().meshes, &second.shell().meshes));
    assert_eq!(first.face_ids().len(), 2);
}

#[test]
fn a_missing_file_fails_without_shapes() {
    let path = std::env::temp_dir().join("redring_step_shapes_missing.step");
    let mut task =
        BackgroundTask::run_on_current_thread(|context| read_step_file(&path, context).map(|_| ()));

    let Some(TaskOutcome::Failed(ProductShapeError::StepFile { kind, .. })) = task.take_outcome()
    else {
        panic!("the reading fails with the error of the file");
    };
    assert_eq!(kind, std::io::ErrorKind::NotFound);
}

#[test]
fn reading_stops_when_cancellation_is_requested() {
    let path = temporary_step(&step(PART));
    let (start, started) = std::sync::mpsc::channel::<()>();
    let reading_path = path.clone();
    let mut task = BackgroundTask::spawn("step-shapes-test", move |context| {
        // 取り消しを頼まれてから読み始める
        started.recv().unwrap();
        read_step_file(&reading_path, context).map(|_| ())
    })
    .unwrap();
    task.request_cancel();
    start.send(()).unwrap();

    assert_eq!(wait_for(&mut task), TaskOutcome::Cancelled);
    std::fs::remove_file(&path).unwrap();
    // 初めの段の初めを知らせたところで止まる
    assert_eq!(
        task.progress(),
        Some(StepImportProgress::in_stage(StepImportStage::Syntax, 0, 1))
    );
}

#[test]
fn display_meshes_are_built_once_for_each_brep_before_the_shapes_are_added() {
    let (shapes, _) = read(&step(&(PART.to_string() + ASSEMBLY)));
    let work = shapes.display_mesh_work(CHORD_TOLERANCE);
    // 2 つの出現は、同じ B-rep を使う
    assert_eq!(work.brep_count(), 1);
    let before = work.progress();
    assert_eq!(before.stage, StepImportStage::DisplayMeshes);
    assert!(before.done < before.total);

    assert!(work.triangulate_next());
    assert!(!work.triangulate_next());

    let after = work.progress();
    assert_eq!(after.done, after.total);
    // 加えた後の表示は、作ってある三角形を使う
    let mut store = ProductShapeStore::new();
    store.add_step_shapes(shapes);
    let [first, second] = store.shells() else {
        panic!("two shells are added");
    };
    assert!(first.shell().mesh_ready(CHORD_TOLERANCE));
    for shell in [first, second] {
        for face_id in shell.face_ids() {
            assert!(shell.face_display_mesh(face_id, CHORD_TOLERANCE).is_ok());
        }
    }
    assert_eq!(first.shell().meshes.lock().meshes.len(), 1);
}

#[test]
fn display_meshes_are_built_from_several_threads_without_repeating_a_brep() {
    let breps: Vec<Arc<Brep<f64>>> = [
        test_fixtures::tetrahedron_brep(),
        test_fixtures::cube_brep(),
        test_fixtures::cylinder_brep(),
        test_fixtures::sphere_brep(),
        test_fixtures::torus_brep(),
        test_fixtures::cone_brep(),
    ]
    .into_iter()
    .map(Arc::new)
    .collect();
    // 初めの B-rep は 2 回置く
    let shells = breps
        .iter()
        .chain(breps.first())
        .map(imported)
        .collect::<Vec<_>>();
    let shapes = StepShapes::new(shells, StepImportReport::default());
    let work = shapes.display_mesh_work(CHORD_TOLERANCE);
    assert_eq!(work.brep_count(), breps.len());
    // 面の多い B-rep から取る
    assert!(
        work.items
            .is_sorted_by_key(|item| std::cmp::Reverse(item.faces.len()))
    );

    let taken = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                while work.triangulate_next() {
                    taken.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    // どの B-rep も、1 回だけ取られ、三角形が置き場に残る
    assert_eq!(taken.load(Ordering::Relaxed), breps.len());
    let progress = work.progress();
    assert_eq!(progress.done, progress.total);
    let (shells, _) = shapes.into_parts();
    for (_, meshes) in &shells {
        let meshes = meshes.lock();
        assert_eq!(meshes.meshes.len(), 1);
        assert!(meshes.get(CHORD_TOLERANCE.to_bits()).is_some());
        assert!(meshes.building.is_empty());
    }
}
