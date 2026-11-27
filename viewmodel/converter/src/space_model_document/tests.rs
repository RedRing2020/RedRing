use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use application::background_task::{TaskContext, TaskFailure};
use application::product_shape::{
    ProductFaceGeometry, StepImportProgress, StepImportStage, StepShapes, read_step_file,
};
use geo_algorithms::{Point3D, TriangleMesh3D};
use geo_contracts::TriangleMesh3DConstructor;

use super::*;

/// 試験で作る加工手順・加工工程の名前
pub(crate) const NAMES: DefaultNames<'static> = DefaultNames {
    sequence: "加工手順",
    operation: "スキャン加工",
};

/// 試験ごとの一時のファイルの名前（並べて走る試験どうしで分ける）
fn temporary_file(extension: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "redring_space_model_document_{}_{}.{extension}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn pyramid() -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(20.0, 0.0, 0.0),
            Point3D::new(20.0, 20.0, 0.0),
            Point3D::new(0.0, 20.0, 0.0),
            Point3D::new(10.0, 10.0, 6.0),
        ],
        vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
    )
    .unwrap()
}

/// 四角錐（0〜20, 0〜20, 0〜6）の STL を取り込んだ文書
pub(crate) fn document_with_pyramid() -> SpaceModelDocument {
    let path = temporary_file("stl");
    geo_io::stl::save_stl(&pyramid(), &path).unwrap();
    let mut document = SpaceModelDocument::new(NAMES);
    document.import_stl(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    document
}

#[test]
fn new_documents_have_one_operation_and_no_changes() {
    let document = SpaceModelDocument::new(NAMES);
    assert!(!document.is_modified());
    assert_eq!(document.path(), None);
    assert!(document.shapes().entities().is_empty());
    assert_eq!(
        document.operation_content(),
        &initial_operation_content(NAMES.operation.to_string())
    );
    assert_eq!(document.stock(), initial_stock());
}

#[test]
fn imported_shapes_are_displayed_and_are_a_change() {
    let document = document_with_pyramid();
    assert!(document.is_modified());
    let meshes = document.visible_meshes();
    assert_eq!(meshes.len(), 1);
    assert_eq!(meshes[0].indices.len(), 12);
}

#[test]
fn only_edits_that_change_the_conditions_are_changes() {
    let mut document = SpaceModelDocument::new(NAMES);
    document.set_operation_content(document.operation_content().clone());
    document.set_stock(document.stock());
    assert!(!document.is_modified());

    let mut content = document.operation_content().clone();
    content.attributes.name = "変えた名前".to_string();
    document.set_operation_content(content);
    assert!(document.is_modified());

    let mut document = SpaceModelDocument::new(NAMES);
    document.set_stock(MachiningStock {
        precision: 0.2,
        ..document.stock()
    });
    assert!(document.is_modified());
}

#[test]
fn edited_conditions_are_held_by_the_operation_and_the_sequence() {
    let mut document = SpaceModelDocument::new(NAMES);
    let mut content = document.operation_content().clone();
    content.conditions.tool = CuttingToolKind::FlatEndMill.tool(8.0, Some(20.0));
    let stock = MachiningStock {
        precision: 0.2,
        ..document.stock()
    };
    document.set_operation_content(content.clone());
    document.set_stock(stock);
    assert_eq!(document.operation_content(), &content);
    assert_eq!(document.stock(), stock);
}

/// 製品形状（四角錐の面単体 2 つ。1 つ目は隠す）と、隠した 1 つ目の面だけを加工の対象にした加工手順を保存した
/// モデルファイル（加工の対象は、表示中の面と重ならない）
fn model_file_targeting_the_first_face(path: &Path) {
    let mut shapes = ProductShapeStore::new();
    let first = shapes.add_face(ProductFaceGeometry::Mesh(pyramid()));
    shapes.add_face(ProductFaceGeometry::Mesh(pyramid()));
    shapes.set_visible(first.entity_id, false).unwrap();
    let mut sequences = MachiningSequenceStore::new();
    let sequence =
        sequences.create_sequence("手順".to_string(), vec![first.face_id], initial_stock());
    sequences
        .add_operation(
            sequence,
            initial_operation_content(NAMES.operation.to_string()),
        )
        .unwrap();
    let info = SpaceModelSaveInfo {
        program_version: "0.1.0".to_string(),
        saved_at_ms: 0,
    };
    save_space_model(path, &SpaceModel::new(shapes, sequences), &info).unwrap();
}

#[test]
fn opened_documents_keep_the_saved_target_faces_without_a_change() {
    let path = temporary_file("rrm");
    model_file_targeting_the_first_face(&path);
    let (saved, _) = load_space_model(&path).unwrap();

    // 加工の対象は、保存したまま（隠した面を含み、表示中の面を含まない。表示中の面に合わせない）。工程の改訂も
    // 変わらない
    let document = SpaceModelDocument::open(&path, NAMES).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(!document.is_modified());
    assert_eq!(document.target_face_count(), 1);
    let (shapes, sequences, _, _) = document.machining();
    assert_ne!(
        sequences.records()[0].target_faces,
        shapes.visible_face_ids()
    );
    assert_eq!(sequences.records(), saved.sequences.records());
}

#[test]
fn hiding_target_faces_keeps_the_target_and_the_revisions() {
    let mut document = document_with_pyramid();
    let path = temporary_file("rrm");
    document.save_as(&path, "0.1.0").unwrap();
    std::fs::remove_file(&path).unwrap();
    let (_, sequences, _, _) = document.machining();
    let records = sequences.records();

    // 加工の対象の面を隠しても、加工の対象・工程の改訂は変わらず、保存していない変更にもならない
    document.hide_face_entity(0);
    assert!(document.shapes().visible_face_ids().is_empty());
    assert_eq!(document.target_face_count(), 1);
    let (_, sequences, _, _) = document.machining();
    assert_eq!(sequences.records(), records);
    assert!(!document.is_modified());
}

#[test]
fn imported_faces_are_added_to_the_target_even_when_other_faces_are_hidden() {
    let path = temporary_file("rrm");
    model_file_targeting_the_first_face(&path);
    let mut document = SpaceModelDocument::open(&path, NAMES).unwrap();
    std::fs::remove_file(&path).unwrap();
    let keys: Vec<_> = document
        .operations()
        .iter()
        .map(|item| document.revision_key_of(item.id).unwrap())
        .collect();

    // 取り込んだ面だけを加える（加工の対象にしていない表示中の面は加えず、隠した加工の対象の面は残す）
    let stl = temporary_file("stl");
    geo_io::stl::save_stl(&pyramid(), &stl).unwrap();
    document.import_stl(&stl).unwrap();
    std::fs::remove_file(&stl).unwrap();
    assert!(document.is_modified());
    assert_eq!(document.target_face_count(), 2);
    let (shapes, sequences, _, _) = document.machining();
    let added = shapes.entities()[2].face_ids();
    assert!(sequences.records()[0].target_faces.ends_with(&added));
    // 加工の対象が変わり、手順のすべての工程の改訂が進む
    for (item, key) in document.operations().iter().zip(keys) {
        assert_ne!(document.revision_key_of(item.id).unwrap(), key);
    }
}

#[test]
fn saved_documents_are_opened_with_the_same_contents_and_no_changes() {
    let path = temporary_file("rrm");
    let mut document = document_with_pyramid();
    let mut content = document.operation_content().clone();
    content.attributes.name = "保存した工程".to_string();
    document.set_operation_content(content.clone());

    document.save_as(&path, "0.1.0").unwrap();
    assert!(!document.is_modified());
    assert_eq!(document.path(), Some(path.as_path()));

    let mut opened = SpaceModelDocument::open(&path, NAMES).unwrap();
    assert!(!opened.is_modified());
    assert_eq!(opened.path(), Some(path.as_path()));
    assert_eq!(opened.operation_content(), &content);
    let (shapes, sequences, sequence, operation) = opened.machining();
    let (saved_shapes, saved_sequences, saved_sequence, saved_operation) = document.machining();
    assert_eq!((sequence, operation), (saved_sequence, saved_operation));
    assert_eq!(sequences.records(), saved_sequences.records());
    assert_eq!(shapes.visible_face_ids(), saved_shapes.visible_face_ids());

    // 変えてから保存し直すと、変更なしに戻る
    opened.set_stock(MachiningStock {
        precision: 0.3,
        ..opened.stock()
    });
    assert!(opened.is_modified());
    opened.save_as(&path, "0.1.0").unwrap();
    assert!(!opened.is_modified());
    assert_eq!(
        SpaceModelDocument::open(&path, NAMES)
            .unwrap()
            .stock()
            .precision,
        0.3
    );
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn model_files_without_machining_sequences_get_one_operation() {
    let path = temporary_file("rrm");
    let mut shapes = ProductShapeStore::new();
    shapes.add_face(ProductFaceGeometry::Mesh(pyramid()));
    let hidden = shapes.add_face(ProductFaceGeometry::Mesh(pyramid()));
    shapes.set_visible(hidden.entity_id, false).unwrap();
    let model = SpaceModel::new(shapes, MachiningSequenceStore::new());
    let info = SpaceModelSaveInfo {
        program_version: "0.1.0".to_string(),
        saved_at_ms: 0,
    };
    save_space_model(&path, &model, &info).unwrap();

    let document = SpaceModelDocument::open(&path, NAMES).unwrap();
    assert_eq!(
        document.operation_content(),
        &initial_operation_content(NAMES.operation.to_string())
    );
    assert_eq!(document.shapes().entities().len(), 2);
    // 作った加工手順の加工の対象は、隠した面も含む、すべての面
    assert_eq!(document.target_face_count(), 2);
    assert!(!document.is_modified());
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn unreadable_model_files_are_not_opened() {
    let path = temporary_file("rrm");
    std::fs::write(&path, b"not a model file").unwrap();
    assert!(SpaceModelDocument::open(&path, NAMES).is_err());
    std::fs::remove_file(&path).unwrap();
    assert!(matches!(
        SpaceModelDocument::open(&path, NAMES),
        Err(SpaceModelFileError::Io(_))
    ));
}

#[test]
fn operations_are_added_duplicated_moved_selected_and_removed() {
    use application::machining_sequence::OperationSpec;
    let mut document = SpaceModelDocument::new(NAMES);
    let first = document.selected_operation();
    assert_eq!(document.operations().len(), 1);
    assert!(!document.is_modified());
    // 工程が 1 つだけなら消さない
    assert!(!document.remove_selected_operation());

    // 加えると、選んだ工程のすぐ後に入り、編集の対象になる
    document.add_operation("荒".to_string());
    let rough = document.selected_operation();
    assert!(document.is_modified());
    assert_eq!(
        document
            .operations()
            .iter()
            .map(|item| (item.id, item.name.as_str()))
            .collect::<Vec<_>>(),
        vec![(first, NAMES.operation), (rough, "荒")]
    );
    // 複写は、選んだ工程の中身を受け継ぐ
    let mut content = document.operation_content().clone();
    let OperationSpec::Scanline(mut scanline) = content.conditions.spec;
    scanline.stepover = 4.0;
    content.conditions.spec = OperationSpec::Scanline(scanline);
    document.set_operation_content(content);
    document.duplicate_operation("荒 2".to_string());
    let copy = document.selected_operation();
    let OperationSpec::Scanline(copied) = document.operation_content().conditions.spec;
    assert_eq!(copied.stepover, 4.0);
    assert_eq!(document.operation_content().attributes.name, "荒 2");

    // 並べ替え（端で止まる）
    assert!(document.move_selected_operation(-5));
    assert_eq!(document.operations()[0].id, copy);
    assert!(!document.move_selected_operation(-1));
    // 選び直し（手順にない工程は選ばない）
    assert!(document.select_operation(first));
    assert_eq!(document.selected_operation(), first);
    let unknown = SpaceModelDocument::new(NAMES).selected_operation();
    assert!(!document.select_operation(unknown));
    // 消すと、次の工程（最後なら前の工程）を選ぶ
    assert!(document.select_operation(rough));
    assert!(document.remove_selected_operation());
    assert_eq!(document.selected_operation(), first);
    assert_eq!(
        document
            .operations()
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        vec![copy, first]
    );

    // 名前を替えても、工程の改訂は進めない（計算の結果を今の改訂の結果のまま使う）。保存していない変更にはなる
    let key = document
        .model
        .sequences
        .operation_revision_key(document.sequence, first)
        .unwrap();
    document.rename_selected_operation("  仕上げ  ");
    assert_eq!(document.operation_content().attributes.name, "仕上げ");
    assert_eq!(
        document
            .model
            .sequences
            .operation_revision_key(document.sequence, first)
            .unwrap(),
        key
    );
    document.rename_selected_operation("   ");
    assert_eq!(document.operation_content().attributes.name, "仕上げ");

    // 保存して開き直しても、工程の並びはそのまま
    let path = temporary_file("rrm");
    document.save_as(&path, "0.1.0").unwrap();
    let reopened = SpaceModelDocument::open(&path, NAMES).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(reopened.operations(), document.operations());
}

/// 試験の STEP ファイル（`tests/fixtures/step` の `name`）
fn step_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/step")
        .join(name)
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
    wait_for(&mut task)
}

#[test]
fn imported_step_shapes_are_displayed_and_added_to_the_target() {
    let mut document = SpaceModelDocument::new(NAMES);

    let report = document
        .import_step(&step_fixture("plate_with_holes.step"))
        .unwrap();

    assert_eq!(report.shells, 1);
    assert_eq!(report.issues(), 0);
    assert!(document.is_modified());
    let [shell] = document.shapes().shells() else {
        panic!("one shell is added");
    };
    assert_eq!(document.target_face_count(), shell.face_ids().len());
    let meshes = document.visible_meshes();
    assert_eq!(meshes.len(), 1);
    assert!(!meshes[0].indices.is_empty());
}

#[test]
fn a_step_file_that_cannot_be_read_adds_nothing() {
    let mut document = SpaceModelDocument::new(NAMES);

    let error = document
        .import_step(&step_fixture("missing.step"))
        .unwrap_err();

    assert!(matches!(error, ProductShapeError::StepFile { .. }));
    assert!(!document.is_modified());
    assert!(document.shapes().shells().is_empty());
    assert_eq!(document.target_face_count(), 0);
}

#[test]
fn a_step_file_is_read_on_another_thread_up_to_the_display_meshes() {
    let path = step_fixture("block_with_void.step");
    let display = ProductShapeDisplaySettings::default();
    let mut task = BackgroundTask::spawn("step-reading-test", move |context| {
        step_reading::read_step_shapes(&path, display, context)
    })
    .unwrap();

    let TaskOutcome::Succeeded(shapes) = wait_for(&mut task) else {
        panic!("the file is read");
    };
    // 最後の進み具合は、表示用の三角形の段の終わり（すべての B-rep の三角形を作り終えている）
    let progress = task.progress().unwrap();
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
fn reading_a_step_file_stops_when_cancellation_is_requested() {
    let path = step_fixture("block.step");
    let display = ProductShapeDisplaySettings::default();

    let outcome = run_cancelled(move |context| {
        step_reading::read_step_shapes(&path, display, context).map(|_| ())
    });

    assert_eq!(outcome, TaskOutcome::Cancelled);
}

#[test]
fn building_the_display_meshes_stops_when_cancellation_is_requested() {
    let path = step_fixture("block.step");
    let mut reading =
        BackgroundTask::run_on_current_thread(|context| read_step_file(&path, context));
    let Some(TaskOutcome::Succeeded(shapes)) = reading.take_outcome() else {
        panic!("the file is read");
    };
    let shapes: StepShapes = shapes;
    let work =
        Arc::new(shapes.display_mesh_work(ProductShapeDisplaySettings::default().chord_tolerance));

    let outcome =
        run_cancelled(move |context| step_reading::triangulate_display_meshes(&work, context));

    assert_eq!(outcome, TaskOutcome::Cancelled);
}
