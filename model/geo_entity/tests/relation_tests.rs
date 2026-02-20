use geo_entity::{EntityId, LayerId, LayerMembershipPolicy, RelationStore};

#[test]
fn single_layer_policy_rejects_second_layer() {
    let mut store = RelationStore::new();
    let entity = EntityId::from_seed("entity-a");
    let layer1 = LayerId::new();
    let layer2 = LayerId::new();

    store.set_layer_policy(entity, LayerMembershipPolicy::SingleLayer);
    store
        .add_to_layer(entity, layer1)
        .expect("first layer should be accepted");

    let result = store.add_to_layer(entity, layer2);
    assert!(result.is_err());
}

#[test]
fn multi_layer_policy_accepts_multiple_layers() {
    let mut store = RelationStore::new();
    let entity = EntityId::from_seed("entity-b");
    let layer1 = LayerId::new();
    let layer2 = LayerId::new();

    store.set_layer_policy(entity, LayerMembershipPolicy::MultiLayer);
    store
        .add_to_layer(entity, layer1)
        .expect("first layer should be accepted");
    store
        .add_to_layer(entity, layer2)
        .expect("second layer should be accepted");

    let layers = store.layers_for_entity(entity);
    assert_eq!(layers.len(), 2);
}
