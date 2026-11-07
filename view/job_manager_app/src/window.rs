//! ジョブマネージャーのウィンドウ（ジョブの一覧・絞り込み・詳細。設計: CAM_OPERATION_UI_DESIGN.md 3.7）
//!
//! ジョブの記録は保存先から一定の間隔で読み直す（記録を書くのはサービスのスレッドだけ）。

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, CentralPanel, Color32, Grid, Panel, RichText, ScrollArea};
use i18n_foundation::UiLocale;
use job_runtime::store::{JobStore, StoredJob, StoredJobKind, StoredJobStatus};

use crate::messages::text;
use crate::rows::{JobRow, elapsed_ms, format_elapsed, format_size, job_rows};
use crate::service::now_ms;

/// ジョブの記録を読み直す間隔
const JOBS_INTERVAL: Duration = Duration::from_secs(1);
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
}

impl JobManagerWindow {
    pub(crate) fn new(store: JobStore, locale: UiLocale) -> Self {
        Self {
            store,
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
        }
    }

    fn text(&self, key: &str) -> String {
        text(self.locale, key, &[])
    }

    fn text_with(&self, key: &str, args: &[(&str, String)]) -> String {
        text(self.locale, key, args)
    }

    fn refresh(&mut self) {
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

    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
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
        ui.heading(self.text("filter.header"));
        let before = self.filter;
        let all = self.text("filter.all");
        ui.radio_value(&mut self.filter, None, all);
        for status in STATUSES {
            let label = self.status_text(status);
            ui.radio_value(&mut self.filter, Some(status), label);
        }
        if self.filter != before {
            self.rows = job_rows(&self.jobs, self.filter);
        }
    }

    fn show_jobs(&mut self, ui: &mut egui::Ui) {
        let now = now_ms();
        let rows = std::mem::take(&mut self.rows);
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("jobs")
                .striped(true)
                .num_columns(6)
                .show(ui, |ui| {
                    for key in [
                        "column.id",
                        "column.source",
                        "column.kind",
                        "column.status",
                        "column.elapsed",
                        "column.submitted",
                    ] {
                        ui.label(RichText::new(self.text(key)).strong());
                    }
                    ui.end_row();
                    for row in rows {
                        let job = &row.job;
                        let id = if row.child {
                            format!("  └ {}", job.job_id)
                        } else {
                            job.job_id.to_string()
                        };
                        let selected = self.selected == Some(job.job_id);
                        if ui.selectable_label(selected, id).clicked() {
                            self.selected = Some(job.job_id);
                        }
                        ui.label(self.source_text(job));
                        ui.label(self.text(match job.kind {
                            StoredJobKind::Cam => "kind.cam",
                            StoredJobKind::Sim => "kind.sim",
                        }));
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(self.status_text(job.status))
                                    .color(status_color(job.status)),
                            );
                            // 進み具合の割合はまだないため、計算中は動いていることだけを出す
                            if job.status == StoredJobStatus::Running {
                                ui.spinner();
                            }
                        });
                        ui.label(
                            elapsed_ms(job, now).map_or_else(|| "-".to_string(), format_elapsed),
                        );
                        ui.label(local_time(job.created_at_ms));
                        ui.end_row();
                    }
                });
        });
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
        let Some(job) = self
            .selected
            .and_then(|id| self.jobs.iter().find(|job| job.job_id == id))
            .cloned()
        else {
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
        self.refresh();
        Panel::top("header").show_inside(ui, |ui| self.show_header(ui));
        Panel::left("filter").show_inside(ui, |ui| self.show_filter(ui));
        Panel::bottom("details")
            .resizable(true)
            .default_size(200.0)
            .show_inside(ui, |ui| self.show_details(ui));
        CentralPanel::default().show_inside(ui, |ui| self.show_jobs(ui));
        // 記録・経過時間を、操作がなくても読み直して描き直す
        ui.ctx().request_repaint_after(JOBS_INTERVAL);
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
