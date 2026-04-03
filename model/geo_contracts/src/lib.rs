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
pub use geometry::core::arc_traits::{Arc2DContainment, Arc2DSampling};
pub use geometry::core::plane3d_traits::{
    Plane3DConstructor, Plane3DContainment, Plane3DCore, Plane3DDerived, Plane3DDistance,
    Plane3DEvaluation, Plane3DMeasure, Plane3DProjection, Plane3DProperties, Plane3DTransform,
};
pub use geometry::core::{Aabb2DDerived, Aabb2DProperties, Aabb3DDerived, Aabb3DProperties};
pub use geometry::core::{
    Arc2DConstructor, Arc2DCore, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DCore,
    Arc3DMeasure, Arc3DProperties, Circle2DConstructor, Circle2DCore, Circle2DMeasure,
    Circle2DProperties, Circle3DConstructor, Circle3DCore, Circle3DMeasure, Circle3DProperties,
    ConicalSolid3DConstructor, ConicalSolid3DCore, ConicalSolid3DMeasure, ConicalSolid3DProperties,
    ConicalSurface3DConstructor, ConicalSurface3DCore, ConicalSurface3DMeasure,
    ConicalSurface3DProperties, CylindricalSolid3DConstructor, CylindricalSolid3DCore,
    CylindricalSolid3DMeasure, CylindricalSolid3DProperties, CylindricalSurface3DConstructor,
    CylindricalSurface3DCore, CylindricalSurface3DMeasure, CylindricalSurface3DProperties,
    Direction2DConstructor, Direction2DCore, Direction2DMeasure, Direction2DProperties,
    Direction3DConstructor, Direction3DCore, Direction3DMeasure, Direction3DProperties,
    Ellipse2DConstructor, Ellipse2DCore, Ellipse2DMeasure, Ellipse2DProperties,
    Ellipse3DConstructor, Ellipse3DCore, Ellipse3DMeasure, Ellipse3DProperties,
    EllipseArc2DConstructor, EllipseArc2DCore, EllipseArc2DMeasure, EllipseArc2DProperties,
    EllipseArc3DConstructor, EllipseArc3DCore, EllipseArc3DMeasure, EllipseArc3DProperties,
    EllipsoidalSolid3DConstructor, EllipsoidalSolid3DCore, EllipsoidalSolid3DMeasure,
    EllipsoidalSolid3DProperties, EllipsoidalSurface3DConstructor, EllipsoidalSurface3DCore,
    EllipsoidalSurface3DMeasure, EllipsoidalSurface3DProperties, Point2DConstructor, Point2DCore,
    Point2DProperties, Point3DConstructor, Point3DCore, Point3DProperties, Rect2DConstructor,
    Rect2DContainment, Rect2DCore, Rect2DDerived, Rect2DMeasure, Rect2DProperties,
    Rect3DConstructor, Rect3DContainment, Rect3DCore, Rect3DDerived, Rect3DEvaluation,
    Rect3DMeasure, Rect3DProperties, SphericalSolid3DConstructor, SphericalSolid3DCore,
    SphericalSolid3DMeasure, SphericalSolid3DProperties, SphericalSurface3DConstructor,
    SphericalSurface3DCore, SphericalSurface3DMeasure, SphericalSurface3DProperties,
    TorusSolid3DConstructor, TorusSolid3DCore, TorusSolid3DMeasure, TorusSolid3DProperties,
    TorusSurface3DConstructor, TorusSurface3DCore, TorusSurface3DMeasure, TorusSurface3DProperties,
    Vector2DConstructor, Vector2DCore, Vector2DProperties, Vector3DConstructor, Vector3DCore,
    Vector3DProperties,
};
pub use geometry::core::{
    InfiniteLine2DConstructor, InfiniteLine2DContainment, InfiniteLine2DCore,
    InfiniteLine2DDistance, InfiniteLine2DEvaluation, InfiniteLine2DMeasure,
    InfiniteLine2DProjection, InfiniteLine2DProperties, InfiniteLine2DTransform,
    InfiniteLine3DConstructor, InfiniteLine3DContainment, InfiniteLine3DCore,
    InfiniteLine3DDistance, InfiniteLine3DEvaluation, InfiniteLine3DMeasure,
    InfiniteLine3DProjection, InfiniteLine3DProperties, InfiniteLine3DTransform,
    LineSegment2DConstructor, LineSegment2DContainment, LineSegment2DCore, LineSegment2DDerived,
    LineSegment2DDistance, LineSegment2DEvaluation, LineSegment2DMeasure, LineSegment2DProjection,
    LineSegment2DProperties, LineSegment3DConstructor, LineSegment3DContainment, LineSegment3DCore,
    LineSegment3DDerived, LineSegment3DDistance, LineSegment3DEvaluation, LineSegment3DMeasure,
    LineSegment3DProjection, LineSegment3DProperties, Ray2DConstructor, Ray2DContainment,
    Ray2DCore, Ray2DDistance, Ray2DEvaluation, Ray2DMeasure, Ray2DProjection, Ray2DProperties,
    Ray2DTransform, Ray3DConstructor, Ray3DContainment, Ray3DCore, Ray3DDistance, Ray3DEvaluation,
    Ray3DMeasure, Ray3DProjection, Ray3DProperties, Ray3DTransform,
};
pub use geometry::core::{
    NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DMeasure, NurbsCurve2DProperties,
    NurbsCurve3DConstructor, NurbsCurve3DCore, NurbsCurve3DMeasure, NurbsCurve3DProperties,
    NurbsSurface3DConstructor, NurbsSurface3DCore, NurbsSurface3DMeasure, NurbsSurface3DProperties,
    Triangle2DConstructor, Triangle2DCore, Triangle2DMeasure, Triangle2DProperties,
    Triangle3DConstructor, Triangle3DCore, Triangle3DMeasure, Triangle3DProperties,
};
pub use geometry::foundation::{
    Bounded, ExtensionFoundation, MeasureFoundation, Point2DMeasure, Point3DMeasure,
    PrimitiveMetadata, Vector2DMeasure, Vector3DMeasure,
};
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
