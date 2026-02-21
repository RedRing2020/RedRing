//! 幾何コア抽象化レイヤー - 基本機能の抽象化
//!
//! このモジュールは、幾何的プリミティブの基本的な抽象化を提供し、
//! 型安全性と責務分離を実現します。

// ============================================================================
// Foundation Pattern Traits (Constructor/Properties/Measure + Transform共通)
// ============================================================================

// 基本図形 (Primitives)
pub mod arc_traits; // Arc traits (Constructor/Properties/Measure)
pub mod circle_traits; // Circle traits (Constructor/Properties/Measure)
pub mod direction_traits; // Direction traits (Constructor/Properties/Measure)
pub mod ellipse_arc_traits; // EllipseArc traits (Constructor/Properties/Measure)
pub mod ellipse_traits; // Ellipse traits (Constructor/Properties/Measure)
pub mod infinite_line_traits; // InfiniteLine traits (Constructor/Properties/Measure)
pub mod linesegment_traits; // LineSegment traits (Constructor/Properties/Measure)
pub mod plane_traits; // Plane traits (Constructor/Properties/Measure)
pub mod point_traits; // Point traits (Constructor/Properties/Measure)
pub mod ray_traits; // Ray traits (Constructor/Properties/Measure)
pub mod rectangle_traits; // Rectangle traits (Constructor/Properties/Measure)
pub mod triangle_traits; // Triangle traits (Constructor/Properties/Measure)
pub mod vector_traits; // Vector traits (Constructor/Properties/Measure)

// Surface/Solid (3D幾何形状)
pub mod conical_solid_traits; // ConicalSolid traits
pub mod conical_surface_traits; // ConicalSurface traits
pub mod cylindrical_solid_traits; // CylindricalSolid traits
pub mod cylindrical_surface_traits; // CylindricalSurface traits
pub mod ellipsoidal_solid_traits; // EllipsoidalSolid traits
pub mod ellipsoidal_surface_traits; // EllipsoidalSurface traits
pub mod spherical_solid_traits; // SphericalSolid traits
pub mod spherical_surface_traits; // SphericalSurface traits
pub mod torus_solid_traits; // TorusSolid traits
pub mod torus_surface_traits; // TorusSurface traits

// NURBS (自由曲線・曲面)
pub mod nurbs_curve_2d_traits; // NurbsCurve2D traits
pub mod nurbs_curve_3d_traits; // NurbsCurve3D traits
pub mod nurbs_surface_3d_traits; // NurbsSurface3D traits

// 共通Transform機能
pub mod transform;
pub mod transform_error;
