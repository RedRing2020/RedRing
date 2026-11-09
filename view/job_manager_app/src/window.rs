//! ジョブマネージャーのウィンドウ（ジョブの一覧・絞り込み・詳細・操作。設計: CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! ジョブの記録は保存先から一定の間隔で読み直す。記録を書くのはサービスのスレッドだけのため、操作はサービスに渡し、
//! 結果が返ったら記録を読み直す。

use std::collections::BTreeSet;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::locale::{self, Language};
use eframe::egui::{self, CentralPanel, Color32, Grid, Id, Modal, Panel, RichText, ScrollArea};
use job_manager::{DeletePlan, delete_plan, imported_sources, old_jobs, superseded_jobs};
use job_runtime::store::{JobStore, StoredJob, StoredJobKind, StoredJobStatus};

use crate::messages::{change_failure_text, job_error_text, store_failure_key, text};
use crate::rows::{JobRow, elapsed_ms, format_elapsed, format_size, job_rows};
use crate::service::{JobCommand, ServiceChannel, StoreFailure, now_ms};
use crate::store_change::{self, ChangeFailure, FolderContents};

/// ジョブの記録を読み直す間隔
const JOBS_INTERVAL: Duration = Duration::from_secs(1);
/// 操作の結果を待つ間に、描き直す間隔
const REPLY_INTERVAL: Duration = Duration::from_millis(50);
/// 保存先の大きさを測り直す間隔
const SIZE_INTERVAL: Duration = Duration::from_secs(10);
/// まとめて消すときの、古いジョブの日数の既定と上限
const DEFAULT_OLD_DAYS: u32 = 30;
const MAX_OLD_DAYS: u32 = 3650;
/// 1 日のミリ秒
const DAY_MS: u64 = 24 * 60 * 60 * 1000;
/// ログのタブに出す、ログの末尾の大きさ
const LOG_TAIL_BYTES: u64 = 64 * 1024;

const STATUSES: [StoredJobStatus; 5] = [
    StoredJobStatus::Queued,
    StoredJobStatus::Running,
    StoredJobStatus::Succeeded,
    StoredJobStatus::Failed,
    StoredJobStatus::Canceled,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailsTab {
    Details,
    Log,
}

/// 削除を確かめている、選んだジョブと、消すジョブの決め方の結果
#[derive(Debug, Clone)]
struct DeleteConfirmation {
    selected: Vec<u64>,
    plan: DeletePlan,
}

/// まとめて消すときに選んだもの
#[derive(Debug, Clone)]
struct BulkDelete {
    /// 前の改訂の結果
    superseded: bool,
    /// 取り込み済みの CL の元
    imported: bool,
    /// 画面を開いたときに読んだ、ジョブの取り込んだ印（ジョブの番号と、取り込んだ切削シミュレーションの番号）
    imported_marks: std::collections::HashMap<u64, u64>,
    /// 古いジョブ（`old_days` 日より前に投入した、終わったジョブ）
    old: bool,
    old_days: u32,
    /// 取り込めなかった投入（画面を開いたときにあったものだけを消す）
    rejected: bool,
    rejected_ids: Vec<String>,
    /// 選んだもの・ジョブの記録が変わったときだけ決め直す、消すジョブ
    targets: Option<BulkTargets>,
}

/// まとめて消すときの、消すジョブ
#[derive(Debug, Clone)]
struct BulkTargets {
    /// 決めたときの選んだもの（前の改訂の結果・取り込み済みの CL の元・古いジョブ・日数）と、ジョブの記録の読み込みの回
    inputs: (bool, bool, bool, u32, u64),
    superseded: usize,
    imported: usize,
    old: usize,
    selected: Vec<u64>,
    plan: DeletePlan,
}

/// 替えるかを確かめている保存先のフォルダと、その中身
#[derive(Debug, Clone)]
struct StoreChange {
    folder: PathBuf,
    contents: FolderContents,
}

/// 選んだジョブの、ファイルから読んだ詳細（ジョブの記録を読み直したとき・選び直したときに読み直す）
struct SelectedFiles {
    job_id: u64,
    /// 成果物（CL）を、投入した側が取り込んだ
    imported: bool,
    input_size: Option<u64>,
    result: Option<(PathBuf, Option<u64>)>,
    log: Option<String>,
}

/// ジョブマネージャーのウィンドウ
pub(crate) struct JobManagerWindow {
    store: JobStore,
    service: ServiceChannel,
    /// ウィンドウの題に出している言語（言語を選び直したら、題を替える）
    title_language: Language,
    /// 選んだ言語を保存できなかった理由
    language_error: Option<String>,
    jobs: Vec<StoredJob>,
    /// 一覧の行（ジョブの記録・絞り込みが変わったときに並べ直す）
    rows: Vec<JobRow>,
    unreadable_jobs: usize,
    store_error: Option<StoreFailure>,
    jobs_read_at: Option<Instant>,
    store_size: Option<u64>,
    size_read_at: Option<Instant>,
    /// 保存先の大きさを測っているスレッドからの結果
    size_receiver: Option<mpsc::Receiver<u64>>,
    filter: Option<StoredJobStatus>,
    /// 詳細を出すジョブ（最後に押した行）
    selected: Option<u64>,
    /// 一覧で選んでいるジョブ（Ctrl・Shift を押しながら押すと、複数を選ぶ）
    selection: BTreeSet<u64>,
    /// Shift を押しながら押したときに、範囲の始まりにする行のジョブ
    anchor: Option<u64>,
    selected_files: Option<SelectedFiles>,
    tab: DetailsTab,
    /// 保存先を読み書きできずに、できなかった操作の理由
    command_error: Option<StoreFailure>,
    delete_confirmation: Option<DeleteConfirmation>,
    /// 計算待ち・計算中のため消さなかった、選んだジョブ
    delete_kept: Option<BTreeSet<u64>>,
    bulk_delete: Option<BulkDelete>,
    /// ジョブの記録を読み込んだ回（読み込むたびに増やす）
    jobs_version: u64,
    store_change: Option<StoreChange>,
    /// 保存先を替えられなかった理由
    store_change_error: Option<ChangeFailure>,
    /// 前の描画でマウスが乗っていた一覧の行（見出しの行が 0）
    hovered_row: Option<usize>,
    /// 閉じたときに、ウィンドウを隠す（タスクトレイから開き直す）
    hide_on_close: bool,
}

impl JobManagerWindow {
    pub(crate) fn new(store: JobStore, service: ServiceChannel, hide_on_close: bool) -> Self {
        Self {
            store,
            service,
            title_language: locale::current(),
            language_error: None,
            jobs: Vec::new(),
            rows: Vec::new(),
            unreadable_jobs: 0,
            store_error: None,
            jobs_read_at: None,
            store_size: None,
            size_read_at: None,
            size_receiver: None,
            filter: None,
            selected: None,
            selection: BTreeSet::new(),
            anchor: None,
            selected_files: None,
            tab: DetailsTab::Details,
            command_error: None,
            delete_confirmation: None,
            delete_kept: None,
            bulk_delete: None,
            jobs_version: 0,
            store_change: None,
            store_change_error: None,
            hovered_row: None,
            hide_on_close,
        }
    }

    fn text(&self, key: &str) -> String {
        text(locale::current(), key, &[])
    }

    fn text_with(&self, key: &str, args: &[(&str, String)]) -> String {
        text(locale::current(), key, args)
    }

    /// 言語を選ぶメニュー（その言語の名前で並べる）
    fn show_language_menu(&mut self, ui: &mut egui::Ui) {
        let current = locale::current();
        ui.menu_button(
            format!("{}: {}", self.text("language"), current.native_name()),
            |ui| {
                for language in Language::ALL {
                    if ui
                        .radio(language == current, language.native_name())
                        .clicked()
                    {
                        self.language_error = crate::select_language(language)
                            .err()
                            .map(|error| error.to_string());
                        self.show_title(ui.ctx());
                        ui.close();
                    }
                }
            },
        );
    }

    /// ほかの所（トレイ）で言語を選び直していれば、ウィンドウの題を今の言語にし、前に保存できなかった理由を消す
    fn update_title(&mut self, ctx: &egui::Context) {
        if locale::current() != self.title_language {
            self.language_error = None;
            self.show_title(ctx);
        }
    }

    /// ウィンドウの題を、今の言語にする
    fn show_title(&mut self, ctx: &egui::Context) {
        self.title_language = locale::current();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.text("window.title")));
    }

    fn refresh(&mut self) {
        // 操作の結果が返ったら、記録をすぐに読み直す
        while let Some(reply) = self.service.try_reply() {
            match reply {
                Ok(kept) => {
                    self.command_error = None;
                    // 確かめた後に後のジョブが計算待ちになったなど、消さなかったジョブを出す
                    if !kept.is_empty() {
                        self.delete_kept = Some(kept.into_iter().collect());
                    }
                }
                Err(failure) => self.command_error = Some(failure),
            }
            self.jobs_read_at = None;
        }
        let due =
            |read_at: Option<Instant>, interval| read_at.is_none_or(|at| at.elapsed() >= interval);
        if due(self.jobs_read_at, JOBS_INTERVAL) {
            self.jobs_read_at = Some(Instant::now());
            match self.store.load_jobs() {
                Ok((jobs, unreadable)) => {
                    self.jobs = jobs;
                    self.jobs_version += 1;
                    self.unreadable_jobs = unreadable.len();
                    self.store_error = None;
                }
                Err(error) => self.store_error = Some(StoreFailure::of(&error)),
            }
            self.update_rows();
            self.selected_files = None;
        }
        // 保存先の大きさは、画面を止めないように別のスレッドで測る
        if let Some(receiver) = &self.size_receiver
            && let Ok(size) = receiver.try_recv()
        {
            self.store_size = Some(size);
            self.size_receiver = None;
        }
        if self.size_receiver.is_none() && due(self.size_read_at, SIZE_INTERVAL) {
            self.size_read_at = Some(Instant::now());
            let (sender, receiver) = mpsc::channel();
            let root = self.store.root().to_path_buf();
            std::thread::spawn(move || {
                let _ = sender.send(folder_size(&root));
            });
            self.size_receiver = Some(receiver);
        }
        if self.selected_files.as_ref().map(|files| files.job_id) != self.selected {
            self.selected_files = self.selected.and_then(|id| self.read_selected_files(id));
        }
    }

    /// 選んだジョブの入力・成果物の大きさとログを読む
    fn read_selected_files(&self, job_id: u64) -> Option<SelectedFiles> {
        let job = self.jobs.iter().find(|job| job.job_id == job_id)?;
        let size = |path: &Path| std::fs::metadata(path).ok().map(|metadata| metadata.len());
        let result = self.store.result_file(job_id, job.kind);
        let log = self.store.log_file(job_id);
        let mut partial = log.clone().into_os_string();
        partial.push(".tmp");
        Some(SelectedFiles {
            job_id,
            imported: self.store.imported_job(job_id).is_some(),
            input_size: size(&self.store.input_file(job_id)),
            result: result.is_file().then(|| {
                let bytes = size(&result);
                (result, bytes)
            }),
            log: log_tail(&log).or_else(|| log_tail(Path::new(&partial))),
        })
    }

    /// 一覧の行を並べ直し、一覧に出ていないジョブ（消した・絞り込みで隠れた）の選びを外す（隠れたジョブを、見えない
    /// まま消さないように）。詳細は、ジョブがある間は出し続ける
    fn update_rows(&mut self) {
        self.rows = job_rows(&self.jobs, self.filter);
        let shown: BTreeSet<u64> = self.rows.iter().map(|row| row.job.job_id).collect();
        self.selection.retain(|id| shown.contains(id));
        if let Some(kept) = self.delete_kept.as_mut() {
            kept.retain(|id| shown.contains(id));
            if kept.is_empty() {
                self.delete_kept = None;
            }
        }
        if self.selected_job().is_none() {
            self.selected = None;
        }
    }

    /// 一覧の行 `rows` のジョブ `job_id` を押したときの選び（Ctrl で選びを足し外し、Shift で範囲を選ぶ）
    fn select(&mut self, rows: &[JobRow], job_id: u64, modifiers: egui::Modifiers) {
        let position = |id: u64| rows.iter().position(|row| row.job.job_id == id);
        let range = self
            .anchor
            .and_then(position)
            .zip(position(job_id))
            .filter(|_| modifiers.shift);
        if let Some((anchor, at)) = range {
            if !modifiers.command {
                self.selection.clear();
            }
            self.selection.extend(
                rows[anchor.min(at)..=anchor.max(at)]
                    .iter()
                    .map(|row| row.job.job_id),
            );
            self.selected = Some(job_id);
        } else if modifiers.command {
            if self.selection.remove(&job_id) {
                if self.selected == Some(job_id) {
                    self.selected = None;
                }
            } else {
                self.selection.insert(job_id);
                self.selected = Some(job_id);
            }
            self.anchor = Some(job_id);
        } else {
            self.selection = BTreeSet::from([job_id]);
            self.selected = Some(job_id);
            self.anchor = Some(job_id);
        }
        self.delete_kept = None;
    }

    fn status_text(&self, status: StoredJobStatus) -> String {
        self.text(match status {
            StoredJobStatus::Queued => "status.queued",
            StoredJobStatus::Running => "status.running",
            StoredJobStatus::Succeeded => "status.succeeded",
            StoredJobStatus::Failed => "status.failed",
            StoredJobStatus::Canceled => "status.canceled",
        })
    }

    fn selected_job(&self) -> Option<&StoredJob> {
        self.selected
            .and_then(|id| self.jobs.iter().find(|job| job.job_id == id))
    }

    fn send(&mut self, command: JobCommand) {
        if let Err(failure) = self.service.send_command(command) {
            self.command_error = Some(failure);
        }
    }

    /// 選んだジョブの操作（状態で操作できるものだけを押せる。削除のほかは、1 つだけ選んだときに押せる）
    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        use StoredJobStatus::{Canceled, Failed, Queued, Running};
        let idle = self.service.outstanding() == 0;
        let selected = self
            .selected_job()
            .filter(|job| idle && self.selection.len() == 1 && self.selection.contains(&job.job_id))
            .map(|job| (job.job_id, job.status));
        let can = |allowed: &[StoredJobStatus]| {
            selected.is_some_and(|(_, status)| allowed.contains(&status))
        };
        let buttons = [
            ("toolbar.cancel", can(&[Queued, Running])),
            ("toolbar.retry", can(&[Failed, Canceled])),
            ("toolbar.delete", idle && !self.selection.is_empty()),
            ("toolbar.raise", can(&[Queued])),
            ("toolbar.lower", can(&[Queued])),
        ];
        let mut clicked = None;
        for (key, enabled) in buttons {
            if ui
                .add_enabled(enabled, egui::Button::new(self.text(key)))
                .clicked()
            {
                clicked = Some(key);
            }
        }
        if ui
            .add_enabled(idle, egui::Button::new(self.text("toolbar.bulk_delete")))
            .clicked()
        {
            self.open_bulk_delete();
        }
        if clicked == Some("toolbar.delete") {
            self.request_delete();
            return;
        }
        let (Some(key), Some((job_id, _))) = (clicked, selected) else {
            return;
        };
        match key {
            "toolbar.cancel" => self.send(JobCommand::Cancel(job_id)),
            "toolbar.retry" => self.send(JobCommand::Retry(job_id)),
            "toolbar.raise" => self.send(JobCommand::ChangePriority { job_id, delta: 1 }),
            _ => self.send(JobCommand::ChangePriority { job_id, delta: -1 }),
        }
    }

    /// 選んだジョブを消す（1 つだけを消すときのほかは、消すジョブ・消さないジョブを出して確かめる）
    fn request_delete(&mut self) {
        let selected: Vec<u64> = self.selection.iter().copied().collect();
        let plan = delete_plan(&self.jobs, &selected);
        if plan.deleted.is_empty() {
            self.delete_kept = Some(plan.kept);
        } else if plan.deleted.len() == 1 && plan.kept.is_empty() {
            self.delete_kept = None;
            self.send(JobCommand::Delete(selected));
        } else {
            self.delete_kept = None;
            self.delete_confirmation = Some(DeleteConfirmation { selected, plan });
        }
    }

    /// まとめて消す画面を開く（取り込めなかった投入を数える）
    fn open_bulk_delete(&mut self) {
        let rejected_ids = self.store.rejected_submission_ids().unwrap_or_default();
        // 取り込んだ印は、終わったツールパス生成のジョブだけ、画面を開いたときに 1 回読む
        let imported_marks = self
            .jobs
            .iter()
            .filter(|job| job.kind == StoredJobKind::Cam && job.status.is_terminal())
            .filter_map(|job| {
                self.store
                    .imported_job(job.job_id)
                    .map(|imported| (job.job_id, imported))
            })
            .collect();
        self.delete_kept = None;
        self.bulk_delete = Some(BulkDelete {
            superseded: false,
            imported: false,
            imported_marks,
            old: false,
            old_days: DEFAULT_OLD_DAYS,
            rejected: false,
            rejected_ids,
            targets: None,
        });
    }

    /// まとめて消すときの、消すジョブ（終わったジョブだけ。選んだもの・ジョブの記録が前と同じなら決め直さない）
    fn bulk_targets(&self, bulk: &BulkDelete) -> BulkTargets {
        let inputs = (
            bulk.superseded,
            bulk.imported,
            bulk.old,
            bulk.old_days,
            self.jobs_version,
        );
        if let Some(targets) = bulk
            .targets
            .as_ref()
            .filter(|targets| targets.inputs == inputs)
        {
            return targets.clone();
        }
        let finished: BTreeSet<u64> = self
            .jobs
            .iter()
            .filter(|job| job.status.is_terminal())
            .map(|job| job.job_id)
            .collect();
        let superseded: Vec<u64> = superseded_jobs(&self.jobs)
            .into_iter()
            .filter(|id| finished.contains(id))
            .collect();
        let imported = imported_sources(&self.jobs, |job_id| {
            bulk.imported_marks.get(&job_id).copied()
        });
        let before = now_ms().saturating_sub(u64::from(bulk.old_days) * DAY_MS);
        let old = old_jobs(&self.jobs, before);
        let mut selected: BTreeSet<u64> = BTreeSet::new();
        if bulk.superseded {
            selected.extend(&superseded);
        }
        if bulk.imported {
            selected.extend(&imported);
        }
        if bulk.old {
            selected.extend(&old);
        }
        let selected: Vec<u64> = selected.into_iter().collect();
        let plan = delete_plan(&self.jobs, &selected);
        BulkTargets {
            inputs,
            superseded: superseded.len(),
            imported: imported.len(),
            old: old.len(),
            selected,
            plan,
        }
    }

    /// まとめて消すものを選び、消すジョブの数を出して確かめる
    fn show_bulk_delete(&mut self, ctx: &egui::Context) {
        let Some(mut bulk) = self.bulk_delete.clone() else {
            return;
        };
        let targets = self.bulk_targets(&bulk);
        let removes_rejected = bulk.rejected && !bulk.rejected_ids.is_empty();
        let count = |n: usize| n.to_string();
        let texts = (
            self.text("bulk.title"),
            self.text_with("bulk.superseded", &[("count", count(targets.superseded))]),
            self.text("bulk.old"),
            self.text_with("bulk.old_count", &[("count", count(targets.old))]),
            self.text_with(
                "bulk.rejected",
                &[("count", count(bulk.rejected_ids.len()))],
            ),
            self.text_with(
                "bulk.summary",
                &[("count", count(targets.plan.deleted.len()))],
            ),
            self.text("bulk.note"),
            self.text("bulk.days_suffix"),
        );
        let imported_text = self.text_with("bulk.imported", &[("count", count(targets.imported))]);
        let kept_text = (!targets.plan.kept.is_empty()).then(|| {
            let jobs: Vec<String> = targets.plan.kept.iter().map(u64::to_string).collect();
            self.text_with("bulk.kept", &[("jobs", jobs.join(", "))])
        });
        let (yes, no) = (self.text("bulk.yes"), self.text("bulk.no"));
        let mut choice = None;
        let response = Modal::new(Id::unique("bulk_delete")).show(ctx, |ui| {
            ui.set_max_width(520.0);
            ui.heading(texts.0);
            ui.add_space(8.0);
            ui.checkbox(&mut bulk.superseded, texts.1);
            ui.checkbox(&mut bulk.imported, imported_text);
            ui.horizontal(|ui| {
                ui.checkbox(&mut bulk.old, texts.2);
                ui.add(
                    egui::DragValue::new(&mut bulk.old_days)
                        .range(1..=MAX_OLD_DAYS)
                        .suffix(texts.7),
                );
                ui.label(texts.3);
            });
            ui.checkbox(&mut bulk.rejected, texts.4);
            ui.add_space(8.0);
            ui.label(texts.5);
            if let Some(kept) = kept_text {
                ui.colored_label(ui.visuals().warn_fg_color, kept);
            }
            ui.colored_label(ui.visuals().warn_fg_color, texts.6);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let enabled = !targets.plan.deleted.is_empty() || removes_rejected;
                if ui.add_enabled(enabled, egui::Button::new(yes)).clicked() {
                    choice = Some(true);
                }
                if ui.button(no).clicked() {
                    choice = Some(false);
                }
            });
        });
        // 外を押したとき・Esc は、消さない
        if response.should_close() && choice.is_none() {
            choice = Some(false);
        }
        match choice {
            None => {
                bulk.targets = Some(targets);
                self.bulk_delete = Some(bulk);
            }
            Some(confirmed) => {
                self.bulk_delete = None;
                ctx.request_repaint();
                if confirmed {
                    // 消すとしたジョブだけを頼む（確かめている間に状態が変わったジョブを、消すと出さずに消さないように）
                    let confirmed: Vec<u64> = targets
                        .selected
                        .into_iter()
                        .filter(|id| targets.plan.deleted.contains(id))
                        .collect();
                    if !confirmed.is_empty() {
                        self.send(JobCommand::Delete(confirmed));
                    }
                    if removes_rejected {
                        self.send(JobCommand::RemoveRejected(bulk.rejected_ids));
                    }
                }
            }
        }
    }

    /// 後のジョブが成果物を使うジョブを消すかを確かめる
    fn show_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(confirmation) = self.delete_confirmation.clone() else {
            return;
        };
        let list = |ids: &mut dyn Iterator<Item = &u64>| {
            ids.map(u64::to_string).collect::<Vec<_>>().join(", ")
        };
        let plan = &confirmation.plan;
        let chosen = list(
            &mut plan
                .deleted
                .iter()
                .filter(|id| confirmation.selected.contains(id)),
        );
        let later = list(
            &mut plan
                .deleted
                .iter()
                .filter(|id| !confirmation.selected.contains(id)),
        );
        let mut lines = vec![self.text_with("delete.confirm", &[("jobs", chosen)])];
        if !later.is_empty() {
            lines.push(self.text_with("delete.confirm_later", &[("jobs", later)]));
        }
        if !plan.kept.is_empty() {
            lines.push(self.text_with(
                "delete.confirm_kept",
                &[("jobs", list(&mut plan.kept.iter()))],
            ));
        }
        lines.push(self.text("delete.confirm_question"));
        let message = lines.join("\n");
        let (yes, no) = (
            self.text("delete.confirm_yes"),
            self.text("delete.confirm_no"),
        );
        let mut choice = None;
        let response = Modal::new(Id::unique("delete_confirmation")).show(ctx, |ui| {
            ui.set_max_width(420.0);
            ui.label(message);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(yes).clicked() {
                    choice = Some(true);
                }
                if ui.button(no).clicked() {
                    choice = Some(false);
                }
            });
        });
        // 外を押したとき・Esc は、消さない
        if response.should_close() && choice.is_none() {
            choice = Some(false);
        }
        if choice.is_some() {
            // 閉じた後の画面をすぐに描く（描くまでは、閉じた確かめの画面が次の操作を受けてしまうため）
            ctx.request_repaint();
        }
        match choice {
            Some(true) => {
                self.delete_confirmation = None;
                // 確かめたときに消すとしたジョブだけを頼む（確かめている間に計算を終えたジョブを、消すと出さずに
                // 消さないように）
                let confirmed: Vec<u64> = confirmation
                    .selected
                    .into_iter()
                    .filter(|id| confirmation.plan.deleted.contains(id))
                    .collect();
                self.send(JobCommand::Delete(confirmed));
            }
            Some(false) => self.delete_confirmation = None,
            None => {}
        }
    }

    /// 保存先のフォルダを選び、替えられるフォルダなら確かめの画面を出す
    fn choose_store(&mut self) {
        self.store_change_error = None;
        if store_change::is_busy(&self.store) {
            self.store_change_error = Some(ChangeFailure::Busy);
            return;
        }
        let mut dialog = rfd::FileDialog::new().set_title(self.text("store.choose"));
        if let Some(parent) = self.store.root().parent() {
            dialog = dialog.set_directory(parent);
        }
        let Some(folder) = dialog.pick_folder() else {
            return;
        };
        match store_change::check_folder(self.store.root(), &folder) {
            Ok(contents) => self.store_change = Some(StoreChange { folder, contents }),
            Err(failure) => self.store_change_error = Some(failure),
        }
    }

    /// 保存先を替えるかを確かめ、替えるときはジョブマネージャーを起動し直す
    fn show_store_change(&mut self, ctx: &egui::Context) {
        let Some(change) = self.store_change.clone() else {
            return;
        };
        let message = self.text_with(
            "store.change_confirm",
            &[
                ("current", self.store.root().display().to_string()),
                ("chosen", change.folder.display().to_string()),
            ],
        );
        let note = match change.contents {
            FolderContents::Empty => None,
            FolderContents::Store => Some(self.text("store.change_existing")),
            FolderContents::Other => Some(self.text("store.change_other_files")),
        };
        let (question, yes, no) = (
            self.text("store.change_question"),
            self.text("store.change_yes"),
            self.text("store.change_no"),
        );
        let mut choice = None;
        let response = Modal::new(Id::unique("store_change")).show(ctx, |ui| {
            ui.set_max_width(520.0);
            ui.label(message);
            if let Some(note) = note {
                ui.add_space(4.0);
                ui.colored_label(ui.visuals().warn_fg_color, note);
            }
            ui.add_space(8.0);
            ui.label(question);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(yes).clicked() {
                    choice = Some(true);
                }
                if ui.button(no).clicked() {
                    choice = Some(false);
                }
            });
        });
        // 外を押したとき・Esc は、替えない
        if response.should_close() && choice.is_none() {
            choice = Some(false);
        }
        if choice.is_some() {
            self.store_change = None;
            ctx.request_repaint();
        }
        if choice == Some(true) {
            // 確かめている間に投入・待ちのジョブが増えていれば、替えない
            self.store_change_error = Some(if store_change::is_busy(&self.store) {
                ChangeFailure::Busy
            } else {
                match store_change::switch(&change.folder, &self.service.requests) {
                    Err(failure) => failure,
                }
            });
        }
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.show_toolbar(ui);
            ui.separator();
            if let Some(failure) = self.command_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    self.text_with(
                        "command.failed",
                        &[("reason", self.text(store_failure_key(failure)))],
                    ),
                );
            }
            if let Some(kept) = &self.delete_kept {
                let jobs: Vec<String> = kept.iter().map(u64::to_string).collect();
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    self.text_with("delete.kept", &[("jobs", jobs.join(", "))]),
                );
            }
            if let Some(size) = self.store_size {
                ui.label(self.text_with("store.size", &[("size", format_size(size))]))
                    .on_hover_text(self.store.root().display().to_string());
            }
            if let Some(failure) = self.store_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    self.text_with(
                        "store.unreadable",
                        &[("reason", self.text(store_failure_key(failure)))],
                    ),
                );
            }
            if self.unreadable_jobs > 0 {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    self.text_with(
                        "store.unreadable_jobs",
                        &[("count", self.unreadable_jobs.to_string())],
                    ),
                );
            }
            if let Some(reason) = &self.language_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    self.text_with("language.save_failed", &[("reason", reason.clone())]),
                );
            }
            if let Some(failure) = self.store_change_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    change_failure_text(locale::current(), failure),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.show_language_menu(ui);
                // 計算待ち・計算中のジョブがある間・保存先を読めない間は替えない（選んだときに、取り込みを待つ投入も
                // 確かめる。保存先のフォルダがない（外したドライブなど）ときは替えられる）
                let busy = self.service.outstanding() > 0
                    || self.jobs.iter().any(|job| !job.status.is_terminal())
                    || self
                        .store_error
                        .is_some_and(|failure| failure != StoreFailure::NotFound);
                if ui
                    .add_enabled(!busy, egui::Button::new(self.text("store.change")))
                    .on_hover_text(self.store.root().display().to_string())
                    .on_disabled_hover_text(self.text("store.change_busy"))
                    .clicked()
                {
                    self.choose_store();
                }
            });
        });
    }

    fn show_filter(&mut self, ui: &mut egui::Ui) {
        let before = self.filter;
        let count = |status: Option<StoredJobStatus>| {
            self.jobs
                .iter()
                .filter(|job| status.is_none_or(|status| job.status == status))
                .count()
        };
        // 選ぶもの・印・印の色・文言（ジョブの数を添える）
        let choices: Vec<(Option<StoredJobStatus>, &str, Color32, String)> =
            std::iter::once((None, self.text("filter.all")))
                .chain(
                    STATUSES
                        .iter()
                        .map(|&status| (Some(status), self.status_text(status))),
                )
                .map(|(choice, label)| {
                    let (icon, color) = match choice {
                        None => ("☰", Color32::from_rgb(230, 160, 60)),
                        Some(status) => (status_icon(status), status_color(status)),
                    };
                    (choice, icon, color, format!("{label} ({})", count(choice)))
                })
                .collect();
        // 開き閉じできる見出し（太字）の下に、印と数を添えた選ぶボタンを幅いっぱいに並べる
        egui::CollapsingHeader::new(RichText::new(self.text("filter.header")).strong())
            .default_open(true)
            .show_unindented(ui, |ui| {
                // 選んだボタンは、目立ちすぎない灰色で示す
                let visuals = ui.visuals_mut();
                visuals.selection.bg_fill = visuals.widgets.active.weak_bg_fill;
                visuals.selection.stroke.color = visuals.strong_text_color();
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    for (choice, icon, color, label) in choices {
                        let selected = self.filter == choice;
                        let button = egui::Button::selectable(
                            selected,
                            (RichText::new(icon).color(color), label),
                        );
                        if ui.add(button).clicked() {
                            self.filter = choice;
                        }
                    }
                });
            });
        if self.filter != before {
            self.update_rows();
        }
    }

    fn show_jobs(&mut self, ui: &mut egui::Ui) {
        let now = now_ms();
        // 行を描く間は自分を借りるため、行を一時に取り出し、描き終えたら戻す
        let rows = std::mem::take(&mut self.rows);
        // 選んだ行・マウスが乗った行の色（見出しの行が 0 のため、ジョブの行は 1 から）。乗った行は、前の描画のもの
        let selected_rows: BTreeSet<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.selection.contains(&row.job.job_id))
            .map(|(index, _)| index + 1)
            .collect();
        let hovered_row = self.hovered_row;
        let mut hovered_now = None;
        let mut clicked = None;
        ScrollArea::vertical().show(ui, |ui| {
            // 行のどこを押しても選べるように、文字は押せないようにする
            ui.style_mut().interaction.selectable_labels = false;
            let row_spacing = ui.spacing().item_spacing.y;
            Grid::new("jobs")
                .num_columns(7)
                .with_row_color(move |row, style| {
                    if selected_rows.contains(&row) {
                        Some(style.visuals.selection.bg_fill)
                    } else if row > 0 && Some(row) == hovered_row {
                        Some(style.visuals.widgets.hovered.weak_bg_fill)
                    } else if row % 2 == 1 {
                        Some(style.visuals.faint_bg_color)
                    } else {
                        None
                    }
                })
                .show(ui, |ui| {
                    for key in [
                        "column.id",
                        "column.source",
                        "column.kind",
                        "column.status",
                        "column.priority",
                        "column.elapsed",
                        "column.submitted",
                    ] {
                        ui.label(RichText::new(self.text(key)).strong());
                    }
                    ui.end_row();
                    for (index, row) in rows.iter().enumerate() {
                        let job = &row.job;
                        let id = if row.child {
                            format!("  └ {}", job.job_id)
                        } else {
                            job.job_id.to_string()
                        };
                        // 欄はすべて、行の高さの中央にそろえる（状態の欄は、計算中の印のぶん高い）
                        let mut cells = vec![centered_label(ui, id)];
                        cells.push(centered_label(ui, self.source_text(job)));
                        cells.push(centered_label(
                            ui,
                            self.text(match job.kind {
                                StoredJobKind::Cam => "kind.cam",
                                StoredJobKind::Sim => "kind.sim",
                            }),
                        ));
                        let status = ui.horizontal_centered(|ui| {
                            ui.label(
                                RichText::new(self.status_text(job.status))
                                    .color(status_color(job.status)),
                            );
                            // 進み具合の割合はまだないため、計算中は動いていることだけを出す
                            if job.status == StoredJobStatus::Running {
                                ui.spinner();
                            }
                        });
                        cells.push(status.response.rect);
                        cells.push(centered_label(ui, job.priority.to_string()));
                        cells.push(centered_label(
                            ui,
                            elapsed_ms(job, now).map_or_else(|| "-".to_string(), format_elapsed),
                        ));
                        cells.push(centered_label(
                            ui,
                            local_time(locale::current(), job.created_at_ms),
                        ));
                        // 行のすべての欄と、行の間の半分を、押せる範囲にする
                        let area = cells
                            .into_iter()
                            .reduce(|a, b| a.union(b))
                            .unwrap_or(egui::Rect::NOTHING)
                            .expand2(egui::vec2(0.0, row_spacing / 2.0));
                        let response = ui.interact(
                            area,
                            ui.scope_id().with(("job_row", job.job_id)),
                            egui::Sense::click(),
                        );
                        if response.clicked() {
                            clicked = Some(job.job_id);
                        }
                        if response.hovered() {
                            hovered_now = Some(index + 1);
                        }
                        ui.end_row();
                    }
                });
        });
        if let Some(job_id) = clicked {
            let modifiers = ui.input(|input| input.modifiers);
            self.select(&rows, job_id, modifiers);
        }
        if hovered_now != self.hovered_row {
            self.hovered_row = hovered_now;
            // 乗った行の色を、すぐに描き直す
            ui.ctx().request_repaint();
        }
        self.rows = rows;
    }

    fn source_text(&self, job: &StoredJob) -> String {
        match &job.source {
            // 工程の ID は長いため、先頭だけを出す
            Some(source) => format!(
                "{} r{}",
                source.source_id.chars().take(8).collect::<String>(),
                source.revision
            ),
            None => self.text("source.none"),
        }
    }

    fn show_details(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let details = self.text("details.tab");
            let log = self.text("details.log_tab");
            ui.selectable_value(&mut self.tab, DetailsTab::Details, details);
            ui.selectable_value(&mut self.tab, DetailsTab::Log, log);
        });
        ui.separator();
        let Some(job) = self.selected_job().cloned() else {
            ui.label(self.text("details.none"));
            return;
        };
        ScrollArea::vertical().show(ui, |ui| match self.tab {
            DetailsTab::Details => self.show_job_details(ui, &job),
            DetailsTab::Log => match self
                .selected_files
                .as_ref()
                .and_then(|files| files.log.as_ref())
            {
                Some(text) => {
                    ui.monospace(text);
                }
                None => {
                    ui.label(self.text("details.no_log"));
                }
            },
        });
    }

    fn show_job_details(&self, ui: &mut egui::Ui, job: &StoredJob) {
        ui.label(self.text_with(
            "details.status",
            &[("status", self.status_text(job.status))],
        ));
        // 理由の種類があれば画面の言語で出し（英語の文の詳しい理由（ログの最後の行など）を添える）、ない記録（前の版）・
        // 知らない種類は英語の文をそのまま出す
        let reason = job
            .error
            .as_ref()
            .and_then(|error| {
                let text = job_error_text(locale::current(), error)?;
                let detail = job.last_error.as_deref().and_then(|last| {
                    last.strip_prefix(&error.to_string())
                        .and_then(|rest| rest.strip_prefix(": "))
                });
                Some(match detail {
                    Some(detail) => format!("{text}（{detail}）"),
                    None => text,
                })
            })
            .or_else(|| job.last_error.clone());
        if let Some(reason) = reason {
            ui.colored_label(
                ui.visuals().error_fg_color,
                self.text_with("details.error", &[("reason", reason)]),
            );
        }
        if !job.depends_on.is_empty() {
            let jobs: Vec<String> = job.depends_on.iter().map(u64::to_string).collect();
            ui.label(self.text_with("details.depends_on", &[("jobs", jobs.join(", "))]));
        }
        ui.label(self.text_with("details.attempts", &[("count", job.attempts.to_string())]));
        let files = self.selected_files.as_ref();
        let size_text = |size: Option<u64>| size.map_or_else(|| "-".to_string(), format_size);
        ui.label(self.text_with(
            "details.input",
            &[
                (
                    "path",
                    self.store.input_file(job.job_id).display().to_string(),
                ),
                ("size", size_text(files.and_then(|files| files.input_size))),
            ],
        ));
        if files.is_some_and(|files| files.imported) {
            ui.label(self.text("details.imported"));
        }
        match files.and_then(|files| files.result.as_ref()) {
            Some((path, size)) => {
                ui.label(self.text_with(
                    "details.result",
                    &[
                        ("path", path.display().to_string()),
                        ("size", size_text(*size)),
                    ],
                ));
            }
            None => {
                ui.label(self.text("details.no_result"));
            }
        }
    }
}

impl eframe::App for JobManagerWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.hide_on_close && ui.ctx().input(|input| input.viewport().close_requested()) {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
        self.update_title(ui.ctx());
        self.refresh();
        self.show_delete_confirmation(ui.ctx());
        self.show_bulk_delete(ui.ctx());
        self.show_store_change(ui.ctx());
        Panel::top("header").show(ui, |ui| self.show_header(ui));
        Panel::left("filter")
            .resizable(true)
            .default_size(150.0)
            .show(ui, |ui| self.show_filter(ui));
        Panel::bottom("details")
            .resizable(true)
            .default_size(200.0)
            .show(ui, |ui| self.show_details(ui));
        CentralPanel::default().show(ui, |ui| self.show_jobs(ui));
        // 記録・経過時間を、操作がなくても読み直して描き直す（操作の結果を待つ間は、結果が返ったらすぐ出す）
        ui.ctx()
            .request_repaint_after(if self.service.outstanding() > 0 {
                REPLY_INTERVAL
            } else {
                JOBS_INTERVAL
            });
    }
}

/// 行の高さの中央にそろえた文字の欄（欄の範囲を返す）
fn centered_label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Rect {
    ui.horizontal_centered(|ui| ui.label(text)).response.rect
}

/// 絞り込みで状態に添える印
fn status_icon(status: StoredJobStatus) -> &'static str {
    match status {
        StoredJobStatus::Queued => "⏸",
        StoredJobStatus::Running => "▶",
        StoredJobStatus::Succeeded => "✔",
        StoredJobStatus::Failed => "⚠",
        StoredJobStatus::Canceled => "⏹",
    }
}

fn status_color(status: StoredJobStatus) -> Color32 {
    match status {
        StoredJobStatus::Queued => Color32::GRAY,
        StoredJobStatus::Running => Color32::from_rgb(80, 160, 255),
        StoredJobStatus::Succeeded => Color32::from_rgb(80, 200, 120),
        StoredJobStatus::Failed => Color32::from_rgb(230, 90, 90),
        StoredJobStatus::Canceled => Color32::from_rgb(200, 160, 60),
    }
}

/// UNIX 時刻のミリ秒を、この PC の時刻で、言語 `language` の書き方（日本語は「月/日 時:分」、英語は「月の名前 日
/// 時:分」）で書く
fn local_time(language: Language, ms: u64) -> String {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_millis_opt(ms as i64)
        .single()
        .map_or_else(
            || "-".to_string(),
            |time| time.format(time_format(language)).to_string(),
        )
}

/// 言語 `language` の時刻の書き方
fn time_format(language: Language) -> &'static str {
    match language {
        Language::Ja => "%m/%d %H:%M",
        Language::En => "%b %-d %H:%M",
    }
}

/// フォルダの中のファイルの大きさの合計（読めないものは数えない）
fn folder_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => folder_size(&entry.path()),
            Ok(kind) if kind.is_file() => entry.metadata().map_or(0, |metadata| metadata.len()),
            _ => 0,
        })
        .sum()
}

/// ログの末尾（`LOG_TAIL_BYTES` バイトまで）
fn log_tail(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(LOG_TAIL_BYTES)))
        .ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}
