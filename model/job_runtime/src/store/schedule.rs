//! ジョブの実行の順（設計: JOB_MANAGER_STORAGE_DESIGN.md 5.5）
//!
//! - 依存するジョブがすべて正常に終わった待ちのジョブの中から、実効の優先順位の高いものを先に実行する。同じ高さは
//!   番号の小さい（先に投入した）ものを先にする
//! - 実効の優先順位は、自分の優先順位と、自分を待っている（終わっていない）ジョブの実効の優先順位の大きいほう。
//!   優先順位を上げたジョブが待っている前のジョブも同じ高さで実行され、上げたジョブが前のジョブを待ち続けない。
//!   依存関係を飛び越えては実行しない
//! - 依存するジョブが失敗・取り消しで終わった・記録がない・自分より後の番号の（書き換えられた記録の）待ちのジョブは、
//!   実行できない。実行できないジョブを待つジョブも実行できない。実行できないジョブは、ほかのジョブの優先順位を
//!   上げない
//! - 依存するジョブが中断の待ちのジョブは、実行せず、実行できないジョブにもしない（中断のジョブを計算で続けたら、
//!   そのまま実行できるように）。中断のジョブは、ほかのジョブの優先順位を上げない
//! - 記録を今は読めない依存先（ほかのプロセスが開いている等）は、読めるようになるまで待つ

use std::collections::{HashMap, HashSet};

use super::{StoredJob, StoredJobError, StoredJobStatus};

/// 実行の順の判断
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulePlan {
    /// 実行できない待ちのジョブと理由（英語。番号の順）。呼び出し側は失敗にする
    pub blocked: Vec<(u64, StoredJobError)>,
    /// 次に実行するジョブ（実行できるジョブがなければ `None`）
    pub next: Option<u64>,
}

/// ジョブの記録 `jobs` と、記録を今は読めないジョブの番号 `unreadable` から、実行の順を判断する
pub fn plan_schedule(jobs: &[StoredJob], unreadable: &[u64]) -> SchedulePlan {
    let by_id: HashMap<u64, &StoredJob> = jobs.iter().map(|job| (job.job_id, job)).collect();
    let unreadable: HashSet<u64> = unreadable.iter().copied().collect();
    let mut ordered: Vec<&StoredJob> = jobs.iter().collect();
    ordered.sort_by_key(|job| job.job_id);

    // 番号の小さいほうから、実行できないジョブを決める（依存先は番号の小さいジョブのため、先に決まっている）
    let mut blocked: Vec<(u64, StoredJobError)> = Vec::new();
    let mut blocked_ids: HashSet<u64> = HashSet::new();
    for job in &ordered {
        if job.status != StoredJobStatus::Queued {
            continue;
        }
        let reason = job.depends_on.iter().find_map(|&id| {
            if id >= job.job_id {
                return Some(StoredJobError::DependencyNotEarlier { job: id });
            }
            if blocked_ids.contains(&id) {
                return Some(StoredJobError::DependencyCannotRun { job: id });
            }
            match by_id.get(&id).map(|dependency| dependency.status) {
                None if unreadable.contains(&id) => None,
                None => Some(StoredJobError::DependencyNotFound { job: id }),
                Some(StoredJobStatus::Failed) => Some(StoredJobError::DependencyFailed { job: id }),
                Some(StoredJobStatus::Canceled) => {
                    Some(StoredJobError::DependencyCanceled { job: id })
                }
                Some(_) => None,
            }
        });
        if let Some(reason) = reason {
            blocked_ids.insert(job.job_id);
            blocked.push((job.job_id, reason));
        }
    }

    // 番号の大きいほうから、待っているジョブの高さを依存先へ伝える
    let mut effective: HashMap<u64, i32> =
        jobs.iter().map(|job| (job.job_id, job.priority)).collect();
    for job in ordered.iter().rev() {
        if job.status.is_terminal()
            || job.status == StoredJobStatus::Interrupted
            || blocked_ids.contains(&job.job_id)
        {
            continue;
        }
        let priority = effective[&job.job_id];
        for id in &job.depends_on {
            if let Some(dependency) = effective.get_mut(id) {
                *dependency = (*dependency).max(priority);
            }
        }
    }

    let next = ordered
        .iter()
        .filter(|job| job.status == StoredJobStatus::Queued && !blocked_ids.contains(&job.job_id))
        .filter(|job| {
            job.depends_on.iter().all(|id| {
                by_id
                    .get(id)
                    .is_some_and(|dependency| dependency.status == StoredJobStatus::Succeeded)
            })
        })
        .max_by(|a, b| {
            effective[&a.job_id]
                .cmp(&effective[&b.job_id])
                .then(b.job_id.cmp(&a.job_id))
        })
        .map(|job| job.job_id);
    SchedulePlan { blocked, next }
}
