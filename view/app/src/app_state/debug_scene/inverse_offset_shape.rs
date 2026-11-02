//! AppState の逆オフセット形状（要素の役割ごと）のデバッグ表示を扱うモジュール。

use super::super::AppState;
use render::mesh_list::MeshListItem;
use render::vertex_3d::convert_vertex_data_to_mesh_vertices;
use viewmodel::inverse_offset_shape_converter::InverseOffsetPartKind;

/// 球面・円柱面・円板を三角形に分ける角度のきざみ（度）
const ANGLE_STEP_DEGREES: f64 = 10.0;

/// 重ねて描くメッシュのキーの始まり（製品形状のメッシュの番号より大きく、素材のブロックのキーより小さい）
const PART_KEY_START: u64 = 1 << 62;

/// 種類ごとの色（RGBA）
fn part_color(kind: InverseOffsetPartKind) -> [f32; 4] {
    match kind {
        InverseOffsetPartKind::IndexedFace => [0.95, 0.55, 0.25, 1.0],
        InverseOffsetPartKind::UnindexedFace => [0.55, 0.6, 0.8, 1.0],
        InverseOffsetPartKind::UpperSide => [0.85, 0.35, 0.85, 1.0],
        InverseOffsetPartKind::UnorientedFace => [0.9, 0.15, 0.15, 1.0],
        InverseOffsetPartKind::Vertex => [0.35, 0.8, 0.4, 1.0],
        InverseOffsetPartKind::Edge => [0.3, 0.75, 0.9, 1.0],
    }
}

impl AppState {
    /// デバッグ用: 表示中の製品形状と計算条件の工具で作った逆オフセット形状の要素を、役割ごとの色で製品形状に重ねて
    /// 表示する
    pub fn load_debug_inverse_offset_shape(&mut self) {
        let meshes = match self
            .cutting_simulation
            .inverse_offset_shape_meshes(&self.product_shapes, ANGLE_STEP_DEGREES.to_radians())
        {
            Ok(meshes) => meshes,
            Err(error) => {
                tracing::warn!("inverse offset shape cannot be displayed: {error}");
                return;
            }
        };
        let counts: Vec<String> = meshes
            .iter()
            .map(|mesh| format!("{:?}={}", mesh.kind, mesh.indices.len() / 3))
            .collect();
        tracing::info!(
            "逆オフセット形状: 工具={:?}, きざみ={}°, 三角形 {}",
            self.cutting_simulation.conditions().tool_kind,
            ANGLE_STEP_DEGREES,
            counts.join(" ")
        );
        let overlays = meshes
            .into_iter()
            .enumerate()
            .map(|(index, mesh)| {
                let item = MeshListItem {
                    vertices: convert_vertex_data_to_mesh_vertices(&mesh.vertices),
                    indices: mesh.indices,
                    color: part_color(mesh.kind),
                };
                (PART_KEY_START + index as u64, item)
            })
            .collect();
        self.show_product_shapes_with(overlays);
    }
}
