//! ランナーの実行ファイルを起動し、終了の番号と、書いたファイルを確かめる

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use cam_algorithms::{CamSolverInput, SolverGeometry};
use cam_core::{
    CoordinateFrame, LengthUnit, OperationSpec, ScanlineCutDirection, ScanlineParams,
    ScanlineToolRange, Tool, read_toolpath_artifact_v1,
};
use cam_sim::{
    CuttingSimulationInput, SnapshotInterval, read_cutting_simulation_artifact_v1, run_cam_solver,
    write_cam_solver_input_v1, write_cutting_simulation_input_v1,
};
use geo_algorithms::{
    Aabb3D, Point3D, TessellationLimits, TriangleMesh3D, TriangleMesh3DConstructor,
};

/// 試験ごとの作業のフォルダ（試験の終わりに消す）
struct WorkDir(PathBuf);

impl WorkDir {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "redring-runner-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 素材（0..60 × 0..60 × 0..10）の中の四角錐（底面 z=0、頂点 z=10）
fn pyramid() -> SolverGeometry {
    SolverGeometry::TriangleMesh(
        TriangleMesh3D::new(
            vec![
                Point3D::new(10.0, 10.0, 0.0),
                Point3D::new(50.0, 10.0, 0.0),
                Point3D::new(50.0, 50.0, 0.0),
                Point3D::new(10.0, 50.0, 0.0),
                Point3D::new(30.0, 30.0, 10.0),
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        )
        .unwrap(),
    )
}

fn tool() -> Tool<f64> {
    Tool::ball_end_mill("BEM10".to_string(), 10.0, 40.0)
}

fn cam_input() -> CamSolverInput {
    CamSolverInput {
        operation_id: "op-1".to_string(),
        tool: tool(),
        geometry: pyramid(),
        operation: OperationSpec::Scanline(ScanlineParams {
            stepover: 5.0,
            feed_rate: 1200.0,
            tool_range: ScanlineToolRange::GeometryExtent,
            cut_direction: ScanlineCutDirection::DownCut,
            angle_degrees: 0.0,
        }),
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: 0.01,
        tessellation_limits: TessellationLimits::default(),
        boundary: None,
    }
}

fn sim_input() -> CuttingSimulationInput {
    CuttingSimulationInput {
        stock: Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(60.0, 60.0, 10.0)),
        tool: tool(),
        precision: 0.5,
        snapshot_interval: SnapshotInterval::ByToolAndGrid,
        product: pyramid(),
        tolerance: 0.01,
    }
}

fn write_cam_input(path: &Path, input: &CamSolverInput) {
    let mut bytes = Vec::new();
    write_cam_solver_input_v1(&mut bytes, input).unwrap();
    fs::write(path, bytes).unwrap();
}

fn write_sim_input(path: &Path) {
    let mut bytes = Vec::new();
    write_cutting_simulation_input_v1(&mut bytes, &sim_input()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// ランナーを起動し、終了の番号を返す
fn run(args: &[&std::ffi::OsStr]) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_redring-batch-runner"))
        .args(args)
        .output()
        .unwrap()
        .status
        .code()
        .unwrap()
}

fn run_job(dir: &WorkDir, job_type: &str, input: &Path, toolpath: Option<&Path>) -> i32 {
    let (output, logs, progress) = (
        dir.path("output"),
        dir.path("logs"),
        dir.path("progress.json"),
    );
    let mut args = vec![
        "--job-type".as_ref(),
        job_type.as_ref(),
        "--input".as_ref(),
        input.as_os_str(),
        "--output".as_ref(),
        output.as_os_str(),
        "--logs".as_ref(),
        logs.as_os_str(),
        "--progress".as_ref(),
        progress.as_os_str(),
    ];
    if let Some(toolpath) = toolpath {
        args.extend(["--toolpath".as_ref(), toolpath.as_os_str()]);
    }
    run(&args)
}

fn log_text(dir: &WorkDir) -> String {
    fs::read_to_string(dir.path("logs").join("runner.log")).unwrap()
}

fn progress_text(dir: &WorkDir) -> String {
    fs::read_to_string(dir.path("progress.json")).unwrap()
}

/// フォルダに一時の名前のファイルが残っていない
fn assert_no_temporary_files(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(
            !name.to_string_lossy().ends_with(".tmp"),
            "{name:?} is left"
        );
    }
}

#[test]
fn cam_job_writes_the_same_toolpath_as_the_in_process_calculation() {
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    write_cam_input(&input, &cam_input());

    assert_eq!(run_job(&dir, "cam", &input, None), 0, "{}", log_text(&dir));

    let artifact = fs::read(dir.path("output").join("toolpath.bin")).unwrap();
    assert_eq!(artifact, run_cam_solver(&cam_input()).unwrap());
    let (_, toolpath) = read_toolpath_artifact_v1(&mut artifact.as_slice()).unwrap();
    assert!(toolpath.level_count() > 0);
    assert!(log_text(&dir).contains("succeeded"));
    assert!(progress_text(&dir).contains("\"succeeded\""));
    assert_no_temporary_files(&dir.path("output"));
    assert_no_temporary_files(&dir.path("logs"));
    assert_no_temporary_files(&dir.0);
}

#[test]
fn sim_job_simulates_the_toolpath_of_the_parent_job() {
    let dir = WorkDir::new();
    let toolpath = dir.path("toolpath.bin");
    fs::write(&toolpath, run_cam_solver(&cam_input()).unwrap()).unwrap();
    let input = dir.path("sim_input.bin");
    write_sim_input(&input);

    assert_eq!(
        run_job(&dir, "sim", &input, Some(&toolpath)),
        0,
        "{}",
        log_text(&dir)
    );

    let artifact = fs::read(dir.path("output").join("cutting_simulation.bin")).unwrap();
    let (_, simulation) = read_cutting_simulation_artifact_v1(&mut artifact.as_slice()).unwrap();
    assert!(simulation.product_deviations.is_some());
    assert!(log_text(&dir).contains("snapshots="));
}

#[test]
fn failures_are_returned_as_exit_codes_with_the_reason_in_the_log() {
    // 引数の誤り
    assert_eq!(run(&["--job-type".as_ref(), "cam".as_ref()]), 2);

    // 入力のファイルがない
    let dir = WorkDir::new();
    assert_eq!(run_job(&dir, "cam", &dir.path("missing.bin"), None), 3);
    assert!(log_text(&dir).contains("cannot read the input"));
    assert!(progress_text(&dir).contains("\"failed\""));

    // 入力のファイルの形式の誤り（ほかの種類の入力を含む）
    let dir = WorkDir::new();
    let garbage = dir.path("garbage.bin");
    fs::write(&garbage, b"not an input").unwrap();
    assert_eq!(run_job(&dir, "cam", &garbage, None), 4);
    let sim = dir.path("sim_input.bin");
    write_sim_input(&sim);
    assert_eq!(run_job(&dir, "cam", &sim, None), 4);
    assert!(log_text(&dir).contains("invalid input"));

    // 条件の値の誤り（工具の半径が 0）。前の実行の成果物は消す
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    write_cam_input(&input, &cam_input());
    assert_eq!(run_job(&dir, "cam", &input, None), 0);
    let mut invalid = cam_input();
    invalid.tool = Tool::ball_end_mill("BEM0".to_string(), 0.0, 40.0);
    write_cam_input(&input, &invalid);
    assert_eq!(run_job(&dir, "cam", &input, None), 4);
    assert!(log_text(&dir).contains("code=invalid_input"));
    assert!(!dir.path("output").join("toolpath.bin").exists());

    // 親のツールパスの成果物が壊れている
    let dir = WorkDir::new();
    let input = dir.path("sim_input.bin");
    write_sim_input(&input);
    let broken = dir.path("toolpath.bin");
    fs::write(&broken, [0_u8, 1, 2, 3]).unwrap();
    assert_eq!(run_job(&dir, "sim", &input, Some(&broken)), 4);
    assert!(log_text(&dir).contains("code=artifact-read-failed"));
}
