use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};

/// 試験ごとの保存先（試験の終わりに消す）
struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "redring-job-store-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn job(kind: StoredJobKind, input_file: &str, depends_on: Vec<DependencyRef>) -> SubmittedJob {
    SubmittedJob {
        kind,
        input_file: input_file.to_string(),
        depends_on,
        priority: 0,
        group_id: None,
        source: None,
        toolpath_file: None,
    }
}

/// 投入を書く（`request` は要求の JSON、`inputs` は入力のファイルの名前と中身）。書き終えてから名前を変える
fn submit(store: &JobStore, id: &str, request: &str, inputs: &[(&str, &[u8])]) {
    let inbox = store.root().join(INBOX_DIR);
    let temporary = inbox.join(format!("{id}.tmp"));
    fs::create_dir_all(&temporary).unwrap();
    fs::write(temporary.join(REQUEST_FILE), request).unwrap();
    for (name, bytes) in inputs {
        fs::write(temporary.join(name), bytes).unwrap();
    }
    fs::rename(&temporary, inbox.join(id)).unwrap();
}

fn request_json(jobs: Vec<SubmittedJob>) -> String {
    serde_json::to_string(&SubmissionRequest {
        format_version: STORE_FORMAT_VERSION,
        jobs,
    })
    .unwrap()
}

fn cam_and_sim() -> String {
    request_json(vec![
        job(StoredJobKind::Cam, "cam.bin", vec![]),
        job(
            StoredJobKind::Sim,
            "sim.bin",
            vec![DependencyRef::Submitted(0)],
        ),
    ])
}

fn rejection_reason(outcome: &IntakeOutcome) -> &str {
    match outcome {
        IntakeOutcome::Rejected { reason, .. } => reason,
        _ => panic!("not rejected: {outcome:?}"),
    }
}

#[test]
fn a_second_lock_is_refused_while_the_first_is_held() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    assert!(matches!(store.lock(), Err(StoreError::AlreadyLocked)));
    drop(lock);
    assert!(store.lock().is_ok());
}

#[test]
fn submissions_are_taken_in_with_numbered_jobs_and_copied_inputs() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    submit(
        &store,
        "a1",
        &cam_and_sim(),
        &[("cam.bin", b"cam input"), ("sim.bin", b"sim input")],
    );

    let outcomes = store.intake(&lock, 1_000).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "a1".to_string(),
            job_ids: vec![1, 2],
        }]
    );
    assert!(!store.root().join(INBOX_DIR).join("a1").exists());
    assert_eq!(fs::read(store.input_file(1)).unwrap(), b"cam input");
    assert_eq!(fs::read(store.input_file(2)).unwrap(), b"sim input");

    let (jobs, unreadable) = store.load_jobs().unwrap();
    assert!(unreadable.is_empty());
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[1].kind, StoredJobKind::Sim);
    assert_eq!(jobs[1].depends_on, vec![1]);
    assert_eq!(jobs[1].status, StoredJobStatus::Queued);
    assert_eq!(jobs[1].created_at_ms, 1_000);

    // 次の投入は、続きの番号で、すでにあるジョブに依存できる
    let request = request_json(vec![job(
        StoredJobKind::Sim,
        "sim.bin",
        vec![DependencyRef::Existing(1)],
    )]);
    submit(&store, "b2", &request, &[("sim.bin", b"x")]);
    let outcomes = store.intake(&lock, 2_000).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "b2".to_string(),
            job_ids: vec![3],
        }]
    );
}

#[test]
fn submissions_being_written_are_not_read() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    let writing = store.root().join(INBOX_DIR).join("c3.tmp");
    fs::create_dir_all(&writing).unwrap();
    fs::write(writing.join(REQUEST_FILE), "{").unwrap();

    assert!(store.intake(&lock, 0).unwrap().is_empty());
    assert!(writing.exists());
    assert!(store.load_jobs().unwrap().0.is_empty());
}

/// 投入の ID・要求・入力のファイル・理由に含まれる語
type RejectionCase = (
    &'static str,
    String,
    Vec<(&'static str, &'static [u8])>,
    &'static str,
);

#[test]
fn invalid_submissions_are_rejected_with_the_reason() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    let cases: Vec<RejectionCase> = vec![
        (
            "broken",
            "{ not json".to_string(),
            vec![],
            "invalid request",
        ),
        (
            "version",
            r#"{"format_version":99,"jobs":[]}"#.to_string(),
            vec![],
            "unsupported format version",
        ),
        (
            "traversal",
            request_json(vec![job(StoredJobKind::Cam, "../outside.bin", vec![])]),
            vec![],
            "invalid input file name",
        ),
        (
            "missing",
            request_json(vec![job(StoredJobKind::Cam, "cam.bin", vec![])]),
            vec![],
            "cannot read the input file",
        ),
        (
            "forward",
            request_json(vec![
                job(
                    StoredJobKind::Sim,
                    "sim.bin",
                    vec![DependencyRef::Submitted(1)],
                ),
                job(StoredJobKind::Cam, "cam.bin", vec![]),
            ]),
            vec![("sim.bin", b"s"), ("cam.bin", b"c")],
            "must come earlier",
        ),
        (
            "unknown",
            request_json(vec![job(
                StoredJobKind::Sim,
                "sim.bin",
                vec![DependencyRef::Existing(42)],
            )]),
            vec![("sim.bin", b"s")],
            "does not exist",
        ),
        (
            "nocam",
            request_json(vec![job(StoredJobKind::Sim, "sim.bin", vec![])]),
            vec![("sim.bin", b"s")],
            "exactly one cam job",
        ),
        (
            "camtoolpath",
            request_json(vec![SubmittedJob {
                toolpath_file: Some("tp.bin".to_string()),
                ..job(StoredJobKind::Cam, "cam.bin", vec![])
            }]),
            vec![("cam.bin", b"c"), ("tp.bin", b"t")],
            "only a sim job can have a toolpath file",
        ),
        (
            "bothinputs",
            request_json(vec![
                job(StoredJobKind::Cam, "cam.bin", vec![]),
                SubmittedJob {
                    toolpath_file: Some("tp.bin".to_string()),
                    ..job(
                        StoredJobKind::Sim,
                        "sim.bin",
                        vec![DependencyRef::Submitted(0)],
                    )
                },
            ]),
            vec![("cam.bin", b"c"), ("sim.bin", b"s"), ("tp.bin", b"t")],
            "exactly one cam job or have a toolpath file",
        ),
        (
            "notoolpath",
            request_json(vec![SubmittedJob {
                toolpath_file: Some("tp.bin".to_string()),
                ..job(StoredJobKind::Sim, "sim.bin", vec![])
            }]),
            vec![("sim.bin", b"s")],
            "cannot read the toolpath file",
        ),
        (
            "toolpathname",
            request_json(vec![SubmittedJob {
                toolpath_file: Some("../tp.bin".to_string()),
                ..job(StoredJobKind::Sim, "sim.bin", vec![])
            }]),
            vec![("sim.bin", b"s")],
            "invalid toolpath file name",
        ),
        (
            "oldversion",
            r#"{"format_version":2,"jobs":[{"kind":"sim","input_file":"sim.bin","toolpath_file":"tp.bin"}]}"#
                .to_string(),
            vec![("sim.bin", b"s"), ("tp.bin", b"t")],
            "needs format version 3",
        ),
        (
            "sharedtoolpath",
            request_json(vec![
                job(StoredJobKind::Cam, "cam.bin", vec![]),
                SubmittedJob {
                    toolpath_file: Some("cam.bin".to_string()),
                    ..job(StoredJobKind::Sim, "sim.bin", vec![])
                },
            ]),
            vec![("cam.bin", b"c"), ("sim.bin", b"s")],
            "toolpath file is shared with another input",
        ),
        (
            "longkey",
            request_json(vec![SubmittedJob {
                source: Some(StoredJobSource {
                    source_id: "op-1".to_string(),
                    revision: 1,
                    revision_key: Some("k".repeat(MAX_LABEL_LEN + 1)),
                }),
                ..job(StoredJobKind::Cam, "cam.bin", vec![])
            }]),
            vec![("cam.bin", b"c")],
            "labels must be at most",
        ),
    ];
    for (id, request, inputs, _) in &cases {
        submit(&store, id, request, inputs);
    }

    let outcomes = store.intake(&lock, 0).unwrap();
    assert_eq!(outcomes.len(), cases.len());
    for outcome in &outcomes {
        let IntakeOutcome::Rejected { submission_id, .. } = outcome else {
            panic!("accepted: {outcome:?}");
        };
        let (_, _, _, expected) = cases.iter().find(|(id, ..)| id == submission_id).unwrap();
        assert!(
            rejection_reason(outcome).contains(expected),
            "{submission_id}: {outcome:?}"
        );
        let rejected = store
            .root()
            .join(INBOX_DIR)
            .join(format!("{submission_id}{REJECTED_SUFFIX}"));
        assert!(rejected.join(REJECTED_REASON_FILE).exists());
    }
    assert!(store.load_jobs().unwrap().0.is_empty());
    // 取り込めなかった投入は、次の取り込みで読まない
    assert!(store.intake(&lock, 0).unwrap().is_empty());
}

#[test]
fn oversized_requests_and_inputs_are_rejected() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    let padding = " ".repeat(MAX_JSON_BYTES as usize + 1);
    submit(&store, "big", &format!("{}{padding}", cam_and_sim()), &[]);
    let too_many = request_json(
        (0..=MAX_JOBS_PER_SUBMISSION)
            .map(|index| job(StoredJobKind::Cam, &format!("c{index}.bin"), vec![]))
            .collect(),
    );
    submit(&store, "many", &too_many, &[]);

    let outcomes = store.intake(&lock, 0).unwrap();
    assert!(rejection_reason(&outcomes[0]).contains("larger than"));
    assert!(rejection_reason(&outcomes[1]).contains("1 to 64 jobs"));
}

#[test]
fn restarting_keeps_the_jobs_cleans_temporary_files_and_continues_the_numbers() {
    let root = TempRoot::new();
    {
        let store = JobStore::open(&root.0).unwrap();
        let lock = store.lock().unwrap();
        submit(
            &store,
            "a1",
            &cam_and_sim(),
            &[("cam.bin", b"c"), ("sim.bin", b"s")],
        );
        store.intake(&lock, 0).unwrap();
        let (mut jobs, _) = store.load_jobs().unwrap();
        jobs[0].status = StoredJobStatus::Succeeded;
        store.write_job(&lock, &jobs[0]).unwrap();
        // 落ちたときに残る書きかけのもの
        fs::create_dir_all(store.root().join(JOBS_DIR).join("3.tmp")).unwrap();
        fs::create_dir_all(store.root().join(INBOX_DIR).join("d4.tmp")).unwrap();
        fs::write(store.job_dir(1).join("job.json.tmp"), "{").unwrap();
        fs::create_dir_all(store.output_dir(1)).unwrap();
        fs::write(store.output_dir(1).join("toolpath.bin.tmp"), "x").unwrap();
        fs::create_dir_all(store.logs_dir(1)).unwrap();
        fs::write(store.logs_dir(1).join("runner.log.tmp"), "partial").unwrap();
    }

    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    store.remove_temporary_files(&lock).unwrap();
    assert!(!store.root().join(JOBS_DIR).join("3.tmp").exists());
    // 受付のフォルダの書きかけは、アプリが書いている途中のことがあるため、新しいものは残す
    assert!(store.root().join(INBOX_DIR).join("d4.tmp").exists());
    assert!(!store.job_dir(1).join("job.json.tmp").exists());
    assert!(!store.output_dir(1).join("toolpath.bin.tmp").exists());
    assert!(store.logs_dir(1).join("runner.log.tmp").exists());

    let (jobs, _) = store.load_jobs().unwrap();
    assert_eq!(jobs[0].status, StoredJobStatus::Succeeded);
    let request = request_json(vec![job(StoredJobKind::Cam, "cam.bin", vec![])]);
    submit(&store, "e5", &request, &[("cam.bin", b"c")]);
    let outcomes = store.intake(&lock, 0).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "e5".to_string(),
            job_ids: vec![3],
        }]
    );
}

#[test]
fn deleted_jobs_leave_no_files_and_their_numbers_are_not_used_again() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    submit(
        &store,
        "a1",
        &cam_and_sim(),
        &[("cam.bin", b"c"), ("sim.bin", b"s")],
    );
    store.intake(&lock, 0).unwrap();

    store.delete_job(&lock, 2).unwrap();
    assert!(!store.job_dir(2).exists());
    assert!(!store.root().join(JOBS_DIR).join("2.tmp").exists());
    let (jobs, unreadable) = store.load_jobs().unwrap();
    assert_eq!(
        jobs.iter().map(|job| job.job_id).collect::<Vec<_>>(),
        vec![1]
    );
    assert!(unreadable.is_empty());

    let request = request_json(vec![job(StoredJobKind::Cam, "cam.bin", vec![])]);
    submit(&store, "b2", &request, &[("cam.bin", b"c")]);
    let outcomes = store.intake(&lock, 0).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "b2".to_string(),
            job_ids: vec![3],
        }]
    );
}

#[test]
fn an_unreadable_last_job_number_postpones_taking_in_new_jobs() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    fs::write(store.root().join("last_job_id"), "not a number").unwrap();
    let request = request_json(vec![job(StoredJobKind::Cam, "cam.bin", vec![])]);
    submit(&store, "a1", &request, &[("cam.bin", b"c")]);

    assert!(store.intake(&lock, 0).is_err());
    assert!(store.load_jobs().unwrap().0.is_empty());
    fs::write(store.root().join("last_job_id"), "7").unwrap();
    let outcomes = store.intake(&lock, 0).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "a1".to_string(),
            job_ids: vec![8],
        }]
    );
}

#[test]
fn removing_job_outputs_keeps_the_record_and_the_input() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    let request = request_json(vec![job(StoredJobKind::Cam, "cam.bin", vec![])]);
    submit(&store, "a1", &request, &[("cam.bin", b"c")]);
    store.intake(&lock, 0).unwrap();
    fs::create_dir_all(store.output_dir(1)).unwrap();
    fs::write(store.result_file(1, StoredJobKind::Cam), "x").unwrap();
    fs::create_dir_all(store.logs_dir(1)).unwrap();
    fs::write(store.log_file(1), "log").unwrap();
    fs::write(store.progress_file(1), "{}").unwrap();

    store.remove_job_outputs(&lock, 1).unwrap();
    assert!(!store.output_dir(1).exists());
    assert!(!store.logs_dir(1).exists());
    assert!(!store.progress_file(1).exists());
    assert!(store.input_file(1).is_file());
    assert_eq!(store.load_jobs().unwrap().0.len(), 1);
    // 消すものがなくても誤りにしない
    store.remove_job_outputs(&lock, 1).unwrap();
}

#[test]
fn a_submission_taken_in_before_a_crash_is_not_taken_in_twice() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    submit(
        &store,
        "a1",
        &cam_and_sim(),
        &[("cam.bin", b"c"), ("sim.bin", b"s")],
    );
    // 投入のフォルダを消す前に止まった状態を作る
    let kept = store.root().join("kept");
    copy_dir(&store.root().join(INBOX_DIR).join("a1"), &kept);
    store.intake(&lock, 0).unwrap();
    copy_dir(&kept, &store.root().join(INBOX_DIR).join("a1"));

    let outcomes = store.intake(&lock, 0).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "a1".to_string(),
            job_ids: vec![1, 2],
        }]
    );
    assert_eq!(store.load_jobs().unwrap().0.len(), 2);
    assert!(!store.root().join(INBOX_DIR).join("a1").exists());
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
    }
}

#[test]
fn broken_job_records_are_reported_without_failing() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    for (id, text) in [("1", "{"), ("2", "[]")] {
        fs::create_dir_all(store.root().join(JOBS_DIR).join(id)).unwrap();
        fs::write(store.root().join(JOBS_DIR).join(id).join(JOB_FILE), text).unwrap();
    }
    // 番号の名前でないフォルダは読まない
    fs::create_dir_all(store.root().join(JOBS_DIR).join("..x")).unwrap();
    fs::create_dir_all(store.root().join(JOBS_DIR).join("007")).unwrap();

    let (jobs, unreadable) = store.load_jobs().unwrap();
    assert!(jobs.is_empty());
    assert_eq!(unreadable.len(), 2);
}

#[test]
fn references_map_to_files_under_the_job_folder() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    assert_eq!(
        store.file_for_ref("result://cam/7/ok").unwrap(),
        store.job_dir(7).join(OUTPUT_DIR).join(TOOLPATH_FILE)
    );
    assert_eq!(
        store.file_for_ref("result://sim/8/ok").unwrap(),
        store
            .job_dir(8)
            .join(OUTPUT_DIR)
            .join(CUTTING_SIMULATION_FILE)
    );
    assert_eq!(
        store.file_for_ref("log://sim/8/ok/snapshots-3").unwrap(),
        store.job_dir(8).join(LOGS_DIR).join(LOG_FILE)
    );
    for value in [
        "result://cam/../ok",
        "result://cam/7/../../etc",
        "result://cam/C:/ok",
        "result://cam/007/ok",
        "result://nc-post/7/ok",
        "result://cam/7",
        "input://cam/7/ok",
        "result://cam/0/ok",
    ] {
        match store.file_for_ref(value) {
            Err(StoreError::InvalidRef(_)) => {}
            other => {
                // `..` を含む接尾辞は、パスに使わないためジョブのフォルダの外を指さない
                let path = other.unwrap();
                assert!(path.starts_with(store.job_dir(7)), "{value}: {path:?}");
            }
        }
    }
}

#[test]
fn a_submission_partly_taken_in_before_a_crash_is_taken_in_again() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    submit(
        &store,
        "a1",
        &cam_and_sim(),
        &[("cam.bin", b"c"), ("sim.bin", b"s")],
    );
    // 1 つ目のジョブだけを作って止まった状態を作る
    let kept = store.root().join("kept");
    copy_dir(&store.root().join(INBOX_DIR).join("a1"), &kept);
    store.intake(&lock, 0).unwrap();
    fs::remove_dir_all(store.job_dir(2)).unwrap();
    copy_dir(&kept, &store.root().join(INBOX_DIR).join("a1"));

    let outcomes = store.intake(&lock, 0).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "a1".to_string(),
            job_ids: vec![2, 3],
        }]
    );
    let (jobs, _) = store.load_jobs().unwrap();
    assert_eq!(
        jobs.iter().map(|job| job.job_id).collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(jobs[1].depends_on, vec![2]);
}

fn scheduled_job(
    job_id: u64,
    depends_on: &[u64],
    priority: i32,
    status: StoredJobStatus,
) -> StoredJob {
    StoredJob {
        format_version: STORE_FORMAT_VERSION,
        job_id,
        submission_id: "s".to_string(),
        kind: StoredJobKind::Cam,
        depends_on: depends_on.to_vec(),
        priority,
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

/// 実行したジョブを正常に終わったことにしながら、実行の順を求める
fn run_order(mut jobs: Vec<StoredJob>) -> Vec<u64> {
    let mut order = Vec::new();
    while let Some(id) = plan_schedule(&jobs, &[]).next {
        order.push(id);
        jobs.iter_mut().find(|job| job.job_id == id).unwrap().status = StoredJobStatus::Succeeded;
    }
    order
}

#[test]
fn higher_priority_runs_first_and_ties_run_in_submission_order() {
    use StoredJobStatus::Queued;
    let jobs = vec![
        scheduled_job(1, &[], 0, Queued),
        scheduled_job(2, &[], 5, Queued),
        scheduled_job(3, &[], 0, Queued),
        scheduled_job(4, &[], 5, Queued),
    ];
    assert_eq!(run_order(jobs), vec![2, 4, 1, 3]);
}

#[test]
fn jobs_wait_for_all_dependencies_and_raise_the_jobs_they_wait_for() {
    use StoredJobStatus::Queued;
    // 3 は優先順位が高いが、1・2 を待つ。1・2 は 3 の高さで先に実行する
    let jobs = vec![
        scheduled_job(1, &[], 0, Queued),
        scheduled_job(2, &[], 0, Queued),
        scheduled_job(3, &[1, 2], 9, Queued),
        scheduled_job(4, &[], 1, Queued),
    ];
    assert_eq!(run_order(jobs), vec![1, 2, 3, 4]);
    // 4 を待つ 2、2 を待つ 1 も 5 の高さになる
    let jobs = vec![
        scheduled_job(1, &[], 0, Queued),
        scheduled_job(2, &[1], 0, Queued),
        scheduled_job(3, &[], 3, Queued),
        scheduled_job(4, &[2], 5, Queued),
    ];
    assert_eq!(run_order(jobs), vec![1, 2, 4, 3]);
}

#[test]
fn running_dependencies_block_the_dependents() {
    use StoredJobStatus::{Queued, Running};
    let jobs = vec![
        scheduled_job(1, &[], 0, Running),
        scheduled_job(2, &[1], 0, Queued),
    ];
    assert_eq!(plan_schedule(&jobs, &[]).next, None);
}

#[test]
fn jobs_that_cannot_run_are_blocked_in_cascade_and_raise_nothing() {
    use StoredJobStatus::{Canceled, Failed, Queued};
    let jobs = vec![
        scheduled_job(1, &[], 0, Failed),
        scheduled_job(2, &[1], 0, Queued),
        scheduled_job(3, &[2], 0, Queued),
        scheduled_job(4, &[], 0, Canceled),
        scheduled_job(5, &[4], 0, Queued),
        scheduled_job(7, &[6], 0, Queued),
        scheduled_job(8, &[9], 0, Queued),
        scheduled_job(10, &[], 0, Queued),
        scheduled_job(11, &[], 5, Queued),
        // 1 が失敗したため実行できないが、10 を待つ。10 の優先順位を上げない
        scheduled_job(12, &[1, 10], 9, Queued),
    ];
    let plan = plan_schedule(&jobs, &[]);
    assert_eq!(
        plan.blocked,
        vec![
            (2, "dependency job 1 failed".to_string()),
            (3, "dependency job 2 cannot run".to_string()),
            (5, "dependency job 4 was canceled".to_string()),
            (7, "dependency job 6 is not found".to_string()),
            (8, "dependency job 9 is not an earlier job".to_string()),
            (12, "dependency job 1 failed".to_string()),
        ]
    );
    assert_eq!(plan.next, Some(11));
}

#[test]
fn jobs_waiting_for_unreadable_records_wait() {
    use StoredJobStatus::Queued;
    let jobs = vec![
        scheduled_job(2, &[1], 0, Queued),
        scheduled_job(3, &[], 0, Queued),
    ];
    let plan = plan_schedule(&jobs, &[1]);
    assert!(plan.blocked.is_empty());
    assert_eq!(plan.next, Some(3));
}

#[test]
fn written_submissions_are_taken_in() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let request = SubmissionRequest {
        format_version: STORE_FORMAT_VERSION,
        jobs: vec![
            job(StoredJobKind::Cam, "cam.bin", vec![]),
            job(
                StoredJobKind::Sim,
                "sim.bin",
                vec![DependencyRef::Submitted(0)],
            ),
        ],
    };
    assert!(!store.is_locked().unwrap());
    store
        .write_submission("a1", &request, &[("cam.bin", b"c"), ("sim.bin", b"s")])
        .unwrap();
    for bad in ["../x", "a.b", ""] {
        assert!(store.write_submission(bad, &request, &[]).is_err());
    }
    assert!(
        store
            .write_submission("b2", &request, &[("../cam.bin", b"c")])
            .is_err()
    );

    let lock = store.lock().unwrap();
    assert!(store.is_locked().unwrap());
    let outcomes = store.intake(&lock, 0).unwrap();
    assert_eq!(
        outcomes,
        vec![IntakeOutcome::Accepted {
            submission_id: "a1".to_string(),
            job_ids: vec![1, 2],
        }]
    );
}

#[test]
fn pending_and_rejected_submissions_are_listed() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    submit(&store, "bad", "{ broken", &[]);
    store.intake(&lock, 0).unwrap();
    submit(
        &store,
        "waiting",
        &cam_and_sim(),
        &[("cam.bin", b"c"), ("sim.bin", b"s")],
    );
    fs::create_dir_all(store.root().join(INBOX_DIR).join("writing.tmp")).unwrap();

    let submissions = store.inbox_submissions().unwrap();
    assert_eq!(submissions.len(), 2);
    assert_eq!(submissions[0].submission_id, "bad");
    assert!(submissions[0].request.is_none());
    assert!(
        submissions[0]
            .rejected
            .as_deref()
            .unwrap()
            .contains("invalid request")
    );
    assert_eq!(submissions[1].submission_id, "waiting");
    assert_eq!(submissions[1].request.as_ref().unwrap().jobs.len(), 2);
    assert!(submissions[1].rejected.is_none());
}

#[test]
fn requests_and_records_of_the_previous_format_version_are_read() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    // 版 1 の要求（投入元に改訂の鍵がない）
    let request = r#"{"format_version":1,"jobs":[{"kind":"cam","input_file":"cam.bin","source":{"source_id":"op-1","revision":2}}]}"#;
    submit(&store, "old", request, &[("cam.bin", b"c")]);
    let outcomes = store.intake(&lock, 1_000).unwrap();
    assert!(
        matches!(&outcomes[..], [IntakeOutcome::Accepted { .. }]),
        "{outcomes:?}"
    );

    // 版 1 の記録
    let (mut jobs, _) = store.load_jobs().unwrap();
    let mut old = jobs.remove(0);
    old.format_version = 1;
    let path = store.root().join(JOBS_DIR).join("1").join(JOB_FILE);
    fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
    let (jobs, unreadable) = store.load_jobs().unwrap();
    assert!(unreadable.is_empty());
    assert_eq!(jobs[0].format_version, 1);
    assert_eq!(jobs[0].source.as_ref().unwrap().revision_key, None);
}

#[test]
fn sources_without_a_revision_key_are_read_and_written_as_before() {
    // 前の版の記録・要求の投入元は、改訂の鍵を持たない
    let old: StoredJobSource =
        serde_json::from_str(r#"{"source_id":"op-1","revision":2}"#).unwrap();
    assert_eq!(old.revision_key, None);
    assert_eq!(
        serde_json::to_string(&old).unwrap(),
        r#"{"source_id":"op-1","revision":2}"#
    );

    let keyed = StoredJobSource {
        revision_key: Some("key-2".to_string()),
        ..old
    };
    let written = serde_json::to_string(&keyed).unwrap();
    assert_eq!(
        serde_json::from_str::<StoredJobSource>(&written).unwrap(),
        keyed
    );
}

#[test]
fn sim_jobs_with_a_toolpath_file_are_taken_in_without_a_cam_job() {
    let root = TempRoot::new();
    let store = JobStore::open(&root.0).unwrap();
    let lock = store.lock().unwrap();
    let request = request_json(vec![SubmittedJob {
        toolpath_file: Some("tp.bin".to_string()),
        ..job(StoredJobKind::Sim, "sim.bin", vec![])
    }]);
    submit(
        &store,
        "a1",
        &request,
        &[("sim.bin", b"sim"), ("tp.bin", b"toolpath")],
    );

    let outcomes = store.intake(&lock, 1_000).unwrap();
    assert!(
        matches!(&outcomes[..], [IntakeOutcome::Accepted { job_ids, .. }] if job_ids == &[1]),
        "{outcomes:?}"
    );
    assert_eq!(fs::read(store.input_file(1)).unwrap(), b"sim");
    assert_eq!(fs::read(store.input_toolpath_file(1)).unwrap(), b"toolpath");
    let (jobs, _) = store.load_jobs().unwrap();
    assert!(jobs[0].depends_on.is_empty());
}

#[test]
fn requests_use_the_oldest_format_version_that_can_express_them() {
    let plain = vec![job(StoredJobKind::Cam, "cam.bin", vec![])];
    assert_eq!(request_format_version(&plain), 2);
    let with_toolpath = vec![SubmittedJob {
        toolpath_file: Some("tp.bin".to_string()),
        ..job(StoredJobKind::Sim, "sim.bin", vec![])
    }];
    assert_eq!(
        request_format_version(&with_toolpath),
        TOOLPATH_FILE_FORMAT_VERSION
    );
}
