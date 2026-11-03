//! mesh_convert - メッシュの頂点データから MeshVertex への変換

use crate::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};

/// メッシュの頂点データを MeshVertex に変換する
pub fn triangle_mesh_to_mesh_vertices_generic(
    vertex_data: Vec<viewmodel::mesh_converter::VertexData>,
) -> Vec<MeshVertex> {
    convert_vertex_data_to_mesh_vertices(&vertex_data)
}

#[cfg(test)]
mod tests {
    use super::triangle_mesh_to_mesh_vertices_generic;
    use viewmodel::mesh_converter::VertexData;

    #[test]
    fn test_mvvm_mesh_conversion() {
        let vertex_data = vec![
            VertexData::new([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]),
            VertexData::new([-1.0, 0.5, 0.25], [0.0, 1.0, 0.0]),
        ];

        let converted = triangle_mesh_to_mesh_vertices_generic(vertex_data);

        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].position, [1.0, 2.0, 3.0]);
        assert_eq!(converted[0].normal, [0.0, 0.0, 1.0]);
        assert_eq!(converted[1].position, [-1.0, 0.5, 0.25]);
        assert_eq!(converted[1].normal, [0.0, 1.0, 0.0]);
    }
}
