//! ジョブマネージャーの口（設計: JOB_IPC_DESIGN.md 3 章）
//!
//! 利用者ごとの口の名前と、同時につなげる数を決める。

use std::io;
use std::time::Duration;

pub use platform_foundation::ipc::{Connection, Listener};

/// 口に同時につなげる数の上限
const MAX_CONNECTIONS: usize = 4;
/// 口のまとまりの名前
const ENDPOINT_GROUP: &str = "RedRing";
/// ジョブマネージャーの口の名前
const ENDPOINT_NAME: &str = "JobManager";

/// 口の場所
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint(platform_foundation::ipc::Endpoint);

impl Endpoint {
    /// この利用者の、ジョブマネージャーの口
    ///
    /// # Errors
    ///
    /// 利用者を確かめられない場合（Windows）。
    pub fn for_current_user() -> io::Result<Self> {
        platform_foundation::ipc::Endpoint::for_current_user(ENDPOINT_GROUP, ENDPOINT_NAME)
            .map(Self)
    }

    /// 名前 `name`（ASCII の英数字・`-`・`_` の 64 バイトまで。ジョブマネージャーの口の名前とは別の名前）の口（試験で、
    /// 利用者の実際の口と分けるため）
    ///
    /// # Errors
    ///
    /// 名前が正しくない・口の場所が長すぎる・利用者を確かめられない場合。
    pub fn with_name(name: &str) -> io::Result<Self> {
        // `.` を使わせない（`JobManager` と大文字と小文字だけが違う名前とともに、実際の口と重ならないように）
        if name.contains('.') || name.eq_ignore_ascii_case(ENDPOINT_NAME) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid endpoint name",
            ));
        }
        platform_foundation::ipc::Endpoint::for_current_user(ENDPOINT_GROUP, name).map(Self)
    }

    /// 口を開く（同じ口を、ほかのプロセスが開いていれば開かない）
    ///
    /// # Errors
    ///
    /// ほかのプロセスが開いている（`AddrInUse`）・開けない場合。
    pub fn listen(&self) -> io::Result<Listener> {
        self.0.listen(MAX_CONNECTIONS)
    }

    /// 口につなぐ。口がない・空いていないときは、`wait` まで待ってつなぎ直す
    ///
    /// # Errors
    ///
    /// `wait` までにつなげない（口がないときは `NotFound`）・口の持ち主が自分でない（`PermissionDenied`）場合。
    pub fn connect(&self, wait: Duration) -> io::Result<Connection> {
        self.0.connect(wait)
    }
}
