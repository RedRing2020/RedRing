//! SVGローダー（View層）
//!
//! SVG形状ファイルを読み込んでレンダリング用のMeshVertexに変換します。

use render::vertex_3d::MeshVertex;
use std::path::Path;
use viewmodel::svg_loader::load_svg_shapes;

/// SVG形状読み込み結果
pub type SvgLoadResult = Result<Vec<MeshVertex>, Box<dyn std::error::Error>>;

/// SVGファイルを読み込み、レンダリング用の頂点データに変換
pub fn load_svg_for_rendering(path: &Path) -> SvgLoadResult {
    tracing::info!("SVGファイル読み込み: {:?}", path);

    // ViewModel層でSVGをVertexDataに変換
    let vertex_data = load_svg_shapes(path)?;

    tracing::info!("SVG変換完了: {} 頂点", vertex_data.len());

    // MeshVertexに変換
    let vertices: Vec<MeshVertex> = vertex_data
        .iter()
        .map(MeshVertex::from_vertex_data)
        .collect();

    Ok(vertices)
}
