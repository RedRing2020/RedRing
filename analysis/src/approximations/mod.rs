//! 近似計算モジュール

pub mod curves;
pub mod ellipse;

// 便利な再エクスポート
pub use curves::{
    bezier_length_approximation, parametric_curve_length, spline_length_approximation,
};
pub use ellipse::{
    ellipse_eccentricity, ellipse_focal_distance, ellipse_perimeter_cantrell,
    ellipse_perimeter_padé, ellipse_perimeter_ramanujan_i, ellipse_perimeter_ramanujan_ii,
};
