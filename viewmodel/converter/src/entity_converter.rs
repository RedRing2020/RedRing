use cam_entity::CAMEntity;
use geo_algorithms::Vector3D;
use geo_contracts::{ArcEntity3DProperties, CircleEntity3DProperties, LineEntity3DProperties};

use crate::mesh_converter::VertexData;
use crate::shape_converter::TessellationQuality;
use crate::toolpath_converter::{
    toolpath_to_vertices, ToolPathVertices, ToolPathVisualizationSettings,
};

#[derive(Debug, thiserror::Error)]
pub enum EntityConvertError {
    #[error("unsupported geometric entity type")]
    UnsupportedGeometricType,
}

pub fn line_entity_to_vertices<E>(entity: &E) -> Vec<VertexData>
where
    E: LineEntity3DProperties<f64>,
{
    let start = entity.line_start();
    let end = entity.line_end();
    let normal = [0.0f32, 0.0, 0.0];

    vec![
        VertexData::new([start.0 as f32, start.1 as f32, start.2 as f32], normal),
        VertexData::new([end.0 as f32, end.1 as f32, end.2 as f32], normal),
    ]
}

pub fn circle_entity_to_vertices<E>(entity: &E, quality: &TessellationQuality) -> Vec<VertexData>
where
    E: CircleEntity3DProperties<f64>,
{
    let segments = quality.circle_segments;
    let center = entity.circle_center();
    let radius = entity.circle_radius();
    let axis = entity.circle_axis();
    let normal_vec = Vector3D::new(axis.0, axis.1, axis.2).normalize();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    let arbitrary = if normal_vec.x().abs() > 0.9 {
        Vector3D::new(0.0, 1.0, 0.0)
    } else {
        Vector3D::new(1.0, 0.0, 0.0)
    };
    let u = normal_vec.cross(&arbitrary).normalize();
    let v = normal_vec.cross(&u).normalize();

    let mut vertices = Vec::with_capacity(segments * 2);
    for index in 0..segments {
        let angle1 = std::f64::consts::TAU * (index as f64) / (segments as f64);
        let angle2 = std::f64::consts::TAU * ((index + 1) as f64) / (segments as f64);

        let point1 = [
            (center.0 + radius * (angle1.cos() * u.x() + angle1.sin() * v.x())) as f32,
            (center.1 + radius * (angle1.cos() * u.y() + angle1.sin() * v.y())) as f32,
            (center.2 + radius * (angle1.cos() * u.z() + angle1.sin() * v.z())) as f32,
        ];
        let point2 = [
            (center.0 + radius * (angle2.cos() * u.x() + angle2.sin() * v.x())) as f32,
            (center.1 + radius * (angle2.cos() * u.y() + angle2.sin() * v.y())) as f32,
            (center.2 + radius * (angle2.cos() * u.z() + angle2.sin() * v.z())) as f32,
        ];

        vertices.push(VertexData::new(point1, normal));
        vertices.push(VertexData::new(point2, normal));
    }

    vertices
}

pub fn arc_entity_to_vertices<E>(entity: &E, quality: &TessellationQuality) -> Vec<VertexData>
where
    E: ArcEntity3DProperties<f64>,
{
    let start_angle = entity.arc_start_angle();
    let end_angle = entity.arc_end_angle();
    let mut angle_range = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += std::f64::consts::TAU;
    }

    let segments = ((angle_range / std::f64::consts::TAU) * (quality.circle_segments as f64))
        .ceil()
        .max(quality.min_segments as f64) as usize;
    let center = entity.arc_center();
    let radius = entity.arc_radius();
    let normal_dir = entity.arc_normal();
    let start_dir = entity.arc_start_direction();
    let normal_vec = Vector3D::new(normal_dir.0, normal_dir.1, normal_dir.2).normalize();
    let start_dir_vec = Vector3D::new(start_dir.0, start_dir.1, start_dir.2).normalize();
    let v = normal_vec.cross(&start_dir_vec).normalize();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    let mut vertices = Vec::with_capacity(segments * 2);
    for index in 0..segments {
        let t1 = (index as f64) / (segments as f64);
        let t2 = ((index + 1) as f64) / (segments as f64);
        let angle1 = start_angle + t1 * angle_range;
        let angle2 = start_angle + t2 * angle_range;

        let point1 = [
            (center.0 + radius * (angle1.cos() * start_dir_vec.x() + angle1.sin() * v.x())) as f32,
            (center.1 + radius * (angle1.cos() * start_dir_vec.y() + angle1.sin() * v.y())) as f32,
            (center.2 + radius * (angle1.cos() * start_dir_vec.z() + angle1.sin() * v.z())) as f32,
        ];
        let point2 = [
            (center.0 + radius * (angle2.cos() * start_dir_vec.x() + angle2.sin() * v.x())) as f32,
            (center.1 + radius * (angle2.cos() * start_dir_vec.y() + angle2.sin() * v.y())) as f32,
            (center.2 + radius * (angle2.cos() * start_dir_vec.z() + angle2.sin() * v.z())) as f32,
        ];

        vertices.push(VertexData::new(point1, normal));
        vertices.push(VertexData::new(point2, normal));
    }

    vertices
}

pub fn geometric_entity_to_vertices<E>(
    entity: &E,
    _quality: &TessellationQuality,
) -> Result<Vec<VertexData>, EntityConvertError>
where
    E: LineEntity3DProperties<f64>,
{
    Ok(line_entity_to_vertices(entity))
}

pub fn cam_entity_to_vertices(
    entity: &CAMEntity,
    settings: &ToolPathVisualizationSettings,
) -> ToolPathVertices {
    toolpath_to_vertices(entity.toolpath(), settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape_converter::TessellationQuality;
    use geo_contracts::{
        ArcEntity3DProperties, CircleEntity3DProperties, EntityDisplayProperties, EntityIdentity,
        LineEntity3DProperties,
    };

    #[derive(Clone, Copy)]
    struct MockLineEntity;

    impl EntityIdentity for MockLineEntity {
        type Id = u64;

        fn entity_id(&self) -> Self::Id {
            1
        }
    }

    impl EntityDisplayProperties for MockLineEntity {
        fn entity_visible(&self) -> bool {
            true
        }

        fn entity_color(&self) -> [f32; 4] {
            [1.0, 1.0, 1.0, 1.0]
        }
    }

    impl LineEntity3DProperties<f64> for MockLineEntity {
        fn line_start(&self) -> (f64, f64, f64) {
            (0.0, 0.0, 0.0)
        }

        fn line_end(&self) -> (f64, f64, f64) {
            (1.0, 0.0, 0.0)
        }
    }

    #[derive(Clone, Copy)]
    struct MockCircleEntity;

    impl EntityIdentity for MockCircleEntity {
        type Id = u64;

        fn entity_id(&self) -> Self::Id {
            2
        }
    }

    impl EntityDisplayProperties for MockCircleEntity {
        fn entity_visible(&self) -> bool {
            true
        }

        fn entity_color(&self) -> [f32; 4] {
            [1.0, 1.0, 1.0, 1.0]
        }
    }

    impl CircleEntity3DProperties<f64> for MockCircleEntity {
        fn circle_center(&self) -> (f64, f64, f64) {
            (0.0, 0.0, 0.0)
        }

        fn circle_radius(&self) -> f64 {
            1.0
        }

        fn circle_axis(&self) -> (f64, f64, f64) {
            (0.0, 0.0, 1.0)
        }
    }

    #[derive(Clone, Copy)]
    struct MockArcEntity;

    impl EntityIdentity for MockArcEntity {
        type Id = u64;

        fn entity_id(&self) -> Self::Id {
            3
        }
    }

    impl EntityDisplayProperties for MockArcEntity {
        fn entity_visible(&self) -> bool {
            true
        }

        fn entity_color(&self) -> [f32; 4] {
            [1.0, 1.0, 1.0, 1.0]
        }
    }

    impl ArcEntity3DProperties<f64> for MockArcEntity {
        fn arc_center(&self) -> (f64, f64, f64) {
            (0.0, 0.0, 0.0)
        }

        fn arc_radius(&self) -> f64 {
            1.0
        }

        fn arc_start_angle(&self) -> f64 {
            0.0
        }

        fn arc_end_angle(&self) -> f64 {
            std::f64::consts::FRAC_PI_2
        }

        fn arc_normal(&self) -> (f64, f64, f64) {
            (0.0, 0.0, 1.0)
        }

        fn arc_start_direction(&self) -> (f64, f64, f64) {
            (1.0, 0.0, 0.0)
        }
    }

    #[test]
    fn line_entity_conversion_returns_two_vertices() {
        let entity = MockLineEntity;
        let vertices = line_entity_to_vertices(&entity);
        assert_eq!(vertices.len(), 2);
    }

    #[test]
    fn generic_geometric_conversion_returns_vertices_for_line_entity_trait() {
        let entity = MockLineEntity;

        let result = geometric_entity_to_vertices(&entity, &TessellationQuality::default());
        assert!(result.is_ok());
        assert_eq!(result.expect("conversion should succeed").len(), 2);
    }

    #[test]
    fn circle_entity_conversion_returns_line_segments() {
        let entity = MockCircleEntity;
        let vertices = circle_entity_to_vertices(&entity, &TessellationQuality::default());
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 2, 0);
    }

    #[test]
    fn arc_entity_conversion_returns_line_segments() {
        let entity = MockArcEntity;
        let vertices = arc_entity_to_vertices(&entity, &TessellationQuality::default());
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 2, 0);
    }
}
