//! ジョブが終わったときに示すか（タスクトレイの吹き出し）

use job_manager::ManagerEvent;
use job_runtime::store::{StoredJob, StoredJobKind, StoredJobStatus};

/// 示す終わり方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Finish {
    Succeeded,
    Failed,
}

/// 出来事 `event` で示すジョブ（終わり方・番号・種類）。ジョブの記録 `jobs` は、出来事の後のもの
///
/// 1 つの投入は、ツールパス生成とその成果物を使う切削シミュレーションのように、続くジョブを持つ。続けて計算する
/// ジョブの途中では示さない。正常に終わったときは、そのジョブを待つ終わっていないジョブがないとき（最後のジョブ）
/// だけ示す。失敗したときは、失敗したジョブだけを示し、その失敗で計算できなくなった後のジョブは示さない。
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn finished_job(
    event: &ManagerEvent,
    jobs: &[StoredJob],
) -> Option<(Finish, u64, StoredJobKind)> {
    let (finish, job_id) = match event {
        ManagerEvent::Finished {
            job_id,
            status: StoredJobStatus::Succeeded,
            ..
        } => (Finish::Succeeded, *job_id),
        ManagerEvent::Finished {
            job_id,
            status: StoredJobStatus::Failed,
            ..
        } => (Finish::Failed, *job_id),
        _ => return None,
    };
    let job = jobs.iter().find(|job| job.job_id == job_id)?;
    // 同じ回の仕事で後のジョブを起動していることがあるため、終わっていない後のジョブはすべて待っているとみなす
    let waited = jobs
        .iter()
        .any(|other| !other.status.is_terminal() && other.depends_on.contains(&job_id));
    if finish == Finish::Succeeded && waited {
        return None;
    }
    Some((finish, job_id, job.kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use job_runtime::store::STORE_FORMAT_VERSION;

    fn job(
        job_id: u64,
        kind: StoredJobKind,
        depends_on: Vec<u64>,
        status: StoredJobStatus,
    ) -> StoredJob {
        StoredJob {
            format_version: STORE_FORMAT_VERSION,
            job_id,
            submission_id: "a1".to_string(),
            kind,
            depends_on,
            priority: 0,
            group_id: None,
            source: None,
            status,
            attempts: 1,
            created_at_ms: 0,
            updated_at_ms: 0,
            started_at_ms: None,
            result_ref: None,
            log_ref: None,
            last_error: None,
            error: None,
        }
    }

    fn finished(job_id: u64, status: StoredJobStatus) -> ManagerEvent {
        ManagerEvent::Finished {
            job_id,
            status,
            error: None,
        }
    }

    #[test]
    fn only_the_last_job_of_a_chain_is_shown_when_it_succeeds() {
        let mut jobs = vec![
            job(1, StoredJobKind::Cam, vec![], StoredJobStatus::Succeeded),
            job(2, StoredJobKind::Sim, vec![1], StoredJobStatus::Queued),
        ];
        assert_eq!(
            finished_job(&finished(1, StoredJobStatus::Succeeded), &jobs),
            None
        );
        // 同じ回の仕事で、後のジョブを起動した
        jobs[1].status = StoredJobStatus::Running;
        assert_eq!(
            finished_job(&finished(1, StoredJobStatus::Succeeded), &jobs),
            None
        );
        jobs[1].status = StoredJobStatus::Succeeded;
        assert_eq!(
            finished_job(&finished(2, StoredJobStatus::Succeeded), &jobs),
            Some((Finish::Succeeded, 2, StoredJobKind::Sim))
        );
    }

    #[test]
    fn a_failed_job_is_shown_but_the_jobs_it_blocked_are_not() {
        let jobs = vec![
            job(1, StoredJobKind::Cam, vec![], StoredJobStatus::Failed),
            job(2, StoredJobKind::Sim, vec![1], StoredJobStatus::Queued),
        ];
        assert_eq!(
            finished_job(&finished(1, StoredJobStatus::Failed), &jobs),
            Some((Finish::Failed, 1, StoredJobKind::Cam))
        );
        let blocked = ManagerEvent::Blocked {
            job_id: 2,
            reason: "dependency job 1 failed".to_string(),
        };
        assert_eq!(finished_job(&blocked, &jobs), None);
        assert_eq!(
            finished_job(&ManagerEvent::Started { job_id: 1 }, &jobs),
            None
        );
    }
}
