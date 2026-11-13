//! AppState の加工手順の画面（工程リストと条件の編集のダイアログ。計算条件の入力と投入）を扱うモジュール。

use std::time::{Duration, Instant};

use viewmodel::cutting_simulation_session::{OperationJobState, OperationJobStatus};
use viewmodel::space_model_document::{ImportedToolpathState, MachiningOperationId};

use super::AppState;
use crate::help_ui::Notice;
use crate::machining_conditions_ui::{
    ApplySubmission, ConditionDialog, DialogResult, JobStatusLine, MachiningConditionsUiState,
    OperationCommand, OperationRow, OperationSetup, submission_on_apply,
};
use viewmodel::cutting_simulation_session::{
    ConditionIssue, MachiningOperationContent, MachiningStock,
};
use viewmodel::message_catalog::UiLocale;

/// 計算の状態を読み直す間隔
const JOB_STATUS_INTERVAL: Duration = Duration::from_secs(1);
/// 終わった計算の状態を読み直す間隔（ジョブマネージャーでやり直す・消すと変わる）
const FINISHED_JOB_STATUS_INTERVAL: Duration = Duration::from_secs(5);

/// 条件の編集のダイアログで編集している条件（適用するまで、工程・手順に入れない）
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ConditionDraft {
    dialog: ConditionDialog,
    content: MachiningOperationContent,
    stock: MachiningStock,
    /// 工具長を指定するときの入力値（キャンセルしたら、前の値に戻す）
    tool_length_input: f64,
}

/// 編集している条件（工程の中身・手順の素材）と、その条件の問題
#[derive(Debug, Clone)]
pub(super) struct DraftIssues {
    content: MachiningOperationContent,
    stock: MachiningStock,
    issues: Vec<ConditionIssue>,
}

impl AppState {
    /// 加工手順を開き、工程リストを出す（開いている場合は何もしない）
    ///
    /// 開いている間は、選んだ加工工程の CL の線を出し、工程リストで計算の状態を読む。
    pub(crate) fn open_machining_sequence(&mut self) {
        if self.machining_sequence_open {
            return;
        }
        // 開く前に読んだ状態は使わず、読み直す
        self.forget_machining_sequence_state();
        self.machining_sequence_open = true;
        self.operation_list_shown = true;
        tracing::info!("Machining sequence opened");
    }

    /// 加工手順を閉じ、CAM の画面（工程リスト・条件の編集のダイアログ）・表示（CL の線・切削アニメーション）と、計算の状態を
    /// 放す（閉じている場合は何もしない）
    ///
    /// 計算中のジョブは、ジョブマネージャーが続ける。
    pub(crate) fn close_machining_sequence(&mut self) {
        if !self.machining_sequence_open {
            return;
        }
        self.forget_machining_sequence_state();
        if self.cutting_animation.is_some() {
            self.close_cutting_animation();
        } else {
            self.refresh_operation_toolpath_lines();
        }
        tracing::info!("Machining sequence closed");
    }

    /// 加工手順を閉じた状態にし（工程リストも閉じ、編集している条件は適用せずに捨てる）、それらに出す状態（計算の
    /// 状態・条件の確かめ・結果の行）を捨てる（CAM の表示は放さない。表示を放すのは、呼び出す側で製品形状を描き直す
    /// とき）
    pub(super) fn forget_machining_sequence_state(&mut self) {
        self.machining_sequence_open = false;
        self.operation_list_shown = false;
        self.condition_draft = None;
        self.draft_issues = None;
        self.forget_selected_operation_state();
        self.operation_job_statuses.clear();
    }

    /// 加工手順を開いている間に、工程リストを出す・隠す（開いていなければ何もしない）
    pub fn toggle_operation_list(&mut self) {
        if !self.machining_sequence_open || self.is_condition_dialog_open() {
            tracing::debug!("The machining sequence is not open; the operation list is not shown");
            return;
        }
        self.operation_list_shown = !self.operation_list_shown;
        tracing::info!(
            "Operation list {}",
            if self.operation_list_shown {
                "shown"
            } else {
                "hidden"
            }
        );
    }

    /// 加工手順を開いている間に、条件の編集のダイアログ `dialog` を、今の工程・手順の条件で開く（開いていない・ほかの
    /// ダイアログを開いている間は何もしない）
    pub(crate) fn open_condition_dialog(&mut self, dialog: ConditionDialog) {
        if !self.machining_sequence_open || self.condition_draft.is_some() {
            tracing::debug!("The condition dialog is not opened");
            return;
        }
        self.condition_draft = Some(ConditionDraft {
            dialog,
            content: self.document.operation_content().clone(),
            stock: self.document.stock(),
            tool_length_input: self.tool_length_input,
        });
        tracing::info!("Condition dialog opened: {dialog:?}");
    }

    /// 条件の編集のダイアログを開いているか（開いている間は、見る操作のほか、工程・文書を替える操作を受けない）
    pub(crate) fn is_condition_dialog_open(&self) -> bool {
        self.condition_draft.is_some()
    }

    /// 条件の編集のダイアログを、編集した条件を捨てて閉じる
    pub(crate) fn cancel_condition_dialog(&mut self) {
        if self.condition_draft.take().is_some() {
            self.draft_issues = None;
            tracing::info!("Condition dialog canceled");
        }
    }

    /// 編集した条件を工程・手順に入れて、ダイアログを閉じる（今と同じなら、工程の改訂を進めない）。工程の設定で「適用と
    /// 同時に投入する」を選んでいれば、選んだ工程を投入する（計算の条件が変わっていなければ、計算のボタンが押せる状態の
    /// ときだけ。取込済みの工程の属性だけを替えたときに、ジョブマネージャーを起動しないように）
    fn apply_condition_draft(&mut self) {
        let Some(draft) = self.condition_draft.take() else {
            return;
        };
        self.draft_issues = None;
        self.tool_length_input = draft.tool_length_input;
        let mut submission = None;
        match draft.dialog {
            ConditionDialog::Operation => {
                // 名前は、前後の空白を除き、空なら替えない。属性（名前・加工の段階）は計算に使わないため、属性だけを
                // 替えても工程の改訂は進めず、条件の確かめ・計算の状態も読み直さない
                let mut content = draft.content;
                let name = std::mem::replace(
                    &mut content.attributes.name,
                    self.document.operation_content().attributes.name.clone(),
                );
                let current = self.document.operation_content();
                let conditions_changed = content.conditions != current.conditions;
                if content != *current {
                    self.document.set_operation_content(content);
                    if conditions_changed {
                        self.invalidate_conditions();
                    }
                }
                self.document.rename_selected_operation(&name);
                // 条件を適用したので、計算の条件を設定したことにする（条件を変えなくても）
                if self.document.mark_conditions_set() {
                    self.invalidate_conditions();
                }
                if self.submit_on_apply {
                    let operation = self.document.selected_operation();
                    let toolpath_current =
                        self.document.imported_toolpath_state() == ImportedToolpathState::Current;
                    submission = Some(submission_on_apply(
                        conditions_changed,
                        &self.operation_job_status(operation),
                        toolpath_current,
                    ));
                }
            }
            ConditionDialog::Sequence => {
                if draft.stock != self.document.stock() {
                    self.document.set_stock(draft.stock);
                    self.invalidate_conditions();
                }
            }
        }
        tracing::info!("Conditions applied: {:?}", draft.dialog);
        match submission {
            Some(ApplySubmission::Submit) => {
                if self.submit_cutting_simulation() {
                    // 投入した計算の状態を、すぐに読み直す
                    self.job_status_read_at = None;
                }
            }
            Some(ApplySubmission::Skip(status)) => {
                // 計算しなかったことと、その理由を出す
                tracing::info!("The applied operation is not submitted: {status:?}");
                self.set_operation_status(status);
            }
            None => {}
        }
    }

    /// 編集している条件の問題（計算の条件・素材が変わったときだけ確かめ直す。属性は計算に使わない）
    fn draft_issues(&mut self) -> Vec<ConditionIssue> {
        let Some(draft) = &self.condition_draft else {
            return Vec::new();
        };
        let current = self.draft_issues.as_ref().is_some_and(|cached| {
            cached.content.conditions == draft.content.conditions && cached.stock == draft.stock
        });
        if !current {
            let (content, stock) = (draft.content.clone(), draft.stock);
            let issues = self
                .cutting_simulation
                .check_draft(&mut self.document, &content, stock);
            self.draft_issues = Some(DraftIssues {
                content,
                stock,
                issues,
            });
        }
        self.draft_issues
            .as_ref()
            .map(|cached| cached.issues.clone())
            .unwrap_or_default()
    }

    /// 計算条件を確かめ直す必要がある場合（条件・製品形状が変わった後）に確かめる
    fn refresh_condition_issues(&mut self) {
        if self.condition_issues.is_none() {
            self.condition_issues = Some(self.cutting_simulation.check(&mut self.document));
        }
    }

    /// 工程リストを出している間、加工工程の今の改訂の計算の状態を、一定の間隔で読み直す
    ///
    /// 終わった状態（正常・失敗・取り消し）は、ジョブマネージャーでやり直す・消すまで変わらないため、間隔を長くして
    /// 読み直す。読むだけでは、ジョブマネージャーを起動しない（動いていなければ、状態不明にする。計算待ちのジョブを
    /// 残して止まったときは、次にジョブマネージャーを使う操作をするまで計算が止まる）。状態は別のスレッドで読むため、
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
                // ジョブマネージャーが動いていない・見つからないときは、状態が分からない（読むときは起動しない）
                let line = if error.is_job_manager_not_running() {
                    JobStatusLine::ManagerNotRunning
                } else if error.is_job_manager_not_found() {
                    JobStatusLine::ManagerNotFound
                } else {
                    if self.cutting_simulation_job_status != JobStatusLine::Unreadable {
                        tracing::warn!("Cannot read the job status: {error}");
                    }
                    JobStatusLine::Unreadable
                };
                self.document
                    .operations()
                    .into_iter()
                    .map(|item| (item.id, line.clone()))
                    .collect()
            }
        };
        self.notify_job_manager_availability();
        let status = self.operation_job_status(self.document.selected_operation());
        self.cutting_simulation_job_status = status;
    }

    /// ジョブマネージャーが動いていない・見つからないことを、変わったときに 1 度だけ画面の下の行に出す（動いていない
    /// のは情報、見つからないのは失敗。最後に読めた状態に計算待ち・計算中の工程があれば、計算されていないことを注意
    /// として出す）。状態を読めた（ジョブマネージャーが動いている）ら、次に変わったときにまた出す。読めない・読んで
    /// いる間は、どちらとも決めない（同じメッセージを出し直さないように）
    fn notify_job_manager_availability(&mut self) {
        let Some(first) = self.operation_job_statuses.first().map(|(_, s)| s.clone()) else {
            return;
        };
        let unavailable = match first {
            JobStatusLine::ManagerNotRunning | JobStatusLine::ManagerNotFound => Some(first),
            JobStatusLine::NotSubmitted | JobStatusLine::Status(_) => {
                self.jobs_pending = self.operation_job_statuses.iter().any(|(_, status)| {
                    matches!(
                        status,
                        JobStatusLine::Status(
                            OperationJobStatus::Queued | OperationJobStatus::Running
                        )
                    )
                });
                None
            }
            JobStatusLine::Unreadable | JobStatusLine::Reading => return,
        };
        if unavailable == self.job_manager_unavailable {
            return;
        }
        match &unavailable {
            Some(JobStatusLine::ManagerNotRunning) if self.jobs_pending => {
                self.set_notice(Notice::JobManagerStopped);
            }
            Some(JobStatusLine::ManagerNotRunning) => self.set_notice(Notice::JobManagerNotRunning),
            Some(JobStatusLine::ManagerNotFound) => self.set_notice(Notice::JobManagerNotFound),
            _ => {}
        }
        self.job_manager_unavailable = unavailable;
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
                let name = copied_operation_name(
                    self.ui_locale,
                    &self.document.operation_content().attributes.name,
                );
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
        }
        if self.document.selected_operation() != before {
            self.forget_selected_operation_state();
            self.show_selected_operation();
        }
    }

    /// 条件・製品形状が変わった後に、計算条件と計算の状態を確かめ直す
    pub(crate) fn invalidate_conditions(&mut self) {
        self.condition_issues = None;
        self.operation_condition_errors = None;
        self.draft_issues = None;
        self.job_status_read_at = None;
    }

    pub(crate) fn prepare_machining_conditions_state(&mut self) -> MachiningConditionsUiState {
        // 計算の条件・素材・加工の対象を変えて、切削アニメーションの工程の CL を除いたら（前の条件の結果のため）、
        // アニメーションを閉じる
        if self.cutting_animation.is_some()
            && self.imported_operation.is_some_and(|operation| {
                self.document.imported_toolpath_state_of(operation) == ImportedToolpathState::None
            })
        {
            tracing::info!(
                "Closing the cutting animation because its CL is of the previous conditions"
            );
            self.close_cutting_animation();
        }
        // 条件・素材・加工の対象を変えて、選んだ工程の CL が今の条件の結果かが変わったら、線の色を描き直す。加工手順を
        // 開いた・閉じたら、線を出す・消す
        let toolpath_state = (
            self.document.selected_operation(),
            self.document.imported_toolpath_state(),
            self.machining_sequence_open,
        );
        if self.cutting_animation.is_none() && self.drawn_toolpath_state != Some(toolpath_state) {
            self.refresh_operation_toolpath_lines();
        }
        // 加工手順の画面は、加工手順を開いている間だけ出す（閉じるときに、画面もすべて閉じる）
        let screen_shown = self.operation_list_shown || self.condition_draft.is_some();
        debug_assert!(self.machining_sequence_open || !screen_shown);
        let dialog_issues = self.draft_issues();
        // 条件の確かめは画面に出すため、計算の状態は工程リストに出すためだけに読む
        if screen_shown {
            self.refresh_condition_issues();
        }
        if self.operation_list_shown {
            self.refresh_job_status();
        }
        let selected = self.document.selected_operation();
        // 条件に誤りがある工程（条件の確かめを捨てたときだけ確かめ直す。改訂が進んでいない工程は、前の結果を使う）
        if self.operation_list_shown && self.operation_condition_errors.is_none() {
            self.operation_condition_errors = Some(
                self.cutting_simulation
                    .operation_condition_issues(&mut self.document)
                    .into_iter()
                    .collect(),
            );
        }
        let condition_errors = self.operation_condition_errors.as_ref();
        let operations = self
            .document
            .operations()
            .into_iter()
            .map(|item| OperationRow {
                status: self.operation_job_status(item.id),
                toolpath_current: self.document.imported_toolpath_state_of(item.id)
                    == ImportedToolpathState::Current,
                setup: if !item.conditions_set {
                    OperationSetup::NotSet
                } else if let Some(issues) =
                    condition_errors.and_then(|errors| errors.get(&item.id))
                {
                    OperationSetup::ConditionError(issues.clone())
                } else {
                    OperationSetup::Ready
                },
                id: item.id,
                name: item.name,
                stage: item.stage,
                spec: item.spec,
                tool: item.tool,
            })
            .collect();
        // ダイアログを開いている間は、編集している条件を出す（ダイアログの外では使わない）
        let (operation, stock, tool_length_input) = match &self.condition_draft {
            Some(draft) => (draft.content.clone(), draft.stock, draft.tool_length_input),
            None => (
                self.document.operation_content().clone(),
                self.document.stock(),
                self.tool_length_input,
            ),
        };
        MachiningConditionsUiState {
            list_shown: self.operation_list_shown,
            dialog: self.condition_draft.as_ref().map(|draft| draft.dialog),
            dialog_page: self.operation_settings_page,
            open_dialog_requested: None,
            dialog_result: None,
            submit_on_apply: self.submit_on_apply,
            locale: self.ui_locale,
            sequence_name: self.document.sequence_name().to_string(),
            operations,
            selected_operation: selected,
            operation_command: None,
            face_count: self.document.shapes().visible_face_ids().len(),
            operation,
            stock,
            tool_length_input,
            toolpath_tolerance: self.document.toolpath_tolerance(),
            issues: self.condition_issues.clone().unwrap_or_default(),
            dialog_issues,
            // 取り込んだ条件は、取り込んだ工程を選んでいるときだけ出す
            imported: self
                .cutting_animation_job_id
                .zip(self.imported_conditions.clone())
                .filter(|_| self.imported_operation == Some(selected)),
            submit_requested: false,
            redo_requested: false,
            stop_requested: false,
            submit_sequence_requested: false,
            import_toolpath_requested: false,
            toggle_simulation_requested: false,
            simulation_shown: self.cutting_simulation_shown(),
            open_job_manager_requested: false,
        }
    }

    pub(crate) fn apply_machining_conditions_state(&mut self, state: MachiningConditionsUiState) {
        self.operation_list_shown = state.list_shown;
        self.operation_settings_page = state.dialog_page;
        self.submit_on_apply = state.submit_on_apply;
        // ダイアログを開いている間は、編集した条件を受け取り、適用・キャンセルだけを行う（工程リストは押せないため、
        // ほかの操作は来ない）
        if let Some(draft) = &mut self.condition_draft {
            draft.content = state.operation;
            draft.stock = state.stock;
            draft.tool_length_input = state.tool_length_input;
            match state.dialog_result {
                Some(DialogResult::Apply) => self.apply_condition_draft(),
                Some(DialogResult::Cancel) => self.cancel_condition_dialog(),
                None => {}
            }
            return;
        }
        // 行のダブルクリックは、工程を選ぶ操作と、ダイアログを開くことを同時に行う。選んだ後に開き、選んだ工程の条件を
        // 編集する
        let command = state.operation_command;
        if let Some(command) = command.clone() {
            self.run_operation_command(command);
        }
        if let Some(dialog) = state.open_dialog_requested {
            self.open_condition_dialog(dialog);
        }
        if command.is_some() {
            return;
        }
        if state.submit_requested && self.submit_cutting_simulation() {
            // 投入した計算の状態を、すぐに読み直す
            self.job_status_read_at = None;
        }
        if state.redo_requested && self.redo_cutting_simulation() {
            self.job_status_read_at = None;
        }
        if state.stop_requested {
            self.stop_cutting_simulation();
        }
        if state.submit_sequence_requested && self.submit_sequence() {
            self.job_status_read_at = None;
        }
        if state.import_toolpath_requested {
            self.import_toolpath();
        }
        if state.toggle_simulation_requested {
            self.toggle_cutting_simulation();
        }
        if state.open_job_manager_requested {
            self.open_job_manager();
        }
    }
}

/// 言語 `locale` の、空間モデルに加工手順・加工工程がないときに作る加工手順・加工工程の名前で、`make` を行う
pub(super) fn with_default_names<R>(
    locale: UiLocale,
    make: impl FnOnce(viewmodel::space_model_document::DefaultNames<'_>) -> R,
) -> R {
    let text = |key| {
        crate::machining_conditions_messages::resolve_machining_conditions_text(locale, key, &[])
    };
    let sequence = text("conditions.sequence.default_name");
    let operation = text("conditions.operations.default_name");
    make(viewmodel::space_model_document::DefaultNames {
        sequence: &sequence,
        operation: &operation,
    })
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
