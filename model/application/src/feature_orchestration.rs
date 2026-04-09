//! Feature向け orchestration 境界。

use crate::geometry_orchestration::{
    CreateArcGeometryRequest, CreateCircleGeometryRequest, CreateLineGeometryRequest,
    DebugShapeGeometryMutationPort, GeometryOrchestrationError, GeometryOrchestrator,
    LineGeometryMutationPort,
};
use crate::primitives::{Arc3D, Circle3D, LineSegment3D};
use crate::topology_orchestration::{
    CreateArcTopologyRequest, CreateCircleTopologyRequest, CreateLineTopologyRequest,
    DebugShapeTopologyMutationPort, LineTopologyMutationPort, TopologyOrchestrationError,
    TopologyOrchestrator,
};
use geo_contracts::{
    Arc3DProperties, ArcEntity3DProperties, Circle3DProperties, CircleEntity3DProperties,
    EntityDisplayProperties, EntityIdentity, LineEntity3DProperties, StrokeDisplayProperties,
    StrokePattern,
};
use geo_entity::{EntityId, GeometricEntity};
use geo_topology::{Edge as TopologyEdge, TopoId};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

type StoredLineEntity = GeometricEntity<f64, LineSegment3D<f64>>;
type StoredLineEntityMap = HashMap<EntityId, StoredLineEntity>;
type SharedLineEntityStore = Mutex<StoredLineEntityMap>;
type StoredCurveTopology = TopologyEdge<f64>;
type StoredCurveTopologyMap = HashMap<TopoId, StoredCurveTopology>;
type SharedCurveTopologyStore = Mutex<StoredCurveTopologyMap>;
type StoredCircleEntity = GeometricEntity<f64, Circle3D<f64>>;
type StoredCircleEntityMap = HashMap<EntityId, StoredCircleEntity>;
type SharedCircleEntityStore = Mutex<StoredCircleEntityMap>;
type StoredArcEntity = GeometricEntity<f64, Arc3D<f64>>;
type StoredArcEntityMap = HashMap<EntityId, StoredArcEntity>;
type SharedArcEntityStore = Mutex<StoredArcEntityMap>;

static LINE_ENTITY_STORE: OnceLock<SharedLineEntityStore> = OnceLock::new();
static CURVE_TOPOLOGY_STORE: OnceLock<SharedCurveTopologyStore> = OnceLock::new();
static CIRCLE_ENTITY_STORE: OnceLock<SharedCircleEntityStore> = OnceLock::new();
static ARC_ENTITY_STORE: OnceLock<SharedArcEntityStore> = OnceLock::new();

fn line_entity_store() -> &'static SharedLineEntityStore {
    LINE_ENTITY_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn curve_topology_store() -> &'static SharedCurveTopologyStore {
    CURVE_TOPOLOGY_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn circle_entity_store() -> &'static SharedCircleEntityStore {
    CIRCLE_ENTITY_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn arc_entity_store() -> &'static SharedArcEntityStore {
    ARC_ENTITY_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 将来の feature orchestration ユースケース向けマーカー入力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureOrchestrationRequest {
    pub feature_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateLineFeatureRequest {
    pub feature_id: String,
    pub feature_name: String,
    pub output_index: u32,
    pub local_key: String,
    pub start: [f64; 3],
    pub end: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateCircleFeatureRequest {
    pub feature_id: String,
    pub feature_name: String,
    pub output_index: u32,
    pub local_key: String,
    pub center: [f64; 3],
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateArcFeatureRequest {
    pub feature_id: String,
    pub feature_name: String,
    pub output_index: u32,
    pub local_key: String,
    pub center: [f64; 3],
    pub radius: f64,
    pub start_angle_radians: f64,
    pub end_angle_radians: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredLineEntityDto {
    pub entity_id: EntityId,
    pub feature_name: String,
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub color: [f32; 4],
    pub line_style: StrokePattern,
    pub line_width: f32,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredCurveTopologyDto {
    pub edge_id: TopoId,
    pub start_vertex_id: TopoId,
    pub end_vertex_id: TopoId,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredCircleEntityDto {
    pub entity_id: EntityId,
    pub feature_name: String,
    pub center: [f64; 3],
    pub radius: f64,
    pub axis: [f64; 3],
    pub color: [f32; 4],
    pub line_style: StrokePattern,
    pub line_width: f32,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredArcEntityDto {
    pub entity_id: EntityId,
    pub feature_name: String,
    pub center: [f64; 3],
    pub radius: f64,
    pub start_angle_radians: f64,
    pub end_angle_radians: f64,
    pub normal: [f64; 3],
    pub start_direction: [f64; 3],
    pub color: [f32; 4],
    pub line_style: StrokePattern,
    pub line_width: f32,
    pub visible: bool,
}

impl EntityIdentity for StoredLineEntityDto {
    type Id = EntityId;

    fn entity_id(&self) -> Self::Id {
        self.entity_id
    }
}

impl EntityDisplayProperties for StoredLineEntityDto {
    fn entity_visible(&self) -> bool {
        self.visible
    }

    fn entity_color(&self) -> [f32; 4] {
        self.color
    }
}

impl LineEntity3DProperties<f64> for StoredLineEntityDto {
    fn line_start(&self) -> (f64, f64, f64) {
        (self.start[0], self.start[1], self.start[2])
    }

    fn line_end(&self) -> (f64, f64, f64) {
        (self.end[0], self.end[1], self.end[2])
    }
}

impl StrokeDisplayProperties for StoredLineEntityDto {
    fn entity_stroke_pattern(&self) -> StrokePattern {
        self.line_style
    }

    fn entity_stroke_width(&self) -> f32 {
        self.line_width
    }
}

impl EntityIdentity for StoredCircleEntityDto {
    type Id = EntityId;

    fn entity_id(&self) -> Self::Id {
        self.entity_id
    }
}

impl EntityDisplayProperties for StoredCircleEntityDto {
    fn entity_visible(&self) -> bool {
        self.visible
    }

    fn entity_color(&self) -> [f32; 4] {
        self.color
    }
}

impl CircleEntity3DProperties<f64> for StoredCircleEntityDto {
    fn circle_center(&self) -> (f64, f64, f64) {
        (self.center[0], self.center[1], self.center[2])
    }

    fn circle_radius(&self) -> f64 {
        self.radius
    }

    fn circle_axis(&self) -> (f64, f64, f64) {
        (self.axis[0], self.axis[1], self.axis[2])
    }
}

impl StrokeDisplayProperties for StoredCircleEntityDto {
    fn entity_stroke_pattern(&self) -> StrokePattern {
        self.line_style
    }

    fn entity_stroke_width(&self) -> f32 {
        self.line_width
    }
}

impl EntityIdentity for StoredArcEntityDto {
    type Id = EntityId;

    fn entity_id(&self) -> Self::Id {
        self.entity_id
    }
}

impl EntityDisplayProperties for StoredArcEntityDto {
    fn entity_visible(&self) -> bool {
        self.visible
    }

    fn entity_color(&self) -> [f32; 4] {
        self.color
    }
}

impl ArcEntity3DProperties<f64> for StoredArcEntityDto {
    fn arc_center(&self) -> (f64, f64, f64) {
        (self.center[0], self.center[1], self.center[2])
    }

    fn arc_radius(&self) -> f64 {
        self.radius
    }

    fn arc_start_angle(&self) -> f64 {
        self.start_angle_radians
    }

    fn arc_end_angle(&self) -> f64 {
        self.end_angle_radians
    }

    fn arc_normal(&self) -> (f64, f64, f64) {
        (self.normal[0], self.normal[1], self.normal[2])
    }

    fn arc_start_direction(&self) -> (f64, f64, f64) {
        (
            self.start_direction[0],
            self.start_direction[1],
            self.start_direction[2],
        )
    }
}

impl StrokeDisplayProperties for StoredArcEntityDto {
    fn entity_stroke_pattern(&self) -> StrokePattern {
        self.line_style
    }

    fn entity_stroke_width(&self) -> f32 {
        self.line_width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateLineFeatureResult {
    pub entity: StoredLineEntityDto,
    pub topology: StoredCurveTopologyDto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateCircleFeatureResult {
    pub entity: StoredCircleEntityDto,
    pub topology: StoredCurveTopologyDto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateArcFeatureResult {
    pub entity: StoredArcEntityDto,
    pub topology: StoredCurveTopologyDto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureOrchestrationError {
    InvalidLineGeometry,
    InvalidLineTopology,
    InvalidCircleGeometry,
    InvalidCircleTopology,
    InvalidArcGeometry,
    InvalidArcTopology,
    InvalidTriangleGeometry,
    InvalidTriangleTopology,
    EntityStorePoisoned,
}

impl std::fmt::Display for FeatureOrchestrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLineGeometry => write!(f, "invalid line geometry"),
            Self::InvalidLineTopology => write!(f, "invalid line topology"),
            Self::InvalidCircleGeometry => write!(f, "invalid circle geometry"),
            Self::InvalidCircleTopology => write!(f, "invalid circle topology"),
            Self::InvalidArcGeometry => write!(f, "invalid arc geometry"),
            Self::InvalidArcTopology => write!(f, "invalid arc topology"),
            Self::InvalidTriangleGeometry => write!(f, "invalid triangle geometry"),
            Self::InvalidTriangleTopology => write!(f, "invalid triangle topology"),
            Self::EntityStorePoisoned => write!(f, "entity store poisoned"),
        }
    }
}

impl std::error::Error for FeatureOrchestrationError {}

impl From<GeometryOrchestrationError> for FeatureOrchestrationError {
    fn from(value: GeometryOrchestrationError) -> Self {
        match value {
            GeometryOrchestrationError::InvalidLineEndpoints => Self::InvalidLineGeometry,
            GeometryOrchestrationError::InvalidCircleParameters => Self::InvalidCircleGeometry,
            GeometryOrchestrationError::InvalidArcParameters => Self::InvalidArcGeometry,
            GeometryOrchestrationError::InvalidTriangleVertices => Self::InvalidTriangleGeometry,
        }
    }
}

impl From<TopologyOrchestrationError> for FeatureOrchestrationError {
    fn from(value: TopologyOrchestrationError) -> Self {
        match value {
            TopologyOrchestrationError::InvalidLineTopology => Self::InvalidLineTopology,
            TopologyOrchestrationError::InvalidArcTopology => Self::InvalidArcTopology,
            TopologyOrchestrationError::InvalidCircleTopology => Self::InvalidCircleTopology,
            TopologyOrchestrationError::InvalidTriangleTopology => Self::InvalidTriangleTopology,
        }
    }
}

pub trait FeatureCommandOrchestration {
    fn create_line_feature(
        &self,
        request: CreateLineFeatureRequest,
    ) -> Result<CreateLineFeatureResult, FeatureOrchestrationError>;

    fn create_circle_feature(
        &self,
        request: CreateCircleFeatureRequest,
    ) -> Result<CreateCircleFeatureResult, FeatureOrchestrationError>;

    fn create_arc_feature(
        &self,
        request: CreateArcFeatureRequest,
    ) -> Result<CreateArcFeatureResult, FeatureOrchestrationError>;
}

pub trait LineEntityStorePort {
    fn insert_line_entity(
        &self,
        entity: StoredLineEntity,
        feature_name: String,
    ) -> Result<StoredLineEntityDto, FeatureOrchestrationError>;
    fn contains(&self, entity_id: EntityId) -> Result<bool, FeatureOrchestrationError>;
}

pub trait CurveTopologyStorePort {
    fn insert_curve_topology(
        &self,
        edge: StoredCurveTopology,
        valid: bool,
    ) -> Result<StoredCurveTopologyDto, FeatureOrchestrationError>;
    fn contains_topology(&self, edge_id: TopoId) -> Result<bool, FeatureOrchestrationError>;
}

pub trait CircleEntityStorePort {
    fn insert_circle_entity(
        &self,
        entity: StoredCircleEntity,
        feature_name: String,
    ) -> Result<StoredCircleEntityDto, FeatureOrchestrationError>;
    fn contains_circle(&self, entity_id: EntityId) -> Result<bool, FeatureOrchestrationError>;
}

pub trait ArcEntityStorePort {
    fn insert_arc_entity(
        &self,
        entity: StoredArcEntity,
        feature_name: String,
    ) -> Result<StoredArcEntityDto, FeatureOrchestrationError>;
    fn contains_arc(&self, entity_id: EntityId) -> Result<bool, FeatureOrchestrationError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InMemoryLineEntityStore;

#[derive(Debug, Default, Clone, Copy)]
pub struct InMemoryCurveTopologyStore;

#[derive(Debug, Default, Clone, Copy)]
pub struct InMemoryCircleEntityStore;

#[derive(Debug, Default, Clone, Copy)]
pub struct InMemoryArcEntityStore;

impl LineEntityStorePort for InMemoryLineEntityStore {
    fn insert_line_entity(
        &self,
        entity: StoredLineEntity,
        feature_name: String,
    ) -> Result<StoredLineEntityDto, FeatureOrchestrationError> {
        let dto = StoredLineEntityDto {
            entity_id: entity.id(),
            feature_name,
            start: [
                entity.geometry().start().x(),
                entity.geometry().start().y(),
                entity.geometry().start().z(),
            ],
            end: [
                entity.geometry().end().x(),
                entity.geometry().end().y(),
                entity.geometry().end().z(),
            ],
            color: entity.display().color,
            line_style: entity.display().line_style,
            line_width: entity.display().line_width,
            visible: entity.display().visible,
        };

        let mut store = line_entity_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        store.insert(entity.id(), entity);
        Ok(dto)
    }

    fn contains(&self, entity_id: EntityId) -> Result<bool, FeatureOrchestrationError> {
        let store = line_entity_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        Ok(store.contains_key(&entity_id))
    }
}

impl CurveTopologyStorePort for InMemoryCurveTopologyStore {
    fn insert_curve_topology(
        &self,
        edge: StoredCurveTopology,
        valid: bool,
    ) -> Result<StoredCurveTopologyDto, FeatureOrchestrationError> {
        let dto = StoredCurveTopologyDto {
            edge_id: edge.id(),
            start_vertex_id: edge.start_vertex().id(),
            end_vertex_id: edge.end_vertex().id(),
            valid,
        };

        let mut store = curve_topology_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        store.insert(edge.id(), edge);
        Ok(dto)
    }

    fn contains_topology(&self, edge_id: TopoId) -> Result<bool, FeatureOrchestrationError> {
        let store = curve_topology_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        Ok(store.contains_key(&edge_id))
    }
}

impl CircleEntityStorePort for InMemoryCircleEntityStore {
    fn insert_circle_entity(
        &self,
        entity: StoredCircleEntity,
        feature_name: String,
    ) -> Result<StoredCircleEntityDto, FeatureOrchestrationError> {
        let center = Circle3DProperties::center(entity.geometry());
        let axis = Circle3DProperties::axis(entity.geometry());
        let dto = StoredCircleEntityDto {
            entity_id: entity.id(),
            feature_name,
            center: [center.0, center.1, center.2],
            radius: Circle3DProperties::radius(entity.geometry()),
            axis: [axis.0, axis.1, axis.2],
            color: entity.display().color,
            line_style: entity.display().line_style,
            line_width: entity.display().line_width,
            visible: entity.display().visible,
        };

        let mut store = circle_entity_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        store.insert(entity.id(), entity);
        Ok(dto)
    }

    fn contains_circle(&self, entity_id: EntityId) -> Result<bool, FeatureOrchestrationError> {
        let store = circle_entity_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        Ok(store.contains_key(&entity_id))
    }
}

impl ArcEntityStorePort for InMemoryArcEntityStore {
    fn insert_arc_entity(
        &self,
        entity: StoredArcEntity,
        feature_name: String,
    ) -> Result<StoredArcEntityDto, FeatureOrchestrationError> {
        let center = Arc3DProperties::center(entity.geometry());
        let normal = entity.geometry().normal().as_vector();
        let start_direction = entity.geometry().start_direction().as_vector();
        let dto = StoredArcEntityDto {
            entity_id: entity.id(),
            feature_name,
            center: [center.0, center.1, center.2],
            radius: Arc3DProperties::radius(entity.geometry()),
            start_angle_radians: entity.geometry().start_angle().to_radians(),
            end_angle_radians: entity.geometry().end_angle().to_radians(),
            normal: [normal.x(), normal.y(), normal.z()],
            start_direction: [
                start_direction.x(),
                start_direction.y(),
                start_direction.z(),
            ],
            color: entity.display().color,
            line_style: entity.display().line_style,
            line_width: entity.display().line_width,
            visible: entity.display().visible,
        };

        let mut store = arc_entity_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        store.insert(entity.id(), entity);
        Ok(dto)
    }

    fn contains_arc(&self, entity_id: EntityId) -> Result<bool, FeatureOrchestrationError> {
        let store = arc_entity_store()
            .lock()
            .map_err(|_| FeatureOrchestrationError::EntityStorePoisoned)?;
        Ok(store.contains_key(&entity_id))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FeatureOrchestrator;

impl FeatureCommandOrchestration for FeatureOrchestrator {
    fn create_line_feature(
        &self,
        request: CreateLineFeatureRequest,
    ) -> Result<CreateLineFeatureResult, FeatureOrchestrationError> {
        let geometry = GeometryOrchestrator.create_line_geometry(CreateLineGeometryRequest {
            start: request.start,
            end: request.end,
        })?;

        let topology = TopologyOrchestrator.create_line_topology(CreateLineTopologyRequest {
            line: geometry.line,
        })?;

        let entity = GeometricEntity::from_feature_output(
            geometry.line,
            &request.feature_id,
            request.output_index,
            &request.local_key,
        );
        let topology_dto = InMemoryCurveTopologyStore
            .insert_curve_topology(topology.edge, topology.validation.is_valid())?;
        let dto = InMemoryLineEntityStore.insert_line_entity(entity, request.feature_name)?;

        Ok(CreateLineFeatureResult {
            entity: dto,
            topology: topology_dto,
        })
    }

    fn create_circle_feature(
        &self,
        request: CreateCircleFeatureRequest,
    ) -> Result<CreateCircleFeatureResult, FeatureOrchestrationError> {
        let geometry =
            GeometryOrchestrator.create_circle_geometry(CreateCircleGeometryRequest {
                center: request.center,
                radius: request.radius,
            })?;

        let topology =
            TopologyOrchestrator.create_circle_topology(CreateCircleTopologyRequest {
                circle: geometry.circle,
            })?;

        let entity = GeometricEntity::from_feature_output(
            geometry.circle,
            &request.feature_id,
            request.output_index,
            &request.local_key,
        );
        let topology_dto = InMemoryCurveTopologyStore
            .insert_curve_topology(topology.edge, topology.validation.is_valid())?;
        let dto = InMemoryCircleEntityStore.insert_circle_entity(entity, request.feature_name)?;

        Ok(CreateCircleFeatureResult {
            entity: dto,
            topology: topology_dto,
        })
    }

    fn create_arc_feature(
        &self,
        request: CreateArcFeatureRequest,
    ) -> Result<CreateArcFeatureResult, FeatureOrchestrationError> {
        let geometry = GeometryOrchestrator.create_arc_geometry(CreateArcGeometryRequest {
            center: request.center,
            radius: request.radius,
            start_angle_radians: request.start_angle_radians,
            end_angle_radians: request.end_angle_radians,
        })?;

        let topology = TopologyOrchestrator
            .create_arc_topology(CreateArcTopologyRequest { arc: geometry.arc })?;

        let entity = GeometricEntity::from_feature_output(
            geometry.arc,
            &request.feature_id,
            request.output_index,
            &request.local_key,
        );
        let topology_dto = InMemoryCurveTopologyStore
            .insert_curve_topology(topology.edge, topology.validation.is_valid())?;
        let dto = InMemoryArcEntityStore.insert_arc_entity(entity, request.feature_name)?;

        Ok(CreateArcFeatureResult {
            entity: dto,
            topology: topology_dto,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArcEntityStorePort, CircleEntityStorePort, CreateArcFeatureRequest,
        CreateCircleFeatureRequest, CreateLineFeatureRequest, CurveTopologyStorePort,
        FeatureCommandOrchestration, FeatureOrchestrator, InMemoryArcEntityStore,
        InMemoryCircleEntityStore, InMemoryCurveTopologyStore, InMemoryLineEntityStore,
        LineEntityStorePort,
    };

    #[test]
    fn create_line_feature_stores_entity_and_returns_line_dto() {
        let result = FeatureOrchestrator
            .create_line_feature(CreateLineFeatureRequest {
                feature_id: "debug_line".to_string(),
                feature_name: "Debug Line".to_string(),
                output_index: 0,
                local_key: "line_a".to_string(),
                start: [-50.0, 0.0, 0.0],
                end: [50.0, 0.0, 0.0],
            })
            .expect("line feature creation should succeed");

        assert_eq!(result.entity.feature_name, "Debug Line");
        assert_eq!(result.entity.start, [-50.0, 0.0, 0.0]);
        assert_eq!(result.entity.end, [50.0, 0.0, 0.0]);
        assert!(result.topology.valid);
        assert!(
            InMemoryLineEntityStore
                .contains(result.entity.entity_id)
                .expect("entity store should be readable")
        );
        assert!(
            InMemoryCurveTopologyStore
                .contains_topology(result.topology.edge_id)
                .expect("topology store should be readable")
        );
    }

    #[test]
    fn create_circle_feature_stores_entity_and_returns_circle_dto() {
        let result = FeatureOrchestrator
            .create_circle_feature(CreateCircleFeatureRequest {
                feature_id: "debug_circle".to_string(),
                feature_name: "Debug Circle".to_string(),
                output_index: 0,
                local_key: "circle_a".to_string(),
                center: [0.0, 0.0, 0.0],
                radius: 5.0,
            })
            .expect("circle feature creation should succeed");

        assert_eq!(result.entity.feature_name, "Debug Circle");
        assert_eq!(result.entity.center, [0.0, 0.0, 0.0]);
        assert_eq!(result.entity.radius, 5.0);
        assert!(result.topology.valid);
        assert!(
            InMemoryCircleEntityStore
                .contains_circle(result.entity.entity_id)
                .expect("circle entity store should be readable")
        );
        assert!(
            InMemoryCurveTopologyStore
                .contains_topology(result.topology.edge_id)
                .expect("topology store should be readable")
        );
    }

    #[test]
    fn create_arc_feature_stores_entity_and_returns_arc_dto() {
        let result = FeatureOrchestrator
            .create_arc_feature(CreateArcFeatureRequest {
                feature_id: "debug_arc".to_string(),
                feature_name: "Debug Arc".to_string(),
                output_index: 0,
                local_key: "arc_a".to_string(),
                center: [0.0, 0.0, 0.0],
                radius: 1.5,
                start_angle_radians: 0.0,
                end_angle_radians: std::f64::consts::FRAC_PI_2,
            })
            .expect("arc feature creation should succeed");

        assert_eq!(result.entity.feature_name, "Debug Arc");
        assert_eq!(result.entity.center, [0.0, 0.0, 0.0]);
        assert_eq!(result.entity.radius, 1.5);
        assert!(result.topology.valid);
        assert!(
            InMemoryArcEntityStore
                .contains_arc(result.entity.entity_id)
                .expect("arc entity store should be readable")
        );
        assert!(
            InMemoryCurveTopologyStore
                .contains_topology(result.topology.edge_id)
                .expect("topology store should be readable")
        );
    }
}
