//! Geometry operation contracts (collision, intersection, relation).

pub mod collision;
pub mod distance;
pub mod ellipse_calculation;
pub mod intersection;
pub mod relation;

pub use collision::{AdvancedCollision, BBoxCollision, BasicCollision, PointDistance};
pub use distance::{CrossDistance, DistanceConvergenceError, FallibleCrossDistance};
pub use ellipse_calculation::{
    EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation,
};
pub use intersection::{BasicIntersection, MultipleIntersection, SelfIntersection};
pub use relation::{
    Aabb2DRelation, Aabb3DRelation, AngleBetween, AngularRelation, ClosestPointPair, Contains,
    DirectionalRelation, IntersectsRelation, OnPlaneRelation, ParallelRelation,
    PerpendicularRelation, PointsTowards, SameLineRelation, SkewRelation,
};
