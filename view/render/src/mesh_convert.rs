//! mesh_convert - TriangleMesh3D から MeshVertex への変換（MVVM準拠版）
//!
//! MVVM アーキテクチャに従い、viewmodel の mesh_converter を使用して変換を実行。
//! render レイヤーから直接 geo_* クレートへの依存を削除。

use crate::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};

/// ジェネリック版のメッシュ変換（型消去経由）
/// geo_primitives への直接依存を避けるため、型消去されたインターフェースを使用
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
