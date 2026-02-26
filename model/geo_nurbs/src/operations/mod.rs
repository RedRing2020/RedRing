//! NURBS編集操作モジュール
//!
//! NURBS曲線・曲面の形状編集操作を提供します。
//! これらは座標変換ではなく、NURBSの内部構造（制御点、ノット、次数）を操作します。
//!
//! ## 主要機能
//! - **`knot_insertion`**: ノット挿入（曲線の形状を保ったまま制御点を増やす）
//! - **`degree_elevation`**: 次数上昇（より滑らかな表現への変換）
//! - **`splitting`**: 曲線分割（指定パラメータで曲線を2つに分割）

pub mod degree_elevation;
pub mod knot_insertion;
pub mod splitting;

// 公開API
pub use degree_elevation::DegreeElevation;
pub use knot_insertion::KnotInsertion;
pub use splitting::CurveSplitting;
