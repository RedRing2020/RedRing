//! geo_primitives の数値計算の定数

/// 楕円弧の長さの数値積分で、区間を 2 等分する最大の深さ
pub(crate) const ELLIPSE_ARC_LENGTH_MAX_BISECTION_DEPTH: usize = 20;
