//! Geometry operation contracts (point distance, bounding box, relation, transform).
//!
//! Cross-shape distance, collision and intersection contracts live in `geo_algorithms::operations`.

pub mod collision;
pub mod relation;
pub mod transform;

pub use collision::{BBoxCollision, PointDistance};

pub use relation::{AngularRelation, ParallelRelation, PerpendicularRelation};
pub use transform::{SimilarityTransformable2D, SimilarityTransformable3D};
