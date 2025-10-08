//! Ray（レイ/半無限直線）基本トレイト定義
//!
//! 2D/3D空間における半無限直線（レイ）の基本的なインターフェースを提供
//! 起点から特定の方向に無限に延びる直線を表現する。
//!
//! # トレイト階層
//! - `Ray<T>`: 2D/3D共通の基本操作
//! - `Ray2D<T>`: 2D固有の基本操作（継承）
//! - `Ray3D<T>`: 3D固有の基本操作（継承）
//!
//! # 注意
//! CAD/CAM向けの高度な機能（レイキャスティング、衝突検出など）は
//! 別の拡張モジュールで提供されます。

use crate::abstract_types::Scalar;

/// レイ（半無限直線）の基本操作を定義する共通トレイト
///
/// 2D/3Dの両方に共通する基本的なレイ操作を提供します。
/// 次元固有の操作は Ray2D/Ray3D トレイトで拡張されます。
pub trait Ray<T: Scalar> {
    /// 点の型（Point2D<T> または Point3D<T>）
    type Point;
    /// ベクトルの型（Vector2D<T> または Vector3D<T>）
    type Vector;
    /// 方向の型（Direction2D<T> または Direction3D<T>）
    type Direction;
    /// エラー型
    type Error;

    /// レイの起点を取得
    fn origin(&self) -> Self::Point;

    /// レイの方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Self::Direction;

    /// 指定されたパラメータ t での点を取得
    /// t >= 0 の範囲でのみ有効（半無限直線のため）
    /// point = origin + t * direction (t >= 0)
    fn point_at_parameter(&self, t: T) -> Option<Self::Point>;

    /// 指定された点がレイ上にあるかを判定（許容誤差内）
    /// 点がレイの起点より後方にある場合はfalseを返す
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 指定された点からレイへの最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 指定された点からレイ上の最近点を取得
    /// 最近点がレイの起点より後方にある場合は起点を返す
    fn closest_point(&self, point: &Self::Point) -> Self::Point;

    /// 指定された点のレイ上でのパラメータ値を取得
    /// 負の値の場合は起点より後方にあることを示す
    fn parameter_at_point(&self, point: &Self::Point) -> T;

    /// 他のレイと平行かどうかを判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他のレイと同一かどうかを判定
    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool;
}

/// 2Dレイの基本操作を定義するトレイト
pub trait Ray2D<T: Scalar>: Ray<T> {
    // 将来、2D固有の基本操作が必要になった場合にここに追加
}

/// 3Dレイの基本操作を定義するトレイト
pub trait Ray3D<T: Scalar>: Ray<T> {
    // 将来、3D固有の基本操作が必要になった場合にここに追加
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_trait_compilation() {
        // トレイト定義のコンパイルテスト
        // 実際の実装は具象型で行われる
    }

    #[test]
    fn test_ray_trait_bounds() {
        // トレイト境界のテスト
        #[allow(dead_code)]
        fn check_ray<T: Scalar, R: Ray<T>>(_ray: &R) {
            // 共通Rayトレイトの境界確認
        }

        #[allow(dead_code)]
        fn check_2d_ray<T: Scalar, R: Ray2D<T>>(_ray: &R) {
            // 2Dレイトレイトの境界確認
        }

        #[allow(dead_code)]
        fn check_3d_ray<T: Scalar, R: Ray3D<T>>(_ray: &R) {
            // 3Dレイトレイトの境界確認
        }
    }
}
