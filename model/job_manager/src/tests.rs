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

/// 偽のランナーの共有の状態（起動した引数・止めたジョブ・種類ごとの終了の番号・標準エラー出力）
#[derive(Default)]
struct FakeState {
    launched: Vec<LaunchSpec>,
    stopped: Vec<PathBuf>,
    exit_codes: HashMap<&'static str, i32>,
    stderr: &'static str,
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
            stderr: state.stderr,
            polls_left: state.polls_until_done,
            state: self.0.clone(),
        }))
    }
}

struct FakeJob {
    spec: LaunchSpec,
    code: i32,
    stderr: &'static str,
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
        fs::write(&self.spec.stderr, self.stderr)?;
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
        if let Some(toolpath) = &job.toolpath_file {
            fs::write(temporary.join(toolpath), b"toolpath").unwrap();
        }
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
        toolpath_file: None,
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
fn a_sim_job_with_a_toolpath_file_reads_the_toolpath_in_its_input_folder() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    let sim = SubmittedJob {
        toolpath_file: Some("toolpath.bin".to_string()),
        ..job(StoredJobKind::Sim, "sim.bin", vec![], 0)
    };
    submit(&root.store(), "a1", vec![sim]);

    let events = service.tick(1);
    assert!(events.contains(&ManagerEvent::Started { job_id: 1 }));
    let launched = launcher.0.borrow().launched.clone();
    assert_eq!(launched[0].job_type, "sim");
    assert_eq!(
        launched[0].toolpath,
        Some(root.store().input_toolpath_file(1))
    );
    assert_eq!(
        fs::read(root.store().input_toolpath_file(1)).unwrap(),
        b"toolpath"
    );
    service.tick(2);
    assert_eq!(
        status_of(&root.store(), 1).status,
        StoredJobStatus::Succeeded
    );
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
fn a_runner_that_ends_with_another_exit_code_records_the_last_line_of_its_standard_error() {
    // メモリを確保できずに止まったとき（Windows の 0xC0000409）は、ランナーはログに理由を書けない
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher
        .0
        .borrow_mut()
        .exit_codes
        .insert("cam", -1_073_740_791);
    launcher.0.borrow_mut().stderr = "memory allocation of 6000000000 bytes failed\n";
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());

    service.tick(1);
    service.tick(2);
    let cam = status_of(&root.store(), 1);
    assert_eq!(
        cam.error,
        Some(StoredJobError::RunnerExitCode {
            code: -1_073_740_791
        })
    );
    let error = cam.last_error.unwrap();
    assert!(
        error.ends_with("memory allocation of 6000000000 bytes failed"),
        "{error}"
    );
}

#[test]
fn a_runner_stopped_at_the_memory_limit_records_that_the_toolpath_is_too_large() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().exit_codes.insert("cam", 7);
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());

    service.tick(1);
    service.tick(2);
    let cam = status_of(&root.store(), 1);
    assert_eq!(cam.status, StoredJobStatus::Failed);
    assert_eq!(cam.error, Some(StoredJobError::ToolpathTooLarge));
}

#[test]
fn a_panic_records_its_message_without_the_note_line() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().exit_codes.insert("cam", 101);
    launcher.0.borrow_mut().stderr = "\nthread 'main' panicked at src/run.rs:1:1:\nindex out of bounds\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n";
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());

    service.tick(1);
    service.tick(2);
    let error = status_of(&root.store(), 1).last_error.unwrap();
    assert!(error.ends_with("index out of bounds"), "{error}");
}

#[test]
fn a_runner_that_reports_its_own_failure_records_the_line_of_its_log() {
    // ランナーが決めた終了の番号では、標準エラー出力に何かあっても、ログの理由を出す
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().exit_codes.insert("cam", 5);
    launcher.0.borrow_mut().stderr = "something else\n";
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());

    service.tick(1);
    service.tick(2);
    let error = status_of(&root.store(), 1).last_error.unwrap();
    assert!(error.contains("code=no_solution"), "{error}");
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
fn quitting_stops_the_running_job_and_keeps_the_waiting_jobs() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 10;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    service.tick(1);

    service.stop_running(2).unwrap();
    assert_eq!(
        launcher.0.borrow().stopped,
        vec![root.store().input_file(1)]
    );
    let stopped = status_of(&root.store(), 1);
    assert_eq!(stopped.status, StoredJobStatus::Failed);
    assert_eq!(
        stopped.last_error.as_deref(),
        Some("the job manager was quit")
    );
    assert_eq!(stopped.started_at_ms, None);
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Queued);
}

#[test]
fn only_finished_jobs_are_deleted() {
    let root = TempRoot::new();
    let launcher = FakeLauncher::default();
    launcher.0.borrow_mut().polls_until_done = 10;
    let (mut service, _) = JobManagerService::start(root.store(), launcher.clone(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    service.tick(1);

    let kept = |ids: &[u64]| DeletePlan {
        deleted: BTreeSet::new(),
        kept: ids.iter().copied().collect(),
    };
    // 計算中のジョブも、計算待ちの後のジョブを持つジョブも消さない
    assert_eq!(service.delete(&[1, 2]).unwrap(), kept(&[1, 2]));
    service.cancel(1, 2).unwrap();
    assert_eq!(service.delete(&[1]).unwrap(), kept(&[1]));
    assert!(root.store().job_dir(1).exists());
    // 後のジョブが失敗で終わったら、前のジョブを選ぶと後のジョブもあわせて消す
    service.tick(3);
    assert_eq!(status_of(&root.store(), 2).status, StoredJobStatus::Failed);
    assert_eq!(
        service.delete(&[1, 9]).unwrap(),
        DeletePlan {
            deleted: BTreeSet::from([1, 2]),
            kept: BTreeSet::new(),
        }
    );
    assert!(!root.store().job_dir(1).exists());
    assert!(!root.store().job_dir(2).exists());
    assert_eq!(service.delete(&[1]).unwrap(), DeletePlan::default());
}

#[test]
fn deleting_a_later_job_keeps_the_earlier_job() {
    let root = TempRoot::new();
    let (mut service, _) =
        JobManagerService::start(root.store(), FakeLauncher::default(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    submit(&root.store(), "a2", cam_and_sim());
    for now in 1..20 {
        service.tick(now);
    }
    assert_eq!(
        status_of(&root.store(), 4).status,
        StoredJobStatus::Succeeded
    );
    // 後のジョブだけを選べば、前のジョブを残す。複数を選べば、それぞれの後のジョブもあわせて消す
    assert_eq!(
        service.delete(&[2, 3]).unwrap(),
        DeletePlan {
            deleted: BTreeSet::from([2, 3, 4]),
            kept: BTreeSet::new(),
        }
    );
    assert!(root.store().job_dir(1).exists());
    assert!(!root.store().job_dir(4).exists());
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
        stderr: PathBuf::from("logs/runner.stderr.log"),
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

/// ジョブの列 `jobs` に、投入元（投入元の ID `source_id`・改訂の番号 `revision`・改訂の鍵 `key`）を持たせる
fn with_source(
    mut jobs: Vec<SubmittedJob>,
    source_id: &str,
    revision: u32,
    key: &str,
) -> Vec<SubmittedJob> {
    for job in &mut jobs {
        job.source = Some(job_runtime::store::StoredJobSource {
            source_id: source_id.to_string(),
            revision,
            revision_key: Some(key.to_string()),
        });
    }
    jobs
}

#[test]
fn jobs_of_other_revision_keys_than_the_latest_submission_are_superseded() {
    let root = TempRoot::new();
    let (mut service, _) =
        JobManagerService::start(root.store(), FakeLauncher::default(), 0).unwrap();
    submit(
        &root.store(),
        "a1",
        with_source(cam_and_sim(), "op-1", 1, "key-a"),
    );
    submit(
        &root.store(),
        "a2",
        with_source(cam_and_sim(), "op-1", 2, "key-b"),
    );
    // 複製したモデルファイルで、前の鍵に戻して投入した（改訂の番号は小さい）
    submit(
        &root.store(),
        "a3",
        with_source(cam_and_sim(), "op-1", 1, "key-a"),
    );
    submit(
        &root.store(),
        "a4",
        with_source(cam_and_sim(), "op-2", 1, "key-c"),
    );
    submit(&root.store(), "a5", cam_and_sim());
    for now in 1..40 {
        service.tick(now);
    }
    let (jobs, _) = root.store().load_jobs().unwrap();
    assert_eq!(jobs.len(), 10);
    assert_eq!(superseded_jobs(&jobs), vec![3, 4]);
    // 後から別の鍵で投入しても、正常に終わるまでは前の結果を前の改訂にしない
    submit(
        &root.store(),
        "a6",
        with_source(cam_and_sim(), "op-2", 2, "key-d"),
    );
    service.tick(40);
    let (jobs, _) = root.store().load_jobs().unwrap();
    assert_eq!(superseded_jobs(&jobs), vec![3, 4]);
}

#[test]
fn old_jobs_do_not_take_newer_later_jobs_with_them() {
    let root = TempRoot::new();
    let (mut service, _) =
        JobManagerService::start(root.store(), FakeLauncher::default(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    for now in 1..20 {
        service.tick(now);
    }
    // 古いツールパス生成の下に、後から切削シミュレーションを投入した
    submit(
        &root.store(),
        "a2",
        vec![job(
            StoredJobKind::Sim,
            "sim.bin",
            vec![DependencyRef::Existing(1)],
            0,
        )],
    );
    for now in 100..120 {
        service.tick(now);
    }
    let (jobs, _) = root.store().load_jobs().unwrap();
    assert_eq!(
        status_of(&root.store(), 3).status,
        StoredJobStatus::Succeeded
    );
    assert_eq!(old_jobs(&jobs, 50), vec![2]);
    assert_eq!(old_jobs(&jobs, 200), vec![1, 2, 3]);
    assert_eq!(old_jobs(&jobs, 1), Vec::<u64>::new());
}

#[test]
fn rejected_submissions_are_removed() {
    let root = TempRoot::new();
    let (mut service, _) =
        JobManagerService::start(root.store(), FakeLauncher::default(), 0).unwrap();
    let broken = root.store().root().join("inbox").join("broken");
    fs::create_dir_all(&broken).unwrap();
    fs::write(broken.join("request.json"), b"broken").unwrap();
    service.tick(1);
    let rejected = |store: &JobStore| {
        store
            .inbox_submissions()
            .unwrap()
            .iter()
            .filter(|submission| submission.rejected.is_some())
            .count()
    };
    assert_eq!(rejected(&root.store()), 1);
    let ids = root.store().rejected_submission_ids().unwrap();
    assert_eq!(ids, vec!["broken".to_string()]);
    service.remove_rejected(&ids).unwrap();
    assert_eq!(rejected(&root.store()), 0);
    // もうない投入・正しくない ID
    service.remove_rejected(&ids).unwrap();
    assert!(service.remove_rejected(&["../x".to_string()]).is_err());
}

#[test]
fn imported_sources_are_finished_cam_jobs_with_a_mark() {
    let root = TempRoot::new();
    let (mut service, _) =
        JobManagerService::start(root.store(), FakeLauncher::default(), 0).unwrap();
    submit(&root.store(), "a1", cam_and_sim());
    submit(&root.store(), "a2", cam_and_sim());
    for now in 1..40 {
        service.tick(now);
    }
    let store = root.store();
    let (jobs, _) = store.load_jobs().unwrap();
    let marks = |store: &JobStore| {
        let store = store.clone();
        move |job_id| store.imported_job(job_id)
    };
    assert!(imported_sources(&jobs, marks(&store)).is_empty());
    store.mark_imported(1, 2).unwrap();
    // 切削シミュレーションのジョブの印は、CL の元として数えない
    store.mark_imported(4, 4).unwrap();
    assert_eq!(imported_sources(&jobs, marks(&store)), vec![1]);
    // 取り込んだのと別の切削シミュレーションが同じツールパス生成の下にあれば、数えない
    store.mark_imported(3, 9).unwrap();
    assert_eq!(imported_sources(&jobs, marks(&store)), vec![1]);
}
