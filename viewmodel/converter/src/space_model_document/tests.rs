use std::sync::atomic::{AtomicUsize, Ordering};

use application::product_shape::ProductFaceGeometry;
use geo_algorithms::{Point3D, TriangleMesh3D};
use geo_contracts::TriangleMesh3DConstructor;

use super::*;

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
    let mut document = SpaceModelDocument::new();
    document.import_stl(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    document
}

#[test]
fn new_documents_have_one_operation_and_no_changes() {
    let document = SpaceModelDocument::new();
    assert!(!document.is_modified());
    assert_eq!(document.path(), None);
    assert!(document.shapes().entities().is_empty());
    assert_eq!(document.operation_content(), &initial_operation_content());
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
    let mut document = SpaceModelDocument::new();
    document.set_operation_content(document.operation_content().clone());
    document.set_stock(document.stock());
    // 表示中の製品形状がなく、加工の対象は空のまま
    document.target_visible_faces();
    assert!(!document.is_modified());

    let mut content = document.operation_content().clone();
    content.name = "変えた名前".to_string();
    document.set_operation_content(content);
    assert!(document.is_modified());

    let mut document = SpaceModelDocument::new();
    document.set_stock(MachiningStock {
        precision: 0.2,
        ..document.stock()
    });
    assert!(document.is_modified());
}

#[test]
fn edited_conditions_are_held_by_the_operation_and_the_sequence() {
    let mut document = SpaceModelDocument::new();
    let mut content = document.operation_content().clone();
    content.tool = CuttingToolKind::FlatEndMill.tool(8.0, Some(20.0));
    let stock = MachiningStock {
        precision: 0.2,
        ..document.stock()
    };
    document.set_operation_content(content.clone());
    document.set_stock(stock);
    assert_eq!(document.operation_content(), &content);
    assert_eq!(document.stock(), stock);
}

#[test]
fn opened_documents_target_the_visible_faces_without_a_change() {
    let path = temporary_file("rrm");
    let mut shapes = ProductShapeStore::new();
    shapes.add_face(ProductFaceGeometry::Mesh(pyramid()));
    let mut sequences = MachiningSequenceStore::new();
    // 加工の対象を合わせていない加工手順
    let sequence = sequences.create_sequence("手順".to_string(), Vec::new(), initial_stock());
    sequences
        .add_operation(sequence, initial_operation_content())
        .unwrap();
    let info = SpaceModelSaveInfo {
        program_version: "0.1.0".to_string(),
        saved_at_ms: 0,
    };
    save_space_model(&path, &SpaceModel::new(shapes, sequences), &info).unwrap();

    let mut document = SpaceModelDocument::open(&path).unwrap();
    assert!(!document.is_modified());
    let (shapes, sequences, _, _) = document.machining();
    assert_eq!(
        sequences.records()[0].target_faces,
        shapes.visible_face_ids()
    );
    document.target_visible_faces();
    assert!(!document.is_modified());
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn saved_documents_are_opened_with_the_same_contents_and_no_changes() {
    let path = temporary_file("rrm");
    let mut document = document_with_pyramid();
    let mut content = document.operation_content().clone();
    content.name = "保存した工程".to_string();
    document.set_operation_content(content.clone());
    document.target_visible_faces();

    document.save_as(&path, "0.1.0").unwrap();
    assert!(!document.is_modified());
    assert_eq!(document.path(), Some(path.as_path()));

    let mut opened = SpaceModelDocument::open(&path).unwrap();
    assert!(!opened.is_modified());
    assert_eq!(opened.path(), Some(path.as_path()));
    assert_eq!(opened.operation_content(), &content);
    let (shapes, sequences, sequence, operation) = opened.machining();
    let (saved_shapes, saved_sequences, saved_sequence, saved_operation) = document.machining();
    assert_eq!((sequence, operation), (saved_sequence, saved_operation));
    assert_eq!(sequences.records(), saved_sequences.records());
    assert_eq!(shapes.visible_face_ids(), saved_shapes.visible_face_ids());
    // 開いたときの表示中の面は、保存した加工の対象と同じため、確かめても変更にならない
    opened.target_visible_faces();
    assert!(!opened.is_modified());

    // 変えてから保存し直すと、変更なしに戻る
    opened.set_stock(MachiningStock {
        precision: 0.3,
        ..opened.stock()
    });
    assert!(opened.is_modified());
    opened.save_as(&path, "0.1.0").unwrap();
    assert!(!opened.is_modified());
    assert_eq!(
        SpaceModelDocument::open(&path).unwrap().stock().precision,
        0.3
    );
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn model_files_without_machining_sequences_get_one_operation() {
    let path = temporary_file("rrm");
    let mut shapes = ProductShapeStore::new();
    shapes.add_face(ProductFaceGeometry::Mesh(pyramid()));
    let model = SpaceModel::new(shapes, MachiningSequenceStore::new());
    let info = SpaceModelSaveInfo {
        program_version: "0.1.0".to_string(),
        saved_at_ms: 0,
    };
    save_space_model(&path, &model, &info).unwrap();

    let document = SpaceModelDocument::open(&path).unwrap();
    assert_eq!(document.operation_content(), &initial_operation_content());
    assert_eq!(document.shapes().entities().len(), 1);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn unreadable_model_files_are_not_opened() {
    let path = temporary_file("rrm");
    std::fs::write(&path, b"not a model file").unwrap();
    assert!(SpaceModelDocument::open(&path).is_err());
    std::fs::remove_file(&path).unwrap();
    assert!(matches!(
        SpaceModelDocument::open(&path),
        Err(SpaceModelFileError::Io(_))
    ));
}

#[test]
fn operations_are_added_duplicated_moved_selected_and_removed() {
    use application::machining_sequence::OperationSpec;
    let mut document = SpaceModelDocument::new();
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
        vec![(first, OPERATION_NAME), (rough, "荒")]
    );
    // 複写は、選んだ工程の中身を受け継ぐ
    let mut content = document.operation_content().clone();
    let OperationSpec::Scanline(mut scanline) = content.spec;
    scanline.stepover = 4.0;
    content.spec = OperationSpec::Scanline(scanline);
    document.set_operation_content(content);
    document.duplicate_operation("荒 2".to_string());
    let copy = document.selected_operation();
    let OperationSpec::Scanline(copied) = document.operation_content().spec;
    assert_eq!(copied.stepover, 4.0);
    assert_eq!(document.operation_content().name, "荒 2");

    // 並べ替え（端で止まる）
    assert!(document.move_selected_operation(-5));
    assert_eq!(document.operations()[0].id, copy);
    assert!(!document.move_selected_operation(-1));
    // 選び直し（手順にない工程は選ばない）
    assert!(document.select_operation(first));
    assert_eq!(document.selected_operation(), first);
    let unknown = SpaceModelDocument::new().selected_operation();
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

    // 保存して開き直しても、工程の並びはそのまま
    let path = temporary_file("rrm");
    document.save_as(&path, "0.1.0").unwrap();
    let reopened = SpaceModelDocument::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(reopened.operations(), document.operations());
}
