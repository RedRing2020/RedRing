//! 幾何プリミティブクレート
//!
//! 形状の型と、形状が使う基本型を再エクスポートする。形状のモジュールは private とし、
//! trait定義は `geo_contracts` から使う。

// 2D の形状
mod arc_2d;
mod circle_2d;
mod ellipse_2d;
mod ellipse_arc_2d;
mod infinite_line_2d;
mod line_segment_2d;
mod ray_2d;
mod rectangle_2d;
mod triangle_2d;

// 3D の形状
mod arc_3d;
mod circle_3d;
mod conical_solid_3d;
mod conical_surface_3d;
mod cylindrical_solid_3d;
mod cylindrical_surface_3d;
mod ellipse_3d;
mod ellipse_arc_3d;
mod ellipsoidal_solid_3d;
mod ellipsoidal_surface_3d;
mod infinite_line_3d;
mod line_segment_3d;
mod plane_3d;
mod ray_3d;
mod rectangle_3d;
mod spherical_solid_3d;
mod spherical_surface_3d;
mod torus_solid_3d;
mod torus_surface_3d;
mod triangle_3d;
mod triangle_mesh_3d;

// 基本型
pub use geo_contracts::{Angle, Scalar};
pub use geo_core::{
    Direction2D, Direction3D, Point2D, Point3D, SimilarityTransform2D, SimilarityTransform3D,
    TransformError, Vector2D, Vector3D,
};

// 2D の形状
pub use arc_2d::Arc2D;
pub use circle_2d::Circle2D;
pub use ellipse_2d::Ellipse2D;
pub use ellipse_arc_2d::EllipseArc2D;
pub use infinite_line_2d::InfiniteLine2D;
pub use line_segment_2d::LineSegment2D;
pub use ray_2d::Ray2D;
pub use rectangle_2d::Rect2D;
pub use triangle_2d::Triangle2D;

// 3D の形状
pub use arc_3d::Arc3D;
pub use circle_3d::Circle3D;
pub use conical_solid_3d::ConicalSolid3D;
pub use conical_surface_3d::{ConeRim3D, ConicalSurface3D};
pub use cylindrical_solid_3d::CylindricalSolid3D;
pub use cylindrical_surface_3d::CylindricalSurface3D;
pub use ellipse_3d::Ellipse3D;
pub use ellipse_arc_3d::EllipseArc3D;
pub use ellipsoidal_solid_3d::EllipsoidalSolid3D;
pub use ellipsoidal_surface_3d::EllipsoidalSurface3D;
pub use infinite_line_3d::InfiniteLine3D;
pub use line_segment_3d::LineSegment3D;
pub use plane_3d::Plane3D;
pub use ray_3d::Ray3D;
pub use rectangle_3d::Rect3D;
pub use spherical_solid_3d::SphericalSolid3D;
pub use spherical_surface_3d::SphericalSurface3D;
pub use torus_solid_3d::TorusSolid3D;
pub use torus_surface_3d::TorusSurface3D;
pub use triangle_3d::Triangle3D;
pub use triangle_mesh_3d::TriangleMesh3D;
