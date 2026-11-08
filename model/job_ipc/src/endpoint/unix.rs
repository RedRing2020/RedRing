//! UNIX ドメインソケットの口（設計: JOB_IPC_DESIGN.md 3.2）
//!
//! ソケットは、利用者ごとの実行時のフォルダ（`$XDG_RUNTIME_DIR`）の下の、ほかの利用者が入れないフォルダに置く。
//! つなぐ側は、ソケットの持ち主がフォルダの持ち主と同じであることを確かめる。

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
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "XDG_RUNTIME_DIR is not set to an absolute path",
                )
            })?;
        Ok(Self(runtime.join("redring").join("job-manager.sock")))
    }

    pub(super) fn with_name(name: &str) -> io::Result<Self> {
        Ok(Self(
            std::env::temp_dir()
                .join("redring-job-ipc")
                .join(format!("{name}.sock")),
        ))
    }

    fn folder(&self) -> io::Result<&Path> {
        self.0
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no socket folder"))
    }
}

/// ソケットのフォルダが、ほかの利用者の入れないフォルダか
fn check_private_folder(folder: &Path) -> io::Result<std::fs::Metadata> {
    let metadata = std::fs::metadata(folder)?;
    if metadata.mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the socket folder is open to other users",
        ));
    }
    Ok(metadata)
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

impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buffer)
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.stream.write(buffer)
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
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(folder)?;
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

pub(super) fn connect(address: &Address) -> io::Result<Stream> {
    let folder = check_private_folder(address.folder()?)?;
    let socket = std::fs::metadata(&address.0)?;
    if socket.uid() != folder.uid() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the job manager endpoint is owned by another user",
        ));
    }
    Ok(Stream {
        stream: UnixStream::connect(&address.0)?,
        slots: None,
    })
}
