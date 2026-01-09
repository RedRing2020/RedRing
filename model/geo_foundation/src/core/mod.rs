//! 幾何コア抽象化レイヤー - 基本機能の抽象化
//!
//! このモジュールは、幾何的プリミティブの基本的な抽象化を提供し、
//! 型安全性と責務分離を実現します。

// ============================================================================
// New Core Traits (ハイブリッド方針: Core3機能統合 + Transform共通)
// ============================================================================

// 基本図形 (Primitives)
pub mod arc_core_traits; // Arc Core traits (Constructor/Properties/Measure)
pub mod bbox_core_traits; // BBox Core traits (Constructor/Properties/Measure)
pub mod circle_core_traits; // Circle Core traits (Constructor/Properties/Measure)
pub mod direction_core_traits; // Direction Core traits (Constructor/Properties/Measure)
pub mod ellipse_arc_core_traits; // EllipseArc Core traits (Constructor/Properties/Measure)
pub mod ellipse_core_traits; // Ellipse Core traits (Constructor/Properties/Measure)
pub mod infinite_line_core_traits; // InfiniteLine Core traits (Constructor/Properties/Measure)
pub mod linesegment_core_traits; // LineSegment Core traits (Constructor/Properties/Measure)
pub mod plane_core_traits; // Plane Core traits (Constructor/Properties/Measure)
pub mod point_core_traits; // Point Core traits (Constructor/Properties/Measure)
pub mod ray_core_traits; // Ray Core traits (Constructor/Properties/Measure)
pub mod triangle_core_traits; // Triangle Core traits (Constructor/Properties/Measure)
pub mod vector_core_traits; // Vector Core traits (Constructor/Properties/Measure)

// Surface/Solid (3D幾何形状)
pub mod conical_solid_core_traits; // ConicalSolid Core traits
pub mod conical_surface_core_traits; // ConicalSurface Core traits
pub mod cylindrical_solid_core_traits; // CylindricalSolid Core traits
pub mod cylindrical_surface_core_traits; // CylindricalSurface Core traits
pub mod ellipsoidal_solid_core_traits; // EllipsoidalSolid Core traits
pub mod ellipsoidal_surface_core_traits; // EllipsoidalSurface Core traits
pub mod spherical_solid_core_traits; // SphericalSolid Core traits
pub mod spherical_surface_core_traits; // SphericalSurface Core traits
pub mod torus_solid_core_traits; // TorusSolid Core traits
pub mod torus_surface_core_traits; // TorusSurface Core traits

// NURBS (自由曲線・曲面)
pub mod nurbs_curve_2d_core_traits; // NurbsCurve2D Core traits
pub mod nurbs_curve_3d_core_traits; // NurbsCurve3D Core traits
pub mod nurbs_surface_3d_core_traits; // NurbsSurface3D Core traits

// 共通Transform機能
pub mod transform; // extensionsから移動した共通Transformトレイト群
pub mod transform_error; // extensionsから移動したTransformError(段階的移行中)

// ============================================================================
// Legacy Traits (段階的移行中)
// ============================================================================
pub mod nurbs_traits;
pub mod triangle_traits;
