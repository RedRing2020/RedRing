mod aabb_2d_trait;
mod aabb_3d_trait;
pub mod area_volume;
pub mod distance;
pub mod ellipse_approximations;
pub use aabb_2d_trait::Aabb2DTrait;
pub use aabb_3d_trait::Aabb3DTrait;
/// geo_foundation commons - 共通計算トレイト
///
/// geo_commons クレートで使用される共通計算トレイトを定義します。
/// Foundation Pattern において、具体的な計算実装とトレイト定義を分離する役割を担います。
pub mod ellipse_calculation_traits;

// 便利な再エクスポート
pub use area_volume::{
    circle_area, cone_volume, cylinder_volume, ellipse_area, polygon_area, sphere_volume,
    triangle_area, triangle_area_from_coords,
};
pub use distance::{
    ellipse_2d_distance_to_point, ellipse_3d_distance_to_point, line_segment_to_aabb_distance,
};
pub use ellipse_approximations::{
    ellipse_circumference_numerical, ellipse_circumference_series, ellipse_eccentricity,
    ellipse_focal_distance, ellipse_foci, ellipse_perimeter_cantrell, ellipse_perimeter_padé,
    ellipse_perimeter_ramanujan_i, ellipse_perimeter_ramanujan_ii,
};
pub use ellipse_calculation_traits::{
    EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation,
};
