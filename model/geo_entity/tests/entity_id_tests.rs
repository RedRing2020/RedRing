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
