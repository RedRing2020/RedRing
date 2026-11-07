//! ランナーの実行ファイルを起動し、終了の番号と、書いたファイルを確かめる

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use cam_core::{Tool, read_toolpath_artifact_v1};
use cam_sim::{read_cutting_simulation_artifact_v1, run_cam_solver};
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
