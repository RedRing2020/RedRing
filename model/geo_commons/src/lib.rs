//! geo_commons - 共通幾何計算機能クレート
//!
//! Foundation Pattern と geo_primitives の間で共通利用される
//! 計算機能を提供します。
//!
//! ## 主要機能
//! - **approximations**: 楕円等の近似計算
//! - **metrics**: 面積・体積・距離等の計量計算
//!
//! ## アーキテクチャでの位置
//! ```text
//! geo_foundation → geo_commons → geo_primitives
//! ```
//!
//! ---
//! © RedRing Project

pub mod approximations;
pub mod metrics;

// 便利な再エクスポート - 名前衝突回避のため明示的にエクスポート
pub use approximations::curves::*;
pub use approximations::ellipse::{
    ellipse_circumference_numerical, ellipse_circumference_series, ellipse_eccentricity,
    ellipse_focal_distance, ellipse_foci, ellipse_perimeter_cantrell, ellipse_perimeter_padé,
    ellipse_perimeter_ramanujan_i, ellipse_perimeter_ramanujan_ii,
};
pub use metrics::area_volume::{
    circle_area, cone_volume, cylinder_volume, ellipse_area, polygon_area, sphere_volume,
    triangle_area, triangle_area_from_coords,
};
