//! GeometryKind module
//!
//! Provides geometric element classification with CurveKind2D / CurveKind3D / SurfaceKind.
//! Each enum is semantically clear and intuitive for international developers.

pub mod curve2d;
pub mod curve3d;
pub mod surface;

// Re-export the sub-module types
pub use curve2d::CurveKind2D;
pub use curve3d::CurveKind3D;
pub use surface::SurfaceKind;

/// GeometryKind: Top-level classification of geometric elements
///
/// Abstract classification encompassing CurveKind2D, CurveKind3D, SurfaceKind, etc.
/// Emphasizes consistency with geometry2d, geometry3d, surface modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryKind {
    /// 2D curve
    Curve2D(CurveKind2D),
    /// 3D curve
    Curve3D(CurveKind3D),
    /// Surface
    Surface(SurfaceKind),
    /// Unclassified/unknown geometric element
    Unknown,
}
