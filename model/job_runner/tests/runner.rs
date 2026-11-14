//! ランナーの実行ファイルを起動し、終了の番号と、書いたファイルを確かめる

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use cam_core::{Tool, read_toolpath_artifact_v1};
use cam_sim::{
    Checkpoints, SavedRecord, read_cutting_simulation_artifact_v1, run_cam_solver,
    run_cam_solver_resumable,
};
use common::{WorkDir, cam_input, write_cam_input, write_sim_input};

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

/// 途中の結果のファイル `resume` を渡して、ツールパス生成のランナーを起動する
fn run_cam_job_with_resume(dir: &WorkDir, input: &Path, resume: &Path) -> i32 {
    let (output, logs) = (dir.path("output"), dir.path("logs"));
    run(&[
        "--job-type".as_ref(),
        "cam".as_ref(),
        "--input".as_ref(),
        input.as_os_str(),
        "--output".as_ref(),
        output.as_os_str(),
        "--logs".as_ref(),
        logs.as_os_str(),
        "--resume".as_ref(),
        resume.as_os_str(),
    ])
}

/// FNV-1a（64 ビット。途中の結果のファイルの検査の値）
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// 区切りの記録を覚える
#[derive(Default)]
struct Recorded(Vec<SavedRecord>);

impl Checkpoints for Recorded {
    fn next_saved(&mut self) -> Option<SavedRecord> {
        None
    }

    fn discard(&mut self, _reason: &str) {}

    fn record(&mut self, state: &[u8], result: &[u8]) {
        self.0.push(SavedRecord {
            state: state.to_vec(),
            result: result.to_vec(),
        });
    }
}

/// ランナーが書くのと同じ形の、途中の結果のファイル（頭と、記録 `records`）
fn resume_file_bytes(input: &[u8], records: &[SavedRecord]) -> Vec<u8> {
    let runner = fs::read(env!("CARGO_BIN_EXE_redring-batch-runner")).unwrap();
    let mut bytes = b"RRRESUME".to_vec();
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&fnv1a(&runner).to_le_bytes());
    bytes.extend_from_slice(&fnv1a(input).to_le_bytes());
    for (sequence, record) in records.iter().enumerate() {
        let mut body = (sequence as u64).to_le_bytes().to_vec();
        body.extend_from_slice(&(record.state.len() as u32).to_le_bytes());
        body.extend_from_slice(&(record.result.len() as u32).to_le_bytes());
        body.extend_from_slice(&record.state);
        body.extend_from_slice(&record.result);
        let checksum = fnv1a(&body);
        bytes.extend_from_slice(&body);
        bytes.extend_from_slice(&checksum.to_le_bytes());
    }
    bytes
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

#[test]
fn a_cam_job_with_a_resume_file_records_while_calculating_and_removes_it_at_the_end() {
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    write_cam_input(&input, &cam_input());
    let resume = dir.path("resume").join("toolpath.resume");

    assert_eq!(
        run_cam_job_with_resume(&dir, &input, &resume),
        0,
        "{}",
        log_text(&dir)
    );
    let artifact = fs::read(dir.path("output").join("toolpath.bin")).unwrap();
    assert_eq!(artifact, run_cam_solver(&cam_input()).unwrap());
    assert!(!resume.exists());
}

#[test]
fn a_cam_job_continues_from_the_records_of_the_previous_run() {
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    write_cam_input(&input, &cam_input());
    let mut recorded = Recorded::default();
    let full = run_cam_solver_resumable(&cam_input(), &mut recorded).unwrap();
    assert!(recorded.0.len() >= 2);
    let resume = dir.path("resume").join("toolpath.resume");
    fs::create_dir_all(resume.parent().unwrap()).unwrap();
    let done = recorded.0.len() / 2;
    fs::write(
        &resume,
        resume_file_bytes(&fs::read(&input).unwrap(), &recorded.0[..done]),
    )
    .unwrap();

    assert_eq!(
        run_cam_job_with_resume(&dir, &input, &resume),
        0,
        "{}",
        log_text(&dir)
    );
    assert_eq!(
        fs::read(dir.path("output").join("toolpath.bin")).unwrap(),
        full
    );
    assert!(
        log_text(&dir).contains(&format!("resume: continuing from {done} records")),
        "{}",
        log_text(&dir)
    );
    assert!(!resume.exists());
}
