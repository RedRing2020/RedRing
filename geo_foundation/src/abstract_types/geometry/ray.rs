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

use super::infinite_line::{InfiniteLine2D, InfiniteLine3D};
use crate::Scalar;

/// 2D空間における半無限直線（レイ）の基本トレイト
pub trait Ray2D<T: Scalar>: InfiniteLine2D<T> {
    /// レイ固有：制約付き点取得（t >= 0のみ）
    fn point_at_ray(&self, t: T) -> Option<Self::Point> {
        if t >= T::ZERO {
            Some(self.point_at_parameter(t))
        } else {
            None
        }
    }

    /// レイ固有：制約付き点判定（前方のみ）
    fn contains_point_ray(&self, point: &Self::Point, tolerance: T) -> bool {
        if self.distance_to_point(point) <= tolerance {
            let param = self.parameter_at_point(point);
            param >= -tolerance // 許容誤差を考慮して前方判定
        } else {
            false
        }
    }

    /// レイの起点から指定点までの距離（レイ上の場合のみ）
    fn distance_from_origin(&self, point: &Self::Point) -> Option<T> {
        if self.contains_point_ray(point, T::from_f64(1e-10)) {
            let param = self.parameter_at_point(point);
            if param >= T::ZERO {
                Some(param)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// 3D空間における半無限直線（レイ）の基本トレイト
pub trait Ray3D<T: Scalar>: InfiniteLine3D<T> {
    /// レイ固有：制約付き点取得（t >= 0のみ）
    fn point_at_ray(&self, t: T) -> Option<Self::Point> {
        if t >= T::ZERO {
            Some(self.point_at_parameter(t))
        } else {
            None
        }
    }

    /// レイ固有：制約付き点判定（前方のみ）
    fn contains_point_ray(&self, point: &Self::Point, tolerance: T) -> bool {
        if self.distance_to_point(point) <= tolerance {
            let param = self.parameter_at_point(point);
            param >= -tolerance // 許容誤差を考慮して前方判定
        } else {
            false
        }
    }

    /// レイの起点から指定点までの距離（レイ上の場合のみ）
    fn distance_from_origin(&self, point: &Self::Point) -> Option<T> {
        if self.contains_point_ray(point, T::from_f64(1e-10)) {
            let param = self.parameter_at_point(point);
            if param >= T::ZERO {
                Some(param)
            } else {
                None
            }
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
