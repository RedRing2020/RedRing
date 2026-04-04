//! Geometry core contracts.
// 実装ロジックは置かず、shape contract の定義だけを保持する。

pub mod aabb_traits;
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

pub use aabb_traits::{Aabb2DDerived, Aabb2DProperties, Aabb3DDerived, Aabb3DProperties};
pub use arc_traits::{
    Arc2DConstructor, Arc2DContainment, Arc2DCore, Arc2DDerived, Arc2DDistance, Arc2DEndpoint,
    Arc2DEvaluation, Arc2DMeasure, Arc2DProperties, Arc2DSampling, Arc3DConstructor,
    Arc3DContainment, Arc3DCore, Arc3DDerived, Arc3DDistance, Arc3DEndpoint, Arc3DEvaluation,
    Arc3DMeasure, Arc3DProperties,
};
pub use circle_traits::{
    Circle2DConstructor, Circle2DContainment, Circle2DCore, Circle2DDerived, Circle2DDistance,
    Circle2DEvaluation, Circle2DMeasure, Circle2DProjection, Circle2DProperties,
    Circle3DConstructor, Circle3DContainment, Circle3DCore, Circle3DDerived, Circle3DDistance,
    Circle3DEvaluation, Circle3DMeasure, Circle3DProjection, Circle3DProperties,
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
    EllipseArc2DConstructor, EllipseArc2DContainment, EllipseArc2DCore, EllipseArc2DDerived,
    EllipseArc2DEndpoint, EllipseArc2DEvaluation, EllipseArc2DMeasure, EllipseArc2DProperties,
    EllipseArc3DConstructor, EllipseArc3DContainment, EllipseArc3DCore, EllipseArc3DDerived,
    EllipseArc3DEndpoint, EllipseArc3DEvaluation, EllipseArc3DMeasure, EllipseArc3DProperties,
};
pub use ellipse_traits::{
    Ellipse2DConstructor, Ellipse2DContainment, Ellipse2DCore, Ellipse2DDerived, Ellipse2DDistance,
    Ellipse2DEvaluation, Ellipse2DMeasure, Ellipse2DProperties, Ellipse3DConstructor,
    Ellipse3DContainment, Ellipse3DCore, Ellipse3DDerived, Ellipse3DDistance, Ellipse3DEvaluation,
    Ellipse3DMeasure, Ellipse3DProperties,
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
    InfiniteLine2DConstructor, InfiniteLine2DContainment, InfiniteLine2DCore,
    InfiniteLine2DDistance, InfiniteLine2DEvaluation, InfiniteLine2DMeasure,
    InfiniteLine2DProjection, InfiniteLine2DProperties, InfiniteLine2DTransform,
    InfiniteLine3DConstructor, InfiniteLine3DContainment, InfiniteLine3DCore,
    InfiniteLine3DDistance, InfiniteLine3DEvaluation, InfiniteLine3DMeasure,
    InfiniteLine3DProjection, InfiniteLine3DProperties, InfiniteLine3DTransform,
};
pub use linesegment_traits::{
    LineSegment2DConstructor, LineSegment2DContainment, LineSegment2DCore, LineSegment2DDerived,
    LineSegment2DDistance, LineSegment2DEvaluation, LineSegment2DMeasure, LineSegment2DProjection,
    LineSegment2DProperties, LineSegment3DConstructor, LineSegment3DContainment, LineSegment3DCore,
    LineSegment3DDerived, LineSegment3DDistance, LineSegment3DEvaluation, LineSegment3DMeasure,
    LineSegment3DProjection, LineSegment3DProperties,
};
pub use nurbs_curve_2d_traits::{
    NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DDerived, NurbsCurve2DEvaluation,
    NurbsCurve2DMeasure, NurbsCurve2DProperties,
};
pub use nurbs_curve_3d_traits::{
    NurbsCurve3DConstructor, NurbsCurve3DCore, NurbsCurve3DDerived, NurbsCurve3DEvaluation,
    NurbsCurve3DMeasure, NurbsCurve3DProperties,
};
pub use nurbs_surface_3d_traits::{
    NurbsSurface3DConstructor, NurbsSurface3DCore, NurbsSurface3DMeasure, NurbsSurface3DProperties,
};
pub use plane3d_traits::{
    Plane3DConstructor, Plane3DContainment, Plane3DCore, Plane3DDerived, Plane3DDistance,
    Plane3DEvaluation, Plane3DMeasure, Plane3DProjection, Plane3DProperties, Plane3DTransform,
};
pub use point_traits::{
    Point2DConstructor, Point2DCore, Point2DProperties, Point3DConstructor, Point3DCore,
    Point3DProperties,
};
pub use ray_traits::{
    Ray2DConstructor, Ray2DContainment, Ray2DCore, Ray2DDistance, Ray2DEvaluation, Ray2DMeasure,
    Ray2DProjection, Ray2DProperties, Ray2DTransform, Ray3DConstructor, Ray3DContainment,
    Ray3DCore, Ray3DDistance, Ray3DEvaluation, Ray3DMeasure, Ray3DProjection, Ray3DProperties,
    Ray3DTransform,
};
pub use rectangle_traits::{
    Rect2DConstructor, Rect2DContainment, Rect2DCore, Rect2DDerived, Rect2DMeasure,
    Rect2DProperties, Rect3DConstructor, Rect3DContainment, Rect3DCore, Rect3DDerived,
    Rect3DEvaluation, Rect3DMeasure, Rect3DProperties,
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
    Triangle2DConstructor, Triangle2DContainment, Triangle2DCore, Triangle2DDerived,
    Triangle2DDistance, Triangle2DMeasure, Triangle2DProperties, Triangle3DConstructor,
    Triangle3DContainment, Triangle3DCore, Triangle3DDerived, Triangle3DDistance,
    Triangle3DMeasure, Triangle3DProperties,
};
pub use vector_traits::{
    Vector2DConstructor, Vector2DCore, Vector2DProperties, Vector3DConstructor, Vector3DCore,
    Vector3DProperties,
};
