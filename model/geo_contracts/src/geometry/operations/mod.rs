//! Geometry operation contracts (collision, intersection).

pub mod collision;
pub mod distance;
pub mod ellipse_calculation_traits;
pub mod intersection;

pub use collision::{AdvancedCollision, BBoxCollision, BasicCollision, PointDistance};
pub use distance::{CrossDistance, DistanceConvergenceError, FallibleCrossDistance};
pub use ellipse_calculation_traits::{
    EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation,
};
pub use intersection::{BasicIntersection, MultipleIntersection, SelfIntersection};
