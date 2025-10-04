// geometry module removed after migration to geo_primitives / geo_core.
pub mod geometry_common;
pub mod geometry_kind;
pub mod geometry_trait;
// pub mod geometry_adapter; // 一時的に無効化
// pub mod geometry_migration; // 一時的に無効化

// Phase 1: geo_core統合アダプター
// pub mod geometry_adapter_curve3d; // 一時的に無効化
// pub mod geometry_integrated_line; // 一時的に無効化
// pub mod geometry_common_adapted; // 一時的に無効化
pub mod geometry_simple_adapter; // シンプルな動作版

#[cfg(feature = "use_geo_core")]
pub mod geometry_compatibility_tests; // 互換性検証

// 段階的移行のメインAPI（一時的に無効化）
// pub use geometry_migration::{
//     Vector2D, Vector3D, Point2D, Point3D, Direction3D,
//     Normalize, Normed,
//     geometry2d, geometry3d,
// };

// #[cfg(feature = "use_geo_core")]
// pub use geometry_migration::conversion;

// Phase 1: geo_core統合API（シンプル版）
#[cfg(feature = "use_geo_core")]
pub use geometry_simple_adapter::{SimpleAdaptedLine, TypeConverter, simple_factory};
