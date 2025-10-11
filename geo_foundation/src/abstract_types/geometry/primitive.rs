//! 幾何プリミティブの共通トレイト
//!
//! 全ての幾何プリミティブが実装すべき基本的なインターフェース
//!
//! # エラー処理の設計方針
//!
//! RedRing では各幾何形状ごとに専用のエラー型を定義する方針を採用しています。
//! 汎用的な `GeometryError` ではなく、具体的で型安全なエラー処理を提供します。
//!
//! ## 専用エラー型の設計パターン
//!
//! ```rust,ignore
//! // 例: Circle 用のエラー型
//! #[derive(Debug, Clone, PartialEq)]
//! pub enum CircleError {
//!     InvalidRadius,
//!     CollinearPoints,
//!     InvalidCenter,
//! }
//!
//! impl std::fmt::Display for CircleError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             CircleError::InvalidRadius => write!(f, "Invalid radius: must be positive"),
//!             CircleError::CollinearPoints => write!(f, "Collinear points cannot form a circle"),
//!             CircleError::InvalidCenter => write!(f, "Invalid center point"),
//!         }
//!     }
//! }
//! impl std::error::Error for CircleError {}
//!
//! // 使用例（仮想的な Circle 構造体）
//! struct Circle { center: Point, radius: f64 }
//! struct Point { x: f64, y: f64 }
//!
//! impl Circle {
//!     pub fn new(center: Point, radius: f64) -> Result<Self, CircleError> {
//!         if radius <= 0.0 {
//!             return Err(CircleError::InvalidRadius);
//!         }
//!         Ok(Circle { center, radius })
//!     }
//! }
//! ```
//!
//! この設計により、コンパイル時の型チェックで適切なエラー処理が保証されます。

use super::classification::PrimitiveKind;
use crate::Scalar;

// AbstractBBoxを抽象境界ボックスとして定義（より柔軟な実装）
/// 抽象的な境界ボックストレイト（BBoxに統一）
pub trait AbstractBBox<T: Scalar> {
    type Point;
    fn min(&self) -> Self::Point;
    fn max(&self) -> Self::Point;
}

/// 全ての幾何プリミティブが実装する共通トレイト（ジェネリック版）
pub trait GeometricPrimitive<T: Scalar = f64> {
    /// 境界ボックスの型（BBoxに統一）
    type BBox: AbstractBBox<T>;

    /// プリミティブの種類を返す
    fn primitive_kind(&self) -> PrimitiveKind;

    /// 境界ボックスを返す（ジェネリック版、BBoxに統一）
    fn bounding_box(&self) -> Self::BBox;

    /// プリミティブの測定値（長さ、面積、体積など）を返す（ジェネリック版）
    fn measure(&self) -> Option<T>;
}

/// 変形可能な幾何プリミティブのトレイト（ジェネリック版）
/// 注意: CurveTransformationと統合されており、より具体的な変換は
/// curve_operations::CurveTransformationを使用することを推奨
pub trait TransformablePrimitive<T: Scalar = f64>: GeometricPrimitive<T> {
    /// 変換結果の型（通常はSelf）
    type Transformed: GeometricPrimitive<T>;

    /// 平行移動（ジェネリック版）
    fn translate(&self, offset: (T, T, T)) -> Self::Transformed;

    /// スケール変換（ジェネリック版）
    fn scale(&self, factor: T) -> Self::Transformed;

    /// 回転（オイラー角、ラジアン、ジェネリック版）
    fn rotate(&self, angles: (T, T, T)) -> Self::Transformed;

    /// 指定した中心点を基準にスケール（デフォルト実装なし、実装側で定義）
    fn scale_about_point(&self, center: (T, T, T), factor: T) -> Self::Transformed;
}

/// 測定可能な幾何プリミティブのトレイト（ジェネリック版）
pub trait MeasurablePrimitive<T: Scalar = f64>: GeometricPrimitive<T> {
    /// 表面積を計算（適用可能な場合、ジェネリック版）
    fn surface_area(&self) -> Option<T>;

    /// 体積を計算（適用可能な場合、ジェネリック版）
    fn volume(&self) -> Option<T>;

    /// 周囲長/周長を計算（適用可能な場合、ジェネリック版）
    fn perimeter(&self) -> Option<T>;
}

/// 幾何プリミティブのコレクション操作（ジェネリック版）
pub trait PrimitiveCollection<T: Scalar = f64> {
    type Item: GeometricPrimitive<T>;

    /// 全プリミティブの結合境界ボックス（ジェネリック版）
    fn combined_bounding_box(&self) -> Option<<Self::Item as GeometricPrimitive<T>>::BBox>;

    /// 指定した点に最も近いプリミティブを取得（ジェネリック版）
    fn nearest_to_point(&self, point: (T, T, T)) -> Option<&Self::Item>;

    /// 指定した境界ボックスと交差するプリミティブを取得（ジェネリック版）
    fn intersecting_with_bbox(
        &self,
        bbox: &<Self::Item as GeometricPrimitive<T>>::BBox,
    ) -> Vec<&Self::Item>;
}

/// プリミティブ同士の空間関係を表現するトレイト
pub trait SpatialRelation<T: Scalar = f64, Other: GeometricPrimitive<T> = Self> {
    /// 他のプリミティブとの距離を計算
    fn distance_to(&self, other: &Other) -> T;

    /// 他のプリミティブと交差するかどうか
    fn intersects_with(&self, other: &Other) -> bool;

    /// 他のプリミティブを含むかどうか
    fn contains(&self, other: &Other) -> bool;
}
