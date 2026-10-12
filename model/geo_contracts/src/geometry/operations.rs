//! Geometry operation contracts (collision, intersection, relation).

pub mod collision;
pub mod distance;
pub mod intersection;
pub mod relation;
pub mod transform;

pub use collision::{BBoxCollision, BasicCollision, PointDistance};
pub use distance::{CrossDistance, DistanceConvergenceError};
pub use intersection::BasicIntersection;
pub use relation::{
    Aabb2DRelation, Aabb3DRelation, AngleBetween, AngularRelation, Contains, IntersectsRelation,
    OnPlaneRelation, ParallelRelation, PerpendicularRelation, PointClassification,
    SameLineRelation,
};
pub use transform::{
    SimilarityTransform2DCore, SimilarityTransform3DCore, SimilarityTransformable2D,
    SimilarityTransformable3D, TransformError,
};
