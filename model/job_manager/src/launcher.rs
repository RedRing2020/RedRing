//! ランナーの起動（実行中のランナーの終わりの確かめと、止めること）

use std::cell::OnceCell;
use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use platform_foundation::process::{ChildGroup, hide_console_window};

/// ランナーに渡す引数（パスはすべて保存先のジョブのフォルダの下）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchSpec {
    /// `cam`・`sim`
    pub job_type: &'static str,
    pub input: PathBuf,
    /// 切削シミュレーションの、依存するツールパス生成のジョブの成果物
    pub toolpath: Option<PathBuf>,
    pub output: PathBuf,
    pub logs: PathBuf,
    pub progress: PathBuf,
    /// ランナーの標準エラー出力を書くファイル（ランナーの引数には渡さない）
    pub stderr: PathBuf,
}

impl LaunchSpec {
    /// ランナーのコマンドラインの引数
    pub fn args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = vec![
            "--job-type".into(),
            self.job_type.into(),
            "--input".into(),
            self.input.clone().into(),
        ];
        if let Some(toolpath) = &self.toolpath {
            args.extend(["--toolpath".into(), toolpath.clone().into()]);
        }
        args.extend([
            "--output".into(),
            self.output.clone().into(),
            "--logs".into(),
            self.logs.clone().into(),
            "--progress".into(),
            self.progress.clone().into(),
        ]);
        args
    }
}

/// 実行中のランナー
pub trait RunningJob {
    /// 終わっていれば終了の番号（番号のない終わり方は `None` の `Some`）、終わっていなければ `Ok(None)`
    fn try_finish(&mut self) -> std::io::Result<Option<Option<i32>>>;
    /// 止めて、終わるのを待つ
    fn stop(&mut self) -> std::io::Result<()>;
    /// ジョブマネージャーが止まると一緒に止まる組に入れられなかった理由（入れたとき・組を使わないときは `None`）
    fn ungrouped_reason(&self) -> Option<&str> {
        None
    }
}

/// ランナーを起動する
pub trait Launcher {
    fn launch(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn RunningJob>>;
}

/// ランナーの実行ファイルを、シェルを通さずに起動する
///
/// ランナーは、このプロセスが終わる（異常終了を含む）と一緒に止まる組に入れる（`ChildGroup`。組を持たない OS では
/// 入れない）。このランチャーを捨てたときも、組のランナーは止まる。組を作れない・組に入れられないときは、組に入れずに
/// 動かし、その理由を `RunningJob::ungrouped_reason` で返す（組を使えない環境でも計算できるように）。
#[derive(Debug)]
pub struct ProcessLauncher {
    runner: PathBuf,
    /// 最初に起動するときに作る（作れなければ、組に入れずに起動し、次の起動で作り直す）
    group: OnceCell<ChildGroup>,
}

impl ProcessLauncher {
    /// 実行ファイル `runner` を起動するランチャー
    pub fn new(runner: PathBuf) -> Self {
        Self {
            runner,
            group: OnceCell::new(),
        }
    }

    /// ランナーの組（まだなければ作る）
    fn group(&self) -> std::io::Result<&ChildGroup> {
        if let Some(group) = self.group.get() {
            return Ok(group);
        }
        let group = ChildGroup::new()?;
        Ok(self.group.get_or_init(|| group))
    }
}

impl Launcher for ProcessLauncher {
    fn launch(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn RunningJob>> {
        // 標準エラー出力を残す（ランナーがログに書けずに止まったときの理由。前の実行の分は消す）。補助の情報のため、
        // ファイルを作れなければ捨てて起動する（ログのフォルダに書けないことは、ランナーが終了の番号で返す）
        let stderr = spec
            .stderr
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| File::create(&spec.stderr))
            .map_or_else(|_| Stdio::null(), Stdio::from);
        let mut command = Command::new(&self.runner);
        command
            .args(spec.args())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr);
        hide_console_window(&mut command);
        let (child, ungrouped) = match self.group() {
            Ok(group) => group.spawn(&mut command)?,
            Err(error) => (command.spawn()?, Some(error)),
        };
        Ok(Box::new(ProcessJob {
            child,
            ungrouped: ungrouped.map(|error| error.to_string()),
        }))
    }
}

struct ProcessJob {
    child: Child,
    ungrouped: Option<String>,
}

impl RunningJob for ProcessJob {
    fn try_finish(&mut self) -> std::io::Result<Option<Option<i32>>> {
        Ok(self.child.try_wait()?.map(|status| status.code()))
    }

    fn stop(&mut self) -> std::io::Result<()> {
        // すでに終わっていれば kill は誤りを返すため、終わりを待つだけにする
        if self.child.try_wait()?.is_none() {
            self.child.kill()?;
        }
        self.child.wait().map(|_| ())
    }

    fn ungrouped_reason(&self) -> Option<&str> {
        self.ungrouped.as_deref()
    }
}
