//! 点基本トレイト - Foundation統一システム参照
//!
//! 点の基本トレイト定義は Foundation統一システムに移行済み
//! このファイルは Foundation システムへの参照ブリッジとして機能

// Foundation統一システムから再エクスポート
pub use crate::abstracts::point_extensions::{
    PointConversion, PointDimensionConversion, PointInterpolation, PointPredicate,
    PointTransformation, UnifiedPointExtensions,
};
pub use crate::abstracts::point_traits::Point2D;

// Foundation統一システムに移行完了
// 旧式の実装は abstracts/point_traits.rs と foundation/point_extensions.rs に統合済み
