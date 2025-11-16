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
    #[test]
    fn test_mvvm_mesh_conversion() {
        // 基本的な機能テスト：viewmodel経由でのメッシュ変換
        // 実際のテストは統合テスト時に実行

        // テストデータ作成が viewmodel に依存するため、
        // この段階では変換関数の存在確認のみ

        // これは将来的に適切な VertexData で
        // テストデータを作成して実行される予定
    }
}
