//! 形状の trait定義と値型の一括 import
//!
//! trait のメソッドを呼ぶには trait定義が `use` されている必要があるため、
//! `use geo_contracts::prelude::*;` の 1 行で形状の API を使えるようにする。

pub use crate::geometry::core::*;
pub use crate::geometry::foundation::{Bounded, PrimitiveMetadata};
pub use crate::geometry::operations::{
    AngularRelation, BBoxCollision, BasicCollision, CrossDistance, IntersectsRelation,
    ParallelRelation, PerpendicularRelation, PointDistance, SimilarityTransformable2D,
    SimilarityTransformable3D,
};
pub use crate::{
    Aabb2D, Aabb3D, Angle, Direction2D, Direction3D, Point2D, Point3D, Scalar,
    SimilarityTransform2D, SimilarityTransform3D, Vector2D, Vector3D,
};
