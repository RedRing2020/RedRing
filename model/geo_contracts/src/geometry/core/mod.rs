//! Geometry core contracts.
// 実装ロジックは置かず、shape contract の定義だけを保持する。

pub mod arc_traits;
pub mod circle_traits;
pub mod infinite_line_traits;
pub mod nurbs_curve_2d_traits;
pub mod nurbs_curve_3d_traits;
pub mod nurbs_surface_3d_traits;
pub mod plane3d_traits;
pub mod point_traits;
pub mod ray_traits;
pub mod triangle_traits;
pub mod vector_traits;

pub use arc_traits::{
    Arc2DConstructor, Arc2DCore, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DCore,
    Arc3DMeasure, Arc3DProperties,
};
pub use circle_traits::{
    Circle2DConstructor, Circle2DCore, Circle2DMeasure, Circle2DProperties, Circle3DConstructor,
    Circle3DCore, Circle3DMeasure, Circle3DProperties,
};
pub use infinite_line_traits::InfiniteLine3DProperties;
pub use nurbs_curve_2d_traits::{
    NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DMeasure, NurbsCurve2DProperties,
};
pub use nurbs_curve_3d_traits::{
    NurbsCurve3DConstructor, NurbsCurve3DCore, NurbsCurve3DMeasure, NurbsCurve3DProperties,
};
pub use nurbs_surface_3d_traits::{
    NurbsSurface3DConstructor, NurbsSurface3DCore, NurbsSurface3DMeasure, NurbsSurface3DProperties,
};
pub use plane3d_traits::Plane3DProperties;
pub use point_traits::{
    Point2DConstructor, Point2DCore, Point2DMeasure, Point2DProperties, Point3DConstructor,
    Point3DCore, Point3DMeasure, Point3DProperties,
};
pub use ray_traits::Ray3DProperties;
pub use triangle_traits::{
    Triangle2DConstructor, Triangle2DCore, Triangle2DMeasure, Triangle2DProperties,
    Triangle3DConstructor, Triangle3DCore, Triangle3DMeasure, Triangle3DProperties,
};
pub use vector_traits::{
    Vector2DConstructor, Vector2DCore, Vector2DMeasure, Vector2DProperties, Vector3DConstructor,
    Vector3DCore, Vector3DMeasure, Vector3DProperties,
};
