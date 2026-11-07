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

    let restored = ProductShapeStore::from_records(records.clone()).unwrap();
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
        ProductShapeStore::from_records(vec![record.clone(), same_entity]).unwrap_err(),
        ProductShapeRestoreError::DuplicateEntity(record.entity_id)
    );

    let mut same_face = record.clone();
    same_face.entity_id = EntityId::new_random();
    assert_eq!(
        ProductShapeStore::from_records(vec![record.clone(), same_face]).unwrap_err(),
        ProductShapeRestoreError::DuplicateFace(record.face_id)
    );
}
