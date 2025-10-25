//! STLファイル読み込み機能
//!
//! STLファイルを読み込んでメッシュデータをGPU用の形式に変換します。

use geo_io::stl;
use geo_primitives::TriangleMesh3D;
use render::mesh_convert;
use render::vertex_3d::MeshVertex;
use std::path::Path;

/// STLファイルを読み込み、レンダリング用の頂点データに変換
pub fn load_stl_for_rendering(
    path: &Path,
) -> Result<(Vec<MeshVertex>, Vec<u32>), Box<dyn std::error::Error>> {
    // STLファイルを読み込み
    let mesh: TriangleMesh3D<f64> = stl::load_stl(path)?;

    tracing::info!(
        "STLファイル読み込み完了: {} 頂点, {} 三角形",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // GPU用の頂点データに変換
    let (vertices, indices) = mesh_convert::triangle_mesh_to_mesh_vertices(&mesh);

    tracing::info!(
        "メッシュ変換完了: {} レンダリング頂点, {} インデックス",
        vertices.len(),
        indices.len()
    );

    Ok((vertices, indices))
}

/// サンプルSTLファイルを生成（テスト用）
pub fn create_sample_stl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use geo_primitives::Point3D;

    // シンプルな三角錐を作成
    let vertices = vec![
        Point3D::new(0.0, 0.0, 0.0), // 底面中心
        Point3D::new(1.0, 0.0, 0.0), // 底面右
        Point3D::new(0.0, 1.0, 0.0), // 底面奥
        Point3D::new(0.5, 0.5, 1.0), // 頂点
    ];

    let indices = vec![
        [0, 1, 2], // 底面
        [0, 1, 3], // 側面1
        [1, 2, 3], // 側面2
        [2, 0, 3], // 側面3
    ];

    let mesh = TriangleMesh3D::new(vertices, indices)?;
    stl::save_stl(&mesh, path)?;

    tracing::info!("サンプルSTLファイル作成: {:?}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sample_stl_creation_and_loading() {
        let temp_file = NamedTempFile::new().unwrap();

        // サンプルSTLを作成
        create_sample_stl(temp_file.path()).unwrap();

        // 読み込んでレンダリング用に変換
        let (vertices, indices) = load_stl_for_rendering(temp_file.path()).unwrap();

        assert!(!vertices.is_empty());
        assert!(!indices.is_empty());
        assert_eq!(indices.len() % 3, 0); // 三角形の倍数
    }
}
