//! `ProductShapeStore` のテスト

use std::path::PathBuf;

use cam_algorithms::SolverGeometry;
use geo_algorithms::{NurbsSurface3D, Point3D, TriangleMesh3D, Vector3D};
use geo_contracts::{
    FaceSetEntity3DProperties, NurbsSurface3DConstructor, TriangleMesh3DConstructor,
    TriangleMesh3DProperties,
};

use super::*;

/// 底面 z = 0・頂点 z = `height` の四角錐（側面の 4 枚の三角形）
fn pyramid(height: f64) -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
            Point3D::new(0.0, 10.0, 0.0),
            Point3D::new(5.0, 5.0, height),
        ],
        vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
    )
    .unwrap()
}

/// 中央が盛り上がった 2 次の曲面
fn dome() -> NurbsSurface3D<f64> {
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let row = |x: f64, middle: f64| {
        vec![
            Point3D::new(x, 0.0, 0.0),
            Point3D::new(x, 10.0, middle),
            Point3D::new(x, 20.0, 0.0),
        ]
    };
    NurbsSurface3D::new(
        2,
        2,
        knots.clone(),
        knots,
        &[row(0.0, 0.0), row(10.0, 5.0), row(20.0, 0.0)],
        None,
    )
    .unwrap()
}

/// テストごとに別の一時ファイルのパス
fn temporary_stl(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "redring_product_shape_{}_{name}.stl",
        std::process::id()
    ))
}

#[test]
fn importing_an_stl_holds_a_single_face_entity_with_a_face_id() {
    let path = temporary_stl("import");
    geo_io::stl::save_stl(&pyramid(8.0), &path).unwrap();
    let mut store = ProductShapeStore::new();
    let added = store.import_stl(&path).unwrap();
    std::fs::remove_file(&path).unwrap();

    let [entity] = store.entities() else {
        panic!("one entity is held");
    };
    assert_eq!(entity.id(), added.entity_id);
    assert_eq!(entity.face().id(), added.face_id);
    let ProductFaceGeometry::Mesh(mesh) = entity.face().geometry() else {
        panic!("an STL becomes a mesh face");
    };
    assert_eq!(mesh.triangle_count(), 4);
    assert!(entity.display().visible);
}

#[test]
fn importing_an_stl_without_triangles_fails() {
    let path = temporary_stl("no_triangles");
    std::fs::write(&path, "this is not an STL file\n").unwrap();
    let mut store = ProductShapeStore::new();
    let error = store.import_stl(&path).unwrap_err();
    std::fs::remove_file(&path).unwrap();
    assert!(matches!(error, ProductShapeError::Stl(_)), "{error}");
    assert!(store.entities().is_empty());
}

#[test]
fn importing_a_missing_stl_fails() {
    let mut store = ProductShapeStore::new();
    let error = store.import_stl(&temporary_stl("missing")).unwrap_err();
    assert!(
        matches!(
            error,
            ProductShapeError::StlFile {
                kind: std::io::ErrorKind::NotFound,
                ..
            }
        ),
        "{error}"
    );
    assert!(store.entities().is_empty());
}

#[test]
fn face_ids_are_unique() {
    let mut store = ProductShapeStore::new();
    let first = store.add_face(ProductFaceGeometry::Mesh(pyramid(8.0)));
    let second = store.add_face(ProductFaceGeometry::Mesh(pyramid(8.0)));
    assert_ne!(first.face_id, second.face_id);
    assert_ne!(first.entity_id, second.entity_id);
}

#[test]
fn hidden_entities_are_excluded_from_the_visible_faces() {
    let mut store = ProductShapeStore::new();
    let shown = store.add_face(ProductFaceGeometry::Mesh(pyramid(8.0)));
    let hidden = store.add_face(ProductFaceGeometry::Mesh(pyramid(4.0)));
    store.set_visible(hidden.entity_id, false).unwrap();
    assert_eq!(store.visible_face_ids(), vec![shown.face_id]);

    let SolverGeometry::TriangleMesh(mesh) = store
        .resolve_solver_geometry(&store.visible_face_ids())
        .unwrap()
    else {
        panic!("mesh faces resolve to a mesh");
    };
    assert_eq!(mesh.triangle_count(), 4);
    assert!(mesh.vertices().iter().all(|point| point.z() <= 8.0));
    assert!(mesh.vertices().iter().any(|point| point.z() == 8.0));

    store.set_visible(hidden.entity_id, true).unwrap();
    assert_eq!(
        store.visible_face_ids(),
        vec![shown.face_id, hidden.face_id]
    );
}

#[test]
fn mesh_faces_are_concatenated_into_one_mesh() {
    let mut store = ProductShapeStore::new();
    let first = store.add_face(ProductFaceGeometry::Mesh(pyramid(8.0)));
    let second = store.add_face(ProductFaceGeometry::Mesh(pyramid(4.0)));
    // 同じ面を重ねて指定しても 1 回だけ使う
    let SolverGeometry::TriangleMesh(mesh) = store
        .resolve_solver_geometry(&[first.face_id, second.face_id, first.face_id])
        .unwrap()
    else {
        panic!("mesh faces resolve to a mesh");
    };
    assert_eq!(mesh.vertex_count(), 10);
    assert_eq!(mesh.triangle_count(), 8);
    // 2 つ目のメッシュの三角形は、2 つ目のメッシュの頂点を指す
    let apex = mesh.indices()[4][2];
    assert_eq!(mesh.vertices()[apex].z(), 4.0);
}

#[test]
fn surface_faces_resolve_to_a_surface_set() {
    let mut store = ProductShapeStore::new();
    let faces: Vec<FaceId> = (0..2)
        .map(|_| store.add_face(ProductFaceGeometry::Surface(dome())).face_id)
        .collect();
    let SolverGeometry::NurbsSurfaceSet(surfaces) = store.resolve_solver_geometry(&faces).unwrap()
    else {
        panic!("surface faces resolve to a surface set");
    };
    assert_eq!(surfaces.len(), 2);
}

#[test]
fn invalid_face_sets_are_rejected() {
    let mut store = ProductShapeStore::new();
    let mesh = store.add_face(ProductFaceGeometry::Mesh(pyramid(8.0)));
    let surface = store.add_face(ProductFaceGeometry::Surface(dome()));
    let unknown = FaceId::new_random();

    assert_eq!(
        store
            .resolve_solver_geometry(&[mesh.face_id, surface.face_id])
            .unwrap_err(),
        ProductShapeError::MixedFaceKinds
    );
    assert_eq!(
        store
            .resolve_solver_geometry(&[surface.face_id, mesh.face_id])
            .unwrap_err(),
        ProductShapeError::MixedFaceKinds
    );
    assert_eq!(
        store
            .resolve_solver_geometry(&[mesh.face_id, unknown])
            .unwrap_err(),
        ProductShapeError::UnknownFace(unknown)
    );
    assert_eq!(
        store.resolve_solver_geometry(&[]).unwrap_err(),
        ProductShapeError::NoFaces
    );
    let missing = EntityId::new_random();
    assert_eq!(
        store.set_visible(missing, false).unwrap_err(),
        ProductShapeError::UnknownEntity(missing)
    );
}

#[test]
fn mesh_faces_are_displayed_with_one_normal_per_triangle() {
    let mut store = ProductShapeStore::new();
    let added = store.add_face(ProductFaceGeometry::Mesh(pyramid(8.0)));
    let entity = &store.entities()[0];
    assert_eq!(entity.face_ids(), vec![added.face_id]);

    let mesh = entity.face_display_mesh(added.face_id, 0.1).unwrap();
    // 三角形ごとに頂点を分け、3 つの頂点は三角形の法線を持つ
    assert_eq!(mesh.triangle_count(), 4);
    assert_eq!(mesh.vertex_count(), 12);
    let normals = mesh.normals().unwrap();
    for &[a, b, c] in mesh.indices() {
        let vertices = mesh.vertices();
        let face = Vector3D::from_points(vertices[a], vertices[b])
            .cross(Vector3D::from_points(vertices[a], vertices[c]))
            .normalize();
        for index in [a, b, c] {
            assert!((normals[index].dot(face) - 1.0).abs() < 1e-12);
        }
    }
}

#[test]
fn surface_faces_are_displayed_with_surface_normals() {
    let mut store = ProductShapeStore::new();
    let added = store.add_face(ProductFaceGeometry::Surface(dome()));
    let mesh = store.entities()[0]
        .face_display_mesh(added.face_id, 0.05)
        .unwrap();
    assert!(mesh.triangle_count() > 2);
    assert_eq!(mesh.normals().unwrap().len(), mesh.vertex_count());

    let unknown = FaceId::new_random();
    assert_eq!(
        store.entities()[0]
            .face_display_mesh(unknown, 0.05)
            .unwrap_err(),
        ProductShapeError::UnknownFace(unknown)
    );
}

/// 表示の属性・属性・メタデータを変えた面を、記録にして取り出す
fn record_with_attributes() -> ProductFaceRecord {
    let mut store = ProductShapeStore::new();
    store.add_face(ProductFaceGeometry::Mesh(pyramid(8.0)));
    let mut record = store.records().remove(0);
    record.display.color = [0.2, 0.4, 0.6, 1.0];
    record.display.layer = "補助".to_string();
    record.display.visible = false;
    record
        .attributes
        .set(user_code(), geo_entity::AttributeValue::Float(180.0))
        .unwrap();
    record.metadata.name = Some("面 1".to_string());
    record.metadata.tags.push("見切り".to_string());
    record
        .metadata
        .custom
        .insert("取り込み元".to_string(), "a.stl".to_string());
    record
}

fn user_code() -> geo_entity::AttributeCode {
    geo_entity::AttributeCode::try_new(10_01_02_02).unwrap()
}

#[test]
fn records_are_restored_with_the_same_ids_and_contents() {
    let mut store = ProductShapeStore::new();
    store.add_face(ProductFaceGeometry::Surface(dome()));
    let mut records = store.records();
    records.push(record_with_attributes());

    let restored = ProductShapeStore::from_records(records.clone(), Vec::new()).unwrap();
    let again = restored.records();
    assert_eq!(again.len(), 2);
    for (before, after) in records.iter().zip(&again) {
        assert_eq!(after.entity_id, before.entity_id);
        assert_eq!(after.face_id, before.face_id);
        assert_eq!(after.display, before.display);
        assert_eq!(after.attributes.len(), before.attributes.len());
        assert_eq!(
            after.attributes.get(user_code()),
            before.attributes.get(user_code())
        );
        let (a, b) = (&after.metadata, &before.metadata);
        assert_eq!(
            (&a.name, &a.description, &a.tags, &a.custom),
            (&b.name, &b.description, &b.tags, &b.custom)
        );
        assert_eq!((a.created_at, a.modified_at), (b.created_at, b.modified_at));
    }
    assert_eq!(
        again[1].attributes.get(user_code()),
        Some(&geo_entity::AttributeValue::Float(180.0))
    );
    assert!(matches!(again[0].geometry, ProductFaceGeometry::Surface(_)));
    let ProductFaceGeometry::Mesh(mesh) = &again[1].geometry else {
        panic!("the second face is a mesh");
    };
    assert_eq!(mesh.triangle_count(), 4);
    // 見えない面は、表示中の面に入らない
    assert_eq!(restored.visible_face_ids(), vec![records[0].face_id]);
}

#[test]
fn records_with_duplicate_ids_are_not_restored() {
    let record = record_with_attributes();
    let mut same_entity = record.clone();
    same_entity.face_id = FaceId::new_random();
    assert_eq!(
        ProductShapeStore::from_records(vec![record.clone(), same_entity], Vec::new()).unwrap_err(),
        ProductShapeRestoreError::DuplicateEntity(record.entity_id)
    );

    let mut same_face = record.clone();
    same_face.entity_id = EntityId::new_random();
    assert_eq!(
        ProductShapeStore::from_records(vec![record.clone(), same_face], Vec::new()).unwrap_err(),
        ProductShapeRestoreError::DuplicateFace(record.face_id)
    );
}

/// 四面体の B-rep の元の要素（ファイルの名前・出現・面の番号）
fn tetrahedron_source() -> ShellSource {
    ShellSource {
        file_name: "part.stp".to_string(),
        occurrences: vec![12, 34],
        faces: vec![101, 102, 103, 104],
    }
}

#[test]
fn a_shell_gets_a_face_id_for_each_face_of_its_brep() {
    let mut store = ProductShapeStore::new();
    let brep = Arc::new(test_fixtures::tetrahedron_brep());
    let placement = SimilarityTransform3D::translation(Vector3D::new(10.0, 0.0, 0.0));
    let first = store
        .add_shell(Arc::clone(&brep), placement, Some(tetrahedron_source()))
        .unwrap();
    // 同じ部品の 2 つ目の出現は、同じ表を共有し、別の面の ID を持つ
    let second = store
        .add_shell(Arc::clone(&brep), SimilarityTransform3D::identity(), None)
        .unwrap();
    assert_ne!(first, second);

    let shells = store.shell_records();
    assert_eq!(shells.len(), 2);
    assert_eq!((shells[0].entity_id, shells[1].entity_id), (first, second));
    assert!(Arc::ptr_eq(&shells[0].brep, &shells[1].brep));
    assert_eq!(shells[0].face_ids.len(), 4);
    let ids: HashSet<FaceId> = shells
        .iter()
        .flat_map(|shell| shell.face_ids.iter().copied())
        .collect();
    assert_eq!(ids.len(), 8);
    assert_eq!(shells[0].source, Some(tetrahedron_source()));
    assert_eq!(shells[0].placement.to_matrix(), placement.to_matrix());
    // シェルの面は、CAM の入力にできるまで、表示中の面に含めない
    assert!(store.visible_face_ids().is_empty());
    store.set_visible(first, false).unwrap();
    assert!(!store.shell_records()[0].display.visible);
}

#[test]
fn a_source_with_another_number_of_faces_is_refused() {
    let mut store = ProductShapeStore::new();
    let mut source = tetrahedron_source();
    source.faces.pop();
    assert_eq!(
        store
            .add_shell(
                Arc::new(test_fixtures::tetrahedron_brep()),
                SimilarityTransform3D::identity(),
                Some(source),
            )
            .unwrap_err(),
        ProductShapeError::SourceFaceCountMismatch
    );
}

#[test]
fn shell_records_are_checked_when_restored() {
    let mut store = ProductShapeStore::new();
    let added = store.add_face(ProductFaceGeometry::Surface(dome()));
    store
        .add_shell(
            Arc::new(test_fixtures::tetrahedron_brep()),
            SimilarityTransform3D::identity(),
            Some(tetrahedron_source()),
        )
        .unwrap();
    let faces = store.records();
    let shells = store.shell_records();

    let restored = ProductShapeStore::from_records(faces.clone(), shells.clone()).unwrap();
    assert_eq!(restored.shell_records()[0].face_ids, shells[0].face_ids);

    // 面単体とシェルをまたいで、面の ID が重なる
    let mut same_face = shells.clone();
    same_face[0].face_ids[2] = added.face_id;
    assert_eq!(
        ProductShapeStore::from_records(faces.clone(), same_face).unwrap_err(),
        ProductShapeRestoreError::DuplicateFace(added.face_id)
    );
    // 面単体とシェルをまたいで、エンティティの ID が重なる
    let mut same_entity = shells.clone();
    same_entity[0].entity_id = added.entity_id;
    assert_eq!(
        ProductShapeStore::from_records(faces.clone(), same_entity).unwrap_err(),
        ProductShapeRestoreError::DuplicateEntity(added.entity_id)
    );
    // 面の ID の数が、B-rep の面の数と合わない
    let mut fewer = shells;
    fewer[0].face_ids.pop();
    assert_eq!(
        ProductShapeStore::from_records(faces, fewer.clone()).unwrap_err(),
        ProductShapeRestoreError::FaceCountMismatch(fewer[0].entity_id)
    );
}

#[test]
fn shell_faces_are_displayed_with_the_placement_and_occurrences_share_the_triangles() {
    let mut store = ProductShapeStore::new();
    let brep = Arc::new(test_fixtures::cube_brep());
    let offset = Vector3D::new(100.0, 0.0, 0.0);
    store
        .add_shell(Arc::clone(&brep), SimilarityTransform3D::identity(), None)
        .unwrap();
    store
        .add_shell(brep, SimilarityTransform3D::translation(offset), None)
        .unwrap();
    let [first, second] = store.shells() else {
        panic!("two shells are held");
    };
    assert!(Arc::ptr_eq(&first.shell().meshes, &second.shell().meshes));
    // 記録から戻しても、同じ部品の出現は置き場を共有する
    let restored = ProductShapeStore::from_records(Vec::new(), store.shell_records()).unwrap();
    let [restored_first, restored_second] = restored.shells() else {
        panic!("two shells are restored");
    };
    assert!(Arc::ptr_eq(
        &restored_first.shell().meshes,
        &restored_second.shell().meshes
    ));

    let face_ids = second.face_ids();
    assert_eq!(face_ids.len(), 6);
    for (&placed_face, &face) in face_ids.iter().zip(&first.face_ids()) {
        let placed = second.face_display_mesh(placed_face, 0.05).unwrap();
        let original = first.face_display_mesh(face, 0.05).unwrap();
        assert_eq!(placed.triangle_count(), original.triangle_count());
        for (placed, original) in placed.vertices().iter().zip(original.vertices()) {
            assert!(placed.distance_to(*original + offset) < 1.0e-9);
        }
        let normals = placed.normals().unwrap();
        assert!(
            normals
                .iter()
                .all(|normal| (normal.length() - 1.0).abs() < 1.0e-9)
        );
    }
    // 2 つの出現で、三角形は 1 回だけ作る
    assert_eq!(first.shell().meshes.0.lock().unwrap().len(), 1);
    // 置き場は、許容弦誤差の違う組を 2 つまで残す
    for tolerance in [0.1, 0.2, 0.3] {
        first
            .face_display_mesh(first.face_ids()[0], tolerance)
            .unwrap();
    }
    assert_eq!(first.shell().meshes.0.lock().unwrap().len(), 2);
    let unknown = FaceId::new_random();
    assert_eq!(
        first.face_display_mesh(unknown, 0.05).unwrap_err(),
        ProductShapeError::UnknownFace(unknown)
    );
}

/// 表示を確かめるモデルファイル（立方体・円柱・球・トーラス・穴のある板のシェル）を書き出す
///
/// `REDRING_SHELL_SAMPLES` に書き出すファイルの場所を渡して、`cargo test -p application --lib write_shell_samples
/// -- --ignored` で実行する。
#[test]
#[ignore = "writes a model file to the path given by REDRING_SHELL_SAMPLES"]
fn write_shell_samples() {
    let Ok(path) = std::env::var("REDRING_SHELL_SAMPLES") else {
        return;
    };
    let mut store = ProductShapeStore::new();
    for (brep, offset) in [
        (test_fixtures::cube_brep(), [0.0, 0.0, 0.0]),
        (test_fixtures::cylinder_brep(), [40.0, 5.0, 0.0]),
        (test_fixtures::sphere_brep(), [80.0, 5.0, 10.0]),
        (test_fixtures::torus_brep(), [140.0, 5.0, 5.0]),
        (test_fixtures::plate_with_hole_brep(), [0.0, 50.0, 0.0]),
    ] {
        let [x, y, z] = offset;
        store
            .add_shell(
                Arc::new(brep),
                SimilarityTransform3D::translation(Vector3D::new(x, y, z)),
                None,
            )
            .unwrap();
    }
    let model = crate::space_model_file::SpaceModel::new(
        store,
        crate::machining_sequence::MachiningSequenceStore::new(),
    );
    crate::space_model_file::save_space_model(
        std::path::Path::new(&path),
        &model,
        &crate::space_model_file::SpaceModelSaveInfo {
            program_version: "shell samples".to_string(),
            saved_at_ms: 0,
        },
    )
    .unwrap();
}
