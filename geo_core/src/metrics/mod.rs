//! 幾何学的計量計算モジュール
//!
//! 距離、長さ、面積、体積などの幾何学的な測定値を計算する機能を提供します。

pub mod area_volume;
pub mod distance;
pub mod ellipse;
pub mod length;

#[cfg(test)]
mod area_volume_tests;
#[cfg(test)]
mod length_tests;

// 便利な再エクスポート（geometry専用機能のみ）
pub use area_volume::{
    box_volume, circle_area, cone_surface_area, cone_volume, cylinder_surface_area,
    cylinder_volume, polygon_area, rectangle_area, sphere_surface_area, sphere_volume,
    trapezoid_area, triangle_area_2d, triangle_area_heron,
};
pub use ellipse::{
    ellipse_area_f64, ellipse_circumference_numerical_f64, ellipse_circumference_ramanujan_f64,
    ellipse_circumference_series_f64, ellipse_eccentricity_f64, ellipse_focal_distance_f64,
    ellipse_foci_f64,
};
pub use length::{arc_length, ellipse_arc_length_approximation};
