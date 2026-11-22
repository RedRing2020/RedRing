//! ツールパスの大きさ（生成に使うメモリの見積もり）の、計算の前に確かめる目安と上限
//!
//! 設定のファイルの項目 `toolpath_confirm_gb`（確かめる目安）・`toolpath_limit_gb`（上限）から、GB（10 億バイト）の
//! 数で読む。項目がない・値が正しくない（数でない・0 以下・目安が上限より大きいなど）ときは既定の値を使い、その理由を
//! ジョブマネージャーのログに書く（計算は止めない）。起動のときに 1 回だけ読む。

use std::path::Path;

use job_ipc::ToolpathLimits;
use serde_json::{Map, Value};

use crate::settings;

/// 確かめる目安の項目
const CONFIRM_KEY: &str = "toolpath_confirm_gb";
/// 上限の項目
const LIMIT_KEY: &str = "toolpath_limit_gb";
/// 確かめる目安の既定の値（バイト。2 GB）
const DEFAULT_CONFIRM_BYTES: u64 = 2_000_000_000;
/// 上限の既定の値の最大（バイト。8 GB。物理メモリの半分がこれより小さければ、物理メモリの半分にする）
const MAX_DEFAULT_LIMIT_BYTES: u64 = 8_000_000_000;
/// 1 GB のバイトの数
const BYTES_PER_GB: f64 = 1e9;

/// 設定のファイル `path`（場所が分からないときは `None`）から、この PC の物理メモリ `physical_memory`（バイト。
/// 分からないときは `None`）での目安と上限を決める。決めた値と、既定の値にした理由をログの行（英語）で返す
pub(crate) fn read(
    path: Option<&Path>,
    physical_memory: Option<u64>,
) -> (ToolpathLimits, Vec<String>) {
    let mut notes = Vec::new();
    let settings = match path.map(settings::read_settings) {
        Some(Ok(settings)) => settings,
        Some(Err(error)) => {
            notes.push(format!(
                "cannot read the settings file for the toolpath limits: {error}"
            ));
            None
        }
        None => None,
    };
    let limits = resolve(settings.as_ref(), physical_memory, &mut notes);
    notes.push(format!(
        "toolpath limits confirm_bytes={} limit_bytes={} physical_memory_bytes={}",
        limits.confirm_bytes,
        limits.limit_bytes,
        physical_memory.map_or_else(|| "unknown".to_string(), |bytes| bytes.to_string())
    ));
    (limits, notes)
}

/// 設定 `settings` の項目と物理メモリ `physical_memory` から、目安と上限を決める（既定の値にした理由を `notes` に
/// 足す）
fn resolve(
    settings: Option<&Map<String, Value>>,
    physical_memory: Option<u64>,
    notes: &mut Vec<String>,
) -> ToolpathLimits {
    let default_limit = match physical_memory {
        Some(bytes) => (bytes / 2).clamp(1, MAX_DEFAULT_LIMIT_BYTES),
        None => {
            notes.push(format!(
                "the physical memory size is unknown; the default {LIMIT_KEY} is {MAX_DEFAULT_LIMIT_BYTES} bytes"
            ));
            MAX_DEFAULT_LIMIT_BYTES
        }
    };
    let limit_bytes = item_bytes(settings, LIMIT_KEY, notes).unwrap_or(default_limit);
    let default_confirm = DEFAULT_CONFIRM_BYTES.min(limit_bytes);
    let confirm_bytes = match item_bytes(settings, CONFIRM_KEY, notes) {
        Some(bytes) if bytes > limit_bytes => {
            notes.push(format!(
                "{CONFIRM_KEY} ({bytes} bytes) is greater than the limit ({limit_bytes} bytes); using the default"
            ));
            default_confirm
        }
        Some(bytes) => bytes,
        None => default_confirm,
    };
    ToolpathLimits {
        confirm_bytes,
        limit_bytes,
    }
}

/// 設定 `settings` の項目 `key`（GB の正の数）のバイトの数（ない・正しくないときは、理由を `notes` に足して `None`）
fn item_bytes(
    settings: Option<&Map<String, Value>>,
    key: &str,
    notes: &mut Vec<String>,
) -> Option<u64> {
    let Some(value) = settings.and_then(|settings| settings.get(key)) else {
        notes.push(format!("{key} is not set; using the default"));
        return None;
    };
    let bytes = value
        .as_f64()
        .map(|gb| (gb * BYTES_PER_GB).round())
        .filter(|&bytes| bytes >= 1.0 && bytes < u64::MAX as f64);
    if bytes.is_none() {
        notes.push(format!(
            "{key} must be a positive number of GB but is {value}; using the default"
        ));
    }
    bytes.map(|bytes| bytes as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一時のフォルダの設定のファイル（利用者の設定のファイルには触れない。試験の終わりに消す）
    struct TempSettings(std::path::PathBuf);

    impl TempSettings {
        fn new(name: &str) -> Self {
            let folder = std::env::temp_dir().join(format!(
                "redring-toolpath-limits-test-{name}-{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&folder).unwrap();
            Self(folder)
        }

        fn path(&self) -> std::path::PathBuf {
            self.0.join("job_manager.json")
        }

        /// 設定のファイルを `json` にして、物理メモリ `physical_memory` で読む
        fn read(&self, json: &str, physical_memory: Option<u64>) -> (ToolpathLimits, Vec<String>) {
            std::fs::write(self.path(), json).unwrap();
            read(Some(&self.path()), physical_memory)
        }
    }

    impl Drop for TempSettings {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const GB: u64 = 1_000_000_000;
    const GIB: u64 = 1 << 30;

    fn limits(confirm_bytes: u64, limit_bytes: u64) -> ToolpathLimits {
        ToolpathLimits {
            confirm_bytes,
            limit_bytes,
        }
    }

    #[test]
    fn the_settings_change_the_confirmation_and_the_limit() {
        let settings = TempSettings::new("set");
        let (limits_read, notes) = settings.read(
            r#"{"store":"D:\\jobs","toolpath_confirm_gb":1.5,"toolpath_limit_gb":24}"#,
            Some(64 * GIB),
        );
        assert_eq!(limits_read, limits(1_500_000_000, 24 * GB));
        // 既定の値にした理由はなく、決めた値だけをログに書く
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("confirm_bytes=1500000000 limit_bytes=24000000000"));
    }

    #[test]
    fn missing_items_use_the_defaults_from_the_physical_memory() {
        let settings = TempSettings::new("missing");
        // 物理メモリが 32 GiB なら、上限は 8 GB
        let (limits_read, notes) = settings.read("{}", Some(32 * GIB));
        assert_eq!(limits_read, limits(2 * GB, 8 * GB));
        assert!(
            notes
                .iter()
                .any(|note| note.contains("toolpath_confirm_gb is not set"))
        );
        assert!(
            notes
                .iter()
                .any(|note| note.contains("toolpath_limit_gb is not set"))
        );
        // 物理メモリが 8 GiB なら、上限は物理メモリの半分
        let (limits_read, _) = settings.read("{}", Some(8 * GIB));
        assert_eq!(limits_read, limits(2 * GB, 4 * GIB));
        // 物理メモリが 2 GiB なら、上限は 1 GiB で、目安は上限に合わせる
        let (limits_read, _) = settings.read("{}", Some(2 * GIB));
        assert_eq!(limits_read, limits(GIB, GIB));
        // 物理メモリが分からなければ、上限は 8 GB
        let (limits_read, notes) = settings.read("{}", None);
        assert_eq!(limits_read, limits(2 * GB, 8 * GB));
        assert!(
            notes
                .iter()
                .any(|note| note.contains("physical memory size is unknown"))
        );
        // 設定のファイルがないときも、既定の値
        std::fs::remove_file(settings.path()).unwrap();
        assert_eq!(
            read(Some(&settings.path()), Some(32 * GIB)).0,
            limits(2 * GB, 8 * GB)
        );
    }

    #[test]
    fn invalid_values_use_the_defaults_and_are_logged() {
        let settings = TempSettings::new("invalid");
        // 数でない（文字列・null・配列）・0 以下・大きすぎる
        for value in [r#""4""#, "null", "[4]", "0", "-2", "1e300"] {
            let json = format!(r#"{{"toolpath_confirm_gb":{value},"toolpath_limit_gb":{value}}}"#);
            let (limits_read, notes) = settings.read(&json, Some(32 * GIB));
            assert_eq!(limits_read, limits(2 * GB, 8 * GB), "{value}");
            for key in ["toolpath_confirm_gb", "toolpath_limit_gb"] {
                assert!(
                    notes
                        .iter()
                        .any(|note| note.starts_with(key) && note.contains("must be a positive")),
                    "{value}: {notes:?}"
                );
            }
        }
        // 目安が上限より大きいときは、目安を既定の値にする
        let (limits_read, notes) = settings.read(
            r#"{"toolpath_confirm_gb":6,"toolpath_limit_gb":4}"#,
            Some(32 * GIB),
        );
        assert_eq!(limits_read, limits(2 * GB, 4 * GB));
        assert!(
            notes
                .iter()
                .any(|note| note.contains("greater than the limit"))
        );
        // 読めない設定のファイルも、既定の値にして、理由を書く
        let (limits_read, notes) = settings.read("not json", Some(32 * GIB));
        assert_eq!(limits_read, limits(2 * GB, 8 * GB));
        assert!(
            notes
                .iter()
                .any(|note| note.contains("cannot read the settings file"))
        );
    }
}
