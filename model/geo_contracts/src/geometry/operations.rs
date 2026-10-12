//! Geometry operation contracts (collision, distance, relation, transform).

pub mod collision;
pub mod distance;
pub mod relation;
pub mod transform;

pub use collision::{BBoxCollision, BasicCollision, PointDistance};
pub use distance::{CrossDistance, DistanceConvergenceError};

pub use relation::{AngularRelation, IntersectsRelation, ParallelRelation, PerpendicularRelation};
pub use transform::{SimilarityTransformable2D, SimilarityTransformable3D};
