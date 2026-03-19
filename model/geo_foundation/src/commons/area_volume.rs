//! Compatibility exports for area and volume formulas.
//!
//! The canonical implementations live in `geo_commons`.

pub use geo_commons::{
    circle_area, cone_volume, cylinder_volume, ellipse_area, polygon_area, sphere_volume,
    triangle_area, triangle_area_from_coords,
};
