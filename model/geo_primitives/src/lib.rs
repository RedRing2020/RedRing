//! 幾何プリミティブクレート
//!
//! モジュール公開と trait定義 の再エクスポートを提供する。

mod ellipse_calculation_analysis;
mod ellipse_calculation_strategy;

// Transform 系は geo_core を正規参照先として再公開する。
pub use geo_core::{
    AnalysisTransform2D, AnalysisTransform3D, AnalysisTransformSupport, TransformError,
};

// 3D プリミティブ
pub mod arc_3d;
pub mod arc_3d_bounds;
pub mod arc_3d_extensions;
pub mod circle_3d;
pub mod circle_3d_bounds;
pub mod circle_3d_extensions;
pub mod circle_3d_tests;
pub mod conical_solid_3d;
pub mod conical_solid_3d_bounds;
pub mod conical_solid_3d_extensions;
pub mod conical_surface_3d;
pub mod conical_surface_3d_bounds;
pub mod conical_surface_3d_extensions;
pub mod cylindrical_solid_3d;
pub mod cylindrical_solid_3d_bounds;
pub mod cylindrical_solid_3d_extensions;
#[cfg(test)]
pub mod cylindrical_solid_3d_tests;
pub mod cylindrical_surface_3d;
pub mod cylindrical_surface_3d_bounds;
pub mod cylindrical_surface_3d_extensions;
#[cfg(test)]
pub mod cylindrical_surface_3d_tests;
pub mod direction_3d;
pub mod direction_3d_extensions;
pub mod ellipse_3d;
pub mod ellipse_3d_bounds;
pub mod ellipse_3d_extensions;
pub mod ellipse_arc_3d;
pub mod ellipse_arc_3d_bounds;
pub mod ellipse_arc_3d_extensions;
pub mod ellipse_arc_3d_tests;
pub mod ellipsoidal_solid_3d;
pub mod ellipsoidal_solid_3d_bounds;
pub mod ellipsoidal_solid_3d_transform;
pub mod ellipsoidal_surface_3d;
pub mod ellipsoidal_surface_3d_bounds;
pub mod infinite_line_3d;
pub mod infinite_line_3d_extensions;
#[cfg(test)]
pub mod infinite_line_3d_tests;
pub mod line_segment_3d;
pub mod line_segment_3d_bounds;
pub mod line_segment_3d_extensions;
pub mod plane_3d;
pub mod plane_3d_extensions;
#[cfg(test)]
pub mod plane_3d_tests;

// Point/Vector 実装は geo_core 側を利用する。

pub mod ray_3d;
pub mod ray_3d_extensions;
pub mod rectangle_3d;
pub mod rectangle_3d_bounds;
pub mod rectangle_3d_transform;
pub mod spherical_solid_3d;
pub mod spherical_solid_3d_bounds;
#[cfg(test)]
pub mod spherical_solid_3d_tests;
pub mod spherical_surface_3d;
pub mod spherical_surface_3d_bounds;
pub mod torus_solid_3d;
pub mod torus_solid_3d_bounds;
pub mod torus_solid_3d_extensions;
pub mod torus_surface_3d;
pub mod torus_surface_3d_bounds;
pub mod torus_surface_3d_extensions;
pub mod triangle_3d;
pub mod triangle_3d_bounds;
pub mod triangle_mesh_3d;
pub mod triangle_mesh_3d_bounds;
pub mod triangle_mesh_3d_transform;

// 3D テストモジュール
#[cfg(test)]
pub mod ray_3d_tests;
#[cfg(test)]
pub mod triangle_3d_tests;
#[cfg(test)]
pub mod triangle_mesh_3d_tests;

// 2D プリミティブ
pub mod arc_2d;
pub mod arc_2d_bounds;
pub mod arc_2d_extensions;

pub mod circle_2d;
pub mod circle_2d_bounds;
pub mod circle_2d_extensions;

pub mod circle_2d_metrics;
pub mod direction_2d;
pub mod direction_2d_extensions;
pub mod ellipse_2d;
pub mod ellipse_2d_bounds;
pub mod ellipse_2d_transform;
pub mod ellipse_arc_2d;
pub mod ellipse_arc_2d_bounds;
pub mod ellipse_arc_2d_extensions;
pub mod infinite_line_2d;
pub mod infinite_line_2d_extensions;
pub mod infinite_line_2d_transform;
pub mod line_segment_2d;
pub mod line_segment_2d_bounds;
pub mod line_segment_2d_extensions;
pub mod ray_2d;
pub mod ray_2d_extensions;
pub mod ray_2d_transform;
pub mod rectangle_2d;
pub mod rectangle_2d_bounds;
pub mod rectangle_2d_transform;
pub mod triangle_2d;
pub mod triangle_2d_bounds;
pub mod triangle_2d_transform;

// テストモジュール
#[cfg(test)]
mod direction_2d_extensions_tests;
#[cfg(test)]
mod direction_3d_extensions_tests;
#[cfg(test)]
mod ellipse_3d_tests;

// capability trait は geo_contracts を正本 export とし、この crate では shape 型公開を優先する。

// 基本型
pub use geo_contracts::{Angle, Scalar};

// 3D プリミティブ
pub use arc_3d::Arc3D;
pub use circle_3d::Circle3D;
pub use conical_solid_3d::{Cone3D, ConicalSolid3D};
pub use conical_surface_3d::{ConeRim3D, ConicalSurface3D};
pub use cylindrical_solid_3d::CylindricalSolid3D;
pub use cylindrical_surface_3d::CylindricalSurface3D;
pub use direction_3d::Direction3D;
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

// 2D プリミティブ
pub use arc_2d::Arc2D;
pub use circle_2d::Circle2D;
pub use direction_2d::Direction2D;
pub use ellipse_2d::Ellipse2D;
pub use ellipse_arc_2d::EllipseArc2D;
pub use geo_core::{Point2D, Point3D, Vector2D, Vector3D};
pub use infinite_line_2d::InfiniteLine2D;
pub use line_segment_2d::LineSegment2D;
pub use ray_2d::Ray2D;
pub use rectangle_2d::Rect2D;
pub use triangle_2d::Triangle2D;
