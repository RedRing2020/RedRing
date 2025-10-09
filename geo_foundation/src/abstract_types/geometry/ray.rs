//! Ray（レイ/半無限直線）基本トレイト定義
//!
//! 2D/3D空間における半無限直線（レイ）の基本的なインターフェースを提供
//! 起点から特定の方向に無限に延びる直線を表現する。
//!
//! # トレイト階層
//! - `Ray2D<T>`: InfiniteLine2D<T>を継承した2D半無限直線
//! - `Ray3D<T>`: InfiniteLine3D<T>を継承した3D半無限直線
//!
//! # 設計思想
//! RayはInfiniteLineの制約版（t >= 0の範囲のみ有効）として設計されています。
//! 基本操作はInfiniteLineから継承し、レイ固有の制約を追加します。

use crate::abstract_types::Scalar;
use super::infinite_line::{InfiniteLine2D, InfiniteLine3D};

/// 2Dレイ（半無限直線）の基本操作を定義するトレイト
///
/// InfiniteLine2Dを継承し、t >= 0の範囲制約を追加します。
/// 基本操作はInfiniteLine2Dから継承し、レイ固有の制約チェックを提供します。
pub trait Ray2D<T: Scalar>: InfiniteLine2D<T> {
    /// レイ固有：制約付きの点取得
    /// t >= 0 の範囲でのみ有効（半無限直線のため）
    fn point_at_parameter_ray(&self, t: T) -> Option<Self::Point> {
        if t >= T::ZERO {
            Some(self.point_at_parameter(t))
        } else {
            None
        }
    }

    /// レイ固有：制約付きの点判定
    /// 点がレイ上にあり、かつ起点より前方にある場合のみtrueを返す
    fn contains_point_ray(&self, point: &Self::Point, tolerance: T) -> bool {
        if self.contains_point(point, tolerance) {
            let param = self.parameter_at_point(point);
            param >= -tolerance // 許容誤差を考慮
        } else {
            false
        }
    }

    /// レイ固有：起点からの距離
    /// 指定された点がレイの起点より前方にある場合の距離を取得
    fn distance_from_origin(&self, point: &Self::Point) -> Option<T> {
        let param = self.parameter_at_point(point);
        if param >= T::ZERO {
            Some(param.abs())
        } else {
            None
        }
    }
}

/// 3Dレイ（半無限直線）の基本操作を定義するトレイト
///
/// InfiniteLine3Dを継承し、t >= 0の範囲制約を追加します。
/// 基本操作はInfiniteLine3Dから継承し、レイ固有の制約チェックを提供します。
pub trait Ray3D<T: Scalar>: InfiniteLine3D<T> {
    /// レイ固有：制約付きの点取得
    /// t >= 0 の範囲でのみ有効（半無限直線のため）
    fn point_at_parameter_ray(&self, t: T) -> Option<Self::Point> {
        if t >= T::ZERO {
            Some(self.point_at_parameter(t))
        } else {
            None
        }
    }

    /// レイ固有：制約付きの点判定
    /// 点がレイ上にあり、かつ起点より前方にある場合のみtrueを返す
    fn contains_point_ray(&self, point: &Self::Point, tolerance: T) -> bool {
        if self.contains_point(point, tolerance) {
            let param = self.parameter_at_point(point);
            param >= -tolerance // 許容誤差を考慮
        } else {
            false
        }
    }

    /// レイ固有：起点からの距離
    /// 指定された点がレイの起点より前方にある場合の距離を取得
    fn distance_from_origin(&self, point: &Self::Point) -> Option<T> {
        let param = self.parameter_at_point(point);
        if param >= T::ZERO {
            Some(param.abs())
        } else {
            None
        }
    }
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
        fn check_2d_ray<T: Scalar, R: Ray2D<T>>(_ray: &R) {
            // 2Dレイトレイトの境界確認
        }

        #[allow(dead_code)]
        fn check_3d_ray<T: Scalar, R: Ray3D<T>>(_ray: &R) {
            // 3Dレイトレイトの境界確認
        }
    }
}
