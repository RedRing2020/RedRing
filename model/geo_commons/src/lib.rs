//! geo_commons - 形状の型に依存しない幾何計算の関数
//!
//! 形状の型を使わず、座標・係数の値で表した幾何計算（楕円・NURBS 曲線の距離や交差など）を提供する。

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
