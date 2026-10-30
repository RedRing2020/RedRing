//! AppState の切削シミュレーションの投入・実行・取り込みと、切削アニメーションの描画を扱うモジュール。

use std::ops::RangeInclusive;

use super::debug_snapshot_state::SnapshotPlaybackMode;
use super::AppState;
use render::mesh_list::MeshListItem;
use render::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};
use stage::MeshStage;
use viewmodel::cutting_animation::{CuttingAnimation, StockBlockMesh};
use viewmodel::cutting_simulation_session::CuttingToolKind;
use viewmodel::octree_converter::WireframeVertex;

/// 重ねて描くメッシュのうち、素材のブロックのキーの範囲（製品形状のキーは 0 から）
const STOCK_BLOCK_KEYS: RangeInclusive<u64> = (1 << 63)..=u64::MAX;

/// 素材のブロック `block` のメッシュのキー（各座標軸のブロックの番号は 2²¹ 未満）
fn stock_block_key(block: [usize; 3]) -> u64 {
    let [x, y, z] = block.map(|index| index as u64);
    STOCK_BLOCK_KEYS.start() | (x << 42) | (y << 21) | z
}

impl AppState {
    /// 表示中の製品形状で、切削シミュレーションを投入する（実行しない）
    pub(crate) fn submit_cutting_simulation(&mut self, tool: CuttingToolKind) {
        match self.cutting_simulation.submit(&self.product_shapes, tool) {
            Ok(()) => tracing::info!(
                ?tool,
                "切削シミュレーションを投入しました（Shift+R で実行、Shift+L で取り込み）"
            ),
            Err(error) => tracing::warn!("切削シミュレーションを投入できません: {error}"),
        }
    }

    /// キューにあるジョブを実行する（ジョブマネージャーの代わり。終わるまで戻らない）
    pub(crate) fn run_cutting_simulation_jobs(&mut self) {
        tracing::info!("キューにあるジョブを実行します");
        let started = std::time::Instant::now();
        match self.cutting_simulation.run_queued_jobs() {
            Ok(executed) => tracing::info!(
                executed,
                "ジョブの実行が終わりました（{:.1} 秒）",
                started.elapsed().as_secs_f64()
            ),
            Err(error) => tracing::warn!("ジョブを実行できません: {error}"),
        }
    }

    /// 正常に実行済みの最も新しい切削シミュレーションを取り込み、製品形状に重ねて再生できるようにする
    pub(crate) fn import_cutting_simulation(&mut self) {
        tracing::info!("切削シミュレーションを取り込みます");
        let started = std::time::Instant::now();
        match self.cutting_simulation.import_latest() {
            Ok(imported) => {
                self.cutting_animation_job_id = Some(imported.job_id);
                self.start_cutting_animation(imported.animation, started);
            }
            Err(error) => tracing::warn!("切削シミュレーションを取り込めません: {error}"),
        }
    }

    fn start_cutting_animation(
        &mut self,
        animation: CuttingAnimation,
        started: std::time::Instant,
    ) {
        self.debug_snapshot.clear();
        self.debug_snapshot.series = Some(animation.snapshot_series("cam_sim"));
        self.rebuild_snapshot_weighted_progress_axis_cache();

        let (mesh_stage, positions) = self.build_cutting_animation_stage(&animation);
        if let Some(fit) = Self::build_camera_fit(&positions) {
            self.apply_camera_fit(fit);
        }
        self.renderer.set_stage(mesh_stage);
        self.cutting_animation = Some(animation);
        self.drawn_animation_frame = None;
        self.update_camera_uniforms();
        self.log_current_snapshot_frame(true);
        tracing::info!(
            "切削シミュレーションを取り込みました（{:.1} 秒。Space で再生、k/j でコマ送り、左上の進捗バーでスクラブ、w でワイヤーフレーム）",
            started.elapsed().as_secs_f64()
        );
    }

    /// 取り込んだ切削アニメーションの、設定で変わる線（ツールパスの線・工具の線）を今の設定で描き直す
    ///
    /// 素材の面と製品形状のメッシュは設定で変わらないため作り直さない。今のコマ・カメラ・ワイヤーフレームの状態は保つ。
    pub(super) fn redraw_cutting_animation(&mut self) {
        let Some(animation) = &self.cutting_animation else {
            tracing::info!(
                "切削シミュレーションが取り込まれていないため、設定は次の取り込みから反映します"
            );
            return;
        };
        let toolpath_lines = line_vertices(
            &animation.toolpath_wireframe(&self.toolpath_visualization_settings),
            None,
        );
        let tool_lines = line_vertices(
            &animation.tool_wireframe(&self.tool_wireframe_visualization_settings),
            Some(self.snapshot_shaded_color_settings.tool_wire_color),
        );
        let device = &self.graphic.device;
        let Some(mesh_stage) = self
            .renderer
            .get_stage_mut()
            .as_any_mut()
            .downcast_mut::<MeshStage>()
        else {
            tracing::warn!("切削アニメーションの表示中ではないため、設定を反映できません");
            return;
        };
        mesh_stage.set_overlay_toolpath_line_data(device, toolpath_lines);
        mesh_stage.set_overlay_tool_line_data(device, tool_lines);
        self.update_camera_uniforms();
        tracing::info!("ツールパスの線と工具の線を、変えた設定で描き直しました");
    }

    /// 切削アニメーションの今のコマのメッシュステージ（製品形状・素材の面・ツールパスの線・工具の線）と、素材の面の頂点の位置
    fn build_cutting_animation_stage(
        &self,
        animation: &CuttingAnimation,
    ) -> (Box<MeshStage>, Vec<[f32; 3]>) {
        let stock_blocks = animation.stock_blocks();
        let positions: Vec<[f32; 3]> = stock_blocks
            .iter()
            .flat_map(|block| block.vertices.iter().map(|vertex| vertex.position))
            .collect();
        let mut items: Vec<(u64, MeshListItem)> = self
            .product_shapes
            .visible_meshes()
            .into_iter()
            .enumerate()
            .map(|(index, mesh)| {
                let item = MeshListItem {
                    vertices: convert_vertex_data_to_mesh_vertices(&mesh.vertices),
                    indices: mesh.indices,
                    color: mesh.color,
                };
                (index as u64, item)
            })
            .collect();
        let stock_color = self.snapshot_shaded_color_settings.work_solid_color;
        items.extend(
            stock_blocks
                .into_iter()
                .map(|block| stock_block_item(block, stock_color)),
        );

        let device = &self.graphic.device;
        let mut mesh_stage = Box::new(MeshStage::new(device, self.graphic.config.format));
        mesh_stage.set_mesh_list(device, items);
        mesh_stage.set_overlay_toolpath_visible(self.toolpath_lines_visible);
        mesh_stage.set_overlay_tool_visible(self.tool_lines_visible);
        mesh_stage.set_overlay_toolpath_line_data(
            device,
            line_vertices(
                &animation.toolpath_wireframe(&self.toolpath_visualization_settings),
                None,
            ),
        );
        mesh_stage.set_overlay_tool_line_data(
            device,
            line_vertices(
                &animation.tool_wireframe(&self.tool_wireframe_visualization_settings),
                Some(self.snapshot_shaded_color_settings.tool_wire_color),
            ),
        );
        (mesh_stage, positions)
    }

    /// 切削アニメーションのツールパスの線の表示・非表示を切り替える
    pub(super) fn toggle_toolpath_lines(&mut self) {
        self.toolpath_lines_visible = !self.toolpath_lines_visible;
        let visible = self.toolpath_lines_visible;
        if let Some(mesh_stage) = self.cutting_animation_stage() {
            mesh_stage.set_overlay_toolpath_visible(visible);
        }
        tracing::info!(
            "ツールパスの線を{}にしました（v キー）",
            if visible { "表示" } else { "非表示" }
        );
    }

    /// 切削アニメーションの工具の線の表示・非表示を切り替える
    pub(super) fn toggle_tool_lines(&mut self) {
        self.tool_lines_visible = !self.tool_lines_visible;
        let visible = self.tool_lines_visible;
        if let Some(mesh_stage) = self.cutting_animation_stage() {
            mesh_stage.set_overlay_tool_visible(visible);
        }
        tracing::info!(
            "工具の線を{}にしました（Shift+V キー）",
            if visible { "表示" } else { "非表示" }
        );
    }

    /// 切削アニメーションの表示中のメッシュステージ（表示中でなければ `None`）
    fn cutting_animation_stage(&mut self) -> Option<&mut MeshStage> {
        self.cutting_animation.as_ref()?;
        self.renderer
            .get_stage_mut()
            .as_any_mut()
            .downcast_mut::<MeshStage>()
    }

    /// 切削アニメーションを今のコマへ移し、素材のブロックと工具の形を差し替える（取り込んでいなければ `false`）
    pub(super) fn sync_cutting_animation_frame(&mut self) -> bool {
        let Some(animation) = &mut self.cutting_animation else {
            return false;
        };
        let previous = self.drawn_animation_frame.unwrap_or(animation.frame());
        let blocks = animation.seek(self.debug_snapshot.cursor);
        // 速い自動再生で画面の 1 回の描画に工具が大きく進む間は、残像で工具が複数に見えるため工具の線を描かない
        // （コマの移動は描画より多く起きることがあるため、前に画面を描いたときの時点から測る）
        let auto_playing = self.debug_snapshot.playback_mode == SnapshotPlaybackMode::AutoPlaying;
        let tool_lines = if auto_playing && animation.travels_too_far_to_draw_tool(previous) {
            Vec::new()
        } else {
            line_vertices(
                &animation.tool_wireframe(&self.tool_wireframe_visualization_settings),
                Some(self.snapshot_shaded_color_settings.tool_wire_color),
            )
        };
        let stock_color = self.snapshot_shaded_color_settings.work_solid_color;
        let device = &self.graphic.device;
        let stage = self.renderer.get_stage_mut();
        let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() else {
            return true;
        };
        for block in blocks {
            let (key, item) = stock_block_item(block, stock_color);
            mesh_stage.set_mesh_list_item(device, key, item);
        }
        mesh_stage.set_overlay_tool_line_data(device, tool_lines);
        true
    }
}

fn stock_block_item(block: StockBlockMesh, color: [f32; 4]) -> (u64, MeshListItem) {
    let item = MeshListItem {
        vertices: convert_vertex_data_to_mesh_vertices(&block.vertices),
        indices: block.indices,
        color,
    };
    (stock_block_key(block.block), item)
}

/// 線の頂点を描画用にする（`color` を指定しない場合は頂点の色）
fn line_vertices(vertices: &[WireframeVertex], color: Option<[f32; 3]>) -> Vec<MeshVertex> {
    vertices
        .iter()
        .map(|vertex| MeshVertex::new(vertex.position, color.unwrap_or(vertex.color)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_block_keys_are_distinct_and_above_product_shape_keys() {
        let blocks = [[0, 0, 0], [1, 0, 0], [0, 1, 0], [0, 0, 1], [15, 15, 15]];
        let keys: Vec<u64> = blocks.iter().map(|&block| stock_block_key(block)).collect();
        for (index, key) in keys.iter().enumerate() {
            assert!(STOCK_BLOCK_KEYS.contains(key));
            assert!(!keys[index + 1..].contains(key));
        }
        // 製品形状のキー（0 から）は素材のブロックの範囲に入らない
        assert!(!STOCK_BLOCK_KEYS.contains(&1_000_000));
    }
}
