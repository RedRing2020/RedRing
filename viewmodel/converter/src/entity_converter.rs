use cam_entity::CAMEntity;
use geo_foundation::LineEntity3DProperties;

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

    #[derive(Clone, Copy)]
    struct MockLineEntity;

    impl geo_foundation::EntityIdentity for MockLineEntity {
        type Id = u64;

        fn entity_id(&self) -> Self::Id {
            1
        }
    }

    impl geo_foundation::EntityDisplayProperties for MockLineEntity {
        fn entity_visible(&self) -> bool {
            true
        }

        fn entity_color(&self) -> [f32; 4] {
            [1.0, 1.0, 1.0, 1.0]
        }
    }

    impl geo_foundation::LineEntity3DProperties<f64> for MockLineEntity {
        fn line_start(&self) -> (f64, f64, f64) {
            (0.0, 0.0, 0.0)
        }

        fn line_end(&self) -> (f64, f64, f64) {
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
}
