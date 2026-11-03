use geo_entity::{EntityId, LayerId, RelationStore};

#[test]
fn layer_membership_rejects_second_distinct_layer() {
    let mut store = RelationStore::new();
    let entity = EntityId::from_seed("entity-a");
    let layer1 = LayerId::new();
    let layer2 = LayerId::new();

    store
        .add_to_layer(entity, layer1)
        .expect("first layer should be accepted");

    let result = store.add_to_layer(entity, layer2);
    assert!(result.is_err());
}

#[test]
fn layer_membership_returns_single_assigned_layer() {
    let mut store = RelationStore::new();
    let entity = EntityId::from_seed("entity-b");
    let layer1 = LayerId::new();

    store
        .add_to_layer(entity, layer1)
        .expect("layer should be accepted");

    assert_eq!(store.layer_for_entity(entity), Some(layer1));
}

#[test]
fn layer_membership_allows_idempotent_reassignment_to_same_layer() {
    let mut store = RelationStore::new();
    let entity = EntityId::from_seed("entity-c");
    let layer = LayerId::new();

    store
        .add_to_layer(entity, layer)
        .expect("first assignment should be accepted");
    store
        .add_to_layer(entity, layer)
        .expect("same layer assignment should be idempotent");

    assert_eq!(store.layer_for_entity(entity), Some(layer));
}

#[test]
fn remove_from_layer_clears_assigned_layer() {
    let mut store = RelationStore::new();
    let entity = EntityId::from_seed("entity-d");
    let layer = LayerId::new();

    store
        .add_to_layer(entity, layer)
        .expect("layer should be accepted");
    store.remove_from_layer(entity, layer);

    assert_eq!(store.layer_for_entity(entity), None);
}
