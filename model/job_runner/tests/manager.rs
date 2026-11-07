//! ジョブマネージャーのサービスから、ランナーの実行ファイルを起動して、投入したジョブを最後まで実行する

mod common;

use std::fs;
use std::time::{Duration, Instant};

use cam_sim::{read_cutting_simulation_artifact_v1, run_cam_solver};
use common::{WorkDir, cam_input, write_cam_input, write_sim_input};
use job_manager::{JobManagerService, ProcessLauncher};
use job_runtime::store::{
    DependencyRef, JobStore, STORE_FORMAT_VERSION, StoredJobKind, StoredJobStatus,
    SubmissionRequest, SubmittedJob,
};

fn submitted(
    kind: StoredJobKind,
    input_file: &str,
    depends_on: Vec<DependencyRef>,
) -> SubmittedJob {
    SubmittedJob {
        kind,
        input_file: input_file.to_string(),
        depends_on,
        priority: 0,
        group_id: None,
        source: None,
    }
}

#[test]
fn submitted_cam_and_sim_jobs_run_with_the_real_runner() {
    let dir = WorkDir::new();
    let store = JobStore::open(dir.path("store")).unwrap();
    let launcher = ProcessLauncher::new(env!("CARGO_BIN_EXE_redring-batch-runner").into());
    let (mut service, _) = JobManagerService::start(store.clone(), launcher, 0).unwrap();

    // 投入（書き終えてから名前を変える）
    let writing = store.root().join("inbox").join("s1.tmp");
    fs::create_dir_all(&writing).unwrap();
    write_cam_input(&writing.join("cam.bin"), &cam_input());
    write_sim_input(&writing.join("sim.bin"));
    let request = SubmissionRequest {
        format_version: STORE_FORMAT_VERSION,
        jobs: vec![
            submitted(StoredJobKind::Cam, "cam.bin", vec![]),
            submitted(
                StoredJobKind::Sim,
                "sim.bin",
                vec![DependencyRef::Submitted(0)],
            ),
        ],
    };
    fs::write(
        writing.join("request.json"),
        serde_json::to_vec(&request).unwrap(),
    )
    .unwrap();
    fs::rename(&writing, store.root().join("inbox").join("s1")).unwrap();

    let deadline = Instant::now() + Duration::from_secs(120);
    let jobs = loop {
        service.tick(0);
        let (jobs, _) = store.load_jobs().unwrap();
        if jobs.len() == 2 && jobs.iter().all(|job| job.status.is_terminal()) {
            break jobs;
        }
        assert!(Instant::now() < deadline, "jobs did not finish: {jobs:?}");
        std::thread::sleep(Duration::from_millis(20));
    };
    for job in &jobs {
        assert_eq!(job.status, StoredJobStatus::Succeeded, "{job:?}");
    }
    let toolpath = fs::read(
        store
            .file_for_ref(jobs[0].result_ref.as_deref().unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(toolpath, run_cam_solver(&cam_input()).unwrap());
    let simulation = fs::read(store.result_file(2, StoredJobKind::Sim)).unwrap();
    assert!(read_cutting_simulation_artifact_v1(&mut simulation.as_slice()).is_ok());
    assert!(store.log_file(2).is_file());
    assert!(
        fs::read_to_string(store.progress_file(2))
            .unwrap()
            .contains("succeeded")
    );
}
