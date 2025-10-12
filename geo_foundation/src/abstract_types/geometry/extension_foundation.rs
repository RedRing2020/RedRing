//! 幾何プリミティブの拡張基盤トレイト
//!
//! RedRing の Core/Extension Foundation パターンにおける拡張部分
//! 幾何プリミティブの高度な操作・分析機能を提供する拡張インターフェース
//!
//! # RedRing Extension Foundation パターン
//!
//! ## Extension Foundation の役割
//!
//! Extension Foundation は Core Foundation の基本機能を前提とし、
//! より高度で専門的な機能を提供します。これらの機能はオプションであり、
//! 用途に応じて選択的に実装・使用できます。
//!
//! ### Extension トレイト群
//!
//! #### 基盤トレイト
//! - `ExtensionFoundation<T>`: 拡張機能の基盤（種類識別、測定値取得）
//!
//! #### 変形・操作系
//! - `TransformableExtension<T>`: 変形操作（平行移動、回転、スケール）
//! - `MeasurableExtension<T>`: 高度な測定（表面積、体積、周囲長）
//!
//! #### 関係・解析系
//! - `SpatialExtension<T>`: 空間関係（距離、交差、包含判定）
//! - `CollectionExtension<T>`: コレクション操作（結合、検索、フィルタ）
//!
//! ## 実装例（Circle2D）
//!
//! ### Core 実装（circle_2d.rs）
//! ```rust,ignore
//! impl<T: Scalar> Circle2D<T> {
//!     // Core Methods（必須）
//!     pub fn new(center: Point2D<T>, radius: T) -> Option<Self> { ... }
//!     pub fn center(&self) -> Point2D<T> { ... }
//!     pub fn radius(&self) -> T { ... }
//!     pub fn area(&self) -> T { ... }
//!     pub fn contains_point_inside(&self, point: &Point2D<T>) -> bool { ... }
//!     pub fn bounding_box(&self) -> BBox2D<T> { ... }
//! }
//!
//! impl<T: Scalar> CoreFoundation<T> for Circle2D<T> { ... }
//! impl<T: Scalar> BasicMetrics<T> for Circle2D<T> { ... }
//! impl<T: Scalar> BasicContainment<T> for Circle2D<T> { ... }
//! ```
//!
//! ### Extension 実装（circle_2d_extensions.rs）
//! ```rust,ignore
//! impl<T: Scalar> Circle2D<T> {
//!     // Extension Methods（拡張）
//!     pub fn from_three_points(...) -> Option<Self> { ... }
//!     pub fn unit_circle() -> Self { ... }
//!     pub fn diameter(&self) -> T { ... }
//!     pub fn intersects_circle(&self, other: &Self) -> bool { ... }
//!     pub fn to_3d(&self) -> Circle3D<T> { ... }
//! }
//!
//! impl<T: Scalar> ExtensionFoundation<T> for Circle2D<T> { ... }
//! impl<T: Scalar> TransformableExtension<T> for Circle2D<T> { ... }
//! impl<T: Scalar> SpatialExtension<T> for Circle2D<T> { ... }
//! ```
//!
//! ## Extension 利用ガイドライン
//!
//! ### 実装方針
//! 1. **選択的実装**: 必要な Extension のみ実装
//! 2. **依存関係**: Core Foundation を前提とする
//! 3. **型安全性**: ジェネリック型による数値型の抽象化
//! 4. **エラー処理**: 幾何形状ごとの専用エラー型を使用
//!
//! ### 使用例
//! ```rust,ignore
//! // Core のみ使用（軽量・高速）
//! let circle = Circle2D::new(center, radius)?;
//! let bbox = circle.bounding_box(); // Core
//!
//! // Extension を含む使用（高機能）
//! let unit = Circle2D::unit_circle(); // Extension
//! let scaled = circle.scale(2.0)?; // Extension
//! let intersects = circle.intersects_circle(&other); // Extension
//! ```
//!
//! ## 将来拡張
//!
//! このパターンにより、以下のような新しい Extension を容易に追加できます：
//! - `MeshExtension<T>`: メッシュ操作
//! - `SimulationExtension<T>`: 物理シミュレーション
//! - `OptimizationExtension<T>`: 最適化アルゴリズム
//! - `CAMExtension<T>`: CAM専用機能
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

/// 全ての幾何プリミティブが実装する拡張基盤トレイト（ジェネリック版）
pub trait ExtensionFoundation<T: Scalar = f64> {
    /// 境界ボックスの型（BBoxに統一）
    type BBox: AbstractBBox<T>;

    /// プリミティブの種類を返す
    fn primitive_kind(&self) -> PrimitiveKind;

    /// 境界ボックスを返す（ジェネリック版、BBoxに統一）
    fn bounding_box(&self) -> Self::BBox;

    /// プリミティブの測定値（長さ、面積、体積など）を返す（ジェネリック版）
    fn measure(&self) -> Option<T>;
}

/// 変形可能な幾何プリミティブの拡張トレイト（ジェネリック版）
/// 注意: CurveTransformationと統合されており、より具体的な変換は
/// curve_operations::CurveTransformationを使用することを推奨
pub trait TransformableExtension<T: Scalar = f64>: ExtensionFoundation<T> {
    /// 変換結果の型（通常はSelf）
    type Transformed: ExtensionFoundation<T>;

    /// 平行移動（ジェネリック版）
    fn translate(&self, offset: (T, T, T)) -> Self::Transformed;

    /// スケール変換（ジェネリック版）
    fn scale(&self, factor: T) -> Self::Transformed;

    /// 回転（オイラー角、ラジアン、ジェネリック版）
    fn rotate(&self, angles: (T, T, T)) -> Self::Transformed;

    /// 指定した中心点を基準にスケール（デフォルト実装なし、実装側で定義）
    fn scale_about_point(&self, center: (T, T, T), factor: T) -> Self::Transformed;
}

/// 測定可能な幾何プリミティブの拡張トレイト（ジェネリック版）
pub trait MeasurableExtension<T: Scalar = f64>: ExtensionFoundation<T> {
    /// 表面積を計算（適用可能な場合、ジェネリック版）
    fn surface_area(&self) -> Option<T>;

    /// 体積を計算（適用可能な場合、ジェネリック版）
    fn volume(&self) -> Option<T>;

    /// 周囲長/周長を計算（適用可能な場合、ジェネリック版）
    fn perimeter(&self) -> Option<T>;
}

/// 幾何プリミティブのコレクション操作拡張（ジェネリック版）
pub trait CollectionExtension<T: Scalar = f64> {
    type Item: ExtensionFoundation<T>;

    /// 全プリミティブの結合境界ボックス（ジェネリック版）
    fn combined_bounding_box(&self) -> Option<<Self::Item as ExtensionFoundation<T>>::BBox>;

    /// 指定した点に最も近いプリミティブを取得（ジェネリック版）
    fn nearest_to_point(&self, point: (T, T, T)) -> Option<&Self::Item>;

    /// 指定した境界ボックスと交差するプリミティブを取得（ジェネリック版）
    fn intersecting_with_bbox(
        &self,
        bbox: &<Self::Item as ExtensionFoundation<T>>::BBox,
    ) -> Vec<&Self::Item>;
}

/// プリミティブ同士の空間関係を表現する拡張トレイト
pub trait SpatialExtension<T: Scalar = f64, Other: ExtensionFoundation<T> = Self> {
    /// 他のプリミティブとの距離を計算
    fn distance_to(&self, other: &Other) -> T;

    /// 他のプリミティブと交差するかどうか
    fn intersects_with(&self, other: &Other) -> bool;

    /// 他のプリミティブを含むかどうか
    fn contains(&self, other: &Other) -> bool;
}
