//! ファイルの置き換えと名前の変更（電源が切れても・ほかのプログラムが一瞬開いていても、古い版か新しい版のどちらかを
//! 残す）

use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;

#[cfg(not(windows))]
mod other;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
use other as platform;
#[cfg(windows)]
use windows as platform;

#[cfg(test)]
mod tests;

/// ファイル `path` を、`write` で中身を書いた一時のファイル `temporary` で置き換える
///
/// 一時のファイルに書き、ドライブまで書き込んでから（`sync_all`）、名前を変えて置き換える。電源が切れても、`path`
/// は古い版か新しい版のどちらかになる。書けなかった・置き換えられなかったときは、一時のファイルを消し、`path` は
/// そのまま残す。`temporary` は `path` と同じフォルダに置く（名前の変更で置き換えるため）。すでにあれば上書きする。
///
/// 名前の変更は [`rename`] で行う（Windows では、一時的な拒否を試し直す）。ほかの OS では、置き換えた後にフォルダも
/// ドライブまで書き込む。Windows では、名前の変更の記録を NTFS のジャーナルに任せ、フォルダは書き込まない。
///
/// # Errors
///
/// 一時のファイルを作れない・書けない・ドライブまで書き込めない場合、`write` の誤り、置き換えられない場合。
pub fn replace_file<E: From<io::Error>>(
    path: &Path,
    temporary: &Path,
    write: impl FnOnce(&mut File) -> Result<(), E>,
) -> Result<(), E> {
    let result =
        write_synced(temporary, write).and_then(|()| rename(temporary, path).map_err(E::from));
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
        return result;
    }
    platform::sync_parent(path);
    Ok(())
}

fn write_synced<E: From<io::Error>>(
    temporary: &Path,
    write: impl FnOnce(&mut File) -> Result<(), E>,
) -> Result<(), E> {
    let mut file = File::create(temporary)?;
    write(&mut file)?;
    file.sync_all()?;
    Ok(())
}

/// ファイル・フォルダ `from` の名前を `to` に変える（`to` のファイルがあれば置き換える）
///
/// Windows では、「アクセス拒否」「共有違反」で断られたら（ウイルス対策ソフト・検索の索引が、書いたばかりのファイル
/// を開いていることがある）、間を延ばしながら合わせて約 2 秒まで試し直す。`to` がすでにあるフォルダか、読み取り専用
/// のファイルのときは、試し直しても通らないため、すぐに誤りにする。ほかの OS では、試し直さない（開いているファイルでも名前を
/// 変えられるため）。
///
/// # Errors
///
/// 名前を変えられない場合（試し直した後の、最後の誤り）。
pub fn rename(from: &Path, to: &Path) -> io::Result<()> {
    rename_with_retry(
        || std::fs::rename(from, to),
        |error| platform::is_transient_refusal(error) && !refused_for_good(to),
        std::thread::sleep,
    )
}

/// 名前の変更の先 `to` が、試し直しても置き換えられないもの（すでにあるフォルダ・読み取り専用のファイル）か
fn refused_for_good(to: &Path) -> bool {
    std::fs::metadata(to)
        .is_ok_and(|metadata| metadata.is_dir() || metadata.permissions().readonly())
}

/// 名前の変更を断られたときに、次に試すまでの間（ミリ秒。倍に延ばし、その後は 250 ずつ。合わせて 2005）
const RENAME_WAITS_MS: [u64; 15] = [
    1, 2, 4, 8, 16, 32, 64, 128, 250, 250, 250, 250, 250, 250, 250,
];

/// `rename` を、一時的な拒否（`transient`）の間、`RENAME_WAITS_MS` の間を `wait` で置いて試し直す。最後の結果を返す
fn rename_with_retry(
    mut rename: impl FnMut() -> io::Result<()>,
    transient: impl Fn(&io::Error) -> bool,
    mut wait: impl FnMut(Duration),
) -> io::Result<()> {
    let mut result = rename();
    for milliseconds in RENAME_WAITS_MS {
        match &result {
            Err(error) if transient(error) => {}
            _ => return result,
        }
        wait(Duration::from_millis(milliseconds));
        result = rename();
    }
    result
}
