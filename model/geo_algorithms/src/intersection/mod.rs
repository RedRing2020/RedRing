//! 交点計算アルゴリズム
//!
//! Phase C で `geo_primitives` 側の複数形状間交点計算ロジックを
//! このモジュール配下へ集約した。
//! pair_base と primitive_* モジュールが現行の正本である。

pub mod nurbs_3d;
pub mod pair_base;
pub mod primitive_2d;
pub mod primitive_3d;

pub use nurbs_3d::*;
pub use primitive_2d::*;
pub use primitive_3d::*;
