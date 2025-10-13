//! Point2D Extension Foundation統一システム
//!
//! Foundation統一システムパターンに基づくPoint2D拡張機能の統合

use crate::Scalar;

// ============================================================================
// Point2D Interpolation Foundation
// ============================================================================

/// Point2D 補間Foundation
pub trait PointInterpolation<T: Scalar> {
    type Point;

    /// 線形補間
    fn lerp(&self, other: &Self::Point, t: T) -> Self::Point;

    /// 中点計算
    fn midpoint(&self, other: &Self::Point) -> Self::Point;
}

/// Point2D 変換Foundation
pub trait PointTransformation<T: Scalar> {
    type Point;

    /// X軸反射
    fn reflect_x(&self) -> Self::Point;

    /// Y軸反射
    fn reflect_y(&self) -> Self::Point;

    /// 原点反射
    fn reflect_origin(&self) -> Self::Point;
}

/// Point2D 述語Foundation  
pub trait PointPredicate<T: Scalar> {
    type Point;

    /// 原点判定
    fn is_origin(&self) -> bool;

    /// 近似等価判定
    fn is_approximately_equal(&self, other: &Self::Point, tolerance: T) -> bool;
}

/// Point2D 変換Foundation
pub trait PointConversion<T: Scalar> {
    type Point;
    type Vector;

    /// Vector2Dに変換
    fn to_vector(&self) -> Self::Vector;

    /// 他の点へのベクトル
    fn vector_to(&self, other: &Self::Point) -> Self::Vector;

    /// ベクトルから点を作成
    fn from_vector(vector: Self::Vector) -> Self::Point;
}

/// Point2D 次元変換Foundation
pub trait PointDimensionConversion<T: Scalar> {
    type Point3D;

    /// 3D点に変換（Z=0）
    fn to_3d(&self) -> Self::Point3D;

    /// 3D点に変換（Z指定）
    fn to_3d_with_z(&self, z: T) -> Self::Point3D;
}

// ============================================================================
// Point2D 統合Extension Foundation
// ============================================================================

/// 全てのPoint2D Extension Foundationを統合
pub trait UnifiedPointExtensions<T: Scalar>:
    PointInterpolation<T>
    + PointTransformation<T>
    + PointPredicate<T>
    + PointConversion<T>
    + PointDimensionConversion<T>
{
}

// ============================================================================
// Point2D Implementation
// ============================================================================

// Point2D implementationは具体的な実装クレートで定義
// この Foundation では trait 定義のみ提供

// 具体的な実装は各実装クレートで行う
