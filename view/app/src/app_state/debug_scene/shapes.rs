//! AppState のSTL/SVG形状デバッグ表示を扱うモジュール。

use super::super::AppState;
use render::vertex_3d::MeshVertex;
use stage::MeshStage;
use std::path::Path;

use crate::app_asset_loader::AppAssetLoaderFacade;

impl AppState {
    fn load_svg_vertices_for_debug(&self, svg_path: &Path) -> Option<Vec<MeshVertex>> {
        match AppAssetLoaderFacade::load_svg(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());
                Some(vertices)
            }
            Err(error) => {
                tracing::error!("SVG読み込みエラー: {}", error);
                None
            }
        }
    }

    fn apply_line_stage(&mut self, vertices: Vec<MeshVertex>, reset_camera: bool) {
        if reset_camera {
            self.camera.reset_to_standard_cad_view();
        }

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);

        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();
    }

    fn apply_mesh_stage(
        &mut self,
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        reset_camera: bool,
    ) {
        if reset_camera {
            self.camera.reset_to_standard_cad_view();
        }

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);

        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();
    }

    /// STLファイルを読み込んでメッシュステージに設定
    pub fn load_stl_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("STLファイル読み込み開始: {:?}", path);

        let (vertices, indices, _bounds) = AppAssetLoaderFacade::load_stl(path)?;
        self.apply_mesh_stage(vertices, indices, true);

        tracing::info!("STLファイル読み込み完了");

        Ok(())
    }

    /// サンプルSTLファイルを作成して読み込み
    pub fn load_sample_stl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sample_path = std::env::temp_dir().join("redring_sample.stl");

        let (vertices, indices, _bounds) =
            AppAssetLoaderFacade::load_sample_stl_with_bounds(&sample_path)?;
        self.apply_mesh_stage(vertices, indices, true);

        Ok(())
    }

    /// デバッグ用：LineSegment3Dを表示（SVGから読み込み）
    /// EntityManager経由の管理対象として扱う。
    pub fn load_debug_line(&mut self) {
        tracing::info!("デバッグ形状: LineSegment3D表示（EntityManager経由）");

        let svg_path = Path::new("tests/fixtures/shapes/line.svg");
        let Some(vertices) = self.load_svg_vertices_for_debug(svg_path) else {
            return;
        };

        let id = self.entity_manager.add_line_entity(vertices);
        let _ = self.entity_manager.select(id);

        self.camera.reset_to_standard_cad_view();
        self.rebuild_stage_from_entities();
    }

    /// デバッグ用：Circle3Dを表示（SVGから読み込み）
    /// 直接MeshStage経路で扱う。
    pub fn load_debug_circle(&mut self) {
        tracing::info!("デバッグ形状: Circle3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/circle.svg");
        let Some(vertices) = self.load_svg_vertices_for_debug(svg_path) else {
            return;
        };

        self.apply_line_stage(vertices, true);
    }

    /// デバッグ用：クリップ空間座標の単純な正方形（SVGから読み込み）
    /// 直接MeshStage経路で扱う。
    pub fn load_debug_clip_square(&mut self) {
        tracing::warn!("DEBUG: クリップ空間正方形を表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/clip_square.svg");
        let Some(vertices) = self.load_svg_vertices_for_debug(svg_path) else {
            return;
        };

        self.apply_line_stage(vertices, false);
    }

    /// デバッグ用：Triangle3Dを表示（SVGから読み込み）
    /// 直接MeshStage経路で扱う。
    pub fn load_debug_triangle(&mut self) {
        tracing::info!("デバッグ形状: Triangle3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/triangle.svg");
        let Some(vertices) = self.load_svg_vertices_for_debug(svg_path) else {
            return;
        };

        let indices: Vec<u32> = vec![0, 1, 2];
        self.apply_mesh_stage(vertices, indices, true);
    }

    /// デバッグ用：Arc3Dを表示（SVGから読み込み）
    /// 直接MeshStage経路で扱う。
    pub fn load_debug_arc(&mut self) {
        tracing::info!("デバッグ形状: Arc3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/arc.svg");
        let Some(vertices) = self.load_svg_vertices_for_debug(svg_path) else {
            return;
        };

        self.apply_line_stage(vertices, true);
    }
}
