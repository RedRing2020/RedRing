use crate::{EntityError, EntityId, EntityResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(Uuid);

impl GroupId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GroupId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(Uuid);

impl LayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerMembershipPolicy {
    SingleLayer,
    MultiLayer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupEntity {
    pub id: GroupId,
    pub name: String,
}

impl GroupEntity {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: GroupId::new(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerEntity {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
}

impl LayerEntity {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityGroupMembership {
    pub entity_id: EntityId,
    pub group_id: GroupId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityLayerMembership {
    pub entity_id: EntityId,
    pub layer_id: LayerId,
}

#[derive(Debug, Default)]
pub struct RelationStore {
    group_memberships: HashSet<EntityGroupMembership>,
    layer_memberships: HashSet<EntityLayerMembership>,
    layer_policies: HashMap<EntityId, LayerMembershipPolicy>,
}

impl RelationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_layer_policy(&mut self, entity_id: EntityId, policy: LayerMembershipPolicy) {
        self.layer_policies.insert(entity_id, policy);
    }

    pub fn layer_policy_for(&self, entity_id: EntityId) -> LayerMembershipPolicy {
        self.layer_policies
            .get(&entity_id)
            .copied()
            .unwrap_or(LayerMembershipPolicy::MultiLayer)
    }

    pub fn add_to_group(&mut self, entity_id: EntityId, group_id: GroupId) {
        self.group_memberships.insert(EntityGroupMembership {
            entity_id,
            group_id,
        });
    }

    pub fn remove_from_group(&mut self, entity_id: EntityId, group_id: GroupId) {
        self.group_memberships.remove(&EntityGroupMembership {
            entity_id,
            group_id,
        });
    }

    pub fn groups_for_entity(&self, entity_id: EntityId) -> Vec<GroupId> {
        self.group_memberships
            .iter()
            .filter(|membership| membership.entity_id == entity_id)
            .map(|membership| membership.group_id)
            .collect()
    }

    pub fn add_to_layer(&mut self, entity_id: EntityId, layer_id: LayerId) -> EntityResult<()> {
        let policy = self.layer_policy_for(entity_id);
        let existing_layers = self.layers_for_entity(entity_id);

        if policy == LayerMembershipPolicy::SingleLayer
            && !existing_layers.is_empty()
            && !existing_layers.contains(&layer_id)
        {
            return Err(EntityError::SingleLayerPolicyViolation {
                entity_id: entity_id.to_string(),
                current_layers: existing_layers.len(),
            });
        }

        self.layer_memberships.insert(EntityLayerMembership {
            entity_id,
            layer_id,
        });
        Ok(())
    }

    pub fn remove_from_layer(&mut self, entity_id: EntityId, layer_id: LayerId) {
        self.layer_memberships.remove(&EntityLayerMembership {
            entity_id,
            layer_id,
        });
    }

    pub fn layers_for_entity(&self, entity_id: EntityId) -> Vec<LayerId> {
        self.layer_memberships
            .iter()
            .filter(|membership| membership.entity_id == entity_id)
            .map(|membership| membership.layer_id)
            .collect()
    }
}
