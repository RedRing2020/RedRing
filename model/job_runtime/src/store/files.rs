//! 保存先のファイルの読み書き（書きかけを読ませない書き方・大きさの上限つきの読み込み・名前の検証）

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// 書きかけのファイル・フォルダの名前の接尾辞
pub(crate) const TEMPORARY_SUFFIX: &str = ".tmp";

/// 一時の名前（`<名前>.tmp`）で書き終えてから、正しい名前に変える
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = temporary_path(path);
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)
}

/// `path` の名前に `.tmp` を足したパス
pub(crate) fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(TEMPORARY_SUFFIX);
    path.with_file_name(name)
}

/// 書きかけの名前か
pub(crate) fn is_temporary(name: &str) -> bool {
    name.ends_with(TEMPORARY_SUFFIX)
}

/// 最大 `limit` バイトまで読む。超えるファイルは誤り
pub(crate) fn read_limited(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{} is larger than {limit} bytes", path.display()),
        ));
    }
    Ok(bytes)
}

/// `source` を `destination` に写す。`limit` バイトを超えるファイルは誤り（開いたファイルから読んだ量で確かめる）
pub(crate) fn copy_limited(source: &Path, destination: &Path, limit: u64) -> std::io::Result<()> {
    let mut reader = fs::File::open(source)?.take(limit + 1);
    let mut writer = fs::File::create(destination)?;
    let copied = std::io::copy(&mut reader, &mut writer)?;
    if copied > limit {
        drop(writer);
        let _ = fs::remove_file(destination);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{} is larger than {limit} bytes", source.display()),
        ));
    }
    writer.sync_all()
}

/// フォルダの中の書きかけのもの（`*.tmp` のファイル・フォルダ）を消す。`older_than` を指定したときは、更新から
/// その時間を過ぎたものだけを消す
pub(crate) fn remove_temporary_entries(
    dir: &Path,
    older_than: Option<Duration>,
) -> std::io::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_name().to_str().is_some_and(is_temporary) {
            continue;
        }
        if let Some(age) = older_than {
            let modified = entry.metadata()?.modified()?;
            // 更新の時刻が今より後のもの（時計の違い）も、新しいものとして残す
            let recent = SystemTime::now()
                .duration_since(modified)
                .map_or(true, |elapsed| elapsed < age);
            if recent {
                continue;
            }
        }
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

/// ファイル・フォルダの名前に使える値か（ASCII の英数字・`-`・`_`・`.` だけで、`.` で始まらず、`max_len` 以下）
///
/// パスの区切り・`..`・ドライブの指定を含む値を拒み、名前から作ったパスがフォルダの外を指さないようにする。
pub(crate) fn is_plain_name(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && !value.starts_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}
