//! ランナーの実行ファイルを起動し、終了の番号と、書いたファイルを確かめる

mod common;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use cam_core::{OperationSpec, Tool, read_toolpath_artifact_v1};
use cam_sim::{
    Checkpoints, SavedRecord, TOOLPATH_MEMORY_LIMIT_BYTES, read_cutting_simulation_artifact_v1,
    run_cam_solver, run_cam_solver_resumable,
};
use common::{WorkDir, cam_input, write_cam_input, write_sim_input};

/// 引数 `args` でランナーを起動するコマンド
fn runner(args: &[&std::ffi::OsStr]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_redring-batch-runner"));
    command.args(args);
    command
}

/// コマンド `command` を終わるまで動かし、終了の番号を返す
fn exit_code(mut command: Command) -> i32 {
    command.output().unwrap().status.code().unwrap()
}

/// ランナーを起動し、終了の番号を返す
fn run(args: &[&std::ffi::OsStr]) -> i32 {
    exit_code(runner(args))
}

fn run_job(dir: &WorkDir, job_type: &str, input: &Path, toolpath: Option<&Path>) -> i32 {
    run_job_with_limit(dir, job_type, input, toolpath, TOOLPATH_MEMORY_LIMIT_BYTES)
}

/// ツールパス生成ではメモリの上限 `memory_limit`（バイト）を渡して、ランナーを起動する
fn run_job_with_limit(
    dir: &WorkDir,
    job_type: &str,
    input: &Path,
    toolpath: Option<&Path>,
    memory_limit: u64,
) -> i32 {
    let memory_limit = memory_limit.to_string();
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
    if job_type == "cam" {
        args.extend([OsStr::new("--memory-limit"), OsStr::new(&memory_limit)]);
    }
    run(&args)
}

/// 途中の結果のファイル `resume` を渡して、ツールパス生成のランナーを起動する
fn run_cam_job_with_resume(dir: &WorkDir, input: &Path, resume: &Path) -> i32 {
    exit_code(cam_job_with_resume(dir, input, resume))
}

/// 途中の結果のファイル `resume` を渡す、ツールパス生成のランナーのコマンド
fn cam_job_with_resume(dir: &WorkDir, input: &Path, resume: &Path) -> Command {
    let (output, logs) = (dir.path("output"), dir.path("logs"));
    let memory_limit = TOOLPATH_MEMORY_LIMIT_BYTES.to_string();
    runner(&[
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
        "--memory-limit".as_ref(),
        memory_limit.as_ref(),
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

/// 途中の結果のファイルの、続きから計算するのに使える記録（頭のあとの、通し番号が続き、状態が長すぎず、検査の値
/// の合う記録。書きかけの記録は数えない）を数える。前に数えた所より先だけを読む
struct UsableRecords<'a> {
    path: &'a Path,
    /// 数えた記録の終わりの位置（まだ数えていなければ 0）
    end: u64,
    count: u64,
}

impl<'a> UsableRecords<'a> {
    /// 頭の長さ（印・形式の版・ランナーと入力の検査の値）
    const HEADER_LEN: u64 = 8 + 4 + 8 + 8;
    /// 状態の長さの上限（ランナーと同じ）
    const MAX_STATE_LEN: usize = 4096;

    fn new(path: &'a Path) -> Self {
        Self {
            path,
            end: 0,
            count: 0,
        }
    }

    /// 今のファイルの記録の数（ファイルがない・数えた所より短いときは、最初から数え直す）
    fn update(&mut self) -> u64 {
        let Ok(mut file) = File::open(self.path) else {
            (self.end, self.count) = (0, 0);
            return 0;
        };
        let len = file.metadata().unwrap().len();
        if len < self.end.max(Self::HEADER_LEN) {
            (self.end, self.count) = (0, 0);
            return 0;
        }
        if self.end == 0 {
            let mut magic = [0; 8];
            file.read_exact(&mut magic).unwrap();
            if &magic != b"RRRESUME" {
                return 0;
            }
            self.end = Self::HEADER_LEN;
        }
        let mut rest = Vec::new();
        file.seek(SeekFrom::Start(self.end)).unwrap();
        file.read_to_end(&mut rest).unwrap();
        let mut rest = rest.as_slice();
        while rest.len() >= 16 {
            let sequence = u64::from_le_bytes(rest[..8].try_into().unwrap());
            let state_len = u32::from_le_bytes(rest[8..12].try_into().unwrap()) as usize;
            let result_len = u32::from_le_bytes(rest[12..16].try_into().unwrap()) as usize;
            let body_len = 16 + state_len + result_len;
            if sequence != self.count
                || state_len > Self::MAX_STATE_LEN
                || rest.len() < body_len + 8
            {
                break;
            }
            let checksum = u64::from_le_bytes(rest[body_len..body_len + 8].try_into().unwrap());
            if checksum != fnv1a(&rest[..body_len]) {
                break;
            }
            self.count += 1;
            self.end += (body_len + 8) as u64;
            rest = &rest[body_len + 8..];
        }
        self.count
    }
}

/// ランナーのログ（途中で止まったときは、書きかけのログ）
fn runner_log(dir: &WorkDir) -> String {
    let logs = dir.path("logs");
    fs::read_to_string(logs.join("runner.log"))
        .or_else(|_| fs::read_to_string(logs.join("runner.log.tmp")))
        .unwrap_or_default()
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
    assert_eq!(
        artifact,
        run_cam_solver(&cam_input(), TOOLPATH_MEMORY_LIMIT_BYTES).unwrap()
    );
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
    fs::write(
        &toolpath,
        run_cam_solver(&cam_input(), TOOLPATH_MEMORY_LIMIT_BYTES).unwrap(),
    )
    .unwrap();
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

    // ツールパスの点の数が、引数で渡したメモリの上限を超える（点 1 つあたり 250 バイトのため、1000 バイトは 4 点）
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    write_cam_input(&input, &cam_input());
    assert_eq!(run_job_with_limit(&dir, "cam", &input, None, 1000), 7);
    assert!(log_text(&dir).contains("for the memory limit of 1000 bytes"));
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
    assert_eq!(
        artifact,
        run_cam_solver(&cam_input(), TOOLPATH_MEMORY_LIMIT_BYTES).unwrap()
    );
    assert!(!resume.exists());
}

#[test]
fn a_cam_job_continues_from_the_records_of_the_previous_run() {
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    write_cam_input(&input, &cam_input());
    let mut recorded = Recorded::default();
    let full = run_cam_solver_resumable(
        &cam_input(),
        TOOLPATH_MEMORY_LIMIT_BYTES,
        &mut recorded,
        &mut |_, _| {},
    )
    .unwrap();
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

#[test]
fn a_failed_cam_job_removes_the_resume_file() {
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    let mut invalid = cam_input();
    invalid.tool = Tool::ball_end_mill("BEM0".to_string(), 0.0, 40.0);
    write_cam_input(&input, &invalid);
    let resume = dir.path("resume").join("toolpath.resume");
    fs::create_dir_all(resume.parent().unwrap()).unwrap();
    fs::write(&resume, b"records of an earlier run").unwrap();

    assert_eq!(run_cam_job_with_resume(&dir, &input, &resume), 4);
    assert!(!resume.exists());
}

#[test]
fn a_cam_job_killed_part_way_again_and_again_continues_to_the_same_toolpath() {
    /// 強制終了する回数
    const KILLS: u64 = 8;
    let dir = WorkDir::new();
    let input = dir.path("input.bin");
    // 走査ラインを多くし、何回止めても計算が終わりきらないようにする（走査ラインごとに 1 つ記録する）。最初から
    // 計算するのに 0.4 秒より短ければ、走査ラインを 10 倍にする（速いビルド・速い PC では、止めるまでに進む記録が多い）
    let mut solver_input = cam_input();
    let mut stepover = 0.1;
    let (full, total) = loop {
        let OperationSpec::Scanline(params) = &mut solver_input.operation;
        params.stepover = stepover;
        let mut recorded = Recorded::default();
        let started = Instant::now();
        let full = run_cam_solver_resumable(
            &solver_input,
            TOOLPATH_MEMORY_LIMIT_BYTES,
            &mut recorded,
            &mut |_, _| {},
        )
        .unwrap();
        if started.elapsed() >= Duration::from_millis(400) || stepover <= 0.01 {
            break (full, recorded.0.len() as u64);
        }
        stepover /= 10.0;
    };
    write_cam_input(&input, &solver_input);
    let resume = dir.path("resume").join("toolpath.resume");
    let mut records = UsableRecords::new(&resume);

    let mut previous = 0;
    for round in 0..KILLS {
        let mut child = cam_job_with_resume(&dir, &input, &resume)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        // 前の回の続きから、回ごとに違う数の記録を書き足すまで待ち、回ごとに違う間を置いて止める。止める前に計算が
        // 終わらないよう、眠らずに確かめ続ける（Windows では短い眠りが十数ミリ秒になるため）
        let target = previous + 1 + round * 3 % 7;
        let started = Instant::now();
        while records.update() < target
            && child.try_wait().unwrap().is_none()
            && started.elapsed() < Duration::from_secs(60)
        {
            std::thread::yield_now();
        }
        let pause = Instant::now();
        while pause.elapsed() < Duration::from_micros(200 * (round % 4)) {
            std::thread::yield_now();
        }
        let observed = records.update();
        let running = child.try_wait().unwrap().is_none();
        if running {
            child.kill().unwrap();
        }
        let status = child.wait().unwrap();
        assert!(
            running && observed >= target,
            "before kill {round}, the runner ended or did not reach {target} records ({observed} of \
             {total}, {status}): {}",
            runner_log(&dir)
        );
        let now = records.update();
        assert!(
            now >= observed,
            "kill {round} lost records: {observed} before it, {now} after it"
        );
        previous = now;
    }

    assert_eq!(
        run_cam_job_with_resume(&dir, &input, &resume),
        0,
        "{}",
        log_text(&dir)
    );
    assert!(
        log_text(&dir).contains(&format!("resume: continuing from {previous} records")),
        "{}",
        log_text(&dir)
    );
    assert_eq!(
        fs::read(dir.path("output").join("toolpath.bin")).unwrap(),
        full
    );
    assert!(!resume.exists());
}
