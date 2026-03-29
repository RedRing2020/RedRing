//! application - Application Layer の orchestration エントリポイント
//!
//! このクレートは、PoC段階における orchestration 責務の単一受け皿です。
//! 初期フェーズでは単一クレート内を module 分割し、境界安定後に必要に応じて昇格します。

pub mod cam_orchestration;
pub mod feature_orchestration;
pub mod geometry_orchestration;
pub mod job_orchestration;
