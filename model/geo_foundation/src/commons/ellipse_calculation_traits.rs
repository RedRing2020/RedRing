//! Ellipse Calculation Implementation - geo_commons を使用した実装
//!
//! EllipseCalculation trait の具体実装を geo_commons 関数で提供

use crate::Scalar;

/// 楕円計算機能の統一インターフェース
pub trait EllipseCalculation<T: Scalar> {
    /// 楕円の長半径を取得
    fn semi_major_axis(&self) -> T;
    /// 楕円の短半径を取得  
    fn semi_minor_axis(&self) -> T;

    /// ラマヌジャン近似式I（標準版）による周長計算
    fn perimeter_ramanujan_i(&self) -> T {
        geo_commons::ellipse_perimeter_ramanujan_i(self.semi_major_axis(), self.semi_minor_axis())
    }

    /// ラマヌジャン近似式II（高精度版）による周長計算
    fn perimeter_ramanujan_ii(&self) -> T {
        geo_commons::ellipse_perimeter_ramanujan_ii(self.semi_major_axis(), self.semi_minor_axis())
    }

    /// パダン近似による周長計算（中程度精度）
    fn perimeter_pade(&self) -> T {
        geo_commons::ellipse_perimeter_padé(self.semi_major_axis(), self.semi_minor_axis())
    }

    /// カントレル近似による周長計算（高精度）
    fn perimeter_cantrell(&self) -> T {
        geo_commons::ellipse_perimeter_cantrell(self.semi_major_axis(), self.semi_minor_axis())
    }

    /// 無限級数展開による周長計算（高精度版）
    fn perimeter_series(&self, terms: usize) -> T {
        geo_commons::ellipse_circumference_series(
            self.semi_major_axis(),
            self.semi_minor_axis(),
            terms,
        )
    }

    /// 数値積分による周長計算（最高精度版）
    fn perimeter_numerical(&self, n_points: usize) -> T {
        geo_commons::ellipse_circumference_numerical(
            self.semi_major_axis(),
            self.semi_minor_axis(),
            n_points,
        )
    }

    /// 楕円の離心率計算
    fn eccentricity(&self) -> T {
        geo_commons::ellipse_eccentricity(self.semi_major_axis(), self.semi_minor_axis())
    }

    /// 楕円の焦点距離計算
    fn focal_distance(&self) -> T {
        geo_commons::ellipse_focal_distance(self.semi_major_axis(), self.semi_minor_axis())
    }

    /// 楕円の面積計算
    fn area(&self) -> T {
        geo_commons::metrics::area_volume::ellipse_area(
            self.semi_major_axis(),
            self.semi_minor_axis(),
        )
    }

    /// 楕円の焦点座標を計算
    fn foci(&self) -> (Self::Point, Self::Point);

    /// 点の型（2Dまたは3D）
    type Point;
}

/// 楕円の適応的周長計算
pub trait EllipseAdaptiveCalculation<T: Scalar>: EllipseCalculation<T> {
    /// 適応的周長計算
    fn perimeter_adaptive(&self, tolerance: T, max_computation_cost: T) -> T {
        if tolerance >= T::from_f64(1e-3) {
            self.perimeter_pade()
        } else if tolerance >= T::from_f64(1e-6) {
            self.perimeter_ramanujan_i()
        } else if tolerance >= T::from_f64(1e-9) {
            self.perimeter_ramanujan_ii()
        } else if max_computation_cost >= T::from_f64(2.0) {
            self.perimeter_series(20)
        } else {
            self.perimeter_cantrell()
        }
    }
}

/// 楕円の精度評価機能
pub trait EllipseAccuracyAnalysis<T: Scalar>: EllipseCalculation<T> {
    /// 各種近似手法の精度比較
    fn compare_approximation_methods(&self) -> Vec<(&'static str, T, T)> {
        let numerical_ref = self.perimeter_numerical(1000);

        vec![
            (
                "Ramanujan I",
                self.perimeter_ramanujan_i(),
                ((self.perimeter_ramanujan_i() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Ramanujan II",
                self.perimeter_ramanujan_ii(),
                ((self.perimeter_ramanujan_ii() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Pade",
                self.perimeter_pade(),
                ((self.perimeter_pade() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Cantrell",
                self.perimeter_cantrell(),
                ((self.perimeter_cantrell() - numerical_ref) / numerical_ref).abs(),
            ),
            (
                "Series(20)",
                self.perimeter_series(20),
                ((self.perimeter_series(20) - numerical_ref) / numerical_ref).abs(),
            ),
        ]
    }
}
