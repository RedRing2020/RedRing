use geo_entity::EntityId;

#[test]
fn deterministic_id_is_stable() {
    let id1 = EntityId::from_feature_output("feature_shell", 0, "edge_a");
    let id2 = EntityId::from_feature_output("feature_shell", 0, "edge_a");
    assert_eq!(id1, id2);
}

#[test]
fn deterministic_id_changes_by_output_index() {
    let id1 = EntityId::from_feature_output("feature_shell", 0, "edge_a");
    let id2 = EntityId::from_feature_output("feature_shell", 1, "edge_a");
    assert_ne!(id1, id2);
}

#[test]
fn ids_are_restored_from_their_uuids() {
    let entity = EntityId::new_random();
    assert_eq!(EntityId::from_uuid(entity.as_uuid()), entity);
    let face = geo_entity::FaceId::new_random();
    assert_eq!(geo_entity::FaceId::from_uuid(face.as_uuid()), face);
}
