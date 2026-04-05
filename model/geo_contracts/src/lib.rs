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
pub use entity::{EntityDisplayProperties, EntityIdentity, LineEntity3DProperties};
pub use geometry::core::plane3d_traits::{
    Plane3DConstructor, Plane3DContainment, Plane3DCore, Plane3DDerived, Plane3DDistance,
    Plane3DEvaluation, Plane3DProjection, Plane3DProperties, Plane3DTransform,
};
pub use geometry::core::{Aabb2DDerived, Aabb2DProperties, Aabb3DDerived, Aabb3DProperties};
pub use geometry::core::{
    Arc2DConstructor, Arc2DContainment, Arc2DCore, Arc2DDerived, Arc2DDistance, Arc2DEndpoint,
    Arc2DEvaluation, Arc2DProperties, Arc2DSampling, Arc2DTrimRange, Arc3DConstructor,
    Arc3DContainment, Arc3DCore, Arc3DDerived, Arc3DDistance, Arc3DEndpoint, Arc3DEvaluation,
    Arc3DProperties, Arc3DTrimRange, Circle2DConstructor, Circle2DContainment, Circle2DCore,
    Circle2DDerived, Circle2DDistance, Circle2DEvaluation, Circle2DProjection, Circle2DProperties,
    Circle3DConstructor, Circle3DContainment, Circle3DCore, Circle3DDerived, Circle3DDistance,
    Circle3DEvaluation, Circle3DProjection, Circle3DProperties, ConicalSolid3DConstructor,
    ConicalSolid3DContainment, ConicalSolid3DCore, ConicalSolid3DDerived, ConicalSolid3DDistance,
    ConicalSolid3DEvaluation, ConicalSolid3DProjection, ConicalSolid3DProperties,
    ConicalSurface3DConstructor, ConicalSurface3DCore, ConicalSurface3DDerived,
    ConicalSurface3DDistance, ConicalSurface3DEvaluation, ConicalSurface3DProperties,
    CylindricalSolid3DConstructor, CylindricalSolid3DContainment, CylindricalSolid3DCore,
    CylindricalSolid3DDerived, CylindricalSolid3DDistance, CylindricalSolid3DEvaluation,
    CylindricalSolid3DProjection, CylindricalSolid3DProperties, CylindricalSurface3DConstructor,
    CylindricalSurface3DCore, CylindricalSurface3DDerived, CylindricalSurface3DDistance,
    CylindricalSurface3DEvaluation, CylindricalSurface3DProperties, Direction2DConstructor,
    Direction2DCore, Direction2DProperties, Direction2DRelation, Direction2DTransform,
    Direction3DConstructor, Direction3DCore, Direction3DProperties, Direction3DRelation,
    Direction3DTransform, Ellipse2DConstructor, Ellipse2DContainment, Ellipse2DCore,
    Ellipse2DDerived, Ellipse2DDistance, Ellipse2DEvaluation, Ellipse2DProperties,
    Ellipse3DConstructor, Ellipse3DContainment, Ellipse3DCore, Ellipse3DDerived, Ellipse3DDistance,
    Ellipse3DEvaluation, Ellipse3DProperties, EllipseArc2DConstructor, EllipseArc2DContainment,
    EllipseArc2DCore, EllipseArc2DDerived, EllipseArc2DEndpoint, EllipseArc2DEvaluation,
    EllipseArc2DProperties, EllipseArc2DTrimRange, EllipseArc3DConstructor,
    EllipseArc3DContainment, EllipseArc3DCore, EllipseArc3DDerived, EllipseArc3DEndpoint,
    EllipseArc3DEvaluation, EllipseArc3DProperties, EllipseArc3DTrimRange,
    EllipsoidalSolid3DConstructor, EllipsoidalSolid3DContainment, EllipsoidalSolid3DCore,
    EllipsoidalSolid3DDerived, EllipsoidalSolid3DDistance, EllipsoidalSolid3DProjection,
    EllipsoidalSolid3DProperties, EllipsoidalSurface3DConstructor, EllipsoidalSurface3DCore,
    EllipsoidalSurface3DDerived, EllipsoidalSurface3DDistance, EllipsoidalSurface3DEvaluation,
    EllipsoidalSurface3DProperties, Point2DConstructor, Point2DCore, Point2DDistance,
    Point2DInterpolation, Point2DProperties, Point3DConstructor, Point3DCore, Point3DDistance,
    Point3DInterpolation, Point3DProperties, Rect2DConstructor, Rect2DContainment, Rect2DCore,
    Rect2DDerived, Rect2DProperties, Rect3DConstructor, Rect3DContainment, Rect3DCore,
    Rect3DDerived, Rect3DEvaluation, Rect3DProperties, SphericalSolid3DConstructor,
    SphericalSolid3DContainment, SphericalSolid3DCore, SphericalSolid3DDerived,
    SphericalSolid3DDistance, SphericalSolid3DEvaluation, SphericalSolid3DProjection,
    SphericalSolid3DProperties, SphericalSurface3DConstructor, SphericalSurface3DCore,
    SphericalSurface3DDerived, SphericalSurface3DDistance, SphericalSurface3DEvaluation,
    SphericalSurface3DProjection, SphericalSurface3DProperties, TorusSolid3DConstructor,
    TorusSolid3DContainment, TorusSolid3DCore, TorusSolid3DDerived, TorusSolid3DDistance,
    TorusSolid3DEvaluation, TorusSolid3DProjection, TorusSolid3DProperties,
    TorusSurface3DConstructor, TorusSurface3DCore, TorusSurface3DDerived, TorusSurface3DDistance,
    TorusSurface3DEvaluation, TorusSurface3DProperties, Vector2DConstructor, Vector2DCore,
    Vector2DMetric, Vector2DProduct, Vector2DProjection, Vector2DProperties, Vector2DRelation,
    Vector3DConstructor, Vector3DCore, Vector3DMetric, Vector3DProduct, Vector3DProjection,
    Vector3DProperties, Vector3DRelation,
};
pub use geometry::core::{
    InfiniteLine2DConstructor, InfiniteLine2DContainment, InfiniteLine2DCore,
    InfiniteLine2DDistance, InfiniteLine2DEvaluation, InfiniteLine2DProjection,
    InfiniteLine2DProperties, InfiniteLine2DTransform, InfiniteLine3DConstructor,
    InfiniteLine3DContainment, InfiniteLine3DCore, InfiniteLine3DDistance,
    InfiniteLine3DEvaluation, InfiniteLine3DProjection, InfiniteLine3DProperties,
    InfiniteLine3DTransform, LineSegment2DConstructor, LineSegment2DContainment, LineSegment2DCore,
    LineSegment2DDerived, LineSegment2DDistance, LineSegment2DEvaluation, LineSegment2DProjection,
    LineSegment2DProperties, LineSegment3DConstructor, LineSegment3DContainment, LineSegment3DCore,
    LineSegment3DDerived, LineSegment3DDistance, LineSegment3DEvaluation, LineSegment3DProjection,
    LineSegment3DProperties, Ray2DConstructor, Ray2DContainment, Ray2DCore, Ray2DDistance,
    Ray2DEvaluation, Ray2DProjection, Ray2DProperties, Ray2DTransform, Ray3DConstructor,
    Ray3DContainment, Ray3DCore, Ray3DDistance, Ray3DEvaluation, Ray3DProjection, Ray3DProperties,
    Ray3DTransform,
};
pub use geometry::core::{
    NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DDerived, NurbsCurve2DEvaluation,
    NurbsCurve2DProperties, NurbsCurve3DConstructor, NurbsCurve3DCore, NurbsCurve3DDerived,
    NurbsCurve3DEvaluation, NurbsCurve3DProperties, NurbsSurface3DConstructor, NurbsSurface3DCore,
    NurbsSurface3DDerived, NurbsSurface3DEvaluation, NurbsSurface3DProperties,
    Triangle2DConstructor, Triangle2DContainment, Triangle2DCore, Triangle2DDerived,
    Triangle2DDistance, Triangle2DProperties, Triangle3DConstructor, Triangle3DContainment,
    Triangle3DCore, Triangle3DDerived, Triangle3DDistance, Triangle3DProperties,
};
pub use geometry::foundation::{Bounded, PrimitiveMetadata};
pub use geometry::operations::{
    Aabb2DRelation, Aabb3DRelation, AdvancedCollision, AngleBetween, AngularRelation,
    BBoxCollision, BasicCollision, BasicIntersection, ClosestPointPair, CrossDistance,
    DirectionalRelation, DistanceConvergenceError, EllipseAccuracyAnalysis,
    EllipseAdaptiveCalculation, EllipseCalculation, FallibleCrossDistance, IntersectsRelation,
    MultipleIntersection, OnPlaneRelation, ParallelRelation, PerpendicularRelation, PointDistance,
    PointsTowards, SameLineRelation, SelfIntersection, SkewRelation,
};
pub use tolerance::{
    default_angle_tolerance, default_distance_tolerance, default_kernel_numerical_zero_tolerance,
    default_orthogonality_dot_error_tolerance, default_parallel_cross_error_tolerance,
    GeometryContext, ToleranceSettings,
};
