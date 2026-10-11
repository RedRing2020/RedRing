//! geo_contracts - shape contract definitions.
//!
//! This crate will host geometry-facing contracts that are shared across
//! `geo_primitives`, `geo_nurbs`, and higher-level algorithm crates.

pub mod classification;
pub mod entity;
pub mod geometry;
pub mod tolerance;

pub use analysis::abstract_types::{Angle, Scalar, TolerantEq};
pub use classification::{DimensionClass, GeometryPrimitive, PrimitiveKind};
pub use entity::{
    ArcEntity3DProperties, CircleEntity3DProperties, EntityDisplayProperties, EntityIdentity,
    LineEntity3DProperties, PlanarStroke2DProperties, StrokeDisplayProperties, StrokePattern,
};
pub use geometry::core::plane3d_traits::{
    Plane3DConstructor, Plane3DContainment, Plane3DDerived, Plane3DDistance, Plane3DEvaluation,
    Plane3DProjection, Plane3DProperties,
};
pub use geometry::core::{Aabb2DDerived, Aabb2DProperties, Aabb3DDerived, Aabb3DProperties};
pub use geometry::core::{
    Arc2DConstructor, Arc2DContainment, Arc2DDerived, Arc2DDistance, Arc2DEndpoint,
    Arc2DEvaluation, Arc2DProperties, Arc2DSampling, Arc2DTrimRange, Arc3DConstructor,
    Arc3DContainment, Arc3DDerived, Arc3DDistance, Arc3DEndpoint, Arc3DEvaluation, Arc3DProperties,
    Arc3DTrimRange, Circle2DConstructor, Circle2DContainment, Circle2DDistance, Circle2DEvaluation,
    Circle2DProjection, Circle2DProperties, Circle3DConstructor, Circle3DContainment,
    Circle3DDistance, Circle3DEvaluation, Circle3DProjection, Circle3DProperties,
    ConicalSolid3DConstructor, ConicalSolid3DContainment, ConicalSolid3DDerived,
    ConicalSolid3DDistance, ConicalSolid3DProperties, ConicalSurface3DConstructor,
    ConicalSurface3DDerived, ConicalSurface3DDistance, ConicalSurface3DEvaluation,
    ConicalSurface3DProperties, CylindricalSolid3DConstructor, CylindricalSolid3DContainment,
    CylindricalSolid3DDerived, CylindricalSolid3DDistance, CylindricalSolid3DProperties,
    CylindricalSurface3DConstructor, CylindricalSurface3DDerived, CylindricalSurface3DDistance,
    CylindricalSurface3DEvaluation, CylindricalSurface3DProperties, Direction2DConstructor,
    Direction2DProperties, Direction2DRelation, Direction2DTransform, Direction3DConstructor,
    Direction3DProperties, Direction3DRelation, Direction3DTransform, Ellipse2DConstructor,
    Ellipse2DContainment, Ellipse2DDistance, Ellipse2DEvaluation, Ellipse2DProjection,
    Ellipse2DProperties, Ellipse3DConstructor, Ellipse3DContainment, Ellipse3DDistance,
    Ellipse3DEvaluation, Ellipse3DProjection, Ellipse3DProperties, EllipseArc2DConstructor,
    EllipseArc2DContainment, EllipseArc2DDerived, EllipseArc2DEndpoint, EllipseArc2DEvaluation,
    EllipseArc2DProperties, EllipseArc2DTrimRange, EllipseArc3DConstructor,
    EllipseArc3DContainment, EllipseArc3DDerived, EllipseArc3DEndpoint, EllipseArc3DEvaluation,
    EllipseArc3DProperties, EllipseArc3DTrimRange, EllipsoidalSolid3DConstructor,
    EllipsoidalSolid3DContainment, EllipsoidalSolid3DDerived, EllipsoidalSolid3DDistance,
    EllipsoidalSolid3DProperties, EllipsoidalSurface3DConstructor, EllipsoidalSurface3DDerived,
    EllipsoidalSurface3DDistance, EllipsoidalSurface3DEvaluation, EllipsoidalSurface3DProperties,
    Point2DConstructor, Point2DDistance, Point2DInterpolation, Point2DProperties,
    Point3DConstructor, Point3DDistance, Point3DInterpolation, Point3DProperties,
    Rect2DConstructor, Rect2DContainment, Rect2DDerived, Rect2DProperties, Rect3DConstructor,
    Rect3DContainment, Rect3DDerived, Rect3DProperties, SphericalSolid3DConstructor,
    SphericalSolid3DContainment, SphericalSolid3DDerived, SphericalSolid3DDistance,
    SphericalSolid3DProperties, SphericalSurface3DConstructor, SphericalSurface3DDerived,
    SphericalSurface3DDistance, SphericalSurface3DEvaluation, SphericalSurface3DProjection,
    SphericalSurface3DProperties, TorusSolid3DConstructor, TorusSolid3DContainment,
    TorusSolid3DDerived, TorusSolid3DDistance, TorusSolid3DProperties, TorusSurface3DConstructor,
    TorusSurface3DDerived, TorusSurface3DDistance, TorusSurface3DEvaluation,
    TorusSurface3DProperties, Vector2DConstructor, Vector2DMetric, Vector2DProduct,
    Vector2DProjection, Vector2DProperties, Vector2DRelation, Vector3DConstructor, Vector3DMetric,
    Vector3DProduct, Vector3DProjection, Vector3DProperties, Vector3DRelation,
};
pub use geometry::core::{
    InfiniteLine2DConstructor, InfiniteLine2DContainment, InfiniteLine2DDistance,
    InfiniteLine2DEvaluation, InfiniteLine2DProjection, InfiniteLine2DProperties,
    InfiniteLine2DTransform, InfiniteLine3DConstructor, InfiniteLine3DContainment,
    InfiniteLine3DDistance, InfiniteLine3DEvaluation, InfiniteLine3DProjection,
    InfiniteLine3DProperties, InfiniteLine3DTransform, LineSegment2DConstructor,
    LineSegment2DContainment, LineSegment2DDerived, LineSegment2DDistance, LineSegment2DEvaluation,
    LineSegment2DProjection, LineSegment2DProperties, LineSegment3DConstructor,
    LineSegment3DContainment, LineSegment3DDerived, LineSegment3DDistance, LineSegment3DEvaluation,
    LineSegment3DProjection, LineSegment3DProperties, Ray2DConstructor, Ray2DContainment,
    Ray2DDistance, Ray2DEvaluation, Ray2DProjection, Ray2DProperties, Ray2DTransform,
    Ray3DConstructor, Ray3DContainment, Ray3DDistance, Ray3DEvaluation, Ray3DProjection,
    Ray3DProperties, Ray3DTransform,
};
pub use geometry::core::{
    NurbsCurve2DConstructor, NurbsCurve2DDerived, NurbsCurve2DEvaluation, NurbsCurve2DProperties,
    NurbsCurve3DConstructor, NurbsCurve3DDerived, NurbsCurve3DEvaluation, NurbsCurve3DProperties,
    NurbsSurface3DConstructor, NurbsSurface3DDerived, NurbsSurface3DEvaluation,
    NurbsSurface3DProperties, Triangle2DBoundaryAccess, Triangle2DConstructor,
    Triangle2DContainment, Triangle2DDistance, Triangle3DBoundaryAccess, Triangle3DConstructor,
    Triangle3DContainment, Triangle3DDerived, Triangle3DDistance,
};
pub use geometry::foundation::{Bounded, PrimitiveMetadata};
pub use geometry::operations::{
    Aabb2DRelation, Aabb3DRelation, AngleBetween, AngularRelation, BBoxCollision, BasicCollision,
    BasicIntersection, Contains, CrossDistance, DistanceConvergenceError, IntersectsRelation,
    OnPlaneRelation, ParallelRelation, PerpendicularRelation, PointClassification, PointDistance,
    SameLineRelation, SimilarityTransform2DCore, SimilarityTransform3DCore,
    SimilarityTransformable2D, SimilarityTransformable3D, TransformError,
};
pub use tolerance::{
    default_angle_tolerance, default_distance_tolerance, default_kernel_numerical_zero_tolerance,
    default_orthogonality_dot_error_tolerance, default_parallel_cross_error_tolerance,
    GeometryContext, ToleranceSettings,
};
