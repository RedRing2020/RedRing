//! AppState のデバッグ表示ロード（octree/toolpath/svg/nurbs）を扱うモジュール。

use super::AppState;
use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};
use render::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};
use stage::{MeshStage, NurbsCurveStage, NurbsSurfaceStage, OctreeStage};
use std::path::Path;

impl AppState {
    /// デバッグ用：VoxelOctree可視化を表示
    pub fn load_debug_octree(&mut self) {
        use viewmodel::octree_converter::create_sample_swept_cylinder_wireframe_colored_levels_with_settings;

        tracing::info!("VoxelOctree可視化デバッグ開始");

        let depth_levels = create_sample_swept_cylinder_wireframe_colored_levels_with_settings(
            &self.octree_visualization_settings,
        );

        let initial_depth = depth_levels
            .iter()
            .position(|vertices| !vertices.is_empty())
            .unwrap_or(0);

        let positions: Vec<[f32; 3]> = depth_levels
            .get(initial_depth)
            .map(|vertices| vertices.iter().map(|v| v.position).collect())
            .unwrap_or_default();

        if positions.is_empty() {
            tracing::warn!("Octreeワイヤーフレーム頂点が空のため表示をスキップ");
            return;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for pos in &positions {
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        let center_x = (min_x + max_x) * 0.5;
        let center_y = (min_y + max_y) * 0.5;
        let center_z = (min_z + max_z) * 0.5;

        let size_x = (max_x - min_x).max(1.0);
        let size_y = (max_y - min_y).max(1.0);
        let size_z = (max_z - min_z).max(1.0);
        let half_extent_xy = (size_x.max(size_y) * 0.5 * 1.4).max(10.0);

        tracing::info!("ワイヤーフレーム頂点数: {}", positions.len());

        for (i, pos) in positions.iter().take(8).enumerate() {
            tracing::info!("頂点[{}]: [{:.1}, {:.1}, {:.1}]", i, pos[0], pos[1], pos[2]);
        }

        let mut octree_stage = Box::new(OctreeStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        octree_stage.set_depth_levels(&self.graphic.device, depth_levels);
        octree_stage.set_depth(&self.graphic.device, initial_depth);

        self.camera.target = Vec3f::new(center_x, center_y, center_z);
        self.camera.distance = (size_z * 6.0 + half_extent_xy).max(80.0);
        self.camera.zoom = 1.0;
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);

        self.camera.set_orthographic_bounds(
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        tracing::info!(
            "カメラ設定: target=({:.1}, {:.1}, {:.1}), distance={:.1}, bounds(camspace)=({:.1}..{:.1}, {:.1}..{:.1}), 平行投影",
            center_x,
            center_y,
            center_z,
            self.camera.distance,
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        let view_mat = self.camera.view_matrix();
        tracing::info!(
            "view_matrix[3]: [{:.2}, {:.2}, {:.2}, {:.2}]",
            view_mat[3][0],
            view_mat[3][1],
            view_mat[3][2],
            view_mat[3][3]
        );

        self.renderer.set_stage(octree_stage);
        self.update_camera_uniforms();

        tracing::info!("VoxelOctree可視化デバッグ完了");
        tracing::info!("初期表示深さ: {}", initial_depth);
        tracing::info!("o: 深さを1段進める, Shift+O: 深さアニメーション再生");
    }

    /// デバッグ用：CAMシミュレーション可視化（ToolPath + ワークOctree + 除去結果）を表示
    pub fn load_debug_toolpath(&mut self) {
        use viewmodel::cam_sim_visualization_converter::create_sample_cam_simulation_visualization_bundle_with_settings;

        tracing::info!("CAMシミュレーション可視化デバッグ開始（pキー）");

        let bundle = match create_sample_cam_simulation_visualization_bundle_with_settings(
            &self.octree_visualization_settings,
        ) {
            Ok(bundle) => bundle,
            Err(error) => {
                tracing::error!("CAMシミュレーション可視化データ生成失敗: {}", error);
                return;
            }
        };

        if bundle.snapshot_wireframes.is_empty() {
            tracing::warn!("CAMシミュレーション可視化フレームが空のため表示をスキップ");
            return;
        }

        self.debug_snapshot_series = Some(bundle.snapshot_series);
        self.debug_snapshot_wireframes = Some(bundle.snapshot_wireframes);
        self.debug_snapshot_solids = Some(
            bundle
                .snapshot_solid_meshes
                .iter()
                .map(|(vertex_data, indices)| {
                    (
                        convert_vertex_data_to_mesh_vertices(vertex_data),
                        indices.clone(),
                    )
                })
                .collect(),
        );
        self.debug_snapshot_toolpath_lines = Some(
            bundle
                .toolpath_wireframe
                .iter()
                .map(|v| MeshVertex::new(v.position, v.color))
                .collect(),
        );
        self.debug_snapshot_tool_lines = Some(
            bundle
                .snapshot_tool_wireframes
                .iter()
                .map(|frame| {
                    frame
                        .iter()
                        .map(|v| {
                            MeshVertex::new(
                                v.position,
                                self.snapshot_shaded_color_settings.tool_wire_color,
                            )
                        })
                        .collect()
                })
                .collect(),
        );
        self.debug_snapshot_shaded_mode = false;
        self.debug_snapshot_cursor = 0;

        let frame_wireframes = self
            .debug_snapshot_wireframes
            .as_ref()
            .expect("frame wireframes");
        let frame_count = frame_wireframes.len();
        let positions: Vec<[f32; 3]> = frame_wireframes
            .last()
            .map(|vertices| vertices.iter().map(|v| v.position).collect())
            .unwrap_or_default();

        if positions.is_empty() {
            tracing::warn!("CAMシミュレーション可視化頂点が空のため表示をスキップ");
            return;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for pos in &positions {
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        let center_x = (min_x + max_x) * 0.5;
        let center_y = (min_y + max_y) * 0.5;
        let center_z = (min_z + max_z) * 0.5;
        let size_x = (max_x - min_x).max(1.0);
        let size_y = (max_y - min_y).max(1.0);
        let size_z = (max_z - min_z).max(1.0);
        let half_extent_xy = (size_x.max(size_y) * 0.5 * 1.4).max(10.0);

        let mut octree_stage = Box::new(OctreeStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        octree_stage.set_depth_levels(&self.graphic.device, frame_wireframes.clone());
        octree_stage.set_depth(&self.graphic.device, 0);

        self.camera.target = Vec3f::new(center_x, center_y, center_z);
        self.camera.distance = (size_z * 6.0 + half_extent_xy).max(80.0);
        self.camera.zoom = 1.0;
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);
        self.camera.set_orthographic_bounds(
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        self.renderer.set_stage(octree_stage);
        self.update_camera_uniforms();
        self.log_current_snapshot_frame(true);

        tracing::info!(
            "CAMシミュレーション可視化デバッグ完了: frame={}/{}（k/スクラブで時系列再生）",
            1,
            frame_count
        );
    }

    /// デバッグ用：カッターパスのみを表示（pキー）
    pub fn load_debug_cutter_path_only(&mut self) {
        use viewmodel::toolpath_converter::{
            create_sample_toolpath, toolpath_to_vertices, ToolPathVisualizationSettings,
        };

        tracing::info!("カッターパス表示デバッグ開始（pキー）");

        let toolpath = create_sample_toolpath();
        let settings = ToolPathVisualizationSettings::default();
        let toolpath_vertices = toolpath_to_vertices(&toolpath, &settings);

        let vertices: Vec<MeshVertex> = toolpath_vertices
            .vertices
            .iter()
            .enumerate()
            .map(|(index, vertex)| {
                let color = toolpath_vertices
                    .colors
                    .get(index / 2)
                    .copied()
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                MeshVertex::new(vertex.position, [color[0], color[1], color[2]])
            })
            .collect();

        if vertices.is_empty() {
            tracing::warn!("カッターパス頂点が空のため表示をスキップ");
            return;
        }

        self.debug_snapshot_series = None;
        self.debug_snapshot_wireframes = None;
        self.debug_snapshot_solids = None;
        self.debug_snapshot_toolpath_lines = None;
        self.debug_snapshot_tool_lines = None;
        self.debug_snapshot_shaded_mode = false;
        self.debug_snapshot_cursor = 0;

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for vertex in &vertices {
            let pos = vertex.position;
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        let center_x = (min_x + max_x) * 0.5;
        let center_y = (min_y + max_y) * 0.5;
        let center_z = (min_z + max_z) * 0.5;
        let size_x = (max_x - min_x).max(1.0);
        let size_y = (max_y - min_y).max(1.0);
        let size_z = (max_z - min_z).max(1.0);
        let half_extent_xy = (size_x.max(size_y) * 0.5 * 1.4).max(10.0);

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);

        self.camera.target = Vec3f::new(center_x, center_y, center_z);
        self.camera.distance = (size_z * 6.0 + half_extent_xy).max(80.0);
        self.camera.zoom = 1.0;
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);
        self.camera.set_orthographic_bounds(
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();

        tracing::info!(
            "カッターパス表示デバッグ完了: 線分数={}",
            toolpath_vertices.colors.len()
        );
    }

    /// STLファイルを読み込んでメッシュステージに設定
    pub fn load_stl_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("STLファイル読み込み開始: {:?}", path);

        let (vertices, indices, _bounds) = crate::stl_loader::load_stl_for_rendering(path)?;

        self.camera.reset_to_standard_cad_view();

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);

        self.renderer.set_stage(mesh_stage);

        tracing::info!("STLファイル読み込み完了");

        self.update_camera_uniforms();

        Ok(())
    }

    /// サンプルSTLファイルを作成して読み込み
    pub fn load_sample_stl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sample_path = std::env::temp_dir().join("redring_sample.stl");

        let (vertices, indices, _bounds) = crate::stl_loader::create_sample_stl_with_bounds(&sample_path)?;

        self.camera.reset_to_standard_cad_view();

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);

        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();

        Ok(())
    }

    /// デバッグ用：LineSegment3Dを表示（SVGから読み込み）
    pub fn load_debug_line(&mut self) {
        tracing::info!("デバッグ形状: LineSegment3D表示（EntityManager経由）");

        let svg_path = Path::new("tests/fixtures/shapes/line.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                let id = self.entity_manager.add_line_entity(vertices);
                let _ = self.entity_manager.select(id);

                self.camera.reset_to_standard_cad_view();
                self.rebuild_stage_from_entities();
            }
            Err(error) => {
                tracing::error!("SVG読み込みエラー: {}", error);
            }
        }
    }

    /// デバッグ用：Circle3Dを表示（SVGから読み込み）
    pub fn load_debug_circle(&mut self) {
        tracing::info!("デバッグ形状: Circle3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/circle.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                self.camera.reset_to_standard_cad_view();

                let mut mesh_stage = Box::new(MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                mesh_stage.set_line_data(&self.graphic.device, vertices);

                self.renderer.set_stage(mesh_stage);
                self.update_camera_uniforms();
            }
            Err(error) => {
                tracing::error!("SVG読み込みエラー: {}", error);
            }
        }
    }

    /// デバッグ用：クリップ空間座標の単純な正方形（SVGから読み込み）
    pub fn load_debug_clip_square(&mut self) {
        tracing::warn!("DEBUG: クリップ空間正方形を表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/clip_square.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::warn!("SVG読み込み成功: {} 頂点", vertices.len());

                let mut mesh_stage = Box::new(stage::mesh_stage::MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));

                mesh_stage.set_line_data(&self.graphic.device, vertices);
                self.renderer.set_stage(mesh_stage);

                self.update_camera_uniforms();
            }
            Err(error) => {
                tracing::error!("SVG読み込みエラー: {}", error);
            }
        }
    }

    /// デバッグ用：Triangle3Dを表示（SVGから読み込み）
    pub fn load_debug_triangle(&mut self) {
        tracing::info!("デバッグ形状: Triangle3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/triangle.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                let indices: Vec<u32> = vec![0, 1, 2];

                self.camera.reset_to_standard_cad_view();

                let mut mesh_stage = Box::new(MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);

                self.renderer.set_stage(mesh_stage);
                self.update_camera_uniforms();
            }
            Err(error) => {
                tracing::error!("SVG読み込みエラー: {}", error);
            }
        }
    }

    /// デバッグ用：Arc3Dを表示（SVGから読み込み）
    pub fn load_debug_arc(&mut self) {
        tracing::info!("デバッグ形状: Arc3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/arc.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                self.camera.reset_to_standard_cad_view();

                let mut mesh_stage = Box::new(MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                mesh_stage.set_line_data(&self.graphic.device, vertices);

                self.renderer.set_stage(mesh_stage);
                self.update_camera_uniforms();
            }
            Err(error) => {
                tracing::error!("SVG読み込みエラー: {}", error);
            }
        }
    }

    /// デバッグ用：NurbsCurve3Dを表示（SVGから読み込み）
    /// ViewModelレイヤー経由で評価データを生成
    pub fn load_debug_nurbs(&mut self) {
        let tolerance = self.tolerance_in_current_unit();
        tracing::info!("デバッグ形状: NurbsCurve3D表示（GPU評価）");
        tracing::info!(
            "表示トレランス: {:.6} (単位系: {:?})",
            tolerance,
            self.unit_system
        );

        let svg_path = Path::new("tests/fixtures/shapes/nurbs_curve.svg");
        let eval_data =
            match viewmodel::nurbs_debug::load_nurbs_curve_eval_from_svg(svg_path, tolerance) {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!("NURBS曲線データ生成失敗: {}", error);
                    return;
                }
            };

        tracing::info!(
            "GPU評価データ生成完了: params={}, control_points={}, degree={}, knots={}",
            eval_data.num_eval_points(),
            eval_data.num_control_points(),
            eval_data.degree,
            eval_data.knots.len()
        );

        self.camera.reset_to_standard_cad_view();

        let mut nurbs_stage = Box::new(NurbsCurveStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        nurbs_stage.set_eval_data(&self.graphic.device, eval_data);

        self.renderer.set_stage(nurbs_stage);
        self.update_camera_uniforms();
    }

    /// デバッグ用: NURBS曲面をGPU評価で表示（曲率のある曲面）
    /// ViewModelレイヤー経由で評価データを生成
    pub fn load_debug_nurbs_surface(&mut self) {
        tracing::info!("デバッグ形状: NurbsSurface3D表示（GPU評価）- 曲率のある曲面");
        let tolerance_value = self.tolerance_in_current_unit();
        tracing::info!(
            "表示トレランス: {:.6} (単位系: {:?})",
            tolerance_value,
            self.unit_system
        );
        let eval_data =
            match viewmodel::nurbs_debug::create_sample_nurbs_surface_eval(tolerance_value) {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!("NURBS曲面データ生成失敗: {}", error);
                    return;
                }
            };

        tracing::info!(
            "GPU評価データ生成完了: vertices={}, triangles={}, u_degree={}, v_degree={}",
            eval_data.num_vertices(),
            eval_data.num_triangles(),
            eval_data.u_degree,
            eval_data.v_degree
        );

        self.camera.reset_to_standard_cad_view();

        let mut nurbs_surface_stage = Box::new(NurbsSurfaceStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        nurbs_surface_stage.set_eval_data(&self.graphic.device, eval_data);

        self.renderer.set_stage(nurbs_surface_stage);
        self.update_camera_uniforms();
    }
}
