//! STLファイル読み込み機能
//!
//! STLファイルを読み込んでメッシュデータをGPU用の形式に変換します。

use geo_io::stl;
use geo_primitives::TriangleMesh3D;
use render::vertex_3d::MeshVertex;
use std::path::Path;

/// STL読み込み結果の型エイリアス（複雑性軽減のため）
type StlLoadResult =
    Result<(Vec<MeshVertex>, Vec<u32>, ([f32; 3], [f32; 3])), Box<dyn std::error::Error>>;

/// STLファイルを読み込み、レンダリング用の頂点データと境界ボックスに変換
pub fn load_stl_for_rendering(path: &Path) -> StlLoadResult {
    // STLファイルを読み込み
    let mesh: TriangleMesh3D<f64> = stl::load_stl(path)?;

    tracing::info!(
        "STLファイル読み込み完了: {} 頂点, {} 三角形",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // 境界ボックスを計算
    let bounds = mesh.bounding_box();
    let (min_bounds, max_bounds) = if let Some((min_point, max_point)) = bounds {
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
        tracing::warn!("メッシュが空のため、デフォルト境界ボックスを使用");
        ([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])
    };

    tracing::info!(
        "メッシュ境界ボックス: min={:?}, max={:?}",
        min_bounds,
        max_bounds
    );

    // MVVM準拠: viewmodel経由でGPU用頂点データに変換
    let vertex_data = viewmodel::mesh_converter::triangle_mesh_to_vertices(&mesh);
    let (_vertices, indices) =
        viewmodel::mesh_converter::triangle_mesh_to_vertices_with_indices(&mesh);

    // renderクレートのMeshVertex形式に変換
    let render_vertices = render::vertex_3d::convert_vertex_data_to_mesh_vertices(&vertex_data);

    tracing::info!(
        "メッシュ変換完了: {} レンダリング頂点, {} インデックス",
        render_vertices.len(),
        indices.len()
    );

    Ok((render_vertices, indices, (min_bounds, max_bounds)))
}

/// サンプルSTLファイルを作成
pub fn create_sample_stl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use geo_primitives::Point3D;

    // シンプルな立方体を作成（より分かりやすい形状）
    let vertices = vec![
        // 底面の4頂点
        Point3D::new(-0.5, -0.5, -0.5), // 0: 左下後
        Point3D::new(0.5, -0.5, -0.5),  // 1: 右下後
        Point3D::new(0.5, 0.5, -0.5),   // 2: 右上後
        Point3D::new(-0.5, 0.5, -0.5),  // 3: 左上後
        // 上面の4頂点
        Point3D::new(-0.5, -0.5, 0.5), // 4: 左下前
        Point3D::new(0.5, -0.5, 0.5),  // 5: 右下前
        Point3D::new(0.5, 0.5, 0.5),   // 6: 右上前
        Point3D::new(-0.5, 0.5, 0.5),  // 7: 左上前
    ];

    let indices = vec![
        // 底面 (-Z) - 外向き法線のためCCW順序
        [0, 2, 1],
        [0, 3, 2],
        // 上面 (+Z) - 外向き法線のためCCW順序
        [4, 5, 6],
        [4, 6, 7],
        // 左面 (-X) - 外向き法線のためCCW順序
        [0, 4, 7],
        [0, 7, 3],
        // 右面 (+X) - 外向き法線のためCCW順序
        [1, 2, 6],
        [1, 6, 5],
        // 前面 (-Y) - 外向き法線のためCCW順序
        [0, 1, 5],
        [0, 5, 4],
        // 後面 (+Y) - 外向き法線のためCCW順序
        [3, 7, 6],
        [3, 6, 2],
    ];

    let mesh = TriangleMesh3D::new(vertices, indices)?;
    stl::save_stl(&mesh, path)?;

    tracing::info!("サンプルSTLファイル作成（立方体）: {:?}", path);
    Ok(())
}

/// サンプルSTLファイルを作成して読み込み、境界ボックス付きで返す
pub fn create_sample_stl_with_bounds(path: &Path) -> StlLoadResult {
    create_sample_stl(path)?;
    load_stl_for_rendering(path)
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use tempfile::NamedTempFile;

    #[test]
    #[ignore] // tempfile依存のため一時的に無効化
    fn test_sample_stl_creation_and_loading() {
        // let temp_file = NamedTempFile::new().unwrap();

        // サンプルSTLを作成
        // create_sample_stl(temp_file.path()).unwrap();

        // 読み込んでレンダリング用に変換
        // let (vertices, indices, bounds) = load_stl_for_rendering(temp_file.path()).unwrap();

        // assert!(!vertices.is_empty());
        // assert!(!indices.is_empty());
        // assert_ne!(bounds.0, bounds.1);

        // TODO: tempfile依存を追加後に再有効化
    }
}
