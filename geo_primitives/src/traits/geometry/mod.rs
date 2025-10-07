//! Geometry structures module
//! CAD関連の基本幾何構造体

pub mod point;
pub mod vector;

// geo_foundationのトレイトを再エクスポート
pub use geo_foundation::abstract_types::geometry::{
    Direction, Direction2D as Direction2DTrait, Direction3D as Direction3DTrait, StepCompatible,
};

// 具体的な実装構造体をエクスポート
pub use point::Point;
pub use vector::Vector;
