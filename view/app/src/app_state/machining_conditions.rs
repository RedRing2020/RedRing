//! AppState の加工条件のウィンドウ（計算条件の入力と投入）を扱うモジュール。

use std::time::{Duration, Instant};

use super::AppState;
use crate::machining_conditions_ui::{JobStatusLine, MachiningConditionsUiState};

/// 計算の状態を読み直す間隔
const JOB_STATUS_INTERVAL: Duration = Duration::from_secs(1);
/// 終わった計算の状態を読み直す間隔（ジョブマネージャーでやり直す・消すと変わる）
const FINISHED_JOB_STATUS_INTERVAL: Duration = Duration::from_secs(5);

impl AppState {
    pub fn toggle_machining_conditions(&mut self) {
        self.machining_conditions_open = !self.machining_conditions_open;
        tracing::info!(
            "Machining conditions window {} (Shift+M)",
            if self.machining_conditions_open {
                "shown"
            } else {
                "hidden"
            }
        );
    }

    /// 計算条件を確かめ直す必要がある場合（条件・製品形状が変わった後）に確かめる
    fn refresh_condition_issues(&mut self) {
        if self.condition_issues.is_none() {
            self.condition_issues = Some(self.cutting_simulation.check(&mut self.document));
        }
    }

    /// 加工条件のウィンドウを開いている間、加工工程の今の改訂の計算の状態を、一定の間隔で読み直す
    ///
    /// 終わった状態（正常・失敗・取り消し）は、ジョブマネージャーでやり直す・消すまで変わらないため、間隔を長くして
    /// 読み直す。待ち・計算中の間は、ジョブマネージャーが止まっていれば起動し直す。
    fn refresh_job_status(&mut self) {
        let interval = if self.cutting_simulation_job_status.is_final() {
            FINISHED_JOB_STATUS_INTERVAL
        } else {
            JOB_STATUS_INTERVAL
        };
        let due = self
            .job_status_read_at
            .is_none_or(|read_at| read_at.elapsed() >= interval);
        if !due {
            return;
        }
        self.job_status_read_at = Some(Instant::now());
        let status = match self.cutting_simulation.job_status(&mut self.document) {
            Ok(None) => JobStatusLine::NotSubmitted,
            Ok(Some(status)) => JobStatusLine::Status(status),
            Err(error) => {
                if self.cutting_simulation_job_status != JobStatusLine::Unreadable {
                    tracing::warn!("Cannot read the job status: {error}");
                }
                JobStatusLine::Unreadable
            }
        };
        if status.is_pending() {
            if let Err(error) = self.cutting_simulation.run_queued_jobs() {
                tracing::warn!("Cannot start the job manager: {error}");
            }
        }
        self.cutting_simulation_job_status = status;
    }

    /// 条件・製品形状が変わった後に、計算条件と計算の状態を確かめ直す
    pub(crate) fn invalidate_conditions(&mut self) {
        self.condition_issues = None;
        self.job_status_read_at = None;
    }

    pub(crate) fn prepare_machining_conditions_state(&mut self) -> MachiningConditionsUiState {
        if self.machining_conditions_open {
            self.refresh_condition_issues();
            self.refresh_job_status();
        }
        MachiningConditionsUiState {
            open: self.machining_conditions_open,
            locale: self.ui_locale,
            face_count: self.document.shapes().visible_face_ids().len(),
            operation: self.document.operation_content().clone(),
            stock: self.document.stock(),
            tool_length_input: self.tool_length_input,
            toolpath_tolerance: self.document.toolpath_tolerance(),
            issues: self.condition_issues.clone().unwrap_or_default(),
            imported: self
                .cutting_animation_job_id
                .zip(self.imported_conditions.clone()),
            job_status: self.cutting_simulation_job_status.clone(),
            imported_toolpath: self.document.imported_toolpath_state(),
            status: self.cutting_simulation_status.clone(),
            submit_requested: false,
            simulate_toolpath_requested: false,
            import_requested: false,
            open_job_manager_requested: false,
        }
    }

    pub(crate) fn apply_machining_conditions_state(&mut self, state: MachiningConditionsUiState) {
        self.machining_conditions_open = state.open;
        self.tool_length_input = state.tool_length_input;
        if state.operation != *self.document.operation_content() {
            self.document.set_operation_content(state.operation);
            self.invalidate_conditions();
        }
        if state.stock != self.document.stock() {
            self.document.set_stock(state.stock);
            self.invalidate_conditions();
        }
        if state.submit_requested && self.submit_cutting_simulation() {
            // 投入した計算の状態を、すぐに読み直す
            self.job_status_read_at = None;
        }
        if state.simulate_toolpath_requested && self.submit_imported_toolpath_simulation() {
            self.job_status_read_at = None;
        }
        if state.import_requested {
            self.import_cutting_simulation();
        }
        if state.open_job_manager_requested {
            self.open_job_manager();
        }
    }
}
