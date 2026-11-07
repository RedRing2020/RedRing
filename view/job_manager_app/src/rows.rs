//! ジョブの一覧の行（並べ方・絞り込み）と、時間・大きさの書き方

use std::collections::{HashMap, HashSet};

use job_runtime::store::{StoredJob, StoredJobKind, StoredJobStatus};

/// 一覧の 1 行
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JobRow {
    pub job: StoredJob,
    /// 前の行のツールパス生成の成果物を使う切削シミュレーション（字下げして出す）
    pub child: bool,
}

/// ジョブを一覧の行に並べる。ツールパス生成の下に、その成果物を使う切削シミュレーションを置く（番号の順）
///
/// `filter` を指定したときは、その状態のジョブだけを出す（親が出ないときは、子も字下げしない）。
pub(crate) fn job_rows(jobs: &[StoredJob], filter: Option<StoredJobStatus>) -> Vec<JobRow> {
    let shown = |job: &StoredJob| filter.is_none_or(|status| job.status == status);
    let cam_jobs: HashSet<u64> = jobs
        .iter()
        .filter(|job| job.kind == StoredJobKind::Cam)
        .map(|job| job.job_id)
        .collect();
    let parent_of = |job: &StoredJob| {
        (job.kind == StoredJobKind::Sim)
            .then(|| {
                job.depends_on
                    .iter()
                    .copied()
                    .find(|id| cam_jobs.contains(id))
            })
            .flatten()
    };
    let mut children: HashMap<u64, Vec<&StoredJob>> = HashMap::new();
    let mut roots = Vec::new();
    for job in jobs {
        match parent_of(job) {
            Some(parent) => children.entry(parent).or_default().push(job),
            None => roots.push(job),
        }
    }
    let mut rows = Vec::new();
    for job in roots {
        let parent_shown = shown(job);
        if parent_shown {
            rows.push(JobRow {
                job: job.clone(),
                child: false,
            });
        }
        for child in children.get(&job.job_id).into_iter().flatten() {
            if shown(child) {
                rows.push(JobRow {
                    job: (*child).clone(),
                    child: parent_shown,
                });
            }
        }
    }
    rows
}

/// 計算にかかった時間（計算中は今までの時間）。まだ始めていなければ `None`
pub(crate) fn elapsed_ms(job: &StoredJob, now_ms: u64) -> Option<u64> {
    let started = job.started_at_ms?;
    let end = match job.status {
        StoredJobStatus::Running => now_ms,
        StoredJobStatus::Queued => return None,
        _ => job.updated_at_ms,
    };
    Some(end.saturating_sub(started))
}

/// 時間を `分:秒`（1 時間以上は `時:分:秒`）で書く
pub(crate) fn format_elapsed(ms: u64) -> String {
    let seconds = ms / 1000;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// 大きさを B・KB・MB・GB で書く
pub(crate) fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use job_runtime::store::STORE_FORMAT_VERSION;

    fn job(
        job_id: u64,
        kind: StoredJobKind,
        depends_on: &[u64],
        status: StoredJobStatus,
    ) -> StoredJob {
        StoredJob {
            format_version: STORE_FORMAT_VERSION,
            job_id,
            submission_id: "s".to_string(),
            kind,
            depends_on: depends_on.to_vec(),
            priority: 0,
            group_id: None,
            source: None,
            status,
            attempts: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
            started_at_ms: None,
            result_ref: None,
            log_ref: None,
            last_error: None,
        }
    }

    use StoredJobKind::{Cam, Sim};
    use StoredJobStatus::{Failed, Queued, Running, Succeeded};

    fn shape(rows: &[JobRow]) -> Vec<(u64, bool)> {
        rows.iter().map(|row| (row.job.job_id, row.child)).collect()
    }

    #[test]
    fn simulations_are_placed_under_their_toolpath_jobs() {
        let jobs = vec![
            job(1, Cam, &[], Succeeded),
            job(2, Sim, &[1], Running),
            job(3, Cam, &[], Queued),
            job(4, Sim, &[3], Queued),
            // ツールパス生成の記録がない切削シミュレーションは、字下げしない
            job(6, Sim, &[5], Failed),
        ];
        assert_eq!(
            shape(&job_rows(&jobs, None)),
            vec![(1, false), (2, true), (3, false), (4, true), (6, false)]
        );
        // 親が絞り込みで出ないときは、子を字下げしない
        assert_eq!(
            shape(&job_rows(&jobs, Some(Queued))),
            vec![(3, false), (4, true)]
        );
        assert_eq!(shape(&job_rows(&jobs, Some(Running))), vec![(2, false)]);
    }

    #[test]
    fn elapsed_time_runs_until_now_or_the_end() {
        let mut running = job(1, Cam, &[], Running);
        running.started_at_ms = Some(1_000);
        assert_eq!(elapsed_ms(&running, 4_500), Some(3_500));
        let mut done = running.clone();
        done.status = Succeeded;
        done.updated_at_ms = 2_000;
        assert_eq!(elapsed_ms(&done, 9_000), Some(1_000));
        assert_eq!(elapsed_ms(&job(2, Cam, &[], Queued), 9_000), None);
        assert_eq!(format_elapsed(65_000), "1:05");
        assert_eq!(format_elapsed(3_725_000), "1:02:05");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(3 * 1024 * 1024 / 2), "1.5 MB");
    }
}
