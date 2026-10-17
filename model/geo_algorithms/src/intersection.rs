//! 交差の実装
//!
//! 形状の組み合わせごとの関数を置き、`operations::CrossIntersection` の実装から呼ぶ。
//! `common` と `pair_base` は基礎計算を担う。

pub(crate) mod common;
pub(crate) mod pair_base;
pub(crate) mod primitive_2d;
pub(crate) mod primitive_3d;
