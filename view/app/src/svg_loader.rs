//! SVGローダー（View層）
//!
//! SVG形状ファイルを読み込んでレンダリング用のMeshVertexに変換します。

use render::vertex_3d::MeshVertex;
use std::path::Path;
use viewmodel::svg_loader::load_svg_shapes;

/// SVG形状読み込み結果
pub type SvgLoadResult = Result<Vec<MeshVertex>, Box<dyn std::error::Error>>;

/// SVGファイルを読み込み、レンダリング用の頂点データに変換
///
/// # Arguments
/// * `path` - SVGファイルパス
/// * `tolerance` - テッセレーショントレランス（ミリメートル単位、デフォルト: 0.01mm）
pub fn load_svg_for_rendering(path: &Path, tolerance: Option<f64>) -> SvgLoadResult {
    tracing::info!("SVGファイル読み込み: {:?}", path);

    let tol = tolerance.unwrap_or(0.01);
    // ViewModel層でSVGをVertexDataに変換
    let vertex_data = load_svg_shapes(path, tol)?;

    tracing::info!("SVG変換完了: {} 頂点", vertex_data.len());

    // MeshVertexに変換
    let vertices: Vec<MeshVertex> = vertex_data
        .iter()
        .map(MeshVertex::from_vertex_data)
        .collect();

    Ok(vertices)
}
