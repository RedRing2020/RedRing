pub mod geometry;
pub mod geometry_common;
pub mod geometry_kind;
pub mod geometry_trait;
pub mod geometry_adapter;
pub mod geometry_migration;

// 段階的移行のメインAPI
pub use geometry_migration::{
    Vector2D, Vector3D, Point2D, Point3D, Direction3D,
    Normalize, Normed,
    geometry2d, geometry3d,
};

#[cfg(feature = "use_geo_core")]
pub use geometry_migration::conversion;
