//! 距離・計量計算モジュール

pub mod area_volume;
pub mod distance;
pub mod length;

// 便利な再エクスポート
pub use area_volume::{
    box_volume, circle_area, cone_surface_area, cone_volume, cylinder_surface_area,
    cylinder_volume, ellipse_area, polygon_area, rectangle_area, sphere_surface_area,
    sphere_volume, trapezoid_area, triangle_area_2d, triangle_area_heron,
};
pub use distance::{
    chebyshev_distance, manhattan_distance, minkowski_distance, point_distance, point_distance_2d,
    point_distance_3d, point_distance_squared,
};
pub use length::{
    arc_length, ellipse_arc_length_approximation, polyline_length, polyline_length_3d,
    vector_length, vector_length_2d, vector_length_3d, vector_length_squared,
};
