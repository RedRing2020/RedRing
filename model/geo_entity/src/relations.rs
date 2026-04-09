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
    layer_memberships: HashMap<EntityId, LayerId>,
}

impl RelationStore {
    pub fn new() -> Self {
        Self::default()
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
        if let Some(current_layer_id) = self.layer_for_entity(entity_id)
            && current_layer_id != layer_id
        {
            return Err(EntityError::SingleLayerPolicyViolation {
                entity_id: entity_id.to_string(),
                current_layers: 1,
            });
        }

        self.layer_memberships.insert(entity_id, layer_id);
        Ok(())
    }

    pub fn remove_from_layer(&mut self, entity_id: EntityId, layer_id: LayerId) {
        if self.layer_for_entity(entity_id) == Some(layer_id) {
            self.layer_memberships.remove(&entity_id);
        }
    }

    pub fn layer_for_entity(&self, entity_id: EntityId) -> Option<LayerId> {
        self.layer_memberships.get(&entity_id).copied()
    }
}
