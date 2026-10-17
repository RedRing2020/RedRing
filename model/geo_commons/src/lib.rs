//! geo_commons - geo_* で共通する幾何計算の関数
//!
//! 形状の型に依存しない数値計算を、geo_* の各クレートが共通して使う関数として提供する。
//! 形状のクレートは、同じ計算を個別に実装せず、ここの関数を使う。
//!
//! ## 依存関係での位置
//! ```text
//! analysis → geo_commons → geo_primitives
//! ```

mod distance;
mod ellipse;
mod nurbs_curve;

pub use distance::{
    ellipse_2d_closest_point, ellipse_2d_distance_to_point, ellipse_3d_distance_to_point,
    ellipsoid_closest_point, line_segment_to_aabb_distance,
};
pub use ellipse::{
    ellipse_2d_circle_intersection_parameters, ellipse_2d_distance_critical_parameters,
};
pub use nurbs_curve::{
    nurbs_curve_minimum_distance, CircleDistance, ConvexSetDistance, DistanceField,
};
