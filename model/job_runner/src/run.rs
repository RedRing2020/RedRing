//! 入力のファイルを読み、計算し、成果物・ログ・進み具合を書く

use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use cam_sim::{
    JobRunFailure, read_cam_solver_input_v1, read_cutting_simulation_input_v1, run_cam_solver,
    run_cutting_simulation,
};

use crate::ExitCode;
use crate::args::{JobKind, RunnerArgs};

/// ツールパス生成の成果物のファイルの名前
const TOOLPATH_FILE: &str = "toolpath.bin";
/// 切削シミュレーションの成果物のファイルの名前
const CUTTING_SIMULATION_FILE: &str = "cutting_simulation.bin";
/// ログのファイルの名前（書いている間は `runner.log.tmp`）
const LOG_FILE: &str = "runner.log";

/// ジョブ 1 つを実行し、終了の番号を返す
pub(crate) fn run(args: &RunnerArgs) -> ExitCode {
    let mut log = match RunnerLog::create(&args.logs) {
        Ok(log) => log,
        Err(error) => {
            eprintln!("cannot write the log in {}: {error}", args.logs.display());
            return ExitCode::OutputUnwritable;
        }
    };
    log.line(&format!(
        "start job_type={} input={}",
        match args.job_kind {
            JobKind::Cam => "cam",
            JobKind::Sim => "sim",
        },
        args.input.display()
    ));
    let started = Instant::now();

    let code = match write_progress(args, "running", 0.0).and_then(|()| execute(args, &mut log)) {
        Ok(output) => {
            log.line(&format!(
                "succeeded output={} seconds={:.3}",
                output.display(),
                started.elapsed().as_secs_f64()
            ));
            ExitCode::Succeeded
        }
        Err(failure) => {
            log.line(&format!(
                "failed exit_code={} {}",
                failure.code as u8, failure.message
            ));
            failure.code
        }
    };
    let state = if code == ExitCode::Succeeded {
        "succeeded"
    } else {
        "failed"
    };
    let code = match write_progress(args, state, 1.0) {
        Ok(()) => code,
        Err(failure) => {
            log.line(&failure.message);
            // 計算の失敗のほうを返す（進み具合を書けないことより、失敗の理由が大事なため）
            if code == ExitCode::Succeeded {
                failure.code
            } else {
                code
            }
        }
    };
    match log.finish() {
        Ok(()) => code,
        Err(error) => {
            eprintln!("cannot finish the log: {error}");
            if code == ExitCode::Succeeded {
                ExitCode::OutputUnwritable
            } else {
                code
            }
        }
    }
}

/// 実行の失敗（終了の番号と理由）
struct Failure {
    code: ExitCode,
    message: String,
}

impl Failure {
    fn new(code: ExitCode, message: String) -> Self {
        Self { code, message }
    }
}

/// 計算の失敗を、入力の中身の誤り・入力を読めない・計算の失敗で終了の番号に分ける
fn job_failure(failure: JobRunFailure) -> Failure {
    let code = if failure.is_invalid_input() {
        ExitCode::InputInvalid
    } else if failure.is_unreadable_input() {
        ExitCode::InputUnreadable
    } else {
        ExitCode::JobFailed
    };
    Failure::new(code, format!("code={} {}", failure.code, failure.message))
}

/// 入力を読んで計算し、成果物を書く。成果物のパスを返す
fn execute(args: &RunnerArgs, log: &mut RunnerLog) -> Result<PathBuf, Failure> {
    // 前の実行の成果物が残っていると、失敗した実行の成果物に見えるため、計算の前に消す
    for name in [TOOLPATH_FILE, CUTTING_SIMULATION_FILE] {
        remove_if_exists(&args.output.join(name))?;
    }
    let input = read_file(&args.input, "input")?;
    let (artifact, file_name) = match args.job_kind {
        JobKind::Cam => {
            let input = read_cam_solver_input_v1(&mut input.as_slice()).map_err(|error| {
                Failure::new(ExitCode::InputInvalid, format!("invalid input: {error}"))
            })?;
            let artifact = run_cam_solver(&input).map_err(job_failure)?;
            (artifact, TOOLPATH_FILE)
        }
        JobKind::Sim => {
            let toolpath_path = args.toolpath.as_deref().ok_or_else(|| {
                Failure::new(
                    ExitCode::Usage,
                    "--toolpath is required for sim".to_string(),
                )
            })?;
            let input =
                read_cutting_simulation_input_v1(&mut input.as_slice()).map_err(|error| {
                    Failure::new(ExitCode::InputInvalid, format!("invalid input: {error}"))
                })?;
            // ツールパスは大きくなることがあるため、丸ごと読まずに、ファイルから読みながら組み立てる
            let mut toolpath = File::open(toolpath_path)
                .map(BufReader::new)
                .map_err(|error| {
                    Failure::new(
                        ExitCode::InputUnreadable,
                        format!(
                            "cannot read the toolpath {}: {error}",
                            toolpath_path.display()
                        ),
                    )
                })?;
            let run = run_cutting_simulation(&input, &mut toolpath).map_err(job_failure)?;
            log.line(&format!("snapshots={}", run.snapshot_count));
            (run.artifact, CUTTING_SIMULATION_FILE)
        }
    };
    let path = args.output.join(file_name);
    fs::create_dir_all(&args.output)
        .and_then(|()| write_atomically(&path, &artifact))
        .map_err(|error| {
            Failure::new(
                ExitCode::OutputUnwritable,
                format!("cannot write {}: {error}", path.display()),
            )
        })?;
    Ok(path)
}

fn remove_if_exists(path: &Path) -> Result<(), Failure> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(Failure::new(
            ExitCode::OutputUnwritable,
            format!("cannot remove {}: {error}", path.display()),
        )),
        _ => Ok(()),
    }
}

fn read_file(path: &Path, what: &str) -> Result<Vec<u8>, Failure> {
    fs::read(path).map_err(|error| {
        Failure::new(
            ExitCode::InputUnreadable,
            format!("cannot read the {what} {}: {error}", path.display()),
        )
    })
}

/// 進み具合（状態と割合）を、指定された場合だけ書く
fn write_progress(args: &RunnerArgs, state: &str, fraction: f64) -> Result<(), Failure> {
    let Some(path) = &args.progress else {
        return Ok(());
    };
    let json = format!("{{\"state\":\"{state}\",\"fraction\":{fraction:.3}}}\n");
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    parent
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| write_atomically(path, json.as_bytes()))
        .map_err(|error| {
            Failure::new(
                ExitCode::OutputUnwritable,
                format!("cannot write the progress {}: {error}", path.display()),
            )
        })
}

/// 一時の名前（`<名前>.tmp`）で書き終えてから、正しい名前に変える（読む側が書きかけを読まないように）
fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = temporary_path(path);
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// ログ。書いている間は一時の名前で、終わったら正しい名前に変える（ランナーが落ちた場合も、一時の名前のログは残る）
struct RunnerLog {
    file: File,
    temporary: PathBuf,
    path: PathBuf,
    /// 書けなかった行の最初の誤り
    write_error: Option<std::io::Error>,
}

impl RunnerLog {
    fn create(dir: &Path) -> std::io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path = dir.join(LOG_FILE);
        let temporary = temporary_path(&path);
        let file = File::create(&temporary)?;
        Ok(Self {
            file,
            temporary,
            path,
            write_error: None,
        })
    }

    /// 1 行書く。書けなくても計算は続ける（終わりの `finish` で誤りを返す）
    fn line(&mut self, text: &str) {
        if let Err(error) = writeln!(self.file, "{text}") {
            self.write_error.get_or_insert(error);
        }
    }

    fn finish(mut self) -> std::io::Result<()> {
        if let Some(error) = self.write_error.take() {
            return Err(error);
        }
        self.file.flush()?;
        self.file.sync_all()?;
        drop(self.file);
        fs::rename(&self.temporary, &self.path)
    }
}
