//! 利用者ごとの口（Windows では名前付きパイプ、ほかの OS では UNIX ドメインソケット）
//!
//! 口は、同じ PC の同じ利用者だけが使える。口を開いた側（[`Listener`]）は接続を受け、つないだ側
//! （[`Endpoint::connect`]）は口の持ち主が自分であることを確かめてから使う。接続（[`Connection`]）は、読み書きできる
//! バイトの流れである。

use std::io;
use std::time::{Duration, Instant};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use windows as platform;

#[cfg(test)]
mod tests;

/// つなぎ直しを待つ間隔
const RETRY_INTERVAL: Duration = Duration::from_millis(50);
/// 口の名前の 1 つの部分の長さの上限
const MAX_NAME_PART_BYTES: usize = 64;
/// 同時につなげる数の上限の、指定できる最大（Windows のパイプのインスタンスの数の上限 254 から、次の接続を待つ
/// インスタンスの分を除く）
const MAX_CONNECTIONS_LIMIT: usize = 253;

/// 口の場所
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint(platform::Address);

impl Endpoint {
    /// まとまり `group` の、名前 `name` の、この利用者の口
    ///
    /// Windows では `\\.\pipe\<group>.<name>.<利用者の SID>`、ほかの OS では、利用者ごとの実行時のフォルダの下の
    /// `<group>/<name>.sock`（ない場合は、一時のフォルダの下の `<group>-<利用者の番号>/<name>.sock`）。`group` と
    /// `name` は、ASCII の英数字・`-`・`_` の 64 バイトまでの名前とする（`name` には、先頭でなければ `.` も使える。
    /// Windows のパイプの名前で、まとまりと名前の境目が 1 つに決まるように、`group` には使えない）。Windows のパイプの
    /// 名前は大文字と小文字を分けないため、大文字と小文字だけが違う名前は、同じ口になる。
    ///
    /// # Errors
    ///
    /// 名前が正しくない・ソケットのパスが長すぎる（Windows 以外）・利用者を確かめられない場合。
    pub fn for_current_user(group: &str, name: &str) -> io::Result<Self> {
        if !is_plain_name(group) || group.contains('.') || !is_plain_name(name) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid endpoint name",
            ));
        }
        platform::Address::for_current_user(group, name).map(Self)
    }

    /// 口を開く。同時につなげる数を `max_connections`（1 から 253 まで）までにする（同じ口を、
    /// ほかのプロセスが開いていれば開かない）
    ///
    /// # Errors
    ///
    /// 同時につなげる数が範囲の外（`InvalidInput`）・ほかのプロセスが開いている（`AddrInUse`）・開けない場合。
    pub fn listen(&self, max_connections: usize) -> io::Result<Listener> {
        if !(1..=MAX_CONNECTIONS_LIMIT).contains(&max_connections) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the number of connections is out of range",
            ));
        }
        platform::Listener::bind(&self.0, max_connections).map(Listener)
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

/// 口の名前の 1 つの部分に使える名前か
fn is_plain_name(part: &str) -> bool {
    !part.is_empty()
        && part.len() <= MAX_NAME_PART_BYTES
        && !part.starts_with('.')
        && part
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

/// 開いた口（接続を受ける）
#[derive(Debug)]
pub struct Listener(platform::Listener);

impl Listener {
    /// 次の接続を待って受ける（同時につないでいる数が上限のときは、Windows では空くまで待ってから受け、ほかの OS
    /// では受けてすぐに閉じる。つないだ側は、待ち続けずに接続が閉じたと分かる）
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
    /// つないだ側の接続の、読み書き 1 回ごとの時間の上限（`None` は待ち続ける。上限を超えたら `TimedOut`）
    ///
    /// # Errors
    ///
    /// 上限が 0・接続を受けた側の接続（Windows）・設定できない場合。
    pub fn set_io_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        if timeout == Some(Duration::ZERO) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the timeout must not be zero",
            ));
        }
        self.0.set_io_timeout(timeout)
    }

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
