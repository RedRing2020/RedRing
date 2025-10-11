//! geo_foundation - 幾何計算の基盤クレート
//!
//! 抽象型と橋渡し機能を提供し、他のgeoクレート間の
//! 共通基盤として機能する

// 抽象型モジュール
pub mod abstract_types;

// analysisクレートからScalarトレイトを再エクスポート
pub use analysis::abstract_types::{Angle, Scalar, ToleranceContext, TolerantEq};

// analysisクレートから定数を再エクスポート
pub use analysis::{
    game, precision, GeometricTolerance, DEG_TO_RAD, E, GEOMETRIC_ANGLE_TOLERANCE,
    GEOMETRIC_DISTANCE_TOLERANCE, GEOMETRIC_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
};

// 抽象型の幾何トレイトを再エクスポート
pub use abstract_types::geometry::*;

/// 便利な再エクスポート
pub mod prelude {
    // GeometryError は削除済み - 各幾何形状で専用エラー型を使用
    // 必要に応じて NormalizationError などを個別にインポートしてください
    pub use crate::{
        DEG_TO_RAD, E, GEOMETRIC_ANGLE_TOLERANCE, GEOMETRIC_DISTANCE_TOLERANCE,
        GEOMETRIC_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
    };
    pub use analysis::abstract_types::{Angle, Scalar, ToleranceContext, TolerantEq};
}
