pub mod area_volume;
pub mod distance;

// distance モジュールから主要な関数をエクスポート
pub use distance::{
    ellipse_2d_distance_to_point, ellipse_3d_distance_to_point,
    sphere_to_infinite_line_distance, sphere_to_line_segment_distance, sphere_to_ray_distance,
};
