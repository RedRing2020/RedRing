//! STL読み込み機能（ViewModel層）
//!
//! MVVMアーキテクチャに準拠したSTLファイル読み込みとGPU変換
//! Model層（geo_io）からView層（render）への架け橋

use crate::mesh_converter::{
    triangle_mesh_to_vertices, triangle_mesh_to_vertices_with_indices, VertexData,
};
use geo_io::{fixtures, stl};
use std::path::Path;

/// STL読み込み結果
pub struct StlMeshData {
    /// GPU用頂点データ
    pub vertices: Vec<VertexData>,
    /// インデックスデータ
    pub indices: Vec<u32>,
    /// 境界ボックス（min, max）
    pub bounds: ([f32; 3], [f32; 3]),
}

/// STLファイルを読み込み、GPU用データに変換
pub fn load_stl_mesh(path: &Path) -> Result<StlMeshData, Box<dyn std::error::Error>> {
    // Model層（geo_io）からSTLデータを取得
    let mesh = stl::load_stl::<f64>(path)?;

    tracing::info!(
        "STLファイル読み込み完了: {} 頂点, {} 三角形",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // 境界ボックスを計算
    let bounds = calculate_bounds(&mesh);

    tracing::info!(
        "メッシュ境界ボックス: min={:?}, max={:?}",
        bounds.0,
        bounds.1
    );

    // ViewModel層でGPU用データに変換
    let vertices = triangle_mesh_to_vertices(&mesh);
    let (_vertex_positions, indices) = triangle_mesh_to_vertices_with_indices(&mesh);

    tracing::info!(
        "メッシュ変換完了: {} GPU頂点, {} インデックス",
        vertices.len(),
        indices.len()
    );

    Ok(StlMeshData {
        vertices,
        indices,
        bounds,
    })
}

/// サンプルSTLファイルを作成
pub fn create_sample_stl_mesh(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fixtures::create_sample_stl_mesh(path)
}

/// サンプル作成と読み込みを同時実行
pub fn create_and_load_sample_stl(path: &Path) -> Result<StlMeshData, Box<dyn std::error::Error>> {
    create_sample_stl_mesh(path)?;
    load_stl_mesh(path)
}

/// 境界ボックス計算のヘルパー関数
fn calculate_bounds(mesh: &geo_algorithms::TriangleMesh3D<f64>) -> ([f32; 3], [f32; 3]) {
    let bounds = mesh.bounding_box();
    if let Some((min_point, max_point)) = bounds {
        let min_bounds = [
            min_point.x() as f32,
            min_point.y() as f32,
            min_point.z() as f32,
        ];
        let max_bounds = [
            max_point.x() as f32,
            max_point.y() as f32,
            max_point.z() as f32,
        ];
        (min_bounds, max_bounds)
    } else {
        // デフォルトの境界ボックス（空のメッシュの場合）
        tracing::warn!(
            error_kind = logging_foundation::ERROR_KIND_APP,
            "mesh is empty, falling back to default bounding box"
        );
        ([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])
    }
}

#[cfg(test)]
mod tests {
    // use super::*;  // テストが無効化されているため一時的にコメントアウト

    #[test]
    #[ignore] // tempfile依存のため一時的に無効化
    fn test_stl_mesh_data_creation() {
        // let temp_file = tempfile::NamedTempFile::new().unwrap();
        // let mesh_data = create_and_load_sample_stl(temp_file.path()).unwrap();
        //
        // assert!(!mesh_data.vertices.is_empty());
        // assert!(!mesh_data.indices.is_empty());
        // assert_ne!(mesh_data.bounds.0, mesh_data.bounds.1);
    }
}
