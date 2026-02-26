use cam_core::ToolPath;
use geo_entity::{Attributes, DisplayAttributes, EntityId, Metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Roughing,
    Finishing,
    Drilling,
    Contouring,
}

#[derive(Debug, Clone)]
pub struct MachiningAttributes {
    pub tool_number: u32,
    pub spindle_speed: f64,
    pub feed_rate: f64,
    pub depth_of_cut: f64,
    pub operation_type: OperationType,
}

#[derive(Debug, Clone)]
pub struct CAMEntity {
    id: EntityId,
    toolpath: ToolPath<f64>,
    machining: MachiningAttributes,
    display: DisplayAttributes,
    attributes: Attributes,
    metadata: Metadata,
    selected: bool,
}

impl CAMEntity {
    pub fn new(toolpath: ToolPath<f64>, machining: MachiningAttributes) -> Self {
        Self {
            id: EntityId::new_random(),
            toolpath,
            machining,
            display: DisplayAttributes::cam_cutting(),
            attributes: Attributes::new(),
            metadata: Metadata::default(),
            selected: false,
        }
    }

    pub fn with_operation_color(mut self) -> Self {
        self.display.color = match self.machining.operation_type {
            OperationType::Roughing => [0.8, 0.8, 0.8, 1.0],
            OperationType::Finishing => [1.0, 1.0, 1.0, 1.0],
            OperationType::Drilling => [1.0, 0.5, 0.0, 1.0],
            OperationType::Contouring => [0.2, 1.0, 0.2, 1.0],
        };
        self
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn toolpath(&self) -> &ToolPath<f64> {
        &self.toolpath
    }

    pub fn machining(&self) -> &MachiningAttributes {
        &self.machining
    }

    pub fn display(&self) -> &DisplayAttributes {
        &self.display
    }

    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut Attributes {
        self.metadata.touch();
        &mut self.attributes
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
        self.metadata.touch();
    }

    pub fn estimated_time_seconds(&self) -> f64 {
        (self.toolpath.total_length() / self.machining.feed_rate) * 60.0
    }
}
