//! geo_foundation - 幾何計算の基盤クレート
//!
//! 抽象型と橋渡し機能を提供し、他のgeoクレート間の
//! 共通基盤として機能する

// 抽象型モジュール
pub mod abstract_types;

// 共通機能モジュール
pub mod common;

// analysisクレートからScalarトレイトを再エクスポート
pub use analysis::abstract_types::{Angle, Scalar, ToleranceContext, TolerantEq};

// analysisクレートから定数を再エクスポート
pub use analysis::{
    game, precision, GeometricTolerance, DEG_TO_RAD, E, GEOMETRIC_ANGLE_TOLERANCE,
    GEOMETRIC_DISTANCE_TOLERANCE, GEOMETRIC_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
};

// 共通機能を再エクスポート
pub use common::error;

// 抽象型の幾何トレイトを再エクスポート
pub use abstract_types::geometry::*;

/// 便利な再エクスポート
pub mod prelude {
    pub use crate::common::error::GeometryError;
    pub use crate::{
        DEG_TO_RAD, E, GEOMETRIC_ANGLE_TOLERANCE, GEOMETRIC_DISTANCE_TOLERANCE,
        GEOMETRIC_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
    };
    pub use analysis::abstract_types::{Angle, Scalar, ToleranceContext, TolerantEq};
}
