//! AppState の逆オフセット形状（要素の役割ごと）のデバッグ表示を扱うモジュール。

use super::super::AppState;
use render::mesh_list::MeshListItem;
use render::vertex_3d::convert_vertex_data_to_mesh_vertices;
use stage::MeshStage;
use viewmodel::inverse_offset_shape_converter::InverseOffsetPartKind;

/// 球面・円柱面・円板を三角形に分ける角度のきざみ（度）
const ANGLE_STEP_DEGREES: f64 = 10.0;

/// 種類ごとの色（RGBA。製品形状の白・素材の橙と見分けられる色にする）
fn part_color(kind: InverseOffsetPartKind) -> [f32; 4] {
    match kind {
        InverseOffsetPartKind::IndexedFace => [0.9, 0.85, 0.25, 1.0],
        InverseOffsetPartKind::UnindexedFace => [0.25, 0.35, 0.8, 1.0],
        InverseOffsetPartKind::UpperSide => [0.85, 0.35, 0.85, 1.0],
        InverseOffsetPartKind::UnorientedFace => [0.9, 0.15, 0.15, 1.0],
        InverseOffsetPartKind::Vertex => [0.35, 0.8, 0.4, 1.0],
        InverseOffsetPartKind::Edge => [0.3, 0.75, 0.9, 1.0],
    }
}

impl AppState {
    /// デバッグ用: 表示中の製品形状と計算条件の工具で作った逆オフセット形状の要素を、役割ごとの色で表示する
    ///
    /// カメラは動かさず、通常の表示と同じ向き・拡大で見比べられるようにする。取り込んだ切削シミュレーションのツールパスの
    /// 線を重ね（工具の先端が逆オフセット形状の上面を通るかを確かめる）、表示・非表示は通常の表示と同じに切り替える。
    /// 製品形状は描かない（フラットでは、水平な面の要素が製品形状の面と同じ高さにあり、重ねると見分けられないため）。
    pub fn load_debug_inverse_offset_shape(&mut self) {
        let meshes = match self
            .cutting_simulation
            .inverse_offset_shape_meshes(&mut self.document, ANGLE_STEP_DEGREES.to_radians())
        {
            Ok(meshes) => meshes,
            Err(error) => {
                tracing::warn!("Cannot show the inverse offset shape: {error}");
                return;
            }
        };
        let counts: Vec<String> = meshes
            .iter()
            .map(|mesh| format!("{:?}={}", mesh.kind, mesh.indices.len() / 3))
            .collect();
        tracing::info!(
            "Inverse offset shape: tool={:?}, step={} deg, triangles {}",
            self.document.operation_content().tool.tool_type(),
            ANGLE_STEP_DEGREES,
            counts.join(" ")
        );
        let items = meshes
            .into_iter()
            .enumerate()
            .map(|(index, mesh)| {
                let item = MeshListItem {
                    vertices: convert_vertex_data_to_mesh_vertices(&mesh.vertices),
                    indices: mesh.indices,
                    color: part_color(mesh.kind),
                };
                (index as u64, item)
            })
            .collect();
        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_list(&self.graphic.device, items);
        if let Some(animation) = &self.cutting_animation {
            mesh_stage.set_overlay_toolpath_line_data(
                &self.graphic.device,
                self.toolpath_lines(animation),
            );
        }
        mesh_stage.set_overlay_toolpath_visible(self.toolpath_lines_visible);
        // ほかのデバッグ表示から切り替えても、通常の表示と同じカメラで描く
        self.enter_debug_view();
        self.use_main_view_camera();
        self.show_debug_stage(mesh_stage, true);
        // 取り込んだ結果の工具と今の計算条件の工具が違うと、ツールパスの線は逆オフセット形状に乗らない
        let tool = &self.document.operation_content().tool;
        // 取り込んだ工程を選んでいるときだけ比べる（ほかの工程の工具と比べても意味がない）
        let imported_selected = self.imported_operation == Some(self.document.selected_operation());
        let imported_tool_differs = imported_selected
            && self.imported_conditions.as_ref().is_some_and(|conditions| {
                conditions.tool.is_ball_end_mill() != tool.is_ball_end_mill()
                    || conditions.tool.radius() != tool.radius()
            });
        if imported_tool_differs {
            self.set_notice(crate::help_ui::Notice::InverseOffsetToolDiffers);
        }
    }
}
