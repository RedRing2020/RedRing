//! app層向けアセット読み込みFacade。
//!
//! `debug_scene` などの呼び出し側が、個別loader実装詳細
//! (`stl_loader` / `svg_loader`) に直接依存しないための薄い接続層。

use render::vertex_3d::MeshVertex;
use std::path::Path;

pub type StlRenderLoadResult =
    Result<(Vec<MeshVertex>, Vec<u32>, ([f32; 3], [f32; 3])), Box<dyn std::error::Error>>;

pub type SvgRenderLoadResult = Result<Vec<MeshVertex>, Box<dyn std::error::Error>>;

pub struct AppAssetLoaderFacade;

impl AppAssetLoaderFacade {
    pub fn load_stl(path: &Path) -> StlRenderLoadResult {
        crate::stl_loader::load_stl_for_rendering(path)
    }

    pub fn load_sample_stl_with_bounds(path: &Path) -> StlRenderLoadResult {
        crate::stl_loader::create_sample_stl_with_bounds(path)
    }

    pub fn load_svg(path: &Path, tolerance: Option<f64>) -> SvgRenderLoadResult {
        crate::svg_loader::load_svg_for_rendering(path, tolerance)
    }
}

pub fn load_stl(path: &Path) -> StlRenderLoadResult {
    AppAssetLoaderFacade::load_stl(path)
}

pub fn load_sample_stl_with_bounds(path: &Path) -> StlRenderLoadResult {
    AppAssetLoaderFacade::load_sample_stl_with_bounds(path)
}

pub fn load_svg(path: &Path, tolerance: Option<f64>) -> SvgRenderLoadResult {
    AppAssetLoaderFacade::load_svg(path, tolerance)
}
