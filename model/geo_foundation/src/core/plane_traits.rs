//! 平面のコアトレイト定義
//!
//! Foundation パターンに基づく平面の基本トレイト

use crate::Scalar;

/// 平面の基本機能を定義するトレイト
pub trait PlaneCore<T: Scalar, Point, Vector> {
    /// 平面上の基準点を取得
    fn point(&self) -> Point;

    /// 平面の法線ベクトルを取得
    fn normal(&self) -> Vector;

    /// 点が平面上にあるかチェック
    fn contains_point(&self, point: Point, tolerance: T) -> bool;

    /// 点から平面までの符号付き距離
    fn distance_to_point(&self, point: Point) -> T;

    /// 点を平面に投影
    fn project_point(&self, point: Point) -> Point;
}

/// 平面の構築機能を定義するトレイト
pub trait PlaneConstruction<T: Scalar, Point, Vector> {
    /// 点と法線ベクトルから平面を作成
    fn from_point_and_normal(point: Point, normal: Vector) -> Option<Self>
    where
        Self: Sized;

    /// 3つの点から平面を作成
    fn from_three_points(p1: Point, p2: Point, p3: Point) -> Option<Self>
    where
        Self: Sized;
}

/// 平面の方程式機能を定義するトレイト
pub trait PlaneEquation<T: Scalar> {
    /// 平面の方程式係数を取得 (ax + by + cz + d = 0)
    fn equation_coefficients(&self) -> (T, T, T, T);
}

/// 平面の妥当性検証機能を定義するトレイト
pub trait PlaneValidation<T: Scalar> {
    /// 平面が有効かチェック
    fn is_valid(&self) -> bool;
}
