//! geometry3d (legacy Scalar-based primitives during migration)
//! TODO: Remove after f64geom parity & external migration.

pub mod direction3d; // Direction3D (Scalar based wrapper)
pub mod line_segment3d;
pub mod plane;
pub mod circle3d;

pub use direction3d::LegacyDirection3D;
pub use line_segment3d::LegacyLineSegment3D;
pub use plane::LegacyPlane;
pub use circle3d::LegacyCircle3D;

pub use geo_core::{Vector3D};
