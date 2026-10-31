//! AppState の切削シミュレーションの投入・実行・取り込みと、切削アニメーションの描画を扱うモジュール。

use std::ops::RangeInclusive;

use super::AppState;
use crate::machining_conditions_ui::OperationStatus;
use render::mesh_list::MeshListItem;
use render::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};
use stage::MeshStage;
use viewmodel::cutting_animation::{CuttingAnimation, StockBlockMesh};
use viewmodel::cutting_simulation_session::CuttingSimulationSessionError;
use viewmodel::octree_converter::WireframeVertex;

/// 取り込んだ直後のコマ送りで、ツールパスの全長を分ける数
const FRAME_STEP_DIVISIONS: f64 = 100.0;

/// 重ねて描くメッシュのうち、素材のブロックのキーの範囲（製品形状のキーは 0 から）
const STOCK_BLOCK_KEYS: RangeInclusive<u64> = (1 << 63)..=u64::MAX;

/// 素材のブロック `block` のメッシュのキー（各座標軸のブロックの番号は 2²¹ 未満）
fn stock_block_key(block: [usize; 3]) -> u64 {
    let [x, y, z] = block.map(|index| index as u64);
    STOCK_BLOCK_KEYS.start() | (x << 42) | (y << 21) | z
}

impl AppState {
    /// 表示中の製品形状で、加工条件のウィンドウの計算条件の切削シミュレーションを投入する（実行しない）
    ///
    /// 投入できたかを返す。
    pub(crate) fn submit_cutting_simulation(&mut self) -> bool {
        match self.cutting_simulation.submit(&self.product_shapes) {
            Ok(()) => {
                tracing::info!(
                    conditions = ?self.cutting_simulation.conditions(),
                    "切削シミュレーションを投入しました（Shift+R で実行、Shift+L で取り込み）"
                );
                self.cutting_simulation_status = Some(OperationStatus::Submitted);
                true
            }
            Err(error) => {
                tracing::warn!("切削シミュレーションを投入できません: {error}");
                self.cutting_simulation_status =
                    Some(OperationStatus::SubmitFailed(error.to_string()));
                false
            }
        }
    }

    /// キューにあるジョブを実行する（ジョブマネージャーの代わり。終わるまで戻らない）
    pub(crate) fn run_cutting_simulation_jobs(&mut self) {
        tracing::info!("キューにあるジョブを実行します");
        let started = std::time::Instant::now();
        match self.cutting_simulation.run_queued_jobs() {
            Ok(executed) => {
                let seconds = started.elapsed().as_secs_f64();
                tracing::info!(executed, "ジョブの実行が終わりました（{seconds:.1} 秒）");
                self.cutting_simulation_status = Some(OperationStatus::Ran { executed, seconds });
            }
            Err(error) => {
                tracing::warn!("ジョブを実行できません: {error}");
                self.cutting_simulation_status =
                    Some(OperationStatus::RunFailed(error.to_string()));
            }
        }
    }

    /// 正常に実行済みの最も新しい切削シミュレーションを取り込み、製品形状に重ねて再生できるようにする
    pub(crate) fn import_cutting_simulation(&mut self) {
        tracing::info!("切削シミュレーションを取り込みます");
        let started = std::time::Instant::now();
        match self.cutting_simulation.import_latest() {
            Ok(imported) => {
                self.cutting_simulation_status = Some(OperationStatus::Imported {
                    job: imported.job_id,
                });
                self.cutting_animation_job_id = Some(imported.job_id);
                self.imported_conditions = Some(imported.conditions);
                self.start_cutting_animation(imported.animation, started);
            }
            Err(error) => {
                tracing::warn!("切削シミュレーションを取り込めません: {error}");
                self.cutting_simulation_status = Some(match error {
                    CuttingSimulationSessionError::NoSucceededSimulation => {
                        OperationStatus::NothingToImport
                    }
                    other => OperationStatus::ImportFailed(other.to_string()),
                });
            }
        }
    }

    fn start_cutting_animation(
        &mut self,
        mut animation: CuttingAnimation,
        started: std::time::Instant,
    ) {
        self.debug_snapshot.clear();
        self.debug_snapshot.series = Some(animation.snapshot_series("cam_sim"));
        self.debug_snapshot.frame_step_distance_mm =
            self.initial_frame_step_distance_mm().unwrap_or(0.0);
        self.rebuild_snapshot_weighted_progress_axis_cache();

        let stock_blocks = animation.take_initial_stock_blocks();
        let (mesh_stage, positions) = self.build_cutting_animation_stage(&animation, stock_blocks);
        if let Some(fit) = Self::build_camera_fit(&positions) {
            self.apply_camera_fit(fit);
        }
        self.renderer.set_stage(mesh_stage);
        self.cutting_animation = Some(animation);
        self.update_camera_uniforms();
        self.log_current_snapshot_frame(true);
        tracing::info!(
            "切削シミュレーションを取り込みました（{:.1} 秒。Space で再生、k/j でコマ送り、左上の進捗バーでスクラブ、w でワイヤーフレーム）",
            started.elapsed().as_secs_f64()
        );
    }

    /// コマ送りの移動量の初期値（1 回の送りが進捗バーの約 1% になる、ツールパスの全長の 1/100。取り込んでいなければ `None`）
    pub(super) fn initial_frame_step_distance_mm(&self) -> Option<f64> {
        let last = self.debug_snapshot.series.as_ref()?.frames.last()?;
        Some(last.payload.accumulated_distance_mm / FRAME_STEP_DIVISIONS)
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

    /// 切削アニメーションの最初のコマのメッシュステージ（製品形状・素材の面・ツールパスの線・工具の線）と、素材の面の頂点の位置
    fn build_cutting_animation_stage(
        &self,
        animation: &CuttingAnimation,
        stock_blocks: Vec<StockBlockMesh>,
    ) -> (Box<MeshStage>, Vec<[f32; 3]>) {
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

    /// 頼んだコマの素材の面ができていれば、素材のブロックと工具の形を差し替える
    ///
    /// 工具は、素材の面を表示しているコマの位置に描く（素材の面より先に工具だけが進まない）。
    pub(super) fn apply_cutting_animation_update(&mut self) {
        let Some(animation) = &mut self.cutting_animation else {
            return;
        };
        let Some(blocks) = animation.take_stock_update() else {
            return;
        };
        let tool_lines = line_vertices(
            &animation.tool_wireframe(&self.tool_wireframe_visualization_settings),
            Some(self.snapshot_shaded_color_settings.tool_wire_color),
        );
        let stock_color = self.snapshot_shaded_color_settings.work_solid_color;
        let device = &self.graphic.device;
        let stage = self.renderer.get_stage_mut();
        if let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() {
            for block in blocks {
                let (key, item) = stock_block_item(block, stock_color);
                mesh_stage.set_mesh_list_item(device, key, item);
            }
            mesh_stage.set_overlay_tool_line_data(device, tool_lines);
        }
        // 工具の位置（区間の端点に置いたか）は面を受け取った後に決まるため、タイトルは受け取った後にも出す
        self.refresh_snapshot_window_title();
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
