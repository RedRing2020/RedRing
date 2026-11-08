use super::*;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use cam_algorithms::SolverGeometry;
use cam_core::{
    CoordinateFrame, LengthUnit, OperationSpec, ScanlineCutDirection, ScanlineParams,
    ScanlineToolRange, Tool,
};
use cam_sim::{
    SnapshotInterval, read_cam_solver_input_v1, read_cutting_simulation_input_v1, run_cam_solver,
    run_cutting_simulation,
};
use geo_algorithms::{
    Aabb3D, Point3D, TessellationLimits, TriangleMesh3D, TriangleMesh3DConstructor,
};
use job_ipc::Endpoint;
use job_manager::{JobManagerService, LaunchSpec, Launcher, RunningJob, start_endpoint_server};
use job_runtime::store::{
    DependencyRef, JobStore, StoredJobKind, StoredJobStatus, SubmissionRequest,
};

use crate::cam_simulation_workflow::CamSimulationWorkflowError;

/// 試験ごとの保存先・口・ジョブマネージャーの実行ファイル（利用者の実際の保存先と口に触れない。試験の終わりに消す）
struct TempRoot {
    path: PathBuf,
    endpoint: Endpoint,
}

impl TempRoot {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let name = format!(
            "job-client-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(format!("redring-{name}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self {
            path,
            endpoint: Endpoint::with_name(&name).unwrap(),
        }
    }

    fn store_root(&self) -> PathBuf {
        self.path.join("store")
    }

    /// 保存先から答える口の受け手を動かし、ジョブマネージャーの実行ファイルがある（中身は使わない）クライアントを返す
    fn client(&self) -> PipeJobClient {
        start_endpoint_server(
            self.endpoint.clone(),
            JobStore::open(self.store_root()).unwrap(),
            std::sync::Arc::new(|| true),
            "0.0.0",
        );
        // 受け手のスレッドが口を開くまで待つ（開く前につなぐと、ジョブマネージャーを起動しようとするため）
        drop(
            self.endpoint
                .connect(std::time::Duration::from_secs(5))
                .unwrap(),
        );
        let manager = self.path.join("redring-job-manager");
        fs::write(&manager, b"").unwrap();
        PipeJobClient::new(Ok(self.endpoint.clone()), manager)
    }

    /// 口の受け手がなく、ジョブマネージャーもインストールされていないクライアント
    fn client_without_manager(&self) -> PipeJobClient {
        PipeJobClient::new(Ok(self.endpoint.clone()), self.path.join("missing"))
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// 四角錐を直径 6 のボールエンドミルで粗く加工する投入
fn submission() -> JobSubmission {
    let pyramid = SolverGeometry::TriangleMesh(
        TriangleMesh3D::new(
            vec![
                Point3D::new(20.0, 20.0, 80.0),
                Point3D::new(60.0, 20.0, 80.0),
                Point3D::new(60.0, 60.0, 80.0),
                Point3D::new(20.0, 60.0, 80.0),
                Point3D::new(40.0, 40.0, 90.0),
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        )
        .unwrap(),
    );
    let tool = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    JobSubmission {
        solver_input: CamSolverInput {
            operation_id: "op-1".to_string(),
            tool: tool.clone(),
            geometry: pyramid.clone(),
            operation: OperationSpec::Scanline(ScanlineParams {
                stepover: 6.0,
                feed_rate: 1200.0,
                tool_range: ScanlineToolRange::default(),
                cut_direction: ScanlineCutDirection::default(),
                angle_degrees: 0.0,
            }),
            units: LengthUnit::Millimeter,
            coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
            chord_tolerance: 0.01,
            tessellation_limits: TessellationLimits::default(),
            boundary: None,
        },
        simulation_input: CuttingSimulationInput {
            stock: Aabb3D::new(
                Point3D::new(15.0, 15.0, 80.0),
                Point3D::new(65.0, 65.0, 95.0),
            ),
            tool,
            precision: 0.5,
            snapshot_interval: SnapshotInterval::default(),
            product: pyramid,
            tolerance: 0.01,
        },
        source: Some(CamJobSource {
            source_id: "operation-1".to_string(),
            revision: 3,
            revision_key: Some("key-3".to_string()),
        }),
    }
}

/// ランナーの代わりに、同じプロセスの中で計算してファイルを書くランチャー
struct InProcessLauncher;

impl Launcher for InProcessLauncher {
    fn launch(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn RunningJob>> {
        let input = fs::read(&spec.input)?;
        fs::create_dir_all(&spec.output)?;
        let (name, artifact) = if spec.job_type == "cam" {
            let input = read_cam_solver_input_v1(&mut input.as_slice()).unwrap();
            ("toolpath.bin", run_cam_solver(&input).unwrap())
        } else {
            let input = read_cutting_simulation_input_v1(&mut input.as_slice()).unwrap();
            let toolpath = fs::read(spec.toolpath.as_ref().unwrap())?;
            let run = run_cutting_simulation(&input, &toolpath).unwrap();
            ("cutting_simulation.bin", run.artifact)
        };
        fs::write(spec.output.join(name), artifact)?;
        Ok(Box::new(Finished))
    }
}

struct Finished;

impl RunningJob for Finished {
    fn try_finish(&mut self) -> std::io::Result<Option<Option<i32>>> {
        Ok(Some(Some(0)))
    }

    fn stop(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn submitting_without_the_job_manager_installed_fails_without_a_submission() {
    let root = TempRoot::new();
    let mut client = root.client_without_manager();
    assert_eq!(
        client.submit(submission()).unwrap_err(),
        CamSimulationWorkflowError::JobManagerNotFound
    );
    let inbox = root.store_root().join("inbox");
    assert!(!inbox.exists() || fs::read_dir(&inbox).unwrap().next().is_none());
}

#[test]
fn opening_the_job_manager_window_without_the_job_manager_installed_fails() {
    let root = TempRoot::new();
    let mut client = root.client_without_manager();
    assert_eq!(
        client.open_manager_window().unwrap_err(),
        CamSimulationWorkflowError::JobManagerNotFound
    );
}

#[test]
fn a_submission_is_placed_in_the_inbox_with_both_inputs() {
    let root = TempRoot::new();
    let mut client = root.client();
    let submission_id = client.submit(submission()).unwrap();
    let dir = root.store_root().join("inbox").join(&submission_id);
    let request: SubmissionRequest =
        serde_json::from_slice(&fs::read(dir.join("request.json")).unwrap()).unwrap();
    assert_eq!(request.jobs.len(), 2);
    assert_eq!(request.jobs[0].kind, StoredJobKind::Cam);
    assert_eq!(request.jobs[1].kind, StoredJobKind::Sim);
    assert_eq!(
        request.jobs[1].depends_on,
        vec![DependencyRef::Submitted(0)]
    );
    assert_eq!(
        request.jobs[1]
            .source
            .as_ref()
            .map(|source| (source.revision, source.revision_key.as_deref())),
        Some((3, Some("key-3")))
    );
    let cam = fs::read(dir.join(&request.jobs[0].input_file)).unwrap();
    assert_eq!(
        read_cam_solver_input_v1(&mut cam.as_slice())
            .unwrap()
            .operation_id,
        "op-1"
    );
    let sim = fs::read(dir.join(&request.jobs[1].input_file)).unwrap();
    assert!(read_cutting_simulation_input_v1(&mut sim.as_slice()).is_ok());
    assert!(client.simulation_jobs().unwrap().is_empty());
    // ジョブマネージャーが取り込むまでは、まだジョブになっていない投入として読める（別のスレッドが読み直しても同じ）
    std::thread::sleep(std::time::Duration::from_millis(200));
    let pending = client.status_snapshot().unwrap().unwrap().pending;
    assert_eq!(pending.len(), 1);
    assert!(!pending[0].rejected);
    assert_eq!(
        pending[0]
            .source
            .as_ref()
            .map(|source| (source.revision, source.revision_key.as_deref())),
        Some((3, Some("key-3")))
    );
}

#[test]
fn jobs_run_by_the_job_manager_are_read_and_imported_from_the_store() {
    let root = TempRoot::new();
    let mut client = root.client();
    let store = JobStore::open(root.store_root()).unwrap();
    let (mut manager, _) = JobManagerService::start(store.clone(), InProcessLauncher, 0).unwrap();
    let submission_id = client.submit(submission()).unwrap();

    for now in 1..=4 {
        manager.tick(now);
    }
    let jobs = client.simulation_jobs().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].submission_id, submission_id);
    assert_eq!(jobs[0].status, CamJobStatus::Succeeded);
    assert_eq!(jobs[0].source.as_ref().unwrap().source_id, "operation-1");

    let imported = client.import(jobs[0].job_id).unwrap();
    assert_eq!(imported.solver_input.unwrap().operation_id, "op-1");
    assert_eq!(imported.simulation_input.precision, 0.5);
    assert_eq!(
        imported.toolpath_artifact,
        run_cam_solver(&submission().solver_input).unwrap()
    );
    assert!(
        cam_sim::read_cutting_simulation_artifact_v1(&mut imported.simulation_artifact.as_slice())
            .is_ok()
    );
    // 取り込んだと伝えると、別のスレッドで伝え、CL の元のツールパス生成のジョブに印が付く
    assert_eq!(store.imported_job(1), None);
    client.record_imported(jobs[0].job_id).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while store.imported_job(1).is_none() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(store.imported_job(1), Some(jobs[0].job_id));
    // ツールパス生成のジョブ（投入の 1 つ目）は、切削シミュレーションとしては取り込めない
    assert_eq!(
        client.import(1).unwrap_err(),
        CamSimulationWorkflowError::NotSucceededSimulation(1)
    );
    // ツールパス生成の成果物（CL）だけを読める。切削シミュレーションのジョブは、ツールパス生成として読まない
    let toolpath = client.toolpath_result(1).unwrap();
    assert_eq!(toolpath.artifact, imported.toolpath_artifact);
    assert_eq!(toolpath.source.unwrap().source_id, "operation-1");
    assert_eq!(
        client.toolpath_result(jobs[0].job_id).unwrap_err(),
        CamSimulationWorkflowError::NotSucceededToolpath(jobs[0].job_id)
    );
    // 成果物のファイルがなければ、成果物がない誤り
    fs::remove_file(store.result_file(1, StoredJobKind::Cam)).unwrap();
    assert_eq!(
        client.toolpath_result(1).unwrap_err(),
        CamSimulationWorkflowError::MissingArtifact(1)
    );
}

#[test]
fn broken_or_tampered_records_do_not_break_reading_or_importing() {
    let root = TempRoot::new();
    let mut client = root.client();
    let store = JobStore::open(root.store_root()).unwrap();
    let (mut manager, _) = JobManagerService::start(store.clone(), InProcessLauncher, 0).unwrap();
    client.submit(submission()).unwrap();
    for now in 1..=4 {
        manager.tick(now);
    }
    drop(manager);

    // 切削シミュレーションの記録の成果物の参照を、ほかのジョブの成果物に書き換えても、自分の成果物を読む
    let lock = store.lock().unwrap();
    let (mut jobs, _) = store.load_jobs().unwrap();
    jobs[1].result_ref = Some("result://cam/1/ok".to_string());
    store.write_job(&lock, &jobs[1]).unwrap();
    let imported = client.import(2).unwrap();
    assert!(
        cam_sim::read_cutting_simulation_artifact_v1(&mut imported.simulation_artifact.as_slice())
            .is_ok()
    );
    // 成果物のファイルがなければ、成果物がない誤り
    fs::remove_file(store.result_file(2, StoredJobKind::Sim)).unwrap();
    assert_eq!(
        client.import(2).unwrap_err(),
        CamSimulationWorkflowError::MissingArtifact(2)
    );

    // 壊れた記録は読み飛ばす
    fs::write(store.job_dir(2).join("job.json"), "{ broken").unwrap();
    assert!(client.simulation_jobs().unwrap().is_empty());
    assert_eq!(
        client.import(2).unwrap_err(),
        CamSimulationWorkflowError::NotSucceededSimulation(2)
    );
    assert_eq!(jobs[0].status, StoredJobStatus::Succeeded);
}

#[test]
fn the_job_manager_window_is_opened_through_the_endpoint() {
    let root = TempRoot::new();
    let mut client = root.client();
    client.open_manager_window().unwrap();
}

#[test]
fn simulations_of_a_toolpath_file_run_without_a_cam_job_and_are_imported() {
    let root = TempRoot::new();
    let mut client = root.client();
    let store = JobStore::open(root.store_root()).unwrap();
    let (mut manager, _) = JobManagerService::start(store.clone(), InProcessLauncher, 0).unwrap();
    let base = submission();
    let toolpath = run_cam_solver(&base.solver_input).unwrap();
    let submission_id = client
        .submit_simulation(SimulationSubmission {
            simulation_input: base.simulation_input,
            toolpath: SimulationToolpath::Artifact(toolpath.clone()),
            source: base.source,
        })
        .unwrap();
    // CL のファイルを持つ投入は、CL のファイルを持てる版の要求で書く
    let request: SubmissionRequest = serde_json::from_slice(
        &fs::read(
            root.store_root()
                .join("inbox")
                .join(&submission_id)
                .join("request.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(request.jobs.len(), 1);
    assert!(request.jobs[0].depends_on.is_empty());
    assert_eq!(
        request.format_version,
        job_runtime::store::TOOLPATH_FILE_FORMAT_VERSION
    );

    for now in 1..=3 {
        manager.tick(now);
    }
    let jobs = client.simulation_jobs().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, CamJobStatus::Succeeded);
    assert_eq!(jobs[0].toolpath_status, CamJobStatus::Succeeded);

    let imported = client.import(jobs[0].job_id).unwrap();
    assert!(imported.solver_input.is_none());
    assert_eq!(imported.toolpath_artifact, toolpath);
    assert!(
        cam_sim::read_cutting_simulation_artifact_v1(&mut imported.simulation_artifact.as_slice())
            .is_ok()
    );
}

#[test]
fn simulations_are_put_under_an_existing_cam_job() {
    let root = TempRoot::new();
    let mut client = root.client();
    let store = JobStore::open(root.store_root()).unwrap();
    let (mut manager, _) = JobManagerService::start(store.clone(), InProcessLauncher, 0).unwrap();
    client.submit(submission()).unwrap();
    for now in 1..=4 {
        manager.tick(now);
    }
    let cam = client.toolpath_jobs().unwrap();
    assert_eq!(cam.len(), 1);
    assert_eq!(cam[0].status, CamJobStatus::Succeeded);
    assert_eq!(cam[0].source.as_ref().unwrap().source_id, "operation-1");
    assert!(cam[0].simulated);
    // 切削シミュレーションを消すと、ツールパス生成のジョブは切削シミュレーションを持たない
    let simulation = client.simulation_jobs().unwrap()[0].job_id;
    assert_eq!(
        manager.delete(&[simulation]).unwrap().deleted,
        std::collections::BTreeSet::from([simulation])
    );
    assert!(!client.toolpath_jobs().unwrap()[0].simulated);

    let base = submission();
    let submission_id = client
        .submit_simulation(SimulationSubmission {
            simulation_input: base.simulation_input,
            toolpath: SimulationToolpath::Job(cam[0].job_id),
            source: base.source,
        })
        .unwrap();
    let request: SubmissionRequest = serde_json::from_slice(
        &fs::read(
            root.store_root()
                .join("inbox")
                .join(&submission_id)
                .join("request.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        request.jobs[0].depends_on,
        vec![DependencyRef::Existing(cam[0].job_id)]
    );
    assert_eq!(request.jobs[0].toolpath_file, None);

    for now in 5..=7 {
        manager.tick(now);
    }
    let jobs = client.simulation_jobs().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, CamJobStatus::Succeeded);
    let imported = client.import(jobs[0].job_id).unwrap();
    assert!(imported.solver_input.is_some());
    assert_eq!(
        imported.toolpath_artifact,
        run_cam_solver(&base.solver_input).unwrap()
    );
}

#[test]
fn the_status_is_read_without_waiting_and_shows_a_submission_at_once() {
    let root = TempRoot::new();
    let mut client = root.client();
    // 投入するとすぐ、取り込みを待つ投入として状態に出る（別のスレッドが読み終えるのを待たない）
    client.submit(submission()).unwrap();
    let snapshot = client.status_snapshot().unwrap().unwrap();
    assert_eq!(snapshot.pending.len(), 1);
    assert_eq!(
        snapshot.pending[0]
            .source
            .as_ref()
            .and_then(|source| source.revision_key.as_deref()),
        Some("key-3")
    );

    // ジョブマネージャーが計算すると、別のスレッドが読み直した状態になる
    let store = JobStore::open(root.store_root()).unwrap();
    let (mut manager, _) = JobManagerService::start(store, InProcessLauncher, 0).unwrap();
    for now in 1..=4 {
        manager.tick(now);
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let snapshot = client.status_snapshot().unwrap().unwrap();
        if snapshot.pending.is_empty()
            && snapshot
                .simulations
                .iter()
                .any(|job| job.status == CamJobStatus::Succeeded)
        {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{snapshot:?}");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
