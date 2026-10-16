//! Geometry core contracts.
// 実装ロジックは置かず、shape contract の定義だけを保持する。

pub mod arc_traits;
pub mod circle_traits;
pub mod conical_solid_traits;
pub mod conical_surface_traits;
pub mod cylindrical_solid_traits;
pub mod cylindrical_surface_traits;
pub mod ellipse_arc_traits;
pub mod ellipse_traits;
pub mod ellipsoidal_solid_traits;
pub mod ellipsoidal_surface_traits;
pub mod infinite_line_traits;
pub mod linesegment_traits;
pub mod nurbs_curve_2d_traits;
pub mod nurbs_curve_3d_traits;
pub mod nurbs_error;
pub mod nurbs_surface_3d_traits;
pub mod plane3d_traits;
pub mod ray_traits;
pub mod rectangle_traits;
pub mod spherical_solid_traits;
pub mod spherical_surface_traits;
pub mod torus_solid_traits;
pub mod torus_surface_traits;
pub mod triangle_mesh_traits;
pub mod triangle_traits;

pub use arc_traits::{
    Arc2DConstructor, Arc2DContainment, Arc2DDerived, Arc2DDistance, Arc2DEndpoint,
    Arc2DEvaluation, Arc2DProperties, Arc2DTrimRange, Arc3DConstructor, Arc3DContainment,
    Arc3DDerived, Arc3DDistance, Arc3DEndpoint, Arc3DEvaluation, Arc3DProperties, Arc3DSampling,
    Arc3DTrimRange,
};
pub use circle_traits::{
    Circle2DConstructor, Circle2DContainment, Circle2DDistance, Circle2DEvaluation,
    Circle2DProjection, Circle2DProperties, Circle3DConstructor, Circle3DContainment,
    Circle3DDistance, Circle3DEvaluation, Circle3DProjection, Circle3DProperties,
};
pub use conical_solid_traits::{
    ConicalSolid3DConstructor, ConicalSolid3DContainment, ConicalSolid3DDistance,
    ConicalSolid3DProperties,
};
pub use conical_surface_traits::{
    ConicalSurface3DConstructor, ConicalSurface3DDistance, ConicalSurface3DEvaluation,
    ConicalSurface3DProperties,
};
pub use cylindrical_solid_traits::{
    CylindricalSolid3DConstructor, CylindricalSolid3DContainment, CylindricalSolid3DDistance,
    CylindricalSolid3DProperties,
};
pub use cylindrical_surface_traits::{
    CylindricalSurface3DConstructor, CylindricalSurface3DDistance, CylindricalSurface3DEvaluation,
    CylindricalSurface3DProperties,
};

pub use ellipse_arc_traits::{
    EllipseArc2DConstructor, EllipseArc2DContainment, EllipseArc2DDerived, EllipseArc2DDistance,
    EllipseArc2DEndpoint, EllipseArc2DEvaluation, EllipseArc2DProperties, EllipseArc2DTrimRange,
    EllipseArc3DConstructor, EllipseArc3DContainment, EllipseArc3DDerived, EllipseArc3DDistance,
    EllipseArc3DEndpoint, EllipseArc3DEvaluation, EllipseArc3DProperties, EllipseArc3DTrimRange,
};
pub use ellipse_traits::{
    Ellipse2DConstructor, Ellipse2DContainment, Ellipse2DDistance, Ellipse2DEvaluation,
    Ellipse2DProjection, Ellipse2DProperties, Ellipse3DConstructor, Ellipse3DContainment,
    Ellipse3DDistance, Ellipse3DEvaluation, Ellipse3DProjection, Ellipse3DProperties,
};
pub use ellipsoidal_solid_traits::{
    EllipsoidalSolid3DConstructor, EllipsoidalSolid3DContainment, EllipsoidalSolid3DDistance,
    EllipsoidalSolid3DProperties,
};
pub use ellipsoidal_surface_traits::{
    EllipsoidalSurface3DConstructor, EllipsoidalSurface3DDistance, EllipsoidalSurface3DEvaluation,
    EllipsoidalSurface3DProperties,
};
pub use infinite_line_traits::{
    InfiniteLine2DConstructor, InfiniteLine2DContainment, InfiniteLine2DDistance,
    InfiniteLine2DEvaluation, InfiniteLine2DProjection, InfiniteLine2DProperties,
    InfiniteLine3DConstructor, InfiniteLine3DContainment, InfiniteLine3DDistance,
    InfiniteLine3DEvaluation, InfiniteLine3DProjection, InfiniteLine3DProperties,
};
pub use linesegment_traits::{
    LineSegment2DConstructor, LineSegment2DContainment, LineSegment2DDistance,
    LineSegment2DEvaluation, LineSegment2DProjection, LineSegment2DProperties,
    LineSegment3DConstructor, LineSegment3DContainment, LineSegment3DDistance,
    LineSegment3DEvaluation, LineSegment3DProjection, LineSegment3DProperties,
    LineSegment3DSupportLine,
};
pub use nurbs_curve_2d_traits::{
    NurbsCurve2DConstructor, NurbsCurve2DDerived, NurbsCurve2DEvaluation, NurbsCurve2DProperties,
};
pub use nurbs_curve_3d_traits::{
    NurbsCurve3DConstructor, NurbsCurve3DDerived, NurbsCurve3DEvaluation, NurbsCurve3DProperties,
};
pub use nurbs_error::NurbsError;
pub use nurbs_surface_3d_traits::{
    NurbsSurface3DConstructor, NurbsSurface3DEvaluation, NurbsSurface3DProperties,
};
pub use plane3d_traits::{
    Plane3DConstructor, Plane3DContainment, Plane3DDistance, Plane3DProjection, Plane3DProperties,
};

pub use ray_traits::{
    Ray2DConstructor, Ray2DContainment, Ray2DDistance, Ray2DEvaluation, Ray2DProjection,
    Ray2DProperties, Ray3DConstructor, Ray3DContainment, Ray3DDistance, Ray3DEvaluation,
    Ray3DProjection, Ray3DProperties,
};
pub use rectangle_traits::{
    Rect2DConstructor, Rect2DContainment, Rect2DProperties, Rect3DConstructor, Rect3DContainment,
    Rect3DProperties,
};
pub use spherical_solid_traits::{
    SphericalSolid3DConstructor, SphericalSolid3DContainment, SphericalSolid3DDistance,
    SphericalSolid3DProperties,
};
pub use spherical_surface_traits::{
    SphericalSurface3DConstructor, SphericalSurface3DDistance, SphericalSurface3DEvaluation,
    SphericalSurface3DProperties,
};
pub use torus_solid_traits::{
    TorusSolid3DConstructor, TorusSolid3DContainment, TorusSolid3DDistance, TorusSolid3DProperties,
};
pub use torus_surface_traits::{
    TorusSurface3DConstructor, TorusSurface3DDistance, TorusSurface3DEvaluation,
    TorusSurface3DProperties,
};
pub use triangle_mesh_traits::{
    TriangleMesh3DConstructor, TriangleMesh3DProperties, TriangleMesh3DTriangleAccess,
    TriangleMeshError,
};
pub use triangle_traits::{
    Triangle2DBoundaryAccess, Triangle2DConstructor, Triangle2DContainment, Triangle2DDistance,
    Triangle3DBoundaryAccess, Triangle3DConstructor, Triangle3DContainment, Triangle3DDerived,
    Triangle3DDistance,
};
