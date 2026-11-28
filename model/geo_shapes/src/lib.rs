//! geo_shapes - 形状の層のクレート
//!
//! 基本の形状（直線・円・楕円・解析的な曲面）と NURBS の曲線・曲面の、どちらの型も持つ形状を提供する。
//! 現在の提供:
//! - CurveRef: 稜線が参照する母曲線
//! - SurfaceRef: 面が載る曲面

pub mod curve_ref;
pub mod surface_ref;

pub use curve_ref::CurveRef;
pub use surface_ref::SurfaceRef;
