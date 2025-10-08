//! Surface module
//!
//! 3次元サーフェス（曲面）の実装を提供する

pub mod sphere;

// 主要な型を再エクスポート
pub use sphere::{Sphere, Sphere3D};
