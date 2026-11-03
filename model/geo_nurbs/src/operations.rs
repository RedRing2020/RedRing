//! NURBS編集操作モジュール
//!
//! NURBS曲線・曲面の形状編集操作を提供します。
//! これらは座標変換ではなく、NURBSの内部構造（制御点、ノット）を操作します。
//!
//! ## 主要機能
//! - **`knot_insertion`**: ノット挿入（曲線の形状を保ったまま制御点を増やす）

mod knot_insertion;

pub use knot_insertion::{KnotInsertResult2D, KnotInsertResult3D, KnotInsertion};
