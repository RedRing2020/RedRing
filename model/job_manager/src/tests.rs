use super::*;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use job_runtime::store::{DependencyRef, STORE_FORMAT_VERSION, SubmissionRequest, SubmittedJob};

/// 試験ごとの保存先（試験の終わりに消す）
struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "redring-job-manager-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        Self(path)
    }

    fn store(&self) -> JobStore {
        JobStore::open(&self.0).unwrap()
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 偽のランナーの共有の状態（起動した引数・止めたジョブ・種類ごとの終了の番号）
#[derive(Default)]
struct FakeState {
    launched: Vec<LaunchSpec>,
    stopped: Vec<PathBuf>,
    exit_codes: HashMap<&'static str, i32>,
    fail_to_launch: bool,
    /// 終わるまでの確かめの回数
    polls_until_done: usize,
}

#[derive(Clone, Default)]
struct FakeLauncher(Rc<RefCell<FakeState>>);

impl Launcher for FakeLauncher {
    fn launch(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn RunningJob>> {
        let mut state = self.0.borrow_mut();
        if state.fail_to_launch {
            return Err(std::io::Error::other("no runner"));
        }
        state.launched.push(spec.clone());
        Ok(Box::new(FakeJob {
            spec: spec.clone(),
            code: state.exit_codes.get(spec.job_type).copied().unwrap_or(0),
            polls_left: state.polls_until_done,
            state: self.0.clone(),
        }))
    }
}

struct FakeJob {
    spec: LaunchSpec,
    code: i32,
    polls_left: usize,
    state: Rc<RefCell<FakeState>>,
}

impl RunningJob for FakeJob {
    fn try_finish(&mut self) -> std::io::Result<Option<Option<i32>>> {
        if self.polls_left > 0 {
            self.polls_left -= 1;
            return Ok(None);
        }
        // ランナーと同じく、成功したら成果物を書き、ログを残す
        fs::create_dir_all(&self.spec.output)?;
        fs::create_dir_all(&self.spec.logs)?;
        if self.code == 0 {
            let name = match self.spec.job_type {
                "cam" => "toolpath.bin",
                _ => "cutting_simulation.bin",
            };
            fs::write(self.spec.output.join(name), b"artifact")?;
        }
        fs::write(
            self.spec.logs.join("runner.log"),
            format!("start\nfailed exit_code={} code=no_solution\n", self.code),
        )?;
        Ok(Some(Some(self.code)))
    }

    fn stop(&mut self) -> std::io::Result<()> {
        self.state
            .borrow_mut()
            .stopped
            .push(self.spec.input.clone());
        Ok(())
    }
}

/// 投入を書く（入力はジョブごとに `job<番号>.bin`）
fn submit(store: &JobStore, id: &str, jobs: Vec<SubmittedJob>) {
    let inbox = store.root().join("inbox");
    let temporary = inbox.join(format!("{id}.tmp"));
    fs::create_dir_all(&temporary).unwrap();
    for job in &jobs {
        fs::write(temporary.join(&job.input_file), b"input").unwrap();
    }
    let request = SubmissionRequest {
        format_version: STORE_FORMAT_VERSION,
        jobs,
    };
    fs::write(
        temporary.join("request.json"),
        serde_json::to_string(&request).unwrap(),
    )
    .unwrap();
    fs::rename(&temporary, inbox.join(id)).unwrap();
}

fn job(
    kind: StoredJobKind,
    input: &str,
    depends_on: Vec<DependencyRef>,
    priority: i32,
) -> SubmittedJob {
    SubmittedJob {
        kind,
        input_file: input.to_string(),
        depends_on,
        priority,
        group_id: None,
        source: None,
    }
}

fn cam_and_sim() -> Vec<SubmittedJob> {
    vec![
        job(StoredJobKind::Cam, "cam.bin", vec![], 0),
        job(
            StoredJobKind::Sim,
            "sim.bin",
            vec![DependencyRef::Submitted(0)],
            0,
        ),
    ]
}

fn status_of(store: &JobStore, job_id: u64) -> StoredJob {
    store
        .load_jobs()
        .unwrap()
        .0
        .into_iter()
        .find(|job| job.job_id == job_id)
        .unwrap()
}

#[test]
fn a_cam_job_runs_before_the_sim_job_that_reads_its_toolpath() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());

    let events = service.tick(1);
    assert!(events.contains(&ManagerEvent::Started { job_id: 1 }));
    assert_eq!(status_of(&root.store(), 1).status, StoredJobStatus::Running);
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Queued);

    service.tick(2);
    let cam = status_of(&root.store(), 1);
    assert_eq!(cam.status, StoredJobStatus::Succeeded);
    assert_eq!(cam.attempts, 1);
    assert_eq!(cam.result_ref.as_deref(), Some("result://cam/1/ok"));
    assert_eq!(
        root.store().file_for_ref("result://cam/1/ok").unwrap(),
        root.store().result_file(1, StoredJobKind::Cam)
    );
    let launched = launcher.0.borrow().launched.clone();
    assert_eq!(launched.len(), 2);
    assert_eq!(launched[1].job_type, "sim");
    assert_eq!(
        launched[1].toolpath,
        Some(root.store().result_file(1, StoredJobKind::Cam))
    );
    assert_eq!(launched[1].input, root.store().input_file(2));

    service.tick(3);
    assert_eq!(
        status_of(&root.store(), 2).status,
        StoredJobStatus::Succeeded
    );
}

#[test]
fn a_failed_job_records_the_reason_and_fails_the_jobs_waiting_for_it() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().exit_codes.insert("cam", 5);
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());

    service.tick(1);
    let events = service.tick(2);
    let cam = status_of(&root.store(), 1);
    assert_eq!(cam.status, StoredJobStatus::Failed);
    let error = cam.last_error.unwrap();
    assert!(error.contains("the calculation failed"), "{error}");
    assert!(error.contains("code=no_solution"), "{error}");
    assert!(events.contains(&ManagerEvent::Blocked {
        job_id: 2,
        reason: "dependency job 1 failed".to_string(),
    }));
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Failed);
    assert_eq!(launcher.0.borrow().launched.len(), 1);
}

#[test]
fn higher_priority_submissions_start_first_one_at_a_time() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 1;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(
        &root.store(),
        "a1",
        vec![job(StoredJobKind::Cam, "low.bin", vec![], 0)],
    );
    submit(
        &root.store(),
        "b2",
        vec![job(StoredJobKind::Cam, "high.bin", vec![], 5)],
    );

    service.tick(1);
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Running);
    assert_eq!(status_of(&root.store(), 1).status, StoredJobStatus::Queued);
    // 実行中のジョブが終わるまで、次は起動しない
    service.tick(2);
    assert_eq!(launcher.0.borrow().launched.len(), 1);
    service.tick(3);
    assert_eq!(status_of(&root.store(), 1).status, StoredJobStatus::Running);
}

#[test]
fn a_second_manager_on_the_same_store_is_refused() {
    let root = TempRoot::new();
    let (_service, _) = JobManagerService::start(root.store(), FakeLauncher::default(), 0).unwrap();
    assert!(matches!(
        JobManagerService::start(root.store(), FakeLauncher::default(), 0),
        Err(StoreError::AlreadyLocked)
    ));
}

#[test]
fn restarting_fails_the_interrupted_job_and_keeps_the_queue() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 10;
    {
        let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
        submit(
            &root.store(),
            "a1",
            vec![job(StoredJobKind::Cam, "a.bin", vec![], 0)],
        );
        submit(
            &root.store(),
            "b2",
            vec![job(StoredJobKind::Cam, "b.bin", vec![], 0)],
        );
        service.tick(1);
        assert_eq!(status_of(&root.store(), 1).status, StoredJobStatus::Running);
        // ジョブマネージャーが落ちた
    }

    let launcher = FakeLauncher::default();
    let (mut service, events) =
        JobManagerService::start(root.store(), launcher.clone(), 5).unwrap();
    assert_eq!(events, vec![ManagerEvent::Interrupted { job_id: 1 }]);
    let interrupted = status_of(&root.store(), 1);
    assert_eq!(interrupted.status, StoredJobStatus::Failed);
    assert!(interrupted.last_error.unwrap().contains("stopped"));
    service.tick(6);
    service.tick(7);
    assert_eq!(
        status_of(&root.store(), 2).status,
        StoredJobStatus::Succeeded
    );
}

#[test]
fn canceling_stops_a_running_job_and_cancels_a_queued_job() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 10;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    submit(
        &root.store(),
        "b2",
        vec![job(StoredJobKind::Cam, "b.bin", vec![], 0)],
    );
    service.tick(1);

    assert!(service.cancel(3, 2).unwrap());
    assert_eq!(
        status_of(&root.store(), 3).status,
        StoredJobStatus::Canceled
    );
    assert!(service.cancel(1, 3).unwrap());
    assert_eq!(
        launcher.0.borrow().stopped,
        vec![root.store().input_file(1)]
    );
    assert_eq!(
        status_of(&root.store(), 1).status,
        StoredJobStatus::Canceled
    );
    // 終わったジョブ・ないジョブは取り消さない
    assert!(!service.cancel(1, 4).unwrap());
    assert!(!service.cancel(99, 4).unwrap());

    let events = service.tick(5);
    assert!(events.contains(&ManagerEvent::Blocked {
        job_id: 2,
        reason: "dependency job 1 was canceled".to_string(),
    }));
}

#[test]
fn retrying_a_failed_job_queues_it_with_the_jobs_it_failed_and_clears_the_old_files() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().exit_codes.insert("cam", 5);
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    service.tick(1);
    service.tick(2);
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Failed);
    assert!(root.store().log_file(1).is_file());

    // 後のジョブから頼んでも、前のジョブも戻す
    assert_eq!(service.retry(2, 3).unwrap(), vec![1, 2]);
    for id in [1, 2] {
        let job = status_of(&root.store(), id);
        assert_eq!(job.status, StoredJobStatus::Queued);
        assert_eq!(job.last_error, None);
        assert_eq!(job.started_at_ms, None);
        assert_eq!(job.updated_at_ms, 3);
    }
    assert!(!root.store().log_file(1).exists());
    assert_eq!(status_of(&root.store(), 1).attempts, 1);

    launcher.0.borrow_mut().exit_codes.clear();
    service.tick(4);
    service.tick(5);
    service.tick(6);
    assert_eq!(status_of(&root.store(), 1).attempts, 2);
    assert_eq!(
        status_of(&root.store(), 2).status,
        StoredJobStatus::Succeeded
    );
    // 正常に終わったジョブ・ないジョブは戻さない
    assert!(service.retry(1, 7).unwrap().is_empty());
    assert!(service.retry(99, 7).unwrap().is_empty());
}

#[test]
fn retrying_a_canceled_job_leaves_other_submissions_alone() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 10;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    submit(
        &root.store(),
        "b2",
        vec![job(StoredJobKind::Cam, "b.bin", vec![], 0)],
    );
    service.tick(1);
    service.cancel(1, 2).unwrap();
    service.cancel(3, 2).unwrap();
    service.tick(3);
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Failed);

    assert_eq!(service.retry(1, 4).unwrap(), vec![1, 2]);
    assert_eq!(
        status_of(&root.store(), 3).status,
        StoredJobStatus::Canceled
    );
}

#[test]
fn retrying_also_queues_the_other_jobs_that_failed_with_the_same_earlier_job() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().exit_codes.insert("cam", 5);
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    let mut jobs = cam_and_sim();
    jobs.push(job(
        StoredJobKind::Sim,
        "sim2.bin",
        vec![DependencyRef::Submitted(0)],
        0,
    ));
    submit(&root.store(), "a1", jobs);
    service.tick(1);
    service.tick(2);
    assert_eq!(status_of(&root.store(), 3).status, StoredJobStatus::Failed);

    assert_eq!(service.retry(2, 3).unwrap(), vec![1, 2, 3]);
}

#[test]
fn lowering_a_waiting_job_also_lowers_the_jobs_waiting_for_it() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 1;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(
        &root.store(),
        "a1",
        vec![job(StoredJobKind::Cam, "a.bin", vec![], 0)],
    );
    submit(&root.store(), "b2", cam_and_sim());
    submit(
        &root.store(),
        "c3",
        vec![job(StoredJobKind::Cam, "c.bin", vec![], 0)],
    );
    service.tick(1);
    assert_eq!(status_of(&root.store(), 1).status, StoredJobStatus::Running);

    assert!(service.change_priority(2, -1, 2).unwrap());
    assert_eq!(status_of(&root.store(), 2).priority, -1);
    assert_eq!(status_of(&root.store(), 3).priority, -1);
    assert_eq!(status_of(&root.store(), 4).priority, 0);

    service.tick(3);
    service.tick(4);
    assert_eq!(status_of(&root.store(), 4).status, StoredJobStatus::Running);
}

#[test]
fn only_finished_jobs_are_deleted() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 10;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    service.tick(1);

    assert!(!service.delete(1).unwrap());
    assert!(!service.delete(2).unwrap());
    service.cancel(1, 2).unwrap();
    assert!(service.delete(1).unwrap());
    assert!(!root.store().job_dir(1).exists());
    assert!(!service.delete(1).unwrap());
    // 消したジョブを待つジョブは、計算できずに失敗になる
    let events = service.tick(3);
    assert!(events.contains(&ManagerEvent::Blocked {
        job_id: 2,
        reason: "dependency job 1 is not found".to_string(),
    }));
}

#[test]
fn raising_the_priority_of_a_waiting_job_runs_it_first() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 1;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    for (id, input) in [("a1", "a.bin"), ("b2", "b.bin"), ("c3", "c.bin")] {
        submit(
            &root.store(),
            id,
            vec![job(StoredJobKind::Cam, input, vec![], 0)],
        );
    }
    service.tick(1);
    assert_eq!(status_of(&root.store(), 1).status, StoredJobStatus::Running);

    assert!(service.change_priority(3, 1, 2).unwrap());
    assert_eq!(status_of(&root.store(), 3).priority, 1);
    assert!(service.change_priority(2, -1, 2).unwrap());
    assert_eq!(status_of(&root.store(), 2).priority, -1);
    // 計算中のジョブ・ないジョブは変えない
    assert!(!service.change_priority(1, 1, 2).unwrap());
    assert!(!service.change_priority(99, 1, 2).unwrap());

    service.tick(3);
    service.tick(4);
    assert_eq!(status_of(&root.store(), 3).status, StoredJobStatus::Running);
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Queued);
}

#[test]
fn a_runner_that_cannot_start_fails_the_job() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().fail_to_launch = true;
    let (mut service, _) = JobManagerService::start(root.store(), launcher, 0).unwrap();
    submit(
        &root.store(),
        "a1",
        vec![job(StoredJobKind::Cam, "a.bin", vec![], 0)],
    );
    service.tick(1);
    let failed = status_of(&root.store(), 1);
    assert_eq!(failed.status, StoredJobStatus::Failed);
    assert!(
        failed
            .last_error
            .unwrap()
            .contains("cannot start the runner")
    );
}

#[test]
fn launch_arguments_name_only_files_under_the_job_folder() {
    let spec = LaunchSpec {
        job_type: "sim",
        input: PathBuf::from("in"),
        toolpath: Some(PathBuf::from("tp")),
        output: PathBuf::from("out"),
        logs: PathBuf::from("logs"),
        progress: PathBuf::from("progress.json"),
    };
    let args: Vec<String> = spec
        .args()
        .into_iter()
        .map(|arg| arg.into_string().unwrap())
        .collect();
    assert_eq!(
        args,
        [
            "--job-type",
            "sim",
            "--input",
            "in",
            "--toolpath",
            "tp",
            "--output",
            "out",
            "--logs",
            "logs",
            "--progress",
            "progress.json"
        ]
    );
}
