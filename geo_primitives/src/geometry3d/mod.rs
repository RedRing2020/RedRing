//! Legacy geometry3d module - REMOVED
//!
//! This module has been removed. Legacy Scalar-based primitives have been replaced 
//! with f64 canonical types for better performance and simpler API.
//!
//! ## Migration Guide
//! Please refer to `../LEGACY_MIGRATION.md` for detailed migration instructions.
//!
//! ## Quick Migration
//! ```rust
//! // OLD: Legacy types
//! // use geo_primitives::geometry3d::{LegacyDirection3D, LegacyPlane};
//!
//! // NEW: f64 canonical types
//! use geo_primitives::{Direction3D, Plane}; // aliases
//! // or use geo_primitives::f64geom::{FDirection3, FPlane}; // direct
//! ```

// All legacy types removed - use f64 canonical types instead
// pub use direction3d::LegacyDirection3D;      -> Direction3D (alias of FDirection3)
// pub use line_segment3d::LegacyLineSegment3D; -> LineSegment3D (alias of FLineSegment3) 
// pub use plane::LegacyPlane;                  -> Plane (alias of FPlane)
// pub use circle3d::LegacyCircle3D;            -> Circle3D (alias of FCircle3)
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
