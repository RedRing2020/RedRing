//! 複数の PC から共有するツールライブラリのファイルへの書き込み（設計: TOOL_LIBRARY_DESIGN.md 4.4）
//!
//! 書くときは、ファイルの横のロックのファイル（`<ファイル名>.lock`）を、ないときだけ作る形で作って書き手を 1 つに
//! 絞り、ファイルを読み直して操作を 1 つだけ当て、一時のファイルに書いてから名前を替え、ロックのファイルを消す。
//! 読めないファイルは上書きせず、書けないファイルには書かない。

use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{
    ToolLibrary, ToolLibraryFileError, ToolLibrarySaveInfo, load_tool_library, save_tool_library,
    valid_tool,
};
use crate::machining_sequence::{ToolSetEntry, ToolSetId};

/// ロックのファイルがあるときに、作り直しを試す回数
const LOCK_ATTEMPTS: u32 = 20;
/// ロックのファイルの作り直しを試す間
const LOCK_RETRY_WAIT: Duration = Duration::from_millis(50);
/// ロックのファイルを古いとみなす時間（書いている PC が止まって残ったもの）
const STALE_LOCK_AGE_MS: u64 = 5 * 60 * 1000;
/// 読むロックのファイルの大きさの上限
const MAX_LOCK_FILE_BYTES: u64 = 4096;

/// ツールライブラリへの操作（1 回の書き込みで 1 つだけ当てる）
#[derive(Debug, Clone, PartialEq)]
pub enum ToolLibraryEdit {
    /// ライブラリの名前を替える
    Rename(String),
    /// ツールセットを最後に足す（同じ ID があれば、その項目を替える）
    AddToolSet(ToolSetEntry),
    /// 同じ ID のツールセットを替える
    UpdateToolSet(ToolSetEntry),
    /// ツールセットを消す
    RemoveToolSet(ToolSetId),
    /// ツールセットを `index` の位置（範囲を超えれば最後）に動かす
    MoveToolSet { id: ToolSetId, index: usize },
}

/// ロックのファイルの中身（書いている PC と時刻）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolLibraryLock {
    /// 書いている PC・利用者（書き手が渡した文字列。読めなければ空）
    pub owner: String,
    /// ロックのファイルを作った時刻（UNIX 時刻のミリ秒。読めなければ `None`）
    pub locked_at_ms: Option<u64>,
}

impl ToolLibraryLock {
    /// 時刻 `now_ms`（UNIX 時刻のミリ秒）で、古いとみなすロックか（時刻の読めないロックも古いとみなす）
    pub fn is_stale(&self, now_ms: u64) -> bool {
        self.locked_at_ms
            .is_none_or(|locked_at| now_ms.saturating_sub(locked_at) >= STALE_LOCK_AGE_MS)
    }
}

/// 共有のツールライブラリへの書き込みの誤り
#[derive(Debug)]
pub enum ToolLibraryEditError {
    /// ほかの PC が書いている（ロックのファイルがある）
    Locked(ToolLibraryLock),
    /// 書けない（読むだけの権限・読むだけの共有）
    ReadOnly(io::Error),
    /// ファイルを読めない（壊れている・つながらないなど。上書きしない）・書けない
    File(ToolLibraryFileError),
    /// 読み直した内容に、操作の対象のツールセットがない（ほかの PC が消した）
    ToolSetNotFound(ToolSetId),
    /// 操作の値が正しくない（空の名前・妥当でない工具。項目の名前）
    InvalidEdit(&'static str),
    /// 一覧の中で使えないライブラリ（一覧にない・読めない・ライブラリの ID が重なる）
    Unavailable,
}

impl fmt::Display for ToolLibraryEditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Locked(lock) => write!(f, "the tool library is locked by {:?}", lock.owner),
            Self::ReadOnly(error) => write!(f, "the tool library is read-only: {error}"),
            Self::File(error) => write!(f, "{error}"),
            Self::ToolSetNotFound(id) => {
                write!(f, "the tool set {} is not in the library", id.as_uuid())
            }
            Self::InvalidEdit(field) => write!(f, "invalid tool library edit: {field}"),
            Self::Unavailable => write!(f, "the tool library is not available"),
        }
    }
}

impl std::error::Error for ToolLibraryEditError {}

impl From<ToolLibraryFileError> for ToolLibraryEditError {
    fn from(error: ToolLibraryFileError) -> Self {
        Self::File(error)
    }
}

/// ライブラリ `library` に操作 `edit` を当てる
pub(super) fn apply_edit(
    library: &mut ToolLibrary,
    edit: ToolLibraryEdit,
) -> Result<(), ToolLibraryEditError> {
    let position = |library: &ToolLibrary, id: ToolSetId| {
        library
            .toolsets
            .iter()
            .position(|entry| entry.id == id)
            .ok_or(ToolLibraryEditError::ToolSetNotFound(id))
    };
    let check = |entry: &ToolSetEntry| {
        if entry.name.is_empty() {
            Err(ToolLibraryEditError::InvalidEdit("toolset.name"))
        } else if !valid_tool(&entry.tool) {
            Err(ToolLibraryEditError::InvalidEdit("toolset.tool"))
        } else {
            Ok(())
        }
    };
    match edit {
        ToolLibraryEdit::Rename(name) => {
            if name.is_empty() {
                return Err(ToolLibraryEditError::InvalidEdit("library.name"));
            }
            library.name = name;
        }
        ToolLibraryEdit::AddToolSet(entry) => {
            check(&entry)?;
            match position(library, entry.id) {
                Ok(at) => library.toolsets[at] = entry,
                Err(_) => library.toolsets.push(entry),
            }
        }
        ToolLibraryEdit::UpdateToolSet(entry) => {
            check(&entry)?;
            let at = position(library, entry.id)?;
            library.toolsets[at] = entry;
        }
        ToolLibraryEdit::RemoveToolSet(id) => {
            let at = position(library, id)?;
            library.toolsets.remove(at);
        }
        ToolLibraryEdit::MoveToolSet { id, index } => {
            let at = position(library, id)?;
            let entry = library.toolsets.remove(at);
            let index = index.min(library.toolsets.len());
            library.toolsets.insert(index, entry);
        }
    }
    Ok(())
}

/// ツールライブラリのファイル `path` のロックのファイルの場所
fn lock_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".lock");
    path.with_file_name(name)
}

/// 今の時刻（UNIX 時刻のミリ秒）
pub(super) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

/// 書き終えたら（誤りで止めても）、ロックのファイルを消す
pub(super) struct LockGuard(PathBuf);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// ロックのファイルを作る（あれば、試し直してもなければ [`ToolLibraryEditError::Locked`]）
pub(super) fn acquire_lock(path: &Path, owner: &str) -> Result<LockGuard, ToolLibraryEditError> {
    let lock = lock_path(path);
    for attempt in 0..LOCK_ATTEMPTS {
        match OpenOptions::new().write(true).create_new(true).open(&lock) {
            Ok(mut file) => {
                let guard = LockGuard(lock);
                // 中身は書いている PC の見分けのため。書けなくてもロックは効く。1 行目を書き手、2 行目を時刻とする
                // ため、書き手の改行は空白にする
                let owner = owner.replace(['\r', '\n'], " ");
                let _ = writeln!(file, "{owner}\n{}", now_ms());
                return Ok(guard);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                if attempt + 1 < LOCK_ATTEMPTS {
                    std::thread::sleep(LOCK_RETRY_WAIT);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                return Err(ToolLibraryEditError::ReadOnly(error));
            }
            Err(error) => return Err(ToolLibraryFileError::Io(error).into()),
        }
    }
    let lock = tool_library_lock(path).unwrap_or(ToolLibraryLock {
        owner: String::new(),
        locked_at_ms: None,
    });
    Err(ToolLibraryEditError::Locked(lock))
}

/// ツールライブラリのファイル `path` のロックのファイルを読む（なければ `None`）
pub fn tool_library_lock(path: &Path) -> Option<ToolLibraryLock> {
    let file = fs::File::open(lock_path(path)).ok()?;
    let mut bytes = Vec::new();
    // 書いている途中・壊れた中身でも、ロックがあることは返す
    let _ = file.take(MAX_LOCK_FILE_BYTES).read_to_end(&mut bytes);
    let text = String::from_utf8_lossy(&bytes);
    let mut lines = text.lines();
    let owner = lines.next().unwrap_or_default().to_string();
    let locked_at_ms = lines.next().and_then(|line| line.trim().parse().ok());
    Some(ToolLibraryLock {
        owner,
        locked_at_ms,
    })
}

/// ツールライブラリのファイル `path` のロックのファイルを消す（書いている PC が止まって残った古いロックを、利用者に
/// 確かめてから消すときに使う）
///
/// # Errors
///
/// ロックのファイルを消せない場合（ない場合を含む）。
pub fn remove_tool_library_lock(path: &Path) -> io::Result<()> {
    fs::remove_file(lock_path(path))
}

/// ファイル `path` に書けるか（書く形で開けるか。中身は変えない）
pub(super) fn writable(path: &Path) -> io::Result<()> {
    OpenOptions::new().write(true).open(path).map(drop)
}

/// 共有のツールライブラリのファイル `path` に操作 `edit` を当てて書き、書いた内容を返す
///
/// ロックのファイル（中身は書き手 `owner` と時刻）で書き手を 1 つに絞り、ファイルを読み直した内容に `edit` を当て、
/// 保存したときの情報 `info` とともに書く。ロックのファイルは、書き終えたら（誤りでも）消す。
///
/// # Errors
///
/// ほかの PC が書いている場合、書けない場合、ファイルを読めない場合（上書きしない）・書けない場合、操作の対象の
/// ツールセットがない場合、操作の値が正しくない場合。
pub fn edit_tool_library(
    path: &Path,
    edit: ToolLibraryEdit,
    owner: &str,
    info: &ToolLibrarySaveInfo,
) -> Result<ToolLibrary, ToolLibraryEditError> {
    match writable(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            return Err(ToolLibraryEditError::ReadOnly(error));
        }
        Err(error) => return Err(ToolLibraryFileError::Io(error).into()),
    }
    let _lock = acquire_lock(path, owner)?;
    let (mut library, _) = load_tool_library(path)?;
    apply_edit(&mut library, edit)?;
    save_tool_library(path, &library, info)?;
    Ok(library)
}
