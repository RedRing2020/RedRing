//! ランナーの引数

use std::ffi::OsString;
use std::path::PathBuf;

pub(crate) const USAGE: &str = "usage: redring-batch-runner --job-type <cam|sim> --input <file> \
--output <dir> --logs <dir> [--memory-limit <bytes> (cam only, required)] \
[--toolpath <file> (sim only, required)] [--stock <file> (sim only)] [--resume <file> (cam only)] [--progress <file>]";

/// ジョブの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JobKind {
    /// ツールパス生成
    Cam,
    /// 切削シミュレーション
    Sim,
}

/// ランナーの引数
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunnerArgs {
    pub job_kind: JobKind,
    pub input: PathBuf,
    /// 切削シミュレーションの、親のジョブのツールパスの成果物
    pub toolpath: Option<PathBuf>,
    /// 切削シミュレーションの始まりの素材（前の区間の終わりの素材のファイル。なければ素材の範囲から削る）
    pub stock: Option<PathBuf>,
    pub output: PathBuf,
    pub logs: PathBuf,
    pub progress: Option<PathBuf>,
    /// ツールパス生成の途中の結果のファイル（区切りごとの記録。あれば続きから計算する）
    pub resume: Option<PathBuf>,
    /// ツールパス生成に使ってよいメモリの上限（バイト。ツールパス生成だけ）
    pub memory_limit: Option<u64>,
}

impl RunnerArgs {
    /// 引数（プログラムの名前を除く）を読む。誤りは理由（英語）を返す
    pub(crate) fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut job_type = None;
        let mut input = None;
        let mut toolpath = None;
        let mut stock = None;
        let mut output = None;
        let mut logs = None;
        let mut progress = None;
        let mut resume = None;
        let mut memory_limit = None;
        let mut args = args.into_iter();
        while let Some(name) = args.next() {
            let slot = match name.to_str() {
                Some("--job-type") => &mut job_type,
                Some("--input") => &mut input,
                Some("--toolpath") => &mut toolpath,
                Some("--stock") => &mut stock,
                Some("--output") => &mut output,
                Some("--logs") => &mut logs,
                Some("--progress") => &mut progress,
                Some("--resume") => &mut resume,
                Some("--memory-limit") => &mut memory_limit,
                _ => return Err(format!("unknown argument: {}", name.to_string_lossy())),
            };
            let value = args
                .next()
                .ok_or_else(|| format!("{} needs a value", name.to_string_lossy()))?;
            if slot.replace(value).is_some() {
                return Err(format!("{} is given twice", name.to_string_lossy()));
            }
        }

        let job_kind = match job_type.as_ref().and_then(|value| value.to_str()) {
            Some("cam") => JobKind::Cam,
            Some("sim") => JobKind::Sim,
            Some(_) => return Err("--job-type must be cam or sim".to_string()),
            None => return Err("--job-type is required".to_string()),
        };
        let required = |value: Option<OsString>, name: &str| {
            value
                .map(PathBuf::from)
                .ok_or_else(|| format!("{name} is required"))
        };
        let toolpath = match (job_kind, toolpath) {
            (JobKind::Sim, None) => return Err("--toolpath is required for sim".to_string()),
            (JobKind::Cam, Some(_)) => return Err("--toolpath is only for sim".to_string()),
            (_, toolpath) => toolpath.map(PathBuf::from),
        };
        if job_kind == JobKind::Sim && resume.is_some() {
            return Err("--resume is only for cam".to_string());
        }
        if job_kind == JobKind::Cam && stock.is_some() {
            return Err("--stock is only for sim".to_string());
        }
        let memory_limit = match (job_kind, memory_limit) {
            (JobKind::Cam, None) => return Err("--memory-limit is required for cam".to_string()),
            (JobKind::Sim, Some(_)) => return Err("--memory-limit is only for cam".to_string()),
            (_, None) => None,
            (_, Some(value)) => Some(
                value
                    .to_str()
                    .and_then(|value| value.parse::<u64>().ok())
                    .filter(|&bytes| bytes > 0)
                    .ok_or("--memory-limit must be a positive number of bytes")?,
            ),
        };
        Ok(Self {
            job_kind,
            input: required(input, "--input")?,
            toolpath,
            stock: stock.map(PathBuf::from),
            output: required(output, "--output")?,
            logs: required(logs, "--logs")?,
            progress: progress.map(PathBuf::from),
            resume: resume.map(PathBuf::from),
            memory_limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<RunnerArgs, String> {
        RunnerArgs::parse(args.iter().map(OsString::from))
    }

    #[test]
    fn reads_cam_and_sim_arguments_in_any_order() {
        let cam = parse(&[
            "--output",
            "out",
            "--job-type",
            "cam",
            "--logs",
            "logs",
            "--input",
            "in.bin",
            "--memory-limit",
            "4000000000",
        ])
        .unwrap();
        assert_eq!(cam.job_kind, JobKind::Cam);
        assert_eq!(cam.memory_limit, Some(4_000_000_000));
        assert_eq!(cam.input, PathBuf::from("in.bin"));
        assert_eq!(cam.toolpath, None);
        assert_eq!(cam.progress, None);
        assert_eq!(cam.resume, None);

        let sim = parse(&[
            "--job-type",
            "sim",
            "--input",
            "in.bin",
            "--toolpath",
            "tp.bin",
            "--output",
            "out",
            "--logs",
            "logs",
            "--progress",
            "progress.json",
        ])
        .unwrap();
        assert_eq!(sim.job_kind, JobKind::Sim);
        assert_eq!(sim.toolpath, Some(PathBuf::from("tp.bin")));
        assert_eq!(sim.stock, None);
        assert_eq!(sim.progress, Some(PathBuf::from("progress.json")));
        assert_eq!(sim.memory_limit, None);
    }

    #[test]
    fn rejects_missing_unknown_repeated_and_misplaced_arguments() {
        let base = ["--input", "in", "--output", "out", "--logs", "logs"];
        let cam = ["--job-type", "cam", "--memory-limit", "1000"];
        let with = |extra: &[&'static str]| {
            let mut args = base.to_vec();
            args.extend_from_slice(extra);
            parse(&args)
        };
        assert!(with(&[]).is_err());
        assert!(with(&["--job-type", "nc"]).is_err());
        assert!(with(&["--job-type", "sim"]).is_err());
        assert!(with(&[&cam[..], &["--toolpath", "tp"]].concat()).is_err());
        assert!(with(&["--job-type", "sim", "--toolpath", "tp", "--resume", "r"]).is_err());
        assert!(with(&[&cam[..], &["--stock", "s"]].concat()).is_err());
        assert_eq!(
            with(&["--job-type", "sim", "--toolpath", "tp", "--stock", "s"])
                .unwrap()
                .stock,
            Some(PathBuf::from("s"))
        );
        assert_eq!(
            with(&[&cam[..], &["--resume", "r"]].concat())
                .unwrap()
                .resume,
            Some(PathBuf::from("r"))
        );
        assert!(with(&[&cam[..], &["--input", "again"]].concat()).is_err());
        assert!(with(&[&cam[..], &["--verbose"]].concat()).is_err());
        // メモリの上限は、ツールパス生成に要り、正の数のバイトに限る
        assert!(with(&["--job-type", "cam"]).is_err());
        for bad in ["0", "-1", "2.5", "abc"] {
            assert!(with(&["--job-type", "cam", "--memory-limit", bad]).is_err());
        }
        assert!(
            with(&[
                "--job-type",
                "sim",
                "--toolpath",
                "tp",
                "--memory-limit",
                "1000"
            ])
            .is_err()
        );
        assert!(with(&["--job-type"]).is_err());
        assert!(parse(&["--job-type", "cam", "--input", "in", "--output", "out"]).is_err());
    }
}
