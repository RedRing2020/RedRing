//! ジョブマネージャーに問い合わせる口（設計: JOB_IPC_DESIGN.md）
//!
//! 利用者ごとの口（Windows では名前付きパイプ、ほかの OS では UNIX ドメインソケット）の開き方・つなぎ方
//! （[`endpoint`]）、枠の送り方（[`frame`]）、要求と応答の型（[`protocol`]）、要求を送る側（[`Client`]）と受ける側
//! （[`serve`]）を持つ。口の型は、ジョブの保存先の形式に依らない。

#[cfg(test)]
mod tests;

pub mod endpoint;
pub mod frame;
pub mod protocol;
mod session;

pub use endpoint::{Connection, Endpoint, Listener};
pub use protocol::{
    Dependency, ErrorKind, InputHeader, JobFile, JobKind, JobSource, JobStatus, JobSummary,
    PROTOCOL_VERSION, PendingSubmission, Request, Response, SubmitJob,
};
pub use session::{Client, ClientError, FileContent, Handler, HandlerError, serve};
