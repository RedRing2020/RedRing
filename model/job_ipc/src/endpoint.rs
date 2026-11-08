//! 利用者ごとの口（Windows では名前付きパイプ、ほかの OS では UNIX ドメインソケット。設計: JOB_IPC_DESIGN.md 3 章）
//!
//! 口を開いた側（[`Listener`]）は接続を受け、つないだ側（[`Endpoint::connect`]）は口の持ち主が自分であることを
//! 確かめてから使う。接続（[`Connection`]）は、読み書きできるバイトの流れである。

use std::io;
use std::time::{Duration, Instant};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as platform;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as platform;

/// 口に同時につなげる数の上限
pub const MAX_CONNECTIONS: usize = 4;
/// つなぎ直しを待つ間隔
const RETRY_INTERVAL: Duration = Duration::from_millis(50);

/// 口の場所
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint(platform::Address);

impl Endpoint {
    /// この利用者の、ジョブマネージャーの口
    ///
    /// # Errors
    ///
    /// 利用者を確かめられない（Windows）・利用者ごとの実行時のフォルダがない（ほかの OS）場合。
    pub fn for_current_user() -> io::Result<Self> {
        platform::Address::for_current_user().map(Self)
    }

    /// 名前 `name`（ASCII の英数字・`-`・`_`）の口（試験で、利用者の実際の口と分けるため）
    ///
    /// # Errors
    ///
    /// 名前が正しくない・利用者を確かめられない場合。
    pub fn with_name(name: &str) -> io::Result<Self> {
        let plain = !name.is_empty()
            && name.len() <= 64
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
        if !plain {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid endpoint name",
            ));
        }
        platform::Address::with_name(name).map(Self)
    }

    /// 口を開く（同じ口を、ほかのプロセスが開いていれば開かない）
    ///
    /// # Errors
    ///
    /// ほかのプロセスが開いている（`AddrInUse`）・開けない場合。
    pub fn listen(&self) -> io::Result<Listener> {
        platform::Listener::bind(&self.0).map(Listener)
    }

    /// 口につなぐ。口がない・空いていないときは、`wait` まで待ってつなぎ直す
    ///
    /// # Errors
    ///
    /// `wait` までにつなげない（口がないときは `NotFound`）・口の持ち主が自分でない（`PermissionDenied`）場合。
    pub fn connect(&self, wait: Duration) -> io::Result<Connection> {
        let deadline = Instant::now() + wait;
        loop {
            match platform::connect(&self.0) {
                Ok(connection) => return Ok(Connection(connection)),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::NotFound
                            | io::ErrorKind::ConnectionRefused
                            | io::ErrorKind::ResourceBusy
                    ) && Instant::now() < deadline =>
                {
                    std::thread::sleep(RETRY_INTERVAL);
                }
                Err(error) => return Err(error),
            }
        }
    }
}

/// 開いた口（接続を受ける）
#[derive(Debug)]
pub struct Listener(platform::Listener);

impl Listener {
    /// 次の接続を待って受ける（同時につないでいる数が上限のときは、空くまで待つ）
    ///
    /// # Errors
    ///
    /// 接続を受けられない場合。
    pub fn accept(&mut self) -> io::Result<Connection> {
        self.0.accept().map(Connection)
    }
}

/// 口の接続（読み書きできるバイトの流れ）
#[derive(Debug)]
pub struct Connection(platform::Stream);

impl Connection {
    /// 口を開いた側のプロセスに、ウィンドウを前に出す権利を渡す（Windows。利用者が操作しているプロセスだけが渡せる。
    /// ほかの OS では何もしない）
    pub fn allow_server_foreground(&self) {
        platform::allow_server_foreground(&self.0);
    }
}

impl io::Read for Connection {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.0.read(buffer)
    }
}

impl io::Write for Connection {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
