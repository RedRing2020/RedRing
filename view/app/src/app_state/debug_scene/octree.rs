//! AppState のOctree系デバッグ表示を扱うモジュール。

use super::debug_scene_camera_fit::CameraFit;
use super::AppState;
use stage::OctreeStage;
use viewmodel::octree_converter::WireframeVertex;

struct OctreeDebugData {
    depth_levels: Vec<Vec<WireframeVertex>>,
    initial_depth: usize,
    positions: Vec<[f32; 3]>,
    fit: CameraFit,
}

impl AppState {
    fn build_octree_debug_data(&self) -> Option<OctreeDebugData> {
        use viewmodel::octree_converter::create_sample_swept_cylinder_wireframe_colored_levels_with_settings;

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

        let fit = Self::build_camera_fit(&positions)?;

        Some(OctreeDebugData {
            depth_levels,
            initial_depth,
            positions,
            fit,
        })
    }

    fn apply_octree_debug_data(&mut self, data: OctreeDebugData) {
        let mut octree_stage = Box::new(OctreeStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        octree_stage.set_depth_levels(&self.graphic.device, data.depth_levels);
        octree_stage.set_depth(&self.graphic.device, data.initial_depth);

        self.apply_camera_fit(data.fit);

        tracing::info!(
            "カメラ設定: target=({:.1}, {:.1}, {:.1}), distance={:.1}, bounds(camspace)=({:.1}..{:.1}, {:.1}..{:.1}), 平行投影",
            data.fit.center_x,
            data.fit.center_y,
            data.fit.center_z,
            self.camera.distance,
            -data.fit.half_extent_xy,
            data.fit.half_extent_xy,
            -data.fit.half_extent_xy,
            data.fit.half_extent_xy,
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
    }

    /// デバッグ用：VoxelOctree可視化を表示
    pub fn load_debug_octree(&mut self) {
        tracing::info!("VoxelOctree可視化デバッグ開始");

        let Some(data) = self.build_octree_debug_data() else {
            tracing::warn!("Octreeワイヤーフレーム頂点が空のため表示をスキップ");
            return;
        };

        tracing::info!("ワイヤーフレーム頂点数: {}", data.positions.len());

        for (i, pos) in data.positions.iter().take(8).enumerate() {
            tracing::info!("頂点[{}]: [{:.1}, {:.1}, {:.1}]", i, pos[0], pos[1], pos[2]);
        }

        let initial_depth = data.initial_depth;
        self.apply_octree_debug_data(data);

        tracing::info!("VoxelOctree可視化デバッグ完了");
        tracing::info!("初期表示深さ: {}", initial_depth);
        tracing::info!("o: 深さを1段進める, Shift+O: 深さアニメーション再生");
    }
}
