//! AppState の加工条件のウィンドウ（計算条件の入力と投入）を扱うモジュール。

use std::time::{Duration, Instant};

use viewmodel::cutting_simulation_session::OperationJobState;
use viewmodel::space_model_document::MachiningOperationId;

use super::AppState;
use crate::machining_conditions_ui::{
    JobStatusLine, MachiningConditionsUiState, OperationCommand, OperationRow,
};
use viewmodel::message_catalog::UiLocale;

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
    /// 読み直す。待ち・計算中の間は、ジョブマネージャーが止まっていれば起動し直す。状態は別のスレッドで読むため、
    /// ここでは最後に読んだ状態を出す（画面を止めない）。
    fn refresh_job_status(&mut self) {
        // 間隔は、一覧のどれかの工程の状態で決める。まだ読めていない間は、描くたびに確かめる（読むのは別のスレッドで、
        // ここでは待たない）
        let statuses = || {
            std::iter::once(&self.cutting_simulation_job_status)
                .chain(self.operation_job_statuses.iter().map(|(_, status)| status))
        };
        let interval = if statuses().any(|status| *status == JobStatusLine::Reading) {
            Duration::ZERO
        } else if statuses().all(JobStatusLine::is_final) {
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
        let line = |state: OperationJobState| match state {
            OperationJobState::NotRead => JobStatusLine::Reading,
            OperationJobState::NotSubmitted => JobStatusLine::NotSubmitted,
            OperationJobState::Status(status) => JobStatusLine::Status(status),
        };
        // 一覧に出すため、手順のすべての工程の状態を読む（最後に読んだ状態から決めるため、工程の数だけ待つことはない）
        self.operation_job_statuses = match self.cutting_simulation.job_states(&mut self.document) {
            Ok(states) => states
                .into_iter()
                .map(|(operation, state)| (operation, line(state)))
                .collect(),
            Err(error) => {
                if self.cutting_simulation_job_status != JobStatusLine::Unreadable {
                    tracing::warn!("Cannot read the job status: {error}");
                }
                self.document
                    .operations()
                    .into_iter()
                    .map(|item| (item.id, JobStatusLine::Unreadable))
                    .collect()
            }
        };
        let status = self.operation_job_status(self.document.selected_operation());
        // どれかの工程が待ち・計算中なら、ジョブマネージャーが止まっていれば起動し直す
        let pending = self
            .operation_job_statuses
            .iter()
            .any(|(_, status)| status.is_pending());
        if pending {
            if let Err(error) = self.cutting_simulation.run_queued_jobs() {
                tracing::warn!("Cannot start the job manager: {error}");
            }
        }
        self.cutting_simulation_job_status = status;
    }

    /// 加工工程 `operation` の、最後に読んだ計算の状態（まだ読んでいなければ、読んでいる）
    fn operation_job_status(&self, operation: MachiningOperationId) -> JobStatusLine {
        self.operation_job_statuses
            .iter()
            .find(|(id, _)| *id == operation)
            .map_or(JobStatusLine::Reading, |(_, status)| status.clone())
    }

    /// 選んだ加工工程が替わった後（文書を替えたときを含む）に、前の工程について持っていた画面の状態を捨てる
    pub(crate) fn forget_selected_operation_state(&mut self) {
        self.invalidate_conditions();
        self.cutting_simulation_job_status = JobStatusLine::Reading;
        self.cutting_simulation_status = None;
        self.operation_name_input = None;
    }

    /// 加工工程の一覧の操作 `command` を行う
    fn run_operation_command(&mut self, command: OperationCommand) {
        let before = self.document.selected_operation();
        match command {
            OperationCommand::Select(operation) => {
                self.document.select_operation(operation);
            }
            OperationCommand::Add => {
                let names: Vec<String> = self
                    .document
                    .operations()
                    .into_iter()
                    .map(|item| item.name)
                    .collect();
                // 使っていない最も小さい番号の名前にする（消した後に、同じ名前を作らないように）
                let name = (1..)
                    .map(|number| new_operation_name(self.ui_locale, number))
                    .find(|name| !names.contains(name))
                    .expect("an unused number exists");
                self.document.add_operation(name);
            }
            OperationCommand::Duplicate => {
                let name =
                    copied_operation_name(self.ui_locale, &self.document.operation_content().name);
                self.document.duplicate_operation(name);
            }
            OperationCommand::Remove => {
                self.document.remove_selected_operation();
            }
            OperationCommand::Move(offset) => {
                // 並べ替えは、選んだ工程を替えない
                self.document.move_selected_operation(offset);
                self.job_status_read_at = None;
            }
            OperationCommand::Rename(name) => {
                self.operation_name_input = None;
                self.document.rename_selected_operation(&name);
            }
        }
        if self.document.selected_operation() != before {
            self.forget_selected_operation_state();
        }
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
        let selected = self.document.selected_operation();
        let operations = self
            .document
            .operations()
            .into_iter()
            .map(|item| OperationRow {
                status: self.operation_job_status(item.id),
                id: item.id,
                name: item.name,
            })
            .collect();
        MachiningConditionsUiState {
            open: self.machining_conditions_open,
            locale: self.ui_locale,
            operations,
            selected_operation: selected,
            operation_name: self
                .operation_name_input
                .clone()
                .unwrap_or_else(|| self.document.operation_content().name.clone()),
            operation_command: None,
            face_count: self.document.shapes().visible_face_ids().len(),
            operation: self.document.operation_content().clone(),
            stock: self.document.stock(),
            tool_length_input: self.tool_length_input,
            toolpath_tolerance: self.document.toolpath_tolerance(),
            issues: self.condition_issues.clone().unwrap_or_default(),
            // 取り込んだ条件は、取り込んだ工程を選んでいるときだけ出す
            imported: self
                .cutting_animation_job_id
                .zip(self.imported_conditions.clone())
                .filter(|_| self.imported_operation == Some(selected)),
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
        // 工程の一覧の操作は、条件の編集より先に行う（選び直した工程に、前の工程の条件を書かないように）
        if let Some(command) = state.operation_command {
            self.run_operation_command(command);
            return;
        }
        // 入力している名前（工程の名前と同じに戻したら、入力していないとする）
        self.operation_name_input = (state.operation_name
            != self.document.operation_content().name)
            .then(|| state.operation_name.clone());
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

/// 言語 `locale` の、`number` 番目の新しい加工工程の名前
fn new_operation_name(locale: UiLocale, number: usize) -> String {
    crate::machining_conditions_messages::resolve_machining_conditions_text(
        locale,
        "conditions.operations.new_name",
        &[("number", number.to_string())],
    )
}

/// 言語 `locale` の、加工工程 `name` を複写した工程の名前
fn copied_operation_name(locale: UiLocale, name: &str) -> String {
    crate::machining_conditions_messages::resolve_machining_conditions_text(
        locale,
        "conditions.operations.copy_name",
        &[("name", name.to_string())],
    )
}
