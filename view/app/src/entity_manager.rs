use std::collections::HashMap;

use render::vertex_3d::MeshVertex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityViewId(u64);

#[derive(Debug, Clone)]
pub struct GeometricEntityItem {
    id: EntityViewId,
    line_vertices: Vec<MeshVertex>,
    visible: bool,
    color: [f32; 4],
    selected: bool,
}

#[derive(Default)]
pub struct EntityManager {
    geometric: HashMap<EntityViewId, GeometricEntityItem>,
    selected: Option<EntityViewId>,
    next_id: u64,
    dirty: bool,
}

impl EntityManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_line_entity(&mut self, vertices: Vec<MeshVertex>) -> EntityViewId {
        self.next_id += 1;
        let id = EntityViewId(self.next_id);
        let entity = GeometricEntityItem {
            id,
            line_vertices: vertices,
            visible: true,
            color: [1.0, 1.0, 1.0, 1.0],
            selected: false,
        };
        self.geometric.insert(id, entity);
        self.dirty = true;
        id
    }

    pub fn remove(&mut self, id: EntityViewId) -> bool {
        let removed = self.geometric.remove(&id).is_some();
        if removed {
            if self.selected == Some(id) {
                self.selected = None;
            }
            self.dirty = true;
        }
        removed
    }

    pub fn select(&mut self, id: EntityViewId) -> bool {
        if !self.geometric.contains_key(&id) {
            return false;
        }

        if let Some(prev_id) = self.selected {
            if let Some(prev) = self.geometric.get_mut(&prev_id) {
                prev.selected = false;
            }
        }

        if let Some(current) = self.geometric.get_mut(&id) {
            current.selected = true;
        }

        self.selected = Some(id);
        self.dirty = true;
        true
    }

    pub fn clear_selection(&mut self) {
        if let Some(prev_id) = self.selected {
            if let Some(prev) = self.geometric.get_mut(&prev_id) {
                prev.selected = false;
            }
        }
        self.selected = None;
        self.dirty = true;
    }

    pub fn selected(&self) -> Option<EntityViewId> {
        self.selected
    }

    pub fn set_visible(&mut self, id: EntityViewId, visible: bool) -> bool {
        if let Some(entity) = self.geometric.get_mut(&id) {
            entity.visible = visible;
            self.dirty = true;
            return true;
        }
        false
    }

    pub fn set_color(&mut self, id: EntityViewId, color: [f32; 4]) -> bool {
        if let Some(entity) = self.geometric.get_mut(&id) {
            entity.color = color;
            self.dirty = true;
            return true;
        }
        false
    }

    pub fn line_vertices(&self) -> Vec<MeshVertex> {
        let mut result = Vec::new();
        for item in self.geometric.values() {
            if item.visible {
                result.extend(item.line_vertices.iter().copied());
            }
        }
        result
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_select_remove_line_entity() {
        let vertices = vec![
            MeshVertex::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            MeshVertex::new([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ];

        let mut manager = EntityManager::new();
        let id = manager.add_line_entity(vertices);

        assert!(manager.select(id));
        assert_eq!(manager.selected(), Some(id));
        assert!(manager.remove(id));
        assert_eq!(manager.selected(), None);
    }
}
