//! UNIX ドメインソケットの口（設計: JOB_IPC_DESIGN.md 3.2）
//!
//! ソケットは、利用者ごとの実行時のフォルダ（`$XDG_RUNTIME_DIR`）の下の、ほかの利用者が入れないフォルダに置く。
//! `XDG_RUNTIME_DIR` がなければ（macOS など）、一時のフォルダ（macOS では利用者ごと）の下に、利用者の番号の名前の
//! フォルダを作る。開く側・つなぐ側とも、フォルダとソケットの持ち主が自分（利用者の番号）であり、フォルダにほかの
//! 利用者が入れないことを確かめる（共有の一時のフォルダに、ほかの利用者が先に同じ名前のフォルダ・リンクを作っても
//! 使わない）。

use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::MAX_CONNECTIONS;

/// ソケットのパス
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Address(PathBuf);

impl Address {
    pub(super) fn for_current_user() -> io::Result<Self> {
        // 指していても、ないフォルダは使わない（`su` の後・コンテナなどで、前の利用者の値が残ることがある）
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_dir());
        Ok(Self(
            socket_folder(runtime, &std::env::temp_dir(), current_uid()).join("job-manager.sock"),
        ))
    }

    pub(super) fn with_name(name: &str) -> io::Result<Self> {
        Ok(Self(
            std::env::temp_dir()
                .join(format!("redring-{}-ipc", current_uid()))
                .join(format!("{name}.sock")),
        ))
    }

    fn folder(&self) -> io::Result<&Path> {
        self.0
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no socket folder"))
    }
}

/// ソケットを置くフォルダ（利用者ごとの実行時のフォルダ `runtime` があればその下、なければ一時のフォルダ
/// `temporary` の下の、利用者の番号 `uid` の名前のフォルダ）
fn socket_folder(runtime: Option<PathBuf>, temporary: &Path, uid: u32) -> PathBuf {
    match runtime {
        Some(runtime) => runtime.join("redring"),
        None => temporary.join(format!("redring-{uid}")),
    }
}

/// この利用者の番号
fn current_uid() -> u32 {
    // SAFETY: 引数を持たず、失敗しない
    unsafe { libc::getuid() }
}

/// 使えないソケット・フォルダの誤り
fn denied(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason.to_string())
}

/// ソケットのフォルダが、自分のもので、ほかの利用者の入れないフォルダか（リンクはたどらない）
fn check_private_folder(folder: &Path) -> io::Result<()> {
    let metadata = std::fs::symlink_metadata(folder)?;
    if !metadata.is_dir() {
        return Err(denied("the socket folder is a link or not a folder"));
    }
    if metadata.uid() != current_uid() {
        return Err(denied("the socket folder is owned by another user"));
    }
    if metadata.mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the socket folder is open to other users",
        ));
    }
    Ok(())
}

/// 同時につないでいる数
type Slots = Arc<AtomicUsize>;

/// 受けた接続（閉じたら、同時につないでいる数を減らす）
#[derive(Debug)]
pub(super) struct Stream {
    stream: UnixStream,
    slots: Option<Slots>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        if let Some(slots) = &self.slots {
            slots.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

impl Stream {
    pub(super) fn set_io_timeout(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> io::Result<()> {
        self.stream.set_read_timeout(timeout)?;
        self.stream.set_write_timeout(timeout)
    }
}

/// ソケットの時間の上限を超えた誤り（`WouldBlock`）を、Windows と同じ `TimedOut` にする
fn timed_out<T>(result: io::Result<T>) -> io::Result<T> {
    result.map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            io::Error::new(io::ErrorKind::TimedOut, error)
        } else {
            error
        }
    })
}

impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        timed_out(self.stream.read(buffer))
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        timed_out(self.stream.write(buffer))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

/// 開いたソケット（閉じたら、ソケットのファイルを消す）
#[derive(Debug)]
pub(super) struct Listener {
    listener: UnixListener,
    path: PathBuf,
    slots: Slots,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl Listener {
    pub(super) fn bind(address: &Address) -> io::Result<Self> {
        let folder = address.folder()?;
        // フォルダだけを作る（親のフォルダは作らない。あれば、自分のものかを確かめる）
        match std::fs::DirBuilder::new().mode(0o700).create(folder) {
            Err(error) if error.kind() != io::ErrorKind::AlreadyExists => return Err(error),
            _ => {}
        }
        check_private_folder(folder)?;
        if address.0.exists() {
            // 動いている口があれば開かない。残っているだけのソケットのファイルは消す
            if UnixStream::connect(&address.0).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "the job manager endpoint is already open",
                ));
            }
            std::fs::remove_file(&address.0)?;
        }
        let listener = UnixListener::bind(&address.0)?;
        std::fs::set_permissions(&address.0, std::fs::Permissions::from_mode(0o600))?;
        Ok(Self {
            listener,
            path: address.0.clone(),
            slots: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// 接続を待って受ける（同時につないでいる数が上限のときは、受けてすぐに閉じる。つないだ側は、待ち続けずに
    /// 接続が閉じたと分かる）
    pub(super) fn accept(&mut self) -> io::Result<Stream> {
        loop {
            let (stream, _) = self.listener.accept()?;
            if self.slots.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
                drop(stream);
                continue;
            }
            self.slots.fetch_add(1, Ordering::SeqCst);
            return Ok(Stream {
                stream,
                slots: Some(Arc::clone(&self.slots)),
            });
        }
    }
}

pub(super) fn allow_server_foreground(_stream: &Stream) {}

pub(super) fn connect(address: &Address) -> io::Result<Stream> {
    check_private_folder(address.folder()?)?;
    let socket = std::fs::symlink_metadata(&address.0)?;
    if socket.uid() != current_uid() {
        return Err(denied("the job manager endpoint is owned by another user"));
    }
    Ok(Stream {
        stream: UnixStream::connect(&address.0)?,
        slots: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_socket_folder_is_in_the_runtime_folder_or_a_per_user_temporary_folder() {
        let temporary = Path::new("/tmp");
        assert_eq!(
            socket_folder(Some(PathBuf::from("/run/user/1000")), temporary, 1000),
            Path::new("/run/user/1000/redring")
        );
        assert_eq!(
            socket_folder(None, temporary, 1000),
            Path::new("/tmp/redring-1000")
        );
    }

    #[test]
    fn folders_of_other_users_or_open_folders_are_not_used() {
        let folder = std::env::temp_dir().join(format!(
            "redring-{}-folder-test-{}",
            current_uid(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&folder)
            .unwrap();
        assert!(check_private_folder(&folder).is_ok());
        // ほかの利用者が入れるフォルダは使わない
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(check_private_folder(&folder).is_err());
        // リンクはたどらない
        let link = folder.with_extension("link");
        let _ = std::fs::remove_file(&link);
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::os::unix::fs::symlink(&folder, &link).unwrap();
        assert!(check_private_folder(&link).is_err());
        std::fs::remove_file(&link).unwrap();
        // 共有の一時のフォルダ（ほかの利用者が入れる・リンク）は使わない
        assert!(check_private_folder(Path::new("/tmp")).is_err());
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
