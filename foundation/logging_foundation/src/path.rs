use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// 残す前の回のログの数
const PREVIOUS_LOGS_KEPT: usize = 5;

/// 前の回のログをずらせず、同じファイルに続けて書くときの区切りの行
const NEXT_START_LINE: &str = "----- next start -----";

/// ログディレクトリとファイル名からログファイルパスを組み立てる。
pub fn compose_log_file_path(log_dir: &str, log_file_name: &str) -> PathBuf {
    Path::new(log_dir).join(log_file_name)
}

/// ログのファイル `log_file_path` を、新しく書き始めるように開く（フォルダがなければ作る）
///
/// 前の回までのログは、番号付きの名前にずらして `PREVIOUS_LOGS_KEPT` 回分を残す（`redring.log` → `redring.1.log`
/// → `redring.2.log` …。それより古いものは消す）。前の回のログをずらせないときは、前の回の中身を消さないように、
/// 区切りの行を書いてから同じファイルの末尾に続けて書く。開けなければ `None`。
pub fn open_log_file(log_file_path: &Path) -> Option<fs::File> {
    let log_dir = log_file_path.parent().unwrap_or(Path::new("."));
    let log_file_name = log_file_path.file_name()?.to_string_lossy();
    if let Err(e) = fs::create_dir_all(log_dir) {
        eprintln!("Failed to create the log directory ({:?}): {}", e.kind(), e);
        return None;
    }
    let opened = if keep_previous_logs(log_dir, &log_file_name, PREVIOUS_LOGS_KEPT) {
        fs::File::create(log_file_path)
    } else {
        fs::OpenOptions::new()
            .append(true)
            .open(log_file_path)
            .and_then(|mut file| {
                writeln!(file)?;
                writeln!(file, "{NEXT_START_LINE}")?;
                Ok(file)
            })
    };
    match opened {
        Ok(file) => Some(file),
        Err(e) => {
            eprintln!("Failed to create the log file ({:?}): {}", e.kind(), e);
            None
        }
    }
}

/// 前の回のログ `log_file_name` を、番号付きの名前に 1 つずつずらし、`kept` 回分を残す
///
/// ずらす先にまだファイルがある（ほかのプログラムが開いていて、消せなかった・ずらせなかった）ときは、ずらさない
/// （名前の変更は置き換えになるため、残っているログを上書きして消さないように）。前の回のログ `log_file_name` が
/// 残っていない（ずらせた・はじめからない）なら `true`。
fn keep_previous_logs(log_dir: &Path, log_file_name: &str, kept: usize) -> bool {
    let numbered = |number: usize| log_dir.join(numbered_name(log_file_name, number));
    let _ = fs::remove_file(numbered(kept));
    let shift = |from: PathBuf, to: PathBuf| {
        if from.exists() && !to.exists() {
            let _ = fs::rename(from, to);
        }
    };
    for number in (1..kept).rev() {
        shift(numbered(number), numbered(number + 1));
    }
    let current = log_dir.join(log_file_name);
    shift(current.clone(), numbered(1));
    !current.exists()
}

/// ログのファイルの名前 `log_file_name` の、`number` 回前の名前（拡張子の前に番号を入れる）
fn numbered_name(log_file_name: &str, number: usize) -> String {
    match log_file_name.rsplit_once('.') {
        Some((stem, extension)) => format!("{stem}.{number}.{extension}"),
        None => format!("{log_file_name}.{number}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 試験ごとの空のフォルダ（一時のフォルダの下。試験の終わりに消す）
    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("redring-logging-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            Self(path)
        }

        fn names(&self) -> Vec<String> {
            let mut names: Vec<String> = fs::read_dir(&self.0)
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }

        fn read(&self, name: &str) -> String {
            fs::read_to_string(self.0.join(name)).unwrap()
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// 起動 1 回分のログを書く
    fn start(folder: &Folder, run: usize) {
        let mut file = open_log_file(&folder.0.join("app.log")).unwrap();
        write!(file, "run {run}").unwrap();
    }

    #[test]
    fn test_compose_log_file_path() {
        let p = compose_log_file_path("logs", "redring.log");
        assert_eq!(p, PathBuf::from("logs/redring.log"));
    }

    #[test]
    fn the_logs_of_the_previous_five_starts_are_kept_in_order() {
        // ログのフォルダがなくても作る
        let folder = Folder::new("kept");
        for run in 1..=7 {
            start(&folder, run);
        }
        assert_eq!(
            folder.names(),
            [
                "app.1.log",
                "app.2.log",
                "app.3.log",
                "app.4.log",
                "app.5.log",
                "app.log"
            ]
        );
        assert_eq!(folder.read("app.log"), "run 7");
        for back in 1..=5 {
            assert_eq!(
                folder.read(&format!("app.{back}.log")),
                format!("run {}", 7 - back)
            );
        }
    }

    #[test]
    fn a_file_already_at_a_numbered_name_is_replaced() {
        let folder = Folder::new("replaced");
        fs::create_dir_all(&folder.0).unwrap();
        fs::write(folder.0.join("app.log"), "previous").unwrap();
        fs::write(folder.0.join("app.1.log"), "left").unwrap();
        fs::write(folder.0.join("app.3.log"), "stray").unwrap();
        start(&folder, 1);
        assert_eq!(folder.read("app.log"), "run 1");
        assert_eq!(folder.read("app.1.log"), "previous");
        assert_eq!(folder.read("app.2.log"), "left");
        assert_eq!(folder.read("app.4.log"), "stray");
    }

    #[test]
    fn the_previous_log_is_kept_when_the_oldest_cannot_be_removed() {
        let folder = Folder::new("blocked");
        fs::create_dir_all(&folder.0).unwrap();
        fs::write(folder.0.join("app.log"), "previous").unwrap();
        for back in 1..=4 {
            fs::write(
                folder.0.join(format!("app.{back}.log")),
                format!("old {back}"),
            )
            .unwrap();
        }
        // 消せないいちばん古いログの代わりに、中身のあるフォルダを置く
        fs::create_dir_all(folder.0.join("app.5.log").join("inside")).unwrap();
        start(&folder, 1);
        assert_eq!(
            folder.read("app.log"),
            format!("previous\n{NEXT_START_LINE}\nrun 1")
        );
        for back in 1..=4 {
            assert_eq!(
                folder.read(&format!("app.{back}.log")),
                format!("old {back}")
            );
        }
    }

    #[test]
    fn a_name_without_an_extension_gets_the_number_at_the_end() {
        assert_eq!(numbered_name("manager", 2), "manager.2");
        assert_eq!(numbered_name("job_manager.log", 1), "job_manager.1.log");
    }
}
