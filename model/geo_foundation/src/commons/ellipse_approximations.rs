//! Compatibility exports for ellipse approximation formulas.
//!
//! The canonical implementations live in `geo_commons`.

pub use geo_commons::{
    ellipse_circumference_numerical, ellipse_circumference_series, ellipse_eccentricity,
    ellipse_focal_distance, ellipse_foci, ellipse_perimeter_cantrell, ellipse_perimeter_padé,
    ellipse_perimeter_ramanujan_i, ellipse_perimeter_ramanujan_ii,
};
