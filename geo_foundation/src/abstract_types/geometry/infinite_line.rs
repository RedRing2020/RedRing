//! InfiniteLine (無限直線) トレイト定義
//!
//! 2D/3D空間における無限直線の抽象的なインターフェースを提供
//!
//! # 使用例
//!
//! ```ignore
//! use geo_foundation::abstract_types::geometry::InfiniteLine2D;
//!
//! // 2D無限直線の基本的な使用例
//! // let line = SomeInfiniteLine2D::from_two_points(point1, point2)?;
//! // let distance = line.distance_to_point(&query_point);
//! // let closest = line.closest_point(&query_point);
//! ```

use crate::{Angle, Scalar};

/// 2D無限直線の基本操作を定義するトレイト
pub trait InfiniteLine2D<T: Scalar> {
    /// 点の型（通常は Point2D）
    type Point;
    /// ベクトルの型（通常は Vector2D）
    type Vector;
    /// 方向の型（通常は Direction2D）
    type Direction;
    /// エラー型
    type Error;

    /// 直線上の基準点を取得
    fn origin(&self) -> Self::Point;

    /// 直線の方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Self::Direction;

    /// 指定された点が直線上にあるかを判定（許容誤差内）
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 指定された点から直線への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 指定された点から直線上の最近点を取得
    fn closest_point(&self, point: &Self::Point) -> Self::Point;

    /// 直線上でのパラメータ t での点を取得
    /// point = origin + t * direction
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定された点の直線上でのパラメータ値を取得
    /// point が直線上にない場合は最近点のパラメータを返す
    fn parameter_at_point(&self, point: &Self::Point) -> T;

    /// 他の無限直線との交点を計算
    fn intersect_line(&self, other: &Self) -> Option<Self::Point>;

    /// 他の無限直線と平行かどうかを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他の無限直線と同一かどうかを判定
    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool;

    /// 直線に垂直なベクトルを取得
    fn normal_vector(&self) -> Self::Vector;
}

/// 3D無限直線の基本操作を定義するトレイト
pub trait InfiniteLine3D<T: Scalar> {
    /// 点の型（通常は Point3D）
    type Point;
    /// ベクトルの型（通常は Vector3D）
    type Vector;
    /// 方向の型（通常は Direction3D）
    type Direction;
    /// エラー型
    type Error;

    /// 直線上の基準点を取得
    fn origin(&self) -> Self::Point;

    /// 直線の方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Self::Direction;

    /// 指定された点が直線上にあるかを判定（許容誤差内）
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 指定された点から直線への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 指定された点から直線上の最近点を取得
    fn closest_point(&self, point: &Self::Point) -> Self::Point;

    /// 直線上でのパラメータ t での点を取得
    /// point = origin + t * direction
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定された点の直線上でのパラメータ値を取得
    /// point が直線上にない場合は最近点のパラメータを返す
    fn parameter_at_point(&self, point: &Self::Point) -> T;

    /// 他の無限直線との交点を計算（3Dでは交差しない場合が多い）
    /// 交差する場合は交点、スキューライン（ねじれの位置）の場合は None を返す
    fn intersect_line(&self, other: &Self) -> Option<Self::Point>;

    /// 他の無限直線と平行かどうかを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他の無限直線と同一かどうかを判定
    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool;

    /// 他の無限直線とスキューライン（ねじれの位置）かどうかを判定
    fn is_skew_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他の無限直線との最短距離を計算（3D特有）
    fn distance_to_line(&self, other: &Self) -> T;

    /// 他の無限直線との最短距離を結ぶ線分の端点を取得
    fn closest_points_to_line(&self, other: &Self) -> Option<(Self::Point, Self::Point)>;

    /// 指定した平面との交点を計算
    fn intersect_plane(
        &self,
        plane_point: &Self::Point,
        plane_normal: &Self::Vector,
    ) -> Option<Self::Point>;

    /// 直線を指定した軸周りに回転
    fn rotate_around_axis(
        &self,
        axis_point: &Self::Point,
        axis_direction: &Self::Direction,
        angle: Angle<T>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// 無限直線の構築操作に関するトレイト
pub trait InfiniteLineBuilder<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 方向の型
    type Direction;
    /// 直線の型
    type InfiniteLine;
    /// エラー型
    type Error;

    /// 点と方向ベクトルから無限直線を構築
    fn from_point_and_direction(
        point: Self::Point,
        direction: Self::Direction,
    ) -> Result<Self::InfiniteLine, Self::Error>;

    /// 2点を通る無限直線を構築
    fn from_two_points(
        point1: Self::Point,
        point2: Self::Point,
    ) -> Result<Self::InfiniteLine, Self::Error>;

    /// 点と別の直線に平行な無限直線を構築
    fn parallel_through_point(
        point: Self::Point,
        reference_line: &Self::InfiniteLine,
    ) -> Result<Self::InfiniteLine, Self::Error>;

    /// 点と別の直線に垂直な無限直線を構築（2D用）
    fn perpendicular_through_point_2d(
        point: Self::Point,
        reference_line: &Self::InfiniteLine,
    ) -> Result<Self::InfiniteLine, Self::Error>;
}

/// 無限直線の変換操作に関するトレイト
pub trait InfiniteLineTransform<T: Scalar> {
    /// 変換行列の型
    type Matrix;
    /// 変換された直線の型
    type TransformedLine;

    /// 直線を平行移動
    fn translate(&self, translation: Self::Matrix) -> Self::TransformedLine;

    /// 直線を回転（2D）
    fn rotate_2d(&self, angle: Angle<T>, center: Self::Matrix) -> Self::TransformedLine;

    /// 直線を一般的な変換行列で変換
    fn transform(&self, matrix: &Self::Matrix) -> Self::TransformedLine;

    /// 直線をスケール変換
    fn scale(&self, factor: T, center: Self::Matrix) -> Self::TransformedLine;

    /// 直線をミラー（指定した直線を中心に反転）
    fn mirror(&self, mirror_line: &Self) -> Self::TransformedLine;
}

/// 無限直線の解析操作に関するトレイト
pub trait InfiniteLineAnalysis<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;

    /// 指定した点における直線の方程式の係数を取得（2D: ax + by + c = 0）
    fn line_equation_2d(&self) -> (T, T, T);

    /// 指定した角度範囲内での直線上の点群を生成
    fn sample_points(&self, start_param: T, end_param: T, num_points: usize) -> Vec<Self::Point>;

    /// 指定した境界ボックス内での直線の部分を取得
    fn clip_to_bounds(
        &self,
        min_point: Self::Point,
        max_point: Self::Point,
    ) -> Option<(Self::Point, Self::Point)>;

    /// 他の幾何学要素との交差判定（一般的なインターフェース）
    fn intersects_with(
        &self,
        other: &dyn InfiniteLineAnalysis<T, Point = Self::Point, Vector = Self::Vector>,
    ) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_line_trait_compilation() {
        // トレイト定義のコンパイルテスト
        // 実際の実装は具象型で行われる
    }

    #[test]
    fn test_trait_bounds() {
        // トレイト境界のテスト
        #[allow(dead_code)]
        fn check_2d_line<T: Scalar, L: InfiniteLine2D<T>>(_line: &L) {
            // このテストはコンパイル時にトレイト境界を確認
        }

        #[allow(dead_code)]
        fn check_3d_line<T: Scalar, L: InfiniteLine3D<T>>(_line: &L) {
            // このテストはコンパイル時にトレイト境界を確認
        }
    }
}
