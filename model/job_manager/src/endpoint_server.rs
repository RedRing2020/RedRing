//! ジョブマネージャーに問い合わせる口の受け手（設計: JOB_IPC_DESIGN.md 5・6 章）
//!
//! 別のスレッドで接続を受け、接続ごとのスレッドで要求を処理する。状態と成果物は保存先から読み、投入は受付のフォルダに
//! 書く（ジョブの記録を書くのは、サービスのスレッドだけ）。読めるファイルは、ジョブのフォルダの決まった名前の
//! ファイルだけとする（記録の参照からパスを作らない）。

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use job_ipc::{
    Dependency, Endpoint, ErrorKind, FileContent, Handler, HandlerError, JobFile, JobKind,
    JobSource, JobStatus, JobSummary, PendingSubmission, SubmitJob,
};
use job_runtime::store::{
    DependencyRef, JobStore, StoreError, StoredJobKind, StoredJobSource, StoredJobStatus,
    SubmissionRequest, SubmittedJob, new_submission_id, request_format_version,
};

/// ほかのプロセスが口を開いているときに、開き直す間隔（保存先を替えて起動し直したときは、前のプロセスが口を閉じる
/// まで開けないため。閉じるまで開き直し続ける）
const LISTEN_RETRY_INTERVAL: Duration = Duration::from_millis(200);
/// 口を開き直している間に、ログに書く間隔
const LISTEN_LOG_INTERVAL: Duration = Duration::from_secs(15);
/// 接続を受けられないときに、受け直すまでの間隔
const ACCEPT_RETRY_INTERVAL: Duration = Duration::from_secs(1);

/// ウィンドウを開く口（開けたら `true`）
pub type OpenWindow = Arc<dyn Fn() -> bool + Send + Sync>;

/// 口 `endpoint` を開き、保存先 `store` の要求を受けるスレッドを始める（ほかのプロセスが口を開いている間は、閉じる
/// まで開き直す）。ウィンドウを開く要求は `open_window` で行い、版を確かめたときに `manager_version` を返す
pub fn start_endpoint_server(
    endpoint: Endpoint,
    store: JobStore,
    open_window: OpenWindow,
    manager_version: &'static str,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut logged_at: Option<Instant> = None;
        let mut listener = loop {
            match endpoint.listen() {
                Ok(listener) => break listener,
                Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                    if logged_at.is_none_or(|at| at.elapsed() >= LISTEN_LOG_INTERVAL) {
                        eprintln!("the job manager endpoint is still open elsewhere: {error}");
                        logged_at = Some(Instant::now());
                    }
                    std::thread::sleep(LISTEN_RETRY_INTERVAL);
                }
                Err(error) => {
                    eprintln!("cannot open the job manager endpoint: {error}");
                    return;
                }
            }
        };
        loop {
            match listener.accept() {
                Ok(mut connection) => {
                    let mut handler = StoreHandler {
                        store: store.clone(),
                        open_window: Arc::clone(&open_window),
                    };
                    std::thread::spawn(move || {
                        if let Err(error) =
                            job_ipc::serve(&mut connection, &mut handler, manager_version)
                        {
                            eprintln!("job manager endpoint connection error: {error}");
                        }
                    });
                }
                Err(error) => {
                    eprintln!("cannot accept a job manager endpoint connection: {error}");
                    std::thread::sleep(ACCEPT_RETRY_INTERVAL);
                }
            }
        }
    })
}

/// 保存先の要求を処理する受け手
struct StoreHandler {
    store: JobStore,
    open_window: OpenWindow,
}

/// 保存先の誤りを、口の誤りにする
fn store_error(error: &StoreError) -> HandlerError {
    match error {
        StoreError::Io(io) if io.kind() == io::ErrorKind::InvalidInput => {
            HandlerError::new(ErrorKind::InvalidRequest, error.to_string())
        }
        _ => HandlerError::new(ErrorKind::StoreUnavailable, error.to_string()),
    }
}

fn kind_to_ipc(kind: StoredJobKind) -> JobKind {
    match kind {
        StoredJobKind::Cam => JobKind::Cam,
        StoredJobKind::Sim => JobKind::Sim,
    }
}

fn kind_from_ipc(kind: JobKind) -> StoredJobKind {
    match kind {
        JobKind::Cam => StoredJobKind::Cam,
        JobKind::Sim => StoredJobKind::Sim,
    }
}

fn status_to_ipc(status: StoredJobStatus) -> JobStatus {
    match status {
        StoredJobStatus::Queued => JobStatus::Queued,
        StoredJobStatus::Running => JobStatus::Running,
        StoredJobStatus::Succeeded => JobStatus::Succeeded,
        StoredJobStatus::Failed => JobStatus::Failed,
        StoredJobStatus::Canceled => JobStatus::Canceled,
    }
}

fn source_to_ipc(source: &StoredJobSource) -> JobSource {
    JobSource {
        source_id: source.source_id.clone(),
        revision: source.revision,
        revision_key: source.revision_key.clone(),
    }
}

fn source_from_ipc(source: JobSource) -> StoredJobSource {
    StoredJobSource {
        source_id: source.source_id,
        revision: source.revision,
        revision_key: source.revision_key,
    }
}

impl Handler for StoreHandler {
    fn submit(
        &mut self,
        jobs: Vec<SubmitJob>,
        inputs: Vec<(String, Vec<u8>)>,
    ) -> Result<String, HandlerError> {
        let invalid = |message: &str| HandlerError::new(ErrorKind::InvalidRequest, message);
        if jobs.is_empty() {
            return Err(invalid("no jobs to submit"));
        }
        // ジョブが使う入力は、すべて送られていること（ほかの決まり（依存の番号・種類と CL の組み合わせなど）は、
        // 取り込みで確かめ、取り込めなかった投入にする）
        let names: HashSet<&str> = inputs.iter().map(|(name, _)| name.as_str()).collect();
        if names.len() != inputs.len() {
            return Err(invalid("an input name is given twice"));
        }
        let uses_missing_input = jobs.iter().any(|job| {
            !names.contains(job.input_file.as_str())
                || job
                    .toolpath_file
                    .as_deref()
                    .is_some_and(|name| !names.contains(name))
        });
        if uses_missing_input {
            return Err(invalid("a job uses an input that is not sent"));
        }
        let jobs: Vec<SubmittedJob> = jobs
            .into_iter()
            .map(|job| SubmittedJob {
                kind: kind_from_ipc(job.kind),
                input_file: job.input_file,
                depends_on: job
                    .depends_on
                    .into_iter()
                    .map(|dependency| match dependency {
                        Dependency::Submitted(index) => DependencyRef::Submitted(index),
                        Dependency::Existing(job_id) => DependencyRef::Existing(job_id),
                    })
                    .collect(),
                priority: job.priority,
                group_id: None,
                source: job.source.map(source_from_ipc),
                toolpath_file: job.toolpath_file,
            })
            .collect();
        let request = SubmissionRequest {
            format_version: request_format_version(&jobs),
            jobs,
        };
        let inputs: Vec<(&str, &[u8])> = inputs
            .iter()
            .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
            .collect();
        let submission_id = new_submission_id();
        self.store
            .write_submission(&submission_id, &request, &inputs)
            .map_err(|error| store_error(&error))?;
        Ok(submission_id)
    }

    fn jobs(&mut self) -> Result<Vec<JobSummary>, HandlerError> {
        let (jobs, _) = self
            .store
            .load_jobs()
            .map_err(|error| store_error(&error))?;
        Ok(jobs
            .into_iter()
            .map(|job| JobSummary {
                job_id: job.job_id,
                kind: kind_to_ipc(job.kind),
                status: status_to_ipc(job.status),
                has_input_toolpath: self.store.input_toolpath_file(job.job_id).is_file(),
                has_result: job.result_ref.is_some()
                    && self.store.result_file(job.job_id, job.kind).is_file(),
                depends_on: job.depends_on,
                submission_id: job.submission_id,
                source: job.source.as_ref().map(source_to_ipc),
            })
            .collect())
    }

    fn pending_submissions(&mut self) -> Result<Vec<PendingSubmission>, HandlerError> {
        let submissions = self
            .store
            .inbox_submissions()
            .map_err(|error| store_error(&error))?;
        Ok(submissions
            .into_iter()
            .map(|submission| PendingSubmission {
                source: submission
                    .request
                    .as_ref()
                    .and_then(|request| request.jobs.first())
                    .and_then(|job| job.source.as_ref())
                    .map(source_to_ipc),
                rejected: submission.rejected.is_some(),
            })
            .collect())
    }

    fn read_job_file(&mut self, job_id: u64, file: JobFile) -> Result<FileContent, HandlerError> {
        let (jobs, _) = self
            .store
            .load_jobs()
            .map_err(|error| store_error(&error))?;
        let job = jobs
            .iter()
            .find(|job| job.job_id == job_id)
            .ok_or_else(|| HandlerError::new(ErrorKind::JobNotFound, format!("job {job_id}")))?;
        let not_found =
            || HandlerError::new(ErrorKind::FileNotFound, format!("{file:?} of job {job_id}"));
        let path = match file {
            JobFile::Input => self.store.input_file(job_id),
            JobFile::InputToolpath => self.store.input_toolpath_file(job_id),
            // 成果物は、記録に成果物があるときだけ、ジョブの番号と種類から決まる名前で読む
            JobFile::Result if job.result_ref.is_some() => self.store.result_file(job_id, job.kind),
            JobFile::Result => return Err(not_found()),
        };
        let opened = File::open(&path).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => not_found(),
            _ => HandlerError::new(ErrorKind::StoreUnavailable, error.to_string()),
        })?;
        let size = opened
            .metadata()
            .map_err(|error| HandlerError::new(ErrorKind::StoreUnavailable, error.to_string()))?
            .len();
        Ok(FileContent {
            size,
            reader: Box::new(opened.take(size)),
        })
    }

    fn open_window(&mut self) -> Result<(), HandlerError> {
        if (self.open_window)() {
            Ok(())
        } else {
            Err(HandlerError::new(
                ErrorKind::StoreUnavailable,
                "cannot open the job manager window",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicU64, Ordering};

    use job_ipc::Client;

    /// 試験ごとの保存先と口（利用者の実際の保存先と口に触れない）
    fn test_store_and_endpoint() -> (std::path::PathBuf, Endpoint) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "endpoint-server-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(format!("redring-{name}"));
        let _ = std::fs::remove_dir_all(&root);
        (root, Endpoint::with_name(&name).unwrap())
    }

    #[test]
    fn requests_are_answered_from_the_store() {
        let (root, endpoint) = test_store_and_endpoint();
        let store = JobStore::open(&root).unwrap();
        let opened = Arc::new(AtomicU64::new(0));
        let counted = Arc::clone(&opened);
        start_endpoint_server(
            endpoint.clone(),
            store.clone(),
            Arc::new(move || {
                counted.fetch_add(1, Ordering::Relaxed);
                true
            }),
            "0.0.0",
        );
        let mut client = Client::new(endpoint.connect(Duration::from_secs(5)).unwrap()).unwrap();
        let source = JobSource {
            source_id: "op-1".to_string(),
            revision: 1,
            revision_key: Some("key".to_string()),
        };
        let job = |kind, input: &str, depends_on| SubmitJob {
            kind,
            input_file: input.to_string(),
            depends_on,
            priority: 0,
            source: Some(source.clone()),
            toolpath_file: None,
        };
        let jobs = vec![
            job(JobKind::Cam, "cam.bin", vec![]),
            job(JobKind::Sim, "sim.bin", vec![Dependency::Submitted(0)]),
        ];
        // 送っていない入力を使う投入は、受付のフォルダに置かない
        assert!(matches!(
            client.submit(jobs.clone(), &[("cam.bin", b"cam")]),
            Err(job_ipc::ClientError::Remote {
                kind: ErrorKind::InvalidRequest,
                ..
            })
        ));
        client
            .submit(jobs, &[("cam.bin", b"cam"), ("sim.bin", b"sim")])
            .unwrap();
        let pending = client.pending_submissions().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].source, Some(source));
        assert!(!pending[0].rejected);

        // ジョブマネージャーが取り込むと、ジョブとその入力を読める
        let (mut service, _) = crate::JobManagerService::start(
            store,
            crate::ProcessLauncher::new(root.join("no-runner")),
            1,
        )
        .unwrap();
        service.tick(2);
        let summaries = client.jobs().unwrap();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[1].depends_on, vec![1]);
        assert!(!summaries[0].has_result);
        assert_eq!(client.read_job_file(1, JobFile::Input).unwrap(), b"cam");
        assert!(matches!(
            client.read_job_file(1, JobFile::Result),
            Err(job_ipc::ClientError::Remote {
                kind: ErrorKind::FileNotFound,
                ..
            })
        ));
        assert!(matches!(
            client.read_job_file(9, JobFile::Input),
            Err(job_ipc::ClientError::Remote {
                kind: ErrorKind::JobNotFound,
                ..
            })
        ));
        client.open_window().unwrap();
        assert_eq!(opened.load(Ordering::Relaxed), 1);
        drop(client);
        drop(service);
        let _ = std::fs::remove_dir_all(&root);
    }
}
