//! STLファイル読み込み機能
//!
//! STLファイルを読み込んでメッシュデータをGPU用の形式に変換します。
//! MVVMアーキテクチャに準拠し、ViewModelレイヤー経由でSTLデータを処理します。

use render::vertex_3d::{MeshVertex, convert_vertex_data_to_mesh_vertices};
use std::path::Path;
use viewmodel::stl_loader::{StlMeshData, create_and_load_sample_stl, load_stl_mesh};

/// STL読み込み結果の型エイリアス（複雑性軽減のため）
type StlLoadResult =
    Result<(Vec<MeshVertex>, Vec<u32>, ([f32; 3], [f32; 3])), Box<dyn std::error::Error>>;

/// STLファイルを読み込み、レンダリング用の頂点データと境界ボックスに変換
/// MVVM準拠: ViewModel経由でModel層にアクセス
pub fn load_stl_for_rendering(path: &Path) -> StlLoadResult {
    // ViewModelレイヤー経由でSTLデータを取得・変換
    let stl_data: StlMeshData = load_stl_mesh(path)?;

    // ViewModel→Render層の変換
    let render_vertices = convert_vertex_data_to_mesh_vertices(&stl_data.vertices);

    tracing::info!(
        "STL→レンダリング変換完了: {} MeshVertex, {} インデックス",
        render_vertices.len(),
        stl_data.indices.len()
    );

    Ok((render_vertices, stl_data.indices, stl_data.bounds))
}

/// サンプルSTLファイルを作成
/// MVVM準拠: ViewModelレイヤー経由でModel層にアクセス
pub fn create_sample_stl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // ViewModelレイヤー経由でサンプル作成
    viewmodel::stl_loader::create_sample_stl_mesh(path)
}

/// サンプルSTLファイルを作成して読み込み、境界ボックス付きで返す
/// MVVM準拠: ViewModelレイヤー経由でデータ処理
pub fn create_sample_stl_with_bounds(path: &Path) -> StlLoadResult {
    // ViewModelレイヤー経由でサンプル作成・読み込み
    let stl_data = create_and_load_sample_stl(path)?;

    // ViewModel→Render層の変換
    let render_vertices = convert_vertex_data_to_mesh_vertices(&stl_data.vertices);

    Ok((render_vertices, stl_data.indices, stl_data.bounds))
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::NamedTempFile;

    #[test]
    #[ignore] // tempfile依存のため一時的に無効化
    fn test_sample_stl_creation_and_loading() {
        // MVVM準拠のテスト実装
        // let temp_file = NamedTempFile::new().unwrap();
        // let (vertices, indices, bounds) = create_sample_stl_with_bounds(temp_file.path()).unwrap();
        //
        // assert!(!vertices.is_empty());
        // assert!(!indices.is_empty());
        // assert_ne!(bounds.0, bounds.1);

        // TODO: tempfile依存を追加後に再有効化
    }
}
