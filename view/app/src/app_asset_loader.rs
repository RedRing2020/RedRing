//! アセット読み込みFacade。
//!
//! SVG のファイルを読み込み、描画用の頂点データを返す。

use render::vertex_3d::MeshVertex;
use std::path::Path;

pub type SvgRenderLoadResult = Result<Vec<MeshVertex>, Box<dyn std::error::Error>>;

pub struct AppAssetLoaderFacade;

impl AppAssetLoaderFacade {
    pub fn load_svg(path: &Path, tolerance: Option<f64>) -> SvgRenderLoadResult {
        crate::svg_loader::load_svg_for_rendering(path, tolerance)
    }
}

pub fn load_svg(path: &Path, tolerance: Option<f64>) -> SvgRenderLoadResult {
    AppAssetLoaderFacade::load_svg(path, tolerance)
}
