use application::feature_orchestration::{
    CreateArcFeatureRequest, CreateCircleFeatureRequest, CreateLineFeatureRequest,
    FeatureCommandOrchestration, FeatureOrchestrator,
};
use application::geometry_orchestration::{
    CreateTriangleGeometryRequest, DebugShapeGeometryMutationPort, GeometryOrchestrator,
};
use application::topology_orchestration::{
    CreateTriangleTopologyRequest, DebugShapeTopologyMutationPort, TopologyOrchestrator,
};

use crate::entity_converter::{
    arc_entity_to_vertices, circle_entity_to_vertices, geometric_entity_to_vertices,
    EntityConvertError,
};
use crate::mesh_converter::VertexData;
use crate::shape_converter::{triangle_to_solid_vertices, TessellationQuality};

#[derive(Debug, thiserror::Error)]
pub enum FeatureCommandConvertError {
    #[error("feature orchestration failed: {0}")]
    Orchestration(String),
    #[error(transparent)]
    EntityConvert(#[from] EntityConvertError),
}

pub fn create_debug_line_vertices() -> Result<Vec<VertexData>, FeatureCommandConvertError> {
    let result = FeatureOrchestrator
        .create_line_feature(CreateLineFeatureRequest {
            feature_id: "debug_line".to_string(),
            feature_name: "Debug Line".to_string(),
            output_index: 0,
            local_key: "line_a".to_string(),
            start: [-50.0, 0.0, 0.0],
            end: [50.0, 0.0, 0.0],
        })
        .map_err(|error| FeatureCommandConvertError::Orchestration(error.to_string()))?;

    geometric_entity_to_vertices(&result.entity, &TessellationQuality::default())
        .map_err(FeatureCommandConvertError::from)
}

pub fn create_debug_circle_vertices() -> Result<Vec<VertexData>, FeatureCommandConvertError> {
    let result = FeatureOrchestrator
        .create_circle_feature(CreateCircleFeatureRequest {
            feature_id: "debug_circle".to_string(),
            feature_name: "Debug Circle".to_string(),
            output_index: 0,
            local_key: "circle_a".to_string(),
            center: [0.0, 0.0, 0.0],
            radius: 5.0,
        })
        .map_err(|error| FeatureCommandConvertError::Orchestration(error.to_string()))?;

    Ok(circle_entity_to_vertices(
        &result.entity,
        &TessellationQuality::default(),
    ))
}

pub fn create_debug_arc_vertices() -> Result<Vec<VertexData>, FeatureCommandConvertError> {
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
        .map_err(|error| FeatureCommandConvertError::Orchestration(error.to_string()))?;

    Ok(arc_entity_to_vertices(
        &result.entity,
        &TessellationQuality::default(),
    ))
}

pub fn create_debug_triangle_vertices() -> Result<Vec<VertexData>, FeatureCommandConvertError> {
    let result = GeometryOrchestrator
        .create_triangle_geometry(CreateTriangleGeometryRequest {
            vertex_a: [0.0, 0.0, 0.0],
            vertex_b: [1.0, 0.0, 0.0],
            vertex_c: [0.5, 1.0, 0.0],
        })
        .map_err(|error| FeatureCommandConvertError::Orchestration(error.to_string()))?;

    TopologyOrchestrator
        .create_triangle_topology(CreateTriangleTopologyRequest {
            triangle: result.triangle,
        })
        .map_err(|error| FeatureCommandConvertError::Orchestration(error.to_string()))?;

    Ok(triangle_to_solid_vertices(&result.triangle))
}

#[cfg(test)]
mod tests {
    use super::{
        create_debug_arc_vertices, create_debug_circle_vertices, create_debug_line_vertices,
        create_debug_triangle_vertices,
    };

    #[test]
    fn create_debug_line_vertices_returns_two_vertices() {
        let vertices = create_debug_line_vertices()
            .expect("debug line command should produce viewmodel vertices");

        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].position, [-50.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [50.0, 0.0, 0.0]);
    }

    #[test]
    fn create_debug_circle_vertices_returns_line_segments() {
        let vertices = create_debug_circle_vertices()
            .expect("debug circle command should produce viewmodel vertices");

        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 2, 0);
    }

    #[test]
    fn create_debug_arc_vertices_returns_line_segments() {
        let vertices = create_debug_arc_vertices()
            .expect("debug arc command should produce viewmodel vertices");

        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 2, 0);
    }

    #[test]
    fn create_debug_triangle_vertices_returns_triangle_vertices() {
        let vertices = create_debug_triangle_vertices()
            .expect("debug triangle command should produce viewmodel vertices");

        assert_eq!(vertices.len(), 3);
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 0.0, 0.0]);
        assert_eq!(vertices[2].position, [0.5, 1.0, 0.0]);
    }
}
