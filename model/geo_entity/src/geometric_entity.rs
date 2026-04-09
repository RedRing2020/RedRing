use crate::{Attributes, DisplayAttributes, EntityId, Metadata};
use geo_contracts::{
    Arc3DProperties, ArcEntity3DProperties, Circle3DProperties, CircleEntity3DProperties,
    EntityDisplayProperties, EntityIdentity, LineEntity3DProperties, Scalar,
    StrokeDisplayProperties,
};

#[derive(Debug, Clone)]
pub struct GeometricEntity<T: Scalar, G> {
    id: EntityId,
    geometry: G,
    display: DisplayAttributes,
    attributes: Attributes,
    metadata: Metadata,
    selected: bool,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Scalar, G> GeometricEntity<T, G> {
    pub fn new(geometry: G) -> Self {
        Self {
            id: EntityId::new_random(),
            geometry,
            display: DisplayAttributes::default(),
            attributes: Attributes::new(),
            metadata: Metadata::default(),
            selected: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn from_feature_output(
        geometry: G,
        feature_id: &str,
        output_index: u32,
        local_key: &str,
    ) -> Self {
        Self {
            id: EntityId::from_feature_output(feature_id, output_index, local_key),
            geometry,
            display: DisplayAttributes::default(),
            attributes: Attributes::new(),
            metadata: Metadata::default(),
            selected: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn geometry(&self) -> &G {
        &self.geometry
    }

    pub fn geometry_mut(&mut self) -> &mut G {
        self.metadata.touch();
        &mut self.geometry
    }

    pub fn display(&self) -> &DisplayAttributes {
        &self.display
    }

    pub fn display_mut(&mut self) -> &mut DisplayAttributes {
        self.metadata.touch();
        &mut self.display
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
}

impl<T: Scalar, G> EntityIdentity for GeometricEntity<T, G> {
    type Id = EntityId;

    fn entity_id(&self) -> Self::Id {
        self.id
    }
}

impl<T: Scalar, G> EntityDisplayProperties for GeometricEntity<T, G> {
    fn entity_visible(&self) -> bool {
        self.display.visible
    }

    fn entity_color(&self) -> [f32; 4] {
        self.display.color
    }
}

impl<T: Scalar, G> StrokeDisplayProperties for GeometricEntity<T, G> {
    fn entity_stroke_pattern(&self) -> geo_contracts::StrokePattern {
        self.display.line_style
    }

    fn entity_stroke_width(&self) -> f32 {
        self.display.line_width
    }
}

impl<T: Scalar> LineEntity3DProperties<T> for GeometricEntity<T, geo_primitives::LineSegment3D<T>> {
    fn line_start(&self) -> (T, T, T) {
        let point = self.geometry.start();
        (point.x(), point.y(), point.z())
    }

    fn line_end(&self) -> (T, T, T) {
        let point = self.geometry.end();
        (point.x(), point.y(), point.z())
    }
}

impl<T: Scalar> CircleEntity3DProperties<T> for GeometricEntity<T, geo_primitives::Circle3D<T>> {
    fn circle_center(&self) -> (T, T, T) {
        Circle3DProperties::center(self.geometry())
    }

    fn circle_radius(&self) -> T {
        Circle3DProperties::radius(self.geometry())
    }

    fn circle_axis(&self) -> (T, T, T) {
        Circle3DProperties::axis(self.geometry())
    }
}

impl<T: Scalar> ArcEntity3DProperties<T> for GeometricEntity<T, geo_primitives::Arc3D<T>> {
    fn arc_center(&self) -> (T, T, T) {
        Arc3DProperties::center(self.geometry())
    }

    fn arc_radius(&self) -> T {
        Arc3DProperties::radius(self.geometry())
    }

    fn arc_start_angle(&self) -> T {
        Arc3DProperties::start_angle(self.geometry())
    }

    fn arc_end_angle(&self) -> T {
        Arc3DProperties::end_angle(self.geometry())
    }

    fn arc_normal(&self) -> (T, T, T) {
        let normal = self.geometry.normal().as_vector();
        (normal.x(), normal.y(), normal.z())
    }

    fn arc_start_direction(&self) -> (T, T, T) {
        let direction = self.geometry.start_direction().as_vector();
        (direction.x(), direction.y(), direction.z())
    }
}
