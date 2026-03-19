//! Geometry core contracts.
// 実装ロジックは置かず、shape contract の定義だけを保持する。

pub mod arc_traits;
pub mod circle_traits;
pub mod conical_solid_traits;
pub mod conical_surface_traits;
pub mod cylindrical_solid_traits;
pub mod cylindrical_surface_traits;
pub mod direction_traits;
pub mod ellipse_arc_traits;
pub mod ellipse_traits;
pub mod ellipsoidal_solid_traits;
pub mod ellipsoidal_surface_traits;
pub mod infinite_line_traits;
pub mod linesegment_traits;
pub mod nurbs_curve_2d_traits;
pub mod nurbs_curve_3d_traits;
pub mod nurbs_surface_3d_traits;
pub mod plane3d_traits;
pub mod point_traits;
pub mod ray_traits;
pub mod rectangle_traits;
pub mod spherical_solid_traits;
pub mod spherical_surface_traits;
pub mod torus_solid_traits;
pub mod torus_surface_traits;
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
pub use conical_solid_traits::{
    ConicalSolid3DConstructor, ConicalSolid3DCore, ConicalSolid3DMeasure, ConicalSolid3DProperties,
};
pub use conical_surface_traits::{
    ConicalSurface3DConstructor, ConicalSurface3DCore, ConicalSurface3DMeasure,
    ConicalSurface3DProperties,
};
pub use cylindrical_solid_traits::{
    CylindricalSolid3DConstructor, CylindricalSolid3DCore, CylindricalSolid3DMeasure,
    CylindricalSolid3DProperties,
};
pub use cylindrical_surface_traits::{
    CylindricalSurface3DConstructor, CylindricalSurface3DCore, CylindricalSurface3DMeasure,
    CylindricalSurface3DProperties,
};
pub use direction_traits::{
    Direction2DConstructor, Direction2DCore, Direction2DMeasure, Direction2DProperties,
    Direction3DConstructor, Direction3DCore, Direction3DMeasure, Direction3DProperties,
};
pub use ellipse_arc_traits::{
    EllipseArc2DConstructor, EllipseArc2DCore, EllipseArc2DMeasure, EllipseArc2DProperties,
    EllipseArc3DConstructor, EllipseArc3DCore, EllipseArc3DMeasure, EllipseArc3DProperties,
};
pub use ellipse_traits::{
    Ellipse2DConstructor, Ellipse2DCore, Ellipse2DMeasure, Ellipse2DProperties,
    Ellipse3DConstructor, Ellipse3DCore, Ellipse3DMeasure, Ellipse3DProperties,
};
pub use ellipsoidal_solid_traits::{
    EllipsoidalSolid3DConstructor, EllipsoidalSolid3DCore, EllipsoidalSolid3DMeasure,
    EllipsoidalSolid3DProperties,
};
pub use ellipsoidal_surface_traits::{
    EllipsoidalSurface3DConstructor, EllipsoidalSurface3DCore, EllipsoidalSurface3DMeasure,
    EllipsoidalSurface3DProperties,
};
pub use infinite_line_traits::{
    InfiniteLine2DConstructor, InfiniteLine2DCore, InfiniteLine2DMeasure, InfiniteLine2DProperties,
    InfiniteLine3DConstructor, InfiniteLine3DCore, InfiniteLine3DMeasure, InfiniteLine3DProperties,
};
pub use linesegment_traits::{
    LineSegment2DConstructor, LineSegment2DCore, LineSegment2DMeasure, LineSegment2DProperties,
    LineSegment3DCollisionDetection, LineSegment3DConstructor, LineSegment3DCore,
    LineSegment3DMeasure, LineSegment3DProperties,
};
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
pub use ray_traits::{
    Ray2DConstructor, Ray2DCore, Ray2DMeasure, Ray2DProperties, Ray3DConstructor, Ray3DCore,
    Ray3DMeasure, Ray3DProperties,
};
pub use rectangle_traits::{
    Rect2DConstructor, Rect2DCore, Rect2DMeasure, Rect2DProperties, Rect3DConstructor, Rect3DCore,
    Rect3DMeasure, Rect3DProperties,
};
pub use spherical_solid_traits::{
    SphericalSolid3DConstructor, SphericalSolid3DCore, SphericalSolid3DMeasure,
    SphericalSolid3DProperties,
};
pub use spherical_surface_traits::{
    SphericalSurface3DConstructor, SphericalSurface3DCore, SphericalSurface3DMeasure,
    SphericalSurface3DProperties,
};
pub use torus_solid_traits::{
    TorusSolid3DConstructor, TorusSolid3DCore, TorusSolid3DMeasure, TorusSolid3DProperties,
};
pub use torus_surface_traits::{
    TorusSurface3DConstructor, TorusSurface3DCore, TorusSurface3DMeasure, TorusSurface3DProperties,
};
pub use triangle_traits::{
    Triangle2DConstructor, Triangle2DCore, Triangle2DMeasure, Triangle2DProperties,
    Triangle3DConstructor, Triangle3DCore, Triangle3DMeasure, Triangle3DProperties,
};
pub use vector_traits::{
    Vector2DConstructor, Vector2DCore, Vector2DMeasure, Vector2DProperties, Vector3DConstructor,
    Vector3DCore, Vector3DMeasure, Vector3DProperties,
};
