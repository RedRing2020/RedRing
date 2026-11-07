//! ランナーの起動（実行中のランナーの終わりの確かめと、止めること）

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

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
}

/// ランナーを起動する
pub trait Launcher {
    fn launch(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn RunningJob>>;
}

/// ランナーの実行ファイルを、シェルを通さずに起動する
#[derive(Debug, Clone)]
pub struct ProcessLauncher {
    runner: PathBuf,
}

impl ProcessLauncher {
    /// 実行ファイル `runner` を起動するランチャー
    pub fn new(runner: PathBuf) -> Self {
        Self { runner }
    }
}

impl Launcher for ProcessLauncher {
    fn launch(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn RunningJob>> {
        let child = Command::new(&self.runner)
            .args(spec.args())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Box::new(ProcessJob { child }))
    }
}

struct ProcessJob {
    child: Child,
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
}
