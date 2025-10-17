//! geo_foundation - 幾何計算の基盤クレート
//!
//! geo_foundation は抽象化・インターフェース層
//! geo_primitives の具体実装を抽象化して呼び出すためのトレイト定義

// Core Foundation - 幾何形状の中核基盤トレイト
pub mod core_foundation;

// Extension Foundation - 幾何形状の拡張基盤トレイト
pub mod extension_foundation;

// Classification - 幾何プリミティブの分類システム
pub mod classification;

// Classification - 幾何プリミティブの分類
pub use classification::{DimensionClass, GeometryPrimitive, PrimitiveKind};

// Abstract Traits - 最小責務抽象化（主要インターフェース）
pub mod abstracts;

// Foundation Traits - 統一操作トレイト群
pub mod traits;

// 許容誤差管理モジュール
pub mod tolerance;

// 許容誤差移行支援モジュール（将来削除予定）
pub mod tolerance_migration;

// Legacy abstract_types (段階的廃止予定)
#[deprecated(note = "Use `abstracts`, `traits`, or `geometry` modules directly")]
pub mod abstract_types;

// テストモジュール
#[cfg(test)]
mod tolerance_tests;

// analysisクレートからScalarトレイトを再エクスポート
pub use analysis::abstract_types::{Angle, Scalar, TolerantEq};

// analysisクレートから定数を再エクスポート
pub use analysis::{
    game, precision, GeometricTolerance, DEG_TO_RAD, E, GEOMETRIC_ANGLE_TOLERANCE,
    GEOMETRIC_DISTANCE_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
};

// Abstract Traitsを再エクスポート（主要インターフェース）
pub use abstracts::{
    circle_traits::Circle2D as Circle2DTrait,
    point_traits::Point2D as Point2DTrait,
    vector_traits::{Vector2D, Vector2DOps, Vector3D, Vector3DOps, VectorMetrics, VectorOps},
};

// Extension Foundation Traitsを再エクスポート
pub use extension_foundation::{
    CollectionExtension, ExtensionFoundation, MeasurableExtension, SpatialExtension,
    TransformableExtension,
};

// Foundation Traitsを再エクスポート
pub use traits::{
    AdvancedCollision, AdvancedTransform, BasicCollision, BasicIntersection, BasicTransform,
    MultipleIntersection, PointDistance, SelfIntersection, TransformHelpers,
};

// Geometry Core Foundationを再エクスポート
// 注意: 実際の実装はabstract_types/foundation/にある
// pub use geometry::{
//     BasicContainment, BasicDirectional, BasicMetrics, BasicParametric, CoreFoundation,
// };

// 許容誤差管理を再エクスポート
pub use tolerance::{GeometryContext, ToleranceSettings};

/// 便利な再エクスポート
pub mod prelude {
    // GeometryError は削除済み - 各幾何形状で専用エラー型を使用
    // 必要に応じて NormalizationError などを個別にインポートしてください
    pub use crate::{
        GeometryContext, ToleranceSettings, DEG_TO_RAD, E, GEOMETRIC_ANGLE_TOLERANCE,
        GEOMETRIC_DISTANCE_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
    };
    pub use analysis::abstract_types::{Angle, Scalar, TolerantEq};
}
