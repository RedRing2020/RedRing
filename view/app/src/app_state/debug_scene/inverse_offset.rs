//! AppState の逆オフセット包絡面（CL 面）デバッグ表示を扱うモジュール。

use super::super::AppState;
use analysis::linalg::vector::Vec3f;
use render::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};
use stage::MeshStage;
use viewmodel::inverse_offset_converter::{
    build_sample_inverse_offset_visualization, InspectionToolKind, InverseOffsetDisplayMode,
};

/// 逆オフセット包絡面デバッグ表示の状態
#[derive(Debug, Clone, Copy)]
pub(crate) struct InverseOffsetDebugState {
    pub(crate) tool_kind: InspectionToolKind,
    pub(crate) mode: InverseOffsetDisplayMode,
}

impl Default for InverseOffsetDebugState {
    fn default() -> Self {
        Self {
            tool_kind: InspectionToolKind::BallEndMill,
            mode: InverseOffsetDisplayMode::EnvelopeSurface,
        }
    }
}

impl AppState {
    /// デバッグ用: サンプルNURBS曲面の逆オフセット包絡面を表示（i キー）
    pub fn load_debug_inverse_offset(&mut self) {
        let state = self.inverse_offset_debug;
        let chord_tolerance = self.tolerance_in_current_unit();
        tracing::info!(
            "デバッグ形状: 逆オフセット包絡面（工具: {}, 表示: {}, 工具半径: {}, 格子線ピッチ: {}, 弦誤差: {:.6}）",
            state.tool_kind.label(),
            state.mode.label(),
            self.inverse_offset_visualization_settings.tool_radius,
            self.inverse_offset_visualization_settings.grid_line_pitch(),
            chord_tolerance
        );

        let view = match build_sample_inverse_offset_visualization(
            state.tool_kind,
            state.mode,
            chord_tolerance,
            &self.inverse_offset_visualization_settings,
        ) {
            Ok(view) => view,
            Err(error) => {
                tracing::error!(
                    error_kind = logging_foundation::ERROR_KIND_APP,
                    "inverse offset visualization failed: {}",
                    error
                );
                return;
            }
        };

        tracing::info!(
            "逆オフセット包絡面生成完了: 評価点={}x{}, 三角形={}, 重畳線分={}, 基準点=先端+{}（ボール: 球中心 / フラット: 底面中心）",
            view.envelope_samples.0,
            view.envelope_samples.1,
            view.mesh_indices.len() / 3,
            view.overlay_lines.len() / 2,
            view.reference_offset
        );

        let overlay_lines: Vec<MeshVertex> = view
            .overlay_lines
            .iter()
            .map(|v| MeshVertex::new(v.position, v.color))
            .collect();

        // サンプル曲面は単位寸法のため、mm 規模向けの既定視点ではなく小形状向けの fit を使う
        self.camera.reset_to_standard_cad_view();
        if let Some((min, max)) = bounding_box(&view.fit_positions) {
            self.camera.fit_to_small_mesh(min, max);
            // 傾斜視点のまま、形状全体が収まる直交投影範囲にする
            let size = max - min;
            let half_extent = size.x().max(size.y()).max(size.z()) * 0.75;
            self.camera
                .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);
            self.camera.set_orthographic_bounds(
                -half_extent,
                half_extent,
                -half_extent,
                half_extent,
            );
        }

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_base_color(view.mesh_color);
        mesh_stage.set_mesh_data(
            &self.graphic.device,
            convert_vertex_data_to_mesh_vertices(&view.mesh_vertices),
            view.mesh_indices,
        );
        mesh_stage.set_overlay_line_data(&self.graphic.device, overlay_lines);

        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();
    }

    /// 逆オフセット包絡面の工具（ボール/フラット）を切り替えて再表示（Shift+I）
    pub fn toggle_debug_inverse_offset_tool(&mut self) {
        self.inverse_offset_debug.tool_kind = self.inverse_offset_debug.tool_kind.toggled();
        self.load_debug_inverse_offset();
    }

    /// 逆オフセット包絡面の表示（包絡面/格子線）を切り替えて再表示（g キー）
    pub fn toggle_debug_inverse_offset_mode(&mut self) {
        self.inverse_offset_debug.mode = self.inverse_offset_debug.mode.toggled();
        self.load_debug_inverse_offset();
    }
}

fn bounding_box(positions: &[[f32; 3]]) -> Option<(Vec3f, Vec3f)> {
    let first = positions.first()?;
    let (mut min, mut max) = (*first, *first);
    for p in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    Some((
        Vec3f::new(min[0], min[1], min[2]),
        Vec3f::new(max[0], max[1], max[2]),
    ))
}
