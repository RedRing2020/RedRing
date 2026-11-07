//! ジョブ 1 つの計算だけを行うランナー（設計: JOB_MANAGER_STORAGE_DESIGN.md 3 章）
//!
//! ```text
//! redring-batch-runner --job-type cam --input <入力> --output <フォルダ> --logs <フォルダ> [--progress <ファイル>]
//! redring-batch-runner --job-type sim --input <入力> --toolpath <ツールパスの成果物> --output <フォルダ> --logs <フォルダ>
//!                      [--progress <ファイル>]
//! ```
//!
//! 入力のファイルを読み、計算し、成果物（cam: `toolpath.bin`、sim: `cutting_simulation.bin`）を出力のフォルダに、
//! ログ（`runner.log`）をログのフォルダに書く。ファイルは一時の名前（`*.tmp`）で書き終えてから名前を変える。
//! 成否は終了の番号で返す（[`ExitCode`] の値）。

mod args;
mod run;

use std::process::ExitCode as ProcessExitCode;

use args::RunnerArgs;

/// ランナーの終了の番号
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ExitCode {
    Succeeded = 0,
    /// 引数の誤り
    Usage = 2,
    /// 入力のファイルを読めない
    InputUnreadable = 3,
    /// 入力のファイルの形式・中身の誤り
    InputInvalid = 4,
    /// 計算の失敗
    JobFailed = 5,
    /// 成果物・ログ・進み具合を書けない
    OutputUnwritable = 6,
}

fn main() -> ProcessExitCode {
    let code = match RunnerArgs::parse(std::env::args_os().skip(1)) {
        Ok(args) => run::run(&args),
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", args::USAGE);
            ExitCode::Usage
        }
    };
    ProcessExitCode::from(code as u8)
}
