//! application - Application Layer の orchestration エントリポイント
//!
//! このクレートは、PoC段階における orchestration 責務の単一受け皿です。
//! 初期フェーズでは単一クレート内を module 分割し、境界安定後に必要に応じて昇格します。

pub extern crate geo_primitives as primitives;

/// 時間のかかる処理を別のスレッドで行い、進み具合と結果を待たずに受け取る仕組み
pub use command_framework::background_task;
/// コマンドの宣言と、実行中のコマンドの最中に来たコマンドの受け付け方を決める仕組み
pub use command_framework::command;

mod binary_section;
pub mod cam_simulation_workflow;
pub mod cutting_simulation_playback;
pub mod feature_orchestration;
pub mod geometry_orchestration;
pub mod imported_toolpath;
mod job_client;
pub mod job_orchestration;
pub mod machining_sequence;
pub mod product_shape;
pub mod space_model_file;
pub mod tool_library;
pub mod toolpath;
pub mod topology_orchestration;
