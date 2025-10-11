//! 幾何プリミティブの共通トレイト
//!
//! 全ての幾何プリミティブが実装すべき基本的なインターフェース
//!
//! 注意: 抽象的なトレイト定義は geo_foundation に移動されました。
//! 以下のトレイトは geo_foundation::abstract_types::geometry::primitive で利用可能です:
//! - GeometricPrimitive<T: Scalar>
//! - TransformablePrimitive<T: Scalar>
//! - MeasurablePrimitive<T: Scalar>
//! - PrimitiveCollection<T: Scalar>
//! - SpatialRelation<T: Scalar, Other>

// geo_foundation の抽象プリミティブトレイトを再エクスポート
pub use geo_foundation::abstract_types::geometry::primitive::{
    GeometricPrimitive, MeasurablePrimitive, PrimitiveCollection, SpatialRelation,
    TransformablePrimitive,
};

// CAD/CAM 固有の拡張トレイトはここで定義
// 例: CAD固有の機能（フィレット、面取り等）のトレイト
