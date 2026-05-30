use super::AppState;
use viewmodel::cam_sim_visualization_converter::CamSimulationDemoScenario;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorRestoreDecision {
    Kept(usize),
    Clamped { from: usize, to: usize },
    ResetToHead,
}

trait StageRestorePolicy {
    fn restore_cursor(previous_cursor: usize, frame_count: usize) -> CursorRestoreDecision;
}

struct DefaultStageRestorePolicy;

impl StageRestorePolicy for DefaultStageRestorePolicy {
    fn restore_cursor(previous_cursor: usize, frame_count: usize) -> CursorRestoreDecision {
        // 再読込で系列長が変わっても、可能な限りユーザーの位置感覚を維持する。
        if frame_count == 0 {
            return CursorRestoreDecision::ResetToHead;
        }
        let tail = frame_count.saturating_sub(1);
        let restored = previous_cursor.min(tail);
        if restored == previous_cursor {
            CursorRestoreDecision::Kept(restored)
        } else {
            CursorRestoreDecision::Clamped {
                from: previous_cursor,
                to: restored,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct StageRestoreSnapshot {
    scenario: CamSimulationDemoScenario,
    cursor: usize,
    shaded_mode: bool,
    camera: viewmodel_graphics::Camera,
}

fn build_stage_restore_snapshot(
    current_cam_demo_scenario: Option<CamSimulationDemoScenario>,
    cursor: usize,
    shaded_mode: bool,
    camera: &viewmodel_graphics::Camera,
) -> Option<StageRestoreSnapshot> {
    let scenario = current_cam_demo_scenario?;
    Some(StageRestoreSnapshot {
        scenario,
        cursor,
        shaded_mode,
        camera: camera.clone(),
    })
}

impl AppState {
    pub(super) fn capture_stage_restore_snapshot(&self) -> Option<StageRestoreSnapshot> {
        build_stage_restore_snapshot(
            self.current_cam_demo_scenario,
            self.debug_snapshot.cursor,
            self.debug_snapshot.shaded_mode,
            &self.camera,
        )
    }

    pub(super) fn reload_current_demo_with_stage_restore(&mut self, trigger_label: &str) {
        let Some(snapshot) = self.capture_stage_restore_snapshot() else {
            return;
        };

        // load後に snapshot を適用し、カメラ・カーソル・表示モードを一貫復元する。
        self.load_sample_toolpath_with_scenario(snapshot.scenario, trigger_label);
        self.restore_stage_from_snapshot(snapshot);
    }

    pub(super) fn restore_stage_from_snapshot(&mut self, snapshot: StageRestoreSnapshot) {
        self.camera = snapshot.camera;
        self.update_camera_uniforms();

        let Some(series) = &self.debug_snapshot.series else {
            tracing::warn!("設定再読込後にスナップショット系列が空のため、カーソルを先頭へ初期化");
            self.debug_snapshot.cursor = 0;
            return;
        };

        match DefaultStageRestorePolicy::restore_cursor(snapshot.cursor, series.frames.len()) {
            CursorRestoreDecision::Kept(cursor) => {
                self.debug_snapshot.cursor = cursor;
                tracing::info!("設定再読込: カーソル {} を維持して復元", cursor);
            }
            CursorRestoreDecision::Clamped { from, to } => {
                self.debug_snapshot.cursor = to;
                tracing::info!(
                    "設定再読込: 旧カーソル {} をフレーム末尾 {} へclampして復元",
                    from,
                    to
                );
            }
            CursorRestoreDecision::ResetToHead => {
                self.debug_snapshot.cursor = 0;
                tracing::info!("設定再読込: カーソルを先頭へ復元");
            }
        }

        self.log_current_snapshot_frame(true);

        if snapshot.shaded_mode {
            // 再読込後はワイヤー初期化されるため、保存前がソリッドなら戻す。
            self.toggle_wireframe();
        }
    }
}

#[cfg(test)]
mod tests {
    use viewmodel::cam_sim_visualization_converter::CamSimulationDemoScenario;
    use viewmodel_graphics::Camera;

    use super::{CursorRestoreDecision, DefaultStageRestorePolicy, StageRestorePolicy};

    #[test]
    fn restore_cursor_keeps_index_when_in_range() {
        assert_eq!(
            DefaultStageRestorePolicy::restore_cursor(5, 10),
            CursorRestoreDecision::Kept(5)
        );
    }

    #[test]
    fn restore_cursor_clamps_to_tail_when_out_of_range() {
        assert_eq!(
            DefaultStageRestorePolicy::restore_cursor(12, 8),
            CursorRestoreDecision::Clamped { from: 12, to: 7 }
        );
    }

    #[test]
    fn restore_cursor_resets_to_zero_when_empty() {
        assert_eq!(
            DefaultStageRestorePolicy::restore_cursor(3, 0),
            CursorRestoreDecision::ResetToHead
        );
    }

    #[test]
    fn build_stage_restore_snapshot_returns_none_when_demo_is_not_ready() {
        let camera = Camera::new();

        assert!(
            super::build_stage_restore_snapshot(None, 3, true, &camera).is_none(),
            "missing demo scenario should not produce a stage restore snapshot"
        );
    }

    #[test]
    fn build_stage_restore_snapshot_returns_snapshot_when_demo_is_ready() {
        let camera = Camera::new();

        let snapshot = super::build_stage_restore_snapshot(
            Some(CamSimulationDemoScenario::Success),
            3,
            true,
            &camera,
        )
        .expect("ready demo should produce a stage restore snapshot");

        assert_eq!(snapshot.scenario, CamSimulationDemoScenario::Success);
        assert_eq!(snapshot.cursor, 3);
        assert!(snapshot.shaded_mode);
    }
}
