//! GeometryKind モジュール
//!
//! 幾何学要素の分類を責務ごとに分離し、CurveKind2D / CurveKind3D / SurfaceKind を提供する。
//! 各 enum は語義的に明確で、国際開発者にも直感的に理解可能な構成となっている。
pub mod geometry_kind;
pub mod curve2d;
pub mod curve3d;
pub mod surface;

pub use geometry_kind::GeometryKind;
pub use curve2d::CurveKind2D;
pub use curve3d::CurveKind3D;
pub use surface::SurfaceKind;
