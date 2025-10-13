//! 基本幾何形状トレイト
//!
//! 各幾何形状の基本的な属性アクセスとデータ構造に直接関連する計算のみ

use super::foundation::{
    BasicContainment, BasicDirectional, BasicMetrics, BasicParametric, GeometryFoundation,
};
use crate::Scalar;

// =============================================================================
// 点 (Point)
// =============================================================================

/// 点の基本トレイト
pub trait PointCore<T: Scalar>: GeometryFoundation<T> {
    /// 座標成分を取得
    fn coordinates(&self) -> Vec<T>;

    /// 次元数を取得
    fn dimension(&self) -> usize;
}

/// 2D点の基本トレイト
pub trait Point2DCore<T: Scalar>: PointCore<T> {
    /// X座標を取得
    fn x(&self) -> T;

    /// Y座標を取得
    fn y(&self) -> T;
}

/// 3D点の基本トレイト
pub trait Point3DCore<T: Scalar>: Point2DCore<T> {
    /// Z座標を取得
    fn z(&self) -> T;
}

// =============================================================================
// ベクトル (Vector)
// =============================================================================

/// ベクトルの基本トレイト
pub trait VectorCore<T: Scalar>: GeometryFoundation<T> {
    /// ベクトルの成分を取得
    fn components(&self) -> Vec<T>;

    /// ベクトルの長さ（ノルム）を取得
    fn magnitude(&self) -> T;

    /// ベクトルの長さの二乗を取得（平方根計算の回避）
    fn magnitude_squared(&self) -> T;

    /// ゼロベクトルかどうかを判定
    fn is_zero(&self) -> bool;

    /// 単位ベクトル化（Result型で安全性を保証）
    fn normalize(&self) -> Result<Self, VectorNormalizationError>
    where
        Self: Sized;
}

/// ベクトル正規化エラー
#[derive(Debug, Clone, PartialEq)]
pub enum VectorNormalizationError {
    ZeroLength,
    NumericalInstability,
}

/// 2Dベクトルの基本トレイト
pub trait Vector2DCore<T: Scalar>: VectorCore<T> {
    /// X成分を取得
    fn x(&self) -> T;

    /// Y成分を取得
    fn y(&self) -> T;

    /// 2D外積（スカラー値）を計算
    fn cross_2d(&self, other: &Self) -> T;

    /// 垂直ベクトルを取得
    fn perpendicular(&self) -> Self;
}

/// 3Dベクトルの基本トレイト
pub trait Vector3DCore<T: Scalar>: Vector2DCore<T> {
    /// Z成分を取得
    fn z(&self) -> T;

    /// 3D外積を計算
    fn cross_3d(&self, other: &Self) -> Self;
}

// =============================================================================
// 直線・線分 (Line)
// =============================================================================

/// 無限直線の基本トレイト
pub trait InfiniteLineCore<T: Scalar>:
    GeometryFoundation<T> + BasicParametric<T> + BasicDirectional<T>
{
    /// 直線上の任意の点を取得
    fn origin(&self) -> Self::Point;

    /// 方向ベクトルを取得
    fn direction_vector(&self) -> Self::Vector;

    /// 点から直線への射影点を取得
    fn project_point(&self, point: &Self::Point) -> Self::Point;

    /// 点から直線への垂直距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;
}

/// 線分の基本トレイト（RedRing設計：InfiniteLineを基盤としパラメータ範囲で有効範囲表現）
pub trait LineCore<T: Scalar>:
    GeometryFoundation<T> + BasicMetrics<T> + BasicContainment<T> + BasicParametric<T>
{
    /// 基盤となる無限直線の型
    type InfiniteLine: GeometryFoundation<T>;

    /// 基盤となる無限直線を取得
    fn infinite_line(&self) -> &Self::InfiniteLine;

    /// 開始パラメータを取得
    fn start_parameter(&self) -> T;

    /// 終了パラメータを取得
    fn end_parameter(&self) -> T;

    /// 開始点を取得（無限直線上の開始パラメータ位置）
    fn start_point(&self) -> Self::Point;

    /// 終了点を取得（無限直線上の終了パラメータ位置）
    fn end_point(&self) -> Self::Point;

    /// 中点を取得
    fn midpoint(&self) -> Self::Point;

    /// 線分ベクトルを取得
    fn vector(&self) -> Self::Vector;

    /// 方向ベクトル（正規化済み）を取得
    fn direction(&self) -> Self::Vector;
}

// =============================================================================
// 楕円 (Ellipse)
// =============================================================================

/// 楕円の基本トレイト
pub trait EllipseCore<T: Scalar>:
    GeometryFoundation<T> + BasicMetrics<T> + BasicContainment<T> + BasicParametric<T>
{
    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 長軸半径を取得
    fn semi_major_axis(&self) -> T;

    /// 短軸半径を取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角度を取得（ラジアン）
    fn rotation_angle(&self) -> Angle<T>;

    /// 離心率を取得
    fn eccentricity(&self) -> T;

    /// 指定角度での点を取得（パラメトリック角度）
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;
}

// =============================================================================
// 円弧 (Arc)
// =============================================================================

/// 円弧の基本トレイト
pub trait ArcCore<T: Scalar>:
    GeometryFoundation<T> + BasicMetrics<T> + BasicContainment<T> + BasicParametric<T>
{
    /// 基盤となる円/楕円を取得
    type BaseShape: GeometryFoundation<T>;

    /// 基盤図形を取得
    fn base_shape(&self) -> &Self::BaseShape;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    /// 角度範囲を取得
    fn angle_span(&self) -> T {
        self.end_angle() - self.start_angle()
    }

    /// 開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 中点を取得（角度的中点）
    fn midpoint(&self) -> Self::Point;

    /// 指定角度が弧の範囲内にあるかを判定
    fn contains_angle(&self, angle: Angle<T>) -> bool;
}

// =============================================================================
// 境界ボックス (BBox) - BoundingBoxから統一
// =============================================================================

/// 境界ボックスの基本トレイト（BBoxに統一）
pub trait BBoxCore<T: Scalar>:
    GeometryFoundation<T> + BasicMetrics<T> + BasicContainment<T>
{
    /// 最小点を取得
    fn min_point(&self) -> Self::Point;

    /// 最大点を取得
    fn max_point(&self) -> Self::Point;

    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 境界ボックスが有効かを判定（min <= max）
    fn is_valid(&self) -> bool;

    /// 対角線の長さを取得
    fn diagonal_length(&self) -> T;
}

/// 2D境界ボックスの基本トレイト（BBoxに統一）
pub trait BBox2DCore<T: Scalar>: BBoxCore<T> {
    /// 幅を取得
    fn width(&self) -> T;

    /// 高さを取得
    fn height(&self) -> T;
}

/// 3D境界ボックスの基本トレイト（BBoxに統一）
pub trait BBox3DCore<T: Scalar>: BBox2DCore<T> {
    /// 奥行きを取得
    fn depth(&self) -> T;
}
