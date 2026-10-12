//! Geometry operation contracts (collision, distance, relation, transform).

pub mod collision;
pub mod distance;
pub mod relation;
pub mod transform;

pub use collision::{BBoxCollision, BasicCollision, PointDistance};
pub use distance::{CrossDistance, DistanceConvergenceError};

pub use relation::{
    Aabb2DRelation, Aabb3DRelation, AngleBetween, AngularRelation, Contains, IntersectsRelation,
    ParallelRelation, PerpendicularRelation, PointClassification, SameLineRelation,
};
pub use transform::{
    SimilarityTransform2DCore, SimilarityTransform3DCore, SimilarityTransformable2D,
    SimilarityTransformable3D, TransformError,
};
