//! Compatibility exports for distance helpers.
//!
//! The canonical implementations live in `geo_commons`.

pub use geo_commons::{
    ellipse_2d_distance_to_point, ellipse_3d_distance_to_point, line_segment_to_aabb_distance,
    sphere_to_infinite_line_distance, sphere_to_line_segment_distance, sphere_to_ray_distance,
};
