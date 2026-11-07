//! ジョブマネージャーのウィンドウ（ジョブの一覧・絞り込み・詳細・操作。設計: CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! ジョブの記録は保存先から一定の間隔で読み直す。記録を書くのはサービスのスレッドだけのため、操作はサービスに渡し、
//! 結果が返ったら記録を読み直す。

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, CentralPanel, Color32, Grid, Id, Modal, Panel, RichText, ScrollArea};
use i18n_foundation::UiLocale;
use job_runtime::store::{JobStore, StoredJob, StoredJobKind, StoredJobStatus};

use crate::messages::text;
use crate::rows::{JobRow, elapsed_ms, format_elapsed, format_size, job_rows};
use crate::service::{JobCommand, ServiceChannel, ServiceRequest, now_ms};

/// ジョブの記録を読み直す間隔
const JOBS_INTERVAL: Duration = Duration::from_secs(1);
/// 操作の結果を待つ間に、描き直す間隔
const REPLY_INTERVAL: Duration = Duration::from_millis(50);
/// 保存先の大きさを測り直す間隔
const SIZE_INTERVAL: Duration = Duration::from_secs(10);
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

/// 削除を確かめているジョブと、その成果物を使う後のジョブ
#[derive(Debug, Clone)]
struct DeleteConfirmation {
    job_id: u64,
    dependents: Vec<u64>,
}

/// 選んだジョブの、ファイルから読んだ詳細（ジョブの記録を読み直したとき・選び直したときに読み直す）
struct SelectedFiles {
    job_id: u64,
    input_size: Option<u64>,
    result: Option<(PathBuf, Option<u64>)>,
    log: Option<String>,
}

/// ジョブマネージャーのウィンドウ
pub(crate) struct JobManagerWindow {
    store: JobStore,
    service: ServiceChannel,
    locale: UiLocale,
    jobs: Vec<StoredJob>,
    /// 一覧の行（ジョブの記録・絞り込みが変わったときに並べ直す）
    rows: Vec<JobRow>,
    unreadable_jobs: usize,
    store_error: Option<String>,
    jobs_read_at: Option<Instant>,
    store_size: Option<u64>,
    size_read_at: Option<Instant>,
    /// 保存先の大きさを測っているスレッドからの結果
    size_receiver: Option<mpsc::Receiver<u64>>,
    filter: Option<StoredJobStatus>,
    selected: Option<u64>,
    selected_files: Option<SelectedFiles>,
    tab: DetailsTab,
    /// 保存先を読み書きできずに、できなかった操作の理由
    command_error: Option<String>,
    /// 結果を待っている操作の数（待つ間は、二重に頼まないように操作できなくする）
    pending_replies: usize,
    delete_confirmation: Option<DeleteConfirmation>,
    /// 前の描画でマウスが乗っていた一覧の行（見出しの行が 0）
    hovered_row: Option<usize>,
    /// 閉じたときに、ウィンドウを隠す（タスクトレイから開き直す）
    hide_on_close: bool,
}

impl JobManagerWindow {
    pub(crate) fn new(
        store: JobStore,
        service: ServiceChannel,
        locale: UiLocale,
        hide_on_close: bool,
    ) -> Self {
        Self {
            store,
            service,
            locale,
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
            selected_files: None,
            tab: DetailsTab::Details,
            command_error: None,
            pending_replies: 0,
            delete_confirmation: None,
            hovered_row: None,
            hide_on_close,
        }
    }

    fn text(&self, key: &str) -> String {
        text(self.locale, key, &[])
    }

    fn text_with(&self, key: &str, args: &[(&str, String)]) -> String {
        text(self.locale, key, args)
    }

    fn refresh(&mut self) {
        // 操作の結果が返ったら、記録をすぐに読み直す
        while let Ok(reply) = self.service.replies.try_recv() {
            self.pending_replies = self.pending_replies.saturating_sub(1);
            self.command_error = reply.err();
            self.jobs_read_at = None;
        }
        let due =
            |read_at: Option<Instant>, interval| read_at.is_none_or(|at| at.elapsed() >= interval);
        if due(self.jobs_read_at, JOBS_INTERVAL) {
            self.jobs_read_at = Some(Instant::now());
            match self.store.load_jobs() {
                Ok((jobs, unreadable)) => {
                    self.jobs = jobs;
                    self.unreadable_jobs = unreadable.len();
                    self.store_error = None;
                }
                Err(error) => self.store_error = Some(error.to_string()),
            }
            self.rows = job_rows(&self.jobs, self.filter);
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
            input_size: size(&self.store.input_file(job_id)),
            result: result.is_file().then(|| {
                let bytes = size(&result);
                (result, bytes)
            }),
            log: log_tail(&log).or_else(|| log_tail(Path::new(&partial))),
        })
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
        match self.service.requests.send(ServiceRequest::Command(command)) {
            Ok(()) => self.pending_replies += 1,
            Err(_) => self.command_error = Some("the job manager service has stopped".to_string()),
        }
    }

    /// 選んだジョブの操作（状態で操作できるものだけを押せる）
    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        use StoredJobStatus::{Canceled, Failed, Queued, Running};
        let selected = self
            .selected_job()
            .filter(|_| self.pending_replies == 0)
            .map(|job| (job.job_id, job.status));
        let can = |allowed: &[StoredJobStatus]| {
            selected.is_some_and(|(_, status)| allowed.contains(&status))
        };
        let buttons = [
            ("toolbar.cancel", can(&[Queued, Running])),
            ("toolbar.retry", can(&[Failed, Canceled])),
            (
                "toolbar.delete",
                selected.is_some_and(|(_, status)| status.is_terminal()),
            ),
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
        let (Some(key), Some((job_id, _))) = (clicked, selected) else {
            return;
        };
        match key {
            "toolbar.cancel" => self.send(JobCommand::Cancel(job_id)),
            "toolbar.retry" => self.send(JobCommand::Retry(job_id)),
            "toolbar.delete" => {
                let dependents: Vec<u64> = self
                    .jobs
                    .iter()
                    .filter(|job| job.depends_on.contains(&job_id))
                    .map(|job| job.job_id)
                    .collect();
                if dependents.is_empty() {
                    self.send(JobCommand::Delete(job_id));
                } else {
                    self.delete_confirmation = Some(DeleteConfirmation { job_id, dependents });
                }
            }
            "toolbar.raise" => self.send(JobCommand::ChangePriority { job_id, delta: 1 }),
            _ => self.send(JobCommand::ChangePriority { job_id, delta: -1 }),
        }
    }

    /// 後のジョブが成果物を使うジョブを消すかを確かめる
    fn show_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(confirmation) = self.delete_confirmation.clone() else {
            return;
        };
        let jobs: Vec<String> = confirmation.dependents.iter().map(u64::to_string).collect();
        let message = self.text_with(
            "delete.confirm",
            &[
                ("job", confirmation.job_id.to_string()),
                ("jobs", jobs.join(", ")),
            ],
        );
        let (yes, no) = (
            self.text("delete.confirm_yes"),
            self.text("delete.confirm_no"),
        );
        let mut choice = None;
        let response = Modal::new(Id::new("delete_confirmation")).show(ctx, |ui| {
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
                self.send(JobCommand::Delete(confirmation.job_id));
            }
            Some(false) => self.delete_confirmation = None,
            None => {}
        }
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.show_toolbar(ui);
            ui.separator();
            if let Some(reason) = &self.command_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    self.text_with("command.failed", &[("reason", reason.clone())]),
                );
            }
            if let Some(size) = self.store_size {
                ui.label(self.text_with("store.size", &[("size", format_size(size))]));
            }
            if let Some(reason) = &self.store_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    self.text_with("store.unreadable", &[("reason", reason.clone())]),
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
            self.rows = job_rows(&self.jobs, self.filter);
        }
    }

    fn show_jobs(&mut self, ui: &mut egui::Ui) {
        let now = now_ms();
        // 行を描く間は自分を借りるため、行を一時に取り出し、描き終えたら戻す
        let rows = std::mem::take(&mut self.rows);
        // 選んだ行・マウスが乗った行の色（見出しの行が 0 のため、ジョブの行は 1 から）。乗った行は、前の描画のもの
        let selected_row = rows
            .iter()
            .position(|row| Some(row.job.job_id) == self.selected)
            .map(|index| index + 1);
        let hovered_row = self.hovered_row;
        let mut hovered_now = None;
        ScrollArea::vertical().show(ui, |ui| {
            // 行のどこを押しても選べるように、文字は押せないようにする
            ui.style_mut().interaction.selectable_labels = false;
            let row_spacing = ui.spacing().item_spacing.y;
            Grid::new("jobs")
                .num_columns(7)
                .with_row_color(move |row, style| {
                    if Some(row) == selected_row {
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
                        cells.push(centered_label(ui, local_time(job.created_at_ms)));
                        // 行のすべての欄と、行の間の半分を、押せる範囲にする
                        let area = cells
                            .into_iter()
                            .reduce(|a, b| a.union(b))
                            .unwrap_or(egui::Rect::NOTHING)
                            .expand2(egui::vec2(0.0, row_spacing / 2.0));
                        let response = ui.interact(
                            area,
                            ui.id().with(("job_row", job.job_id)),
                            egui::Sense::click(),
                        );
                        if response.clicked() {
                            self.selected = Some(job.job_id);
                        }
                        if response.hovered() {
                            hovered_now = Some(index + 1);
                        }
                        ui.end_row();
                    }
                });
        });
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
        if let Some(reason) = &job.last_error {
            ui.colored_label(
                ui.visuals().error_fg_color,
                self.text_with("details.error", &[("reason", reason.clone())]),
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
        self.refresh();
        self.show_delete_confirmation(ui.ctx());
        Panel::top("header").show_inside(ui, |ui| self.show_header(ui));
        Panel::left("filter")
            .resizable(true)
            .default_size(150.0)
            .show_inside(ui, |ui| self.show_filter(ui));
        Panel::bottom("details")
            .resizable(true)
            .default_size(200.0)
            .show_inside(ui, |ui| self.show_details(ui));
        CentralPanel::default().show_inside(ui, |ui| self.show_jobs(ui));
        // 記録・経過時間を、操作がなくても読み直して描き直す（操作の結果を待つ間は、結果が返ったらすぐ出す）
        ui.ctx().request_repaint_after(if self.pending_replies > 0 {
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

/// UNIX 時刻のミリ秒を、この PC の時刻（月/日 時:分）で書く
fn local_time(ms: u64) -> String {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_millis_opt(ms as i64)
        .single()
        .map_or_else(
            || "-".to_string(),
            |time| time.format("%m/%d %H:%M").to_string(),
        )
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
