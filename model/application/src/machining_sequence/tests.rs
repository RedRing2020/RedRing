//! `MachiningSequenceStore` のテスト

use cam_core::StockMargins;
use cam_entity::MachiningStock;
use geo_algorithms::{Point3D, TriangleMesh3D};
use geo_contracts::TriangleMesh3DConstructor;

use super::*;
use crate::product_shape::ProductFaceGeometry;

fn triangle() -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(0.0, 10.0, 0.0),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap()
}

fn sequence(target_faces: Vec<FaceId>) -> MachiningSequence {
    MachiningSequence::new(
        "手順".to_string(),
        target_faces,
        MachiningStock {
            margins: StockMargins::default(),
            precision: 0.05,
        },
    )
}

#[test]
fn a_sequence_is_found_and_changed_by_its_id() {
    let mut store = MachiningSequenceStore::new();
    let first = store.add(sequence(Vec::new()));
    let second = store.add(sequence(Vec::new()));
    let face = FaceId::new_random();

    store
        .sequence_mut(second)
        .expect("the sequence is held")
        .set_target_faces(vec![face]);

    assert!(store.sequence(first).unwrap().target_faces().is_empty());
    assert_eq!(store.sequence(second).unwrap().target_faces(), [face]);
}

#[test]
fn target_faces_held_in_the_product_shape_are_accepted() {
    let mut shapes = ProductShapeStore::new();
    let face = shapes
        .add_face(ProductFaceGeometry::Mesh(triangle()))
        .face_id;
    let mut store = MachiningSequenceStore::new();
    let id = store.add(sequence(vec![face]));

    assert_eq!(store.check_target_faces(id, &shapes), Ok(()));
}

#[test]
fn target_faces_missing_from_the_product_shape_are_reported() {
    let mut shapes = ProductShapeStore::new();
    let held = shapes
        .add_face(ProductFaceGeometry::Mesh(triangle()))
        .face_id;
    let missing = [FaceId::new_random(), FaceId::new_random()];
    let mut store = MachiningSequenceStore::new();
    let id = store.add(sequence(vec![missing[0], held, missing[1]]));

    assert_eq!(
        store.check_target_faces(id, &shapes),
        Err(MachiningSequenceError::MissingTargetFaces(missing.to_vec()))
    );
}

#[test]
fn an_unknown_sequence_is_reported() {
    let id = MachiningSequenceStore::new().add(sequence(Vec::new()));

    assert_eq!(
        MachiningSequenceStore::new().check_target_faces(id, &ProductShapeStore::new()),
        Err(MachiningSequenceError::UnknownSequence(id))
    );
}
