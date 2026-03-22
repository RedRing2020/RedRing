//! 距離計算アルゴリズム
//!
//! `geo_primitives` の各形状が持つ距離計算実装を
//! `geo_algorithms` 層の公開 entrypoint として集約するモジュール。
//!
//! - 各関数は形状と点の組み合わせを受け取り、最短距離を返す
//! - 命名規則: `{shape_a}_{shape_b}_distance`

pub mod primitive_2d;
pub mod primitive_3d;

pub use primitive_2d::*;
pub use primitive_3d::*;
