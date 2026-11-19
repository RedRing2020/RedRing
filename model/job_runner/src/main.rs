//! ジョブ 1 つの計算だけを行うランナー（設計: JOB_MANAGER_STORAGE_DESIGN.md 3 章）
//!
//! ```text
//! redring-batch-runner --job-type cam --input <入力> --output <フォルダ> --logs <フォルダ> --memory-limit <バイト>
//!                      [--resume <ファイル>] [--progress <ファイル>]
//! redring-batch-runner --job-type sim --input <入力> --toolpath <ツールパスの成果物> --output <フォルダ> --logs <フォルダ>
//!                      [--progress <ファイル>]
//! ```
//!
//! 入力のファイルを読み、計算し、成果物（cam: `toolpath.bin`、sim: `cutting_simulation.bin`）を出力のフォルダに、
//! ログ（`runner.log`）をログのフォルダに書く。ファイルは一時の名前（`*.tmp`）で書き終えてから名前を変える。
//! 成否は終了の番号で返す（[`ExitCode`] の値）。
//!
//! ツールパス生成は、点の数が `--memory-limit`（ツールパス生成に使ってよいメモリのバイトの数）に収まる数を超えたら止める。
//!
//! ツールパス生成は、`--resume` のファイルに計算の区切りごとの記録を書き足し、記録があれば続きから計算する。成功・失敗で
//! 終わったら、そのファイルを消す（途中で止められたときだけ残る）。
//!
//! 進み具合（`--progress`）は、始まり・終わりと、計算の途中の一定の間隔で書く（割合と、分かれば残りの目安の秒数）。

mod args;
mod progress;
mod resume;
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
    /// ツールパスがメモリの上限を超えるため止めた
    TooLarge = 7,
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
