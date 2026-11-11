//! AppState の切削シミュレーションの投入・実行・取り込みと、切削アニメーションの描画を扱うモジュール。

use std::ops::RangeInclusive;

use super::{AppState, SnapshotShadedColorSettings};
use crate::machining_conditions_ui::OperationStatus;
use render::mesh_list::MeshListItem;
use render::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};
use stage::MeshStage;
use viewmodel::cutting_animation::{CuttingAnimation, DeviationClass, StockBlockMesh};
use viewmodel::cutting_simulation_session::SequenceSubmissionError;
use viewmodel::cutting_simulation_session::{
    CuttingSimulationSessionError, RedoOutcome, StopOutcome, SubmissionKind,
};
use viewmodel::failure_message_mapper::cutting_simulation_failure_message;
use viewmodel::mesh_converter::WireframeVertex;

/// 取り込んだ直後のコマ送りで、ツールパスの全長を分ける数
const FRAME_STEP_DIVISIONS: f64 = 100.0;

/// 重ねて描くメッシュのうち、素材のブロックのキーの範囲（製品形状のキーは 0 から）
const STOCK_BLOCK_KEYS: RangeInclusive<u64> = (1 << 63)..=u64::MAX;

/// 素材の面の分類の、キーの中の位置（分類ごとのキーは続いた範囲になる）
const CLASS_SHIFT: u32 = 60;

/// 素材の面の分類の、キーの中の番号
fn class_index(class: DeviationClass) -> u64 {
    match class {
        DeviationClass::NotCompared => 0,
        DeviationClass::Uncut => 1,
        DeviationClass::Within => 2,
        DeviationClass::Gouge => 3,
    }
}

/// 素材のブロック `block` の、分類 `class` の面のメッシュのキー（各座標軸のブロックの番号は 2²⁰ 未満）
fn stock_block_key(block: [usize; 3], class: DeviationClass) -> u64 {
    let [x, y, z] = block.map(|index| index as u64);
    STOCK_BLOCK_KEYS.start() | (class_index(class) << CLASS_SHIFT) | (x << 40) | (y << 20) | z
}

/// 分類 `class` の素材の面のメッシュのキーの範囲
fn stock_class_keys(class: DeviationClass) -> RangeInclusive<u64> {
    let start = STOCK_BLOCK_KEYS.start() | (class_index(class) << CLASS_SHIFT);
    start..=start | ((1 << CLASS_SHIFT) - 1)
}

/// 分類 `class` の素材の面の色（比べていない所は素材の色）
fn stock_class_color(colors: &SnapshotShadedColorSettings, class: DeviationClass) -> [f32; 4] {
    match class {
        DeviationClass::NotCompared => colors.work_solid_color,
        DeviationClass::Uncut => colors.deviations.uncut_color,
        DeviationClass::Within => colors.deviations.within_color,
        DeviationClass::Gouge => colors.deviations.gouge_color,
    }
}

impl AppState {
    /// 表示中の製品形状で、選んだ加工工程を投入する（実行しない。今の条件の計算が計算待ち・計算中・計算済みなら
    /// 投入しない。今の条件の CL があれば、切削シミュレーションだけ）
    ///
    /// 投入できたかを返す。
    pub(crate) fn submit_cutting_simulation(&mut self) -> bool {
        let Some(kind) = self
            .cutting_simulation
            .submit(&mut self.document)
            .transpose()
        else {
            // 押した後に読んだ状態が、計算待ち・計算中・計算済みだった（ボタンは最後に読んだ状態で決めるため）
            tracing::info!(
                "The machining operation is already waiting, calculating, or calculated"
            );
            self.set_operation_status(OperationStatus::AlreadyCalculated);
            // 押したときに読んだ状態を、すぐ表とボタンに出す
            self.job_status_read_at = None;
            return false;
        };
        let result = kind.map(|kind| {
            let simulation_only = kind == SubmissionKind::SimulationOnly;
            (
                if simulation_only {
                    "the cutting simulation of the imported toolpath"
                } else {
                    "the toolpath generation and the cutting simulation"
                },
                OperationStatus::Submitted { simulation_only },
            )
        });
        self.finish_submission(result, "the machining operation")
    }

    /// 表示中の製品形状で、選んだ加工工程の空間モデルの CL を消して、ツールパス生成から投入し直す（実行しない）。
    /// その工程の切削アニメーションを出していれば閉じる（消した CL の結果を出し続けないように）
    ///
    /// 投入できたかを返す。
    pub(crate) fn redo_cutting_simulation(&mut self) -> bool {
        let result = match self.cutting_simulation.redo(&mut self.document) {
            Ok(RedoOutcome::Waiting) => {
                // 押した後に読んだ状態が計算待ちだった（ボタンは最後に読んだ状態で決めるため）
                tracing::info!("The machining operation is already waiting for the calculation");
                self.set_operation_status(OperationStatus::RecalculationWaiting);
                // 押したときに読んだ状態を、すぐ表とボタンに出す
                self.job_status_read_at = None;
                return false;
            }
            Ok(RedoOutcome::Submitted { canceled }) => Ok((
                "the machining operation again",
                OperationStatus::Redone { canceled },
            )),
            Err(error) => Err(error),
        };
        let submitted = self.finish_submission(result, "the machining operation again");
        if submitted
            && self.cutting_animation.is_some()
            && self.imported_operation == Some(self.document.selected_operation())
        {
            self.close_cutting_animation();
        }
        submitted
    }

    /// 加工手順のすべての加工工程を投入する（実行しない）。1 つでも投入できたかを返す
    pub(crate) fn submit_sequence(&mut self) -> bool {
        match self.cutting_simulation.submit_sequence(&mut self.document) {
            Ok(submission) => {
                tracing::info!(
                    submitted = submission.submitted,
                    simulation_only = submission.simulation_only,
                    skipped = submission.skipped,
                    "Submitted the machining sequence to the job manager"
                );
                self.set_operation_status(OperationStatus::SequenceSubmitted {
                    submitted: submission.submitted,
                    simulation_only: submission.simulation_only,
                    skipped: submission.skipped,
                });
                submission.submitted > 0
            }
            Err(failure) => {
                tracing::warn!("Cannot submit the machining sequence: {failure}");
                match failure {
                    SequenceSubmissionError::StatusUnreadable(error) => {
                        self.set_operation_status(OperationStatus::SequenceStatusUnreadable(
                            cutting_simulation_failure_message(&error),
                        ));
                        false
                    }
                    SequenceSubmissionError::Conditions {
                        position,
                        operation,
                        error,
                    } => {
                        self.set_operation_status(OperationStatus::SequenceSubmitFailed {
                            position,
                            operation,
                            submitted: 0,
                            reason: cutting_simulation_failure_message(&error),
                        });
                        false
                    }
                    SequenceSubmissionError::Submit {
                        position,
                        operation,
                        submitted,
                        error,
                    } => {
                        self.set_operation_status(OperationStatus::SequenceSubmitFailed {
                            position,
                            operation,
                            submitted,
                            reason: cutting_simulation_failure_message(&error),
                        });
                        // 前の工程までは投入したため、状態を読み直す
                        submitted > 0
                    }
                }
            }
        }
    }

    /// 投入の結果 `result`（投入できたら、ログに出す投入したものの名前と、最後の操作の結果。`what` は、投入できない
    /// ときにログに出す投入の名前）を、最後の操作の結果にする。投入できたかを返す
    fn finish_submission(
        &mut self,
        result: Result<(&str, OperationStatus), CuttingSimulationSessionError>,
        what: &str,
    ) -> bool {
        match result {
            Ok((submitted, status)) => {
                tracing::info!(
                    operation = ?self.document.operation_content(),
                    stock = ?self.document.stock(),
                    "Submitted {submitted} to the job manager (import with Shift+L after it finishes)"
                );
                self.set_operation_status(status);
                true
            }
            Err(error) => {
                tracing::warn!("Cannot submit {what}: {error}");
                self.set_operation_status(OperationStatus::SubmitFailed(
                    cutting_simulation_failure_message(&error),
                ));
                false
            }
        }
    }

    /// 選んだ加工工程の今の条件の、計算待ち・計算中の計算を止める（取り消すよう頼み、止まるのを待たない）
    pub(crate) fn stop_cutting_simulation(&mut self) {
        let status = match self.cutting_simulation.stop(&self.document) {
            Ok(StopOutcome::Stopped) => {
                tracing::info!("Requested the job manager to cancel the calculation");
                OperationStatus::Stopped
            }
            Ok(StopOutcome::NotYetAccepted) => {
                tracing::info!("The job manager has not received the calculation yet");
                OperationStatus::StopNotYetAccepted
            }
            Ok(StopOutcome::NothingToStop) => OperationStatus::NothingToStop,
            Err(error) => {
                tracing::warn!("Cannot stop the calculation: {error}");
                OperationStatus::StopFailed(cutting_simulation_failure_message(&error))
            }
        };
        self.set_operation_status(status);
        // 止めた計算の状態を、すぐに読み直す
        self.job_status_read_at = None;
    }

    /// ジョブマネージャーのウィンドウを開く（動いていなければ、ウィンドウを出して起動する）
    pub(crate) fn open_job_manager(&mut self) {
        match self.cutting_simulation.open_job_manager() {
            Ok(()) => tracing::info!("Opened the job manager window"),
            Err(error) => {
                tracing::warn!("Cannot open the job manager: {error}");
                self.set_operation_status(OperationStatus::JobManagerFailed(
                    cutting_simulation_failure_message(&error),
                ));
            }
        }
    }

    /// 選んだ加工工程の、今の条件で正常に終わった最も新しいツールパス生成の CL だけを、空間モデルに取り込む（切削
    /// アニメーションは開かない）
    pub(crate) fn import_toolpath(&mut self) {
        match self.cutting_simulation.import_toolpath(&mut self.document) {
            Ok(job) => {
                tracing::info!(job, "Imported the toolpath into the space model");
                self.set_operation_status(OperationStatus::ToolpathImported { job });
                // 同じ状態の CL を取り込み直しても、線を描き直す
                self.refresh_operation_toolpath_lines();
            }
            Err(error) => {
                tracing::warn!("Cannot import the toolpath: {error}");
                self.set_operation_status(match error {
                    CuttingSimulationSessionError::NoSucceededToolpath => {
                        OperationStatus::NoToolpathToImport
                    }
                    other => {
                        OperationStatus::ImportFailed(cutting_simulation_failure_message(&other))
                    }
                });
            }
        }
    }

    /// 選んだ工程の切削アニメーションを出しているか（ほかの工程を選ぶと閉じるため、出していれば選んだ工程のもの）
    pub(crate) fn cutting_simulation_shown(&self) -> bool {
        self.cutting_animation.is_some()
            && self.imported_operation == Some(self.document.selected_operation())
    }

    /// 選んだ工程の切削シミュレーションの表示を切り替える。出していれば、切削アニメーションを閉じて、製品形状と選んだ
    /// 工程の CL の線に戻す（メモリを放す。カメラは動かさない）。出していなければ、取り込んで出す
    pub(crate) fn toggle_cutting_simulation(&mut self) {
        if !self.cutting_simulation_shown() {
            self.import_cutting_simulation();
            return;
        }
        // 条件の編集のダイアログを開いている間は、取り込みと同じく切り替えない
        if self.is_condition_dialog_open() {
            return;
        }
        self.close_cutting_animation();
        tracing::info!("Cutting simulation display closed");
    }

    /// 正常に実行済みの最も新しい切削シミュレーションを取り込み、製品形状に重ねて再生できるようにする
    pub(crate) fn import_cutting_simulation(&mut self) {
        // CAM の表示は、加工手順を開いている間だけ出す。条件の編集のダイアログを開いている間は取り込まない
        if !self.machining_sequence_open || self.is_condition_dialog_open() {
            tracing::debug!("The machining sequence is not open; nothing is imported");
            return;
        }
        tracing::info!("Importing the cutting simulation");
        let started = std::time::Instant::now();
        match self.cutting_simulation.import_latest(&mut self.document) {
            Ok(imported) => {
                self.set_operation_status(OperationStatus::Imported {
                    job: imported.job_id,
                });
                self.cutting_animation_job_id = Some(imported.job_id);
                self.imported_conditions = Some(imported.conditions);
                self.imported_operation = Some(self.document.selected_operation());
                self.start_cutting_animation(imported.animation, started);
            }
            Err(error) => {
                tracing::warn!("Cannot import the cutting simulation: {error}");
                self.set_operation_status(match error {
                    CuttingSimulationSessionError::NoSucceededSimulation => {
                        OperationStatus::NothingToImport
                    }
                    other => {
                        OperationStatus::ImportFailed(cutting_simulation_failure_message(&other))
                    }
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

        self.cutting_animation_colors = self.snapshot_shaded_color_settings;
        animation.set_deviations_shown(self.cutting_animation_colors.deviations.shown);
        let stock_blocks = animation.take_initial_stock_blocks();
        let (mesh_stage, positions) = self.build_cutting_animation_stage(&animation, stock_blocks);
        // 取り込み直しても見ていた向き・拡大を保つ。カメラは STL の読み込みで製品形状に合わせているため、表示中の
        // 製品形状がない場合だけ合わせる
        if self.document.shapes().visible_face_ids().is_empty() {
            if let Some(fit) = Self::build_camera_fit(&positions) {
                self.apply_camera_fit(fit);
            }
        }
        self.discard_main_view();
        self.renderer.set_stage(mesh_stage);
        self.cutting_animation = Some(animation);
        self.update_camera_uniforms();
        self.log_current_snapshot_frame(true);
        tracing::info!(
            "Imported the cutting simulation ({:.1} s; Space to play, k/j to step, drag the progress bar to scrub, w for wireframe)",
            started.elapsed().as_secs_f64()
        );
    }

    /// コマ送りの移動量の初期値（1 回の送りが進捗バーの約 1% になる、ツールパスの全長の 1/100。取り込んでいなければ `None`）
    pub(super) fn initial_frame_step_distance_mm(&self) -> Option<f64> {
        let last = self.debug_snapshot.series.as_ref()?.frames.last()?;
        Some(last.payload.accumulated_distance_mm / FRAME_STEP_DIVISIONS)
    }

    /// 取り込んだ切削アニメーションの、設定で変わる線（ツールパスの線・工具の線）と素材の色を今の設定で描き直す
    ///
    /// 素材の面と製品形状のメッシュは作り直さない（素材は色だけを変える）。ただし、最後のコマで色分けの入・切が変わった
    /// 場合は、素材の面がすべて作り直されて届く。今のコマ・カメラ・ワイヤーフレームの状態は保つ。
    pub(super) fn redraw_cutting_animation(&mut self) {
        let Some(animation) = &self.cutting_animation else {
            // 切削アニメーションがなければ、選んだ工程の CL の線を描き直す
            self.refresh_operation_toolpath_lines();
            self.update_camera_uniforms();
            return;
        };
        let toolpath_lines = self.toolpath_lines(animation);
        let colors = self.snapshot_shaded_color_settings;
        let tool_lines = line_vertices(
            &animation.tool_wireframe(&self.tool_wireframe_visualization_settings),
            Some(colors.tool_wire_color),
        );
        let device = self.graphic.device.clone();
        let device = &device;
        let Some(mesh_stage) = self.main_mesh_stage() else {
            tracing::warn!("Cannot apply the settings because no cutting animation is shown");
            self.set_notice(crate::help_ui::Notice::ApplyWithoutAnimation);
            return;
        };
        mesh_stage.set_overlay_toolpath_line_data(device, toolpath_lines.clone());
        mesh_stage.set_overlay_tool_line_data(device, tool_lines);
        for class in [
            DeviationClass::NotCompared,
            DeviationClass::Uncut,
            DeviationClass::Within,
            DeviationClass::Gouge,
        ] {
            mesh_stage.set_mesh_list_color(
                device,
                stock_class_keys(class),
                stock_class_color(&colors, class),
            );
        }
        // ツールパスの線を重ねたデバッグ表示にも、同じ線を描く
        if let Some(debug_stage) = self.debug_toolpath_overlay_stage() {
            debug_stage.set_overlay_toolpath_line_data(device, toolpath_lines);
        }
        // 色分けするかが変わった場合は、最後の時点の素材の面が作り直されて届く
        if let Some(animation) = &mut self.cutting_animation {
            animation.set_deviations_shown(colors.deviations.shown);
        }
        // 後から届く素材のブロックと工具の線も、この色で描く
        self.cutting_animation_colors = colors;
        self.update_camera_uniforms();
        tracing::info!(
            "Redrew the toolpath lines, tool lines and stock colors with the changed settings"
        );
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
            .document
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
        let colors = self.cutting_animation_colors;
        items.extend(
            stock_blocks
                .into_iter()
                .flat_map(|block| stock_block_items(block, &colors)),
        );

        let device = &self.graphic.device;
        let mut mesh_stage = Box::new(MeshStage::new(device, self.graphic.config.format));
        mesh_stage.set_mesh_list(device, items);
        mesh_stage.set_overlay_toolpath_visible(self.toolpath_lines_visible);
        mesh_stage.set_overlay_tool_visible(self.tool_lines_visible);
        mesh_stage.set_overlay_toolpath_line_data(device, self.toolpath_lines(animation));
        mesh_stage.set_overlay_tool_line_data(
            device,
            line_vertices(
                &animation.tool_wireframe(&self.tool_wireframe_visualization_settings),
                Some(self.cutting_animation_colors.tool_wire_color),
            ),
        );
        (mesh_stage, positions)
    }

    /// 切削アニメーション `animation` のツールパスの線の頂点（今の表示の設定で作る）
    pub(super) fn toolpath_lines(&self, animation: &CuttingAnimation) -> Vec<MeshVertex> {
        line_vertices(
            &animation.toolpath_wireframe(&self.toolpath_visualization_settings),
            None,
        )
    }

    /// 切削アニメーションのツールパスの線の表示・非表示を切り替える（デバッグ用の表示に重ねた線も同じに切り替える）
    pub(super) fn toggle_toolpath_lines(&mut self) {
        self.toolpath_lines_visible = !self.toolpath_lines_visible;
        let visible = self.toolpath_lines_visible;
        // 切削アニメーションがなくても、選んだ工程の CL の線を重ねている
        if let Some(mesh_stage) = self.main_mesh_stage() {
            mesh_stage.set_overlay_toolpath_visible(visible);
        }
        if let Some(mesh_stage) = self.debug_toolpath_overlay_stage() {
            mesh_stage.set_overlay_toolpath_visible(visible);
        }
        tracing::info!(
            "Toolpath lines {} (v key)",
            if visible { "shown" } else { "hidden" }
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
            "Tool lines {} (Shift+V key)",
            if visible { "shown" } else { "hidden" }
        );
    }

    /// 切削アニメーションの表示中のメッシュステージ（表示中でなければ `None`）
    fn cutting_animation_stage(&mut self) -> Option<&mut MeshStage> {
        self.cutting_animation.as_ref()?;
        self.main_mesh_stage()
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
            Some(self.cutting_animation_colors.tool_wire_color),
        );
        let colors = self.cutting_animation_colors;
        let device = self.graphic.device.clone();
        let device = &device;
        if let Some(mesh_stage) = self.main_mesh_stage() {
            for block in blocks {
                // 三角形のない分類の面も渡し、前の面の同じ分類のメッシュを除く
                for (key, item) in stock_block_items(block, &colors) {
                    mesh_stage.set_mesh_list_item(device, key, item);
                }
            }
            mesh_stage.set_overlay_tool_line_data(device, tool_lines);
        }
        // 工具の位置（区間の端点に置いたか）は面を受け取った後に決まるため、タイトルは受け取った後にも出す
        self.refresh_snapshot_window_title();
    }
}

/// ブロックの素材の面の、分類ごとのメッシュ（三角形のない分類は空のメッシュ）
fn stock_block_items(
    block: StockBlockMesh,
    colors: &SnapshotShadedColorSettings,
) -> [(u64, MeshListItem); 4] {
    let key = block.block;
    block.into_parts().map(|(class, vertices, indices)| {
        let item = MeshListItem {
            vertices: convert_vertex_data_to_mesh_vertices(&vertices),
            indices,
            color: stock_class_color(colors, class),
        };
        (stock_block_key(key, class), item)
    })
}

/// 線の頂点を描画用にする（`color` を指定しない場合は頂点の色）
pub(super) fn line_vertices(
    vertices: &[WireframeVertex],
    color: Option<[f32; 3]>,
) -> Vec<MeshVertex> {
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
        let classes = [
            DeviationClass::NotCompared,
            DeviationClass::Uncut,
            DeviationClass::Within,
            DeviationClass::Gouge,
        ];
        let keys: Vec<(DeviationClass, u64)> = classes
            .iter()
            .flat_map(|&class| {
                blocks
                    .iter()
                    .map(move |&block| (class, stock_block_key(block, class)))
            })
            .collect();
        for (index, (class, key)) in keys.iter().enumerate() {
            assert!(STOCK_BLOCK_KEYS.contains(key));
            assert!(!keys[index + 1..].iter().any(|(_, other)| other == key));
            // 分類ごとのキーの範囲に入り、ほかの分類の範囲には入らない
            for other in classes {
                assert_eq!(stock_class_keys(other).contains(key), other == *class);
            }
        }
        // 製品形状のキー（0 から）は素材のブロックの範囲に入らない
        assert!(!STOCK_BLOCK_KEYS.contains(&1_000_000));
    }
}
