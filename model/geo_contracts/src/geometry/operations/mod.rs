//! Geometry operation contracts (collision, intersection).

pub mod collision;
pub mod intersection;

pub use collision::{AdvancedCollision, BBoxCollision, BasicCollision, PointDistance};
pub use intersection::{BasicIntersection, MultipleIntersection, SelfIntersection};
