//! ジョブマネージャーの設定のファイル（Windows では `%APPDATA%\RedRing\job_manager.json`）
//!
//! 設定は JSON のオブジェクトの項目として持つ。項目を書き替えるときは、ほかの項目を読んだまま書き戻し、読めない設定の
//! ファイルは上書きしない（ほかの項目を失わないように）。

use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// 設定のファイルの大きさの上限
const MAX_SETTINGS_FILE_BYTES: u64 = 64 * 1024;

/// 項目 `key` の値（設定のファイルがない・読めない・項目がないときは `None`）
pub(crate) fn get(key: &str) -> Option<Value> {
    let path = settings_file_path()?;
    read_settings(&path).ok().flatten()?.remove(key)
}

/// 項目 `key` を値 `value` にして保存する
///
/// # Errors
///
/// 設定のファイルの場所が分からない・読めない（上書きしない）・書けない場合。
pub(crate) fn set(key: &str, value: Value) -> std::io::Result<()> {
    let path = settings_file_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "the per-user settings folder is unknown",
        )
    })?;
    let mut settings = read_settings(&path)?.unwrap_or_default();
    settings.insert(key.to_string(), value);
    write_settings(&path, &settings)
}

/// 設定のファイルを読む（ないときは `None`）
///
/// # Errors
///
/// 読めない・大きさの上限を超える・JSON のオブジェクトでない場合。
fn read_settings(path: &Path) -> std::io::Result<Option<Map<String, Value>>> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let invalid =
        |reason: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, reason.to_string());
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SETTINGS_FILE_BYTES {
        return Err(invalid("the settings file is too large"));
    }
    match serde_json::from_slice(&bytes).map_err(|error| invalid(&error.to_string()))? {
        Value::Object(settings) => Ok(Some(settings)),
        _ => Err(invalid("the settings file is not a JSON object")),
    }
}

/// 設定のファイルを書く（一時の名前で書き終えてから名前を変える）
fn write_settings(path: &Path, settings: &Map<String, Value>) -> std::io::Result<()> {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder)?;
    }
    let bytes = serde_json::to_vec_pretty(settings).map_err(std::io::Error::other)?;
    // ほかのプロセスと一時のファイルが重ならないよう、プロセスの番号を名前に入れる
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written =
        std::fs::write(&temporary, bytes).and_then(|()| std::fs::rename(&temporary, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    written
}

/// ジョブマネージャーの設定のファイル（Windows では `%APPDATA%\RedRing\job_manager.json`）
fn settings_file_path() -> Option<PathBuf> {
    let absolute = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    let base = if cfg!(windows) {
        absolute("APPDATA")
    } else {
        absolute("XDG_CONFIG_HOME").or_else(|| absolute("HOME").map(|home| home.join(".config")))
    }?;
    Some(base.join("RedRing").join("job_manager.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_written_and_read_back_keeping_the_other_keys() {
        let folder = std::env::temp_dir().join(format!(
            "redring-job-manager-settings-test-{}",
            std::process::id()
        ));
        let path = folder.join("job_manager.json");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(&path, br#"{"store":"D:\\jobs","language":"ja"}"#).unwrap();

        let mut settings = read_settings(&path).unwrap().unwrap();
        settings.insert("language".to_string(), Value::String("en".to_string()));
        write_settings(&path, &settings).unwrap();
        let read = read_settings(&path).unwrap().unwrap();
        assert_eq!(read.get("language").and_then(Value::as_str), Some("en"));
        assert_eq!(read.get("store").and_then(Value::as_str), Some("D:\\jobs"));
        // 一時のファイルを残さない
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
        // 形の違うファイル・大きすぎるファイルは、読めない（ないファイルとは分ける）
        std::fs::write(&path, b"[1, 2]").unwrap();
        assert!(read_settings(&path).is_err());
        std::fs::write(&path, vec![b' '; MAX_SETTINGS_FILE_BYTES as usize + 1]).unwrap();
        assert!(read_settings(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(read_settings(&path).unwrap().is_none());
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
