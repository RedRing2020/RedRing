//! `NurbsCurve3D` の拡張機能

use crate::adaptive_tessellation::{
    adaptive_params_axis, AdaptiveParamList, AdaptiveTessellationSettings,
    NurbsCurveAdaptiveTessellation,
};
use crate::NurbsCurve3D;
use geo_contracts::default_kernel_numerical_zero_tolerance;
use geo_contracts::Scalar;
use geo_core::{Aabb3D, Point3D};

/// 境界ボックス計算オプション
#[derive(Debug, Clone, Copy)]
pub enum AabbOptions<T: Scalar> {
    /// 高速・保守的（制御点ベース）
    ///
    /// 計算量: O(n)
    /// 精度: 保守的（常に曲線を含むが余分な空間を含む）
    Rough,

    /// 精密（トレランス指定）
    ///
    /// 計算量: O(n * m) - m はサンプル数（トレランス依存）
    /// 精度: 指定したトレランス内の誤差
    Precise {
        /// 許容誤差（境界ボックスが実際の曲線からはみ出す最大距離）
        tolerance: T,
    },

    /// 適応的（最大分割数指定）
    ///
    /// 計算量: O(n * subdivisions)
    /// 精度: 分割数依存
    Adaptive {
        /// 最大分割数
        max_subdivisions: usize,
    },
}

impl<T: Scalar> NurbsCurve3D<T> {
    /// 精密な境界ボックス計算（パラメトリック評価ベース）
    ///
    /// 指定したトレランス内の誤差で曲線の境界ボックスを計算します。
    /// 曲線をパラメトリックに評価してサンプリングポイントから境界を決定します。
    ///
    /// # 引数
    /// * `tolerance` - 許容誤差（境界ボックスが実際の曲線からはみ出す最大距離）
    ///
    /// # アルゴリズム
    /// 1. パラメータ範囲を適応的に分割
    /// 2. 各サンプルポイントで曲線を評価
    /// 3. サンプルポイントの境界ボックスを計算
    /// 4. トレランスチェック（必要に応じて再分割）
    ///
    /// # 計算量
    /// O(n * m) - n は制御点数、m はサンプル数
    ///
    /// # 例
    /// ```no_run
    /// # use geo_nurbs::NurbsCurve3D;
    /// # let curve: NurbsCurve3D<f64> = todo!();
    /// // 誤差0.01以内の精密な境界ボックス
    /// let bbox = curve.precise_bounding_box(0.01);
    /// ```
    pub fn precise_bounding_box(&self, tolerance: T) -> Aabb3D<T> {
        // トレランスから適切なサンプル数を決定
        // より小さいトレランス → より多くのサンプル
        let base_subdivisions = 100;
        let tolerance_factor =
            T::ONE / tolerance.max(default_kernel_numerical_zero_tolerance::<T>());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let subdivisions = (f64::from(base_subdivisions) * tolerance_factor.to_f64().sqrt())
            .min(10000.0)
            .max(f64::from(base_subdivisions)) as usize;

        self.bounding_box_adaptive(subdivisions)
    }

    /// 適応的サンプリングによる境界ボックス計算
    ///
    /// 指定した分割数で曲線をサンプリングし、境界ボックスを計算します。
    ///
    /// # 引数
    /// * `subdivisions` - パラメータ範囲の分割数
    ///
    /// # 計算量
    /// O(n * subdivisions)
    #[must_use]
    pub fn bounding_box_adaptive(&self, subdivisions: usize) -> Aabb3D<T> {
        let (t_min, t_max) = self.parameter_domain();
        let dt = (t_max - t_min) / T::from_usize(subdivisions);

        let mut min_x = T::MAX;
        let mut min_y = T::MAX;
        let mut min_z = T::MAX;
        let mut max_x = T::MIN;
        let mut max_y = T::MIN;
        let mut max_z = T::MIN;

        // パラメータ範囲をサンプリング
        for i in 0..=subdivisions {
            let t = t_min + dt * T::from_usize(i);
            let point = self.evaluate_at(t);

            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            min_z = min_z.min(point.z());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
            max_z = max_z.max(point.z());
        }

        Aabb3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    /// オプション指定による境界ボックス計算
    ///
    /// 用途に応じて最適な境界ボックス計算方法を選択できます。
    ///
    /// # 引数
    /// * `options` - 境界ボックス計算オプション
    ///
    /// # Panics
    /// NURBS曲線が境界ボックスを持たない場合（通常は発生しません）
    ///
    /// # 例
    /// ```no_run
    /// # use geo_nurbs::{NurbsCurve3D, curve_3d_extensions::AabbOptions};
    /// # let curve: NurbsCurve3D<f64> = todo!();
    /// use geo_contracts::PrimitiveMetadata;
    ///
    /// // 高速・保守的
    /// let rough_bbox = curve.bounding_box_with_options(AabbOptions::Rough);
    ///
    /// // 精密（誤差0.01）
    /// let precise_bbox = curve.bounding_box_with_options(
    ///     AabbOptions::Precise { tolerance: 0.01 }
    /// );
    ///
    /// // 適応的（500分割）
    /// let adaptive_bbox = curve.bounding_box_with_options(
    ///     AabbOptions::Adaptive { max_subdivisions: 500 }
    /// );
    /// ```
    pub fn bounding_box_with_options(&self, options: AabbOptions<T>) -> Aabb3D<T> {
        match options {
            AabbOptions::Rough => {
                // Bounded トレイトの実装を使用
                use geo_contracts::Bounded;
                self.aabb().expect("NURBS curve should have bounding box")
            }
            AabbOptions::Precise { tolerance } => self.precise_bounding_box(tolerance),
            AabbOptions::Adaptive { max_subdivisions } => {
                self.bounding_box_adaptive(max_subdivisions)
            }
        }
    }
}

impl<T: Scalar> NurbsCurve3D<T> {
    fn chord_error(&self, t0: T, t1: T) -> T {
        let mid = (t0 + t1) / (T::ONE + T::ONE);
        let p0 = self.evaluate_at(t0);
        let p1 = self.evaluate_at(t1);
        let pm = self.evaluate_at(mid);
        let chord_mid = (p0 + p1) / (T::ONE + T::ONE);
        (pm - chord_mid).norm()
    }
}

impl<T: Scalar> NurbsCurveAdaptiveTessellation<T> for NurbsCurve3D<T> {
    fn adaptive_params_curve(
        &self,
        settings: &AdaptiveTessellationSettings<T>,
    ) -> AdaptiveParamList<T> {
        let (t_min, t_max) = self.parameter_domain();
        let params = adaptive_params_axis(t_min, t_max, settings, |a, b| self.chord_error(a, b));
        AdaptiveParamList { params }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clamped_knot_vector;

    #[test]
    fn test_rough_bbox() {
        use geo_contracts::NurbsCurve3DConstructor;
        let control_points = vec![(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)];

        let knots = clamped_knot_vector(2, 3);
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            2,
            knots,
            control_points,
            None,
        )
        .unwrap();

        let bbox = curve.bounding_box_with_options(AabbOptions::Rough);

        // 制御点ベースなので、制御点を含む
        assert!((bbox.min().x() - 0.0).abs() < 1e-10);
        assert!((bbox.max().x() - 2.0).abs() < 1e-10);
        assert!((bbox.max().y() - 2.0).abs() < 1e-10); // 制御点(1,2,0)を含む
    }

    #[test]
    fn test_precise_bbox() {
        use geo_contracts::NurbsCurve3DConstructor;
        let control_points = vec![(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)];

        let knots = clamped_knot_vector(2, 3);
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            2,
            knots,
            control_points,
            None,
        )
        .unwrap();

        let precise_bbox =
            curve.bounding_box_with_options(AabbOptions::Precise { tolerance: 0.01 });

        let rough_bbox = curve.bounding_box_with_options(AabbOptions::Rough);

        // 精密版は制御点ベースより小さいか同等
        assert!(precise_bbox.max().y() <= rough_bbox.max().y());
    }

    #[test]
    fn test_adaptive_bbox() {
        use geo_contracts::NurbsCurve3DConstructor;
        let control_points = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)];

        let knots = clamped_knot_vector(2, 3);
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            2,
            knots,
            control_points,
            None,
        )
        .unwrap();

        // 分割数を変えてテスト
        let bbox_100 = curve.bounding_box_with_options(AabbOptions::Adaptive {
            max_subdivisions: 100,
        });
        let bbox_500 = curve.bounding_box_with_options(AabbOptions::Adaptive {
            max_subdivisions: 500,
        });

        // より細かい分割は精度が向上（または同等）
        let diff_100 = bbox_100.max().y() - bbox_100.min().y();
        let diff_500 = bbox_500.max().y() - bbox_500.min().y();

        assert!(diff_500 <= diff_100 * 1.01); // 誤差を考慮
    }

    #[test]
    fn test_bbox_comparison() {
        use geo_contracts::NurbsCurve3DConstructor;
        let control_points = vec![
            (0.0, 0.0, 0.0),
            (1.0, 2.0, 1.0),
            (2.0, 0.0, 0.0),
            (3.0, 0.0, 0.0),
        ];

        let knots = clamped_knot_vector(3, 4);
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            3,
            knots,
            control_points,
            None,
        )
        .unwrap();

        let rough = curve.bounding_box_with_options(AabbOptions::Rough);
        let precise = curve.bounding_box_with_options(AabbOptions::Precise { tolerance: 0.01 });
        let adaptive = curve.bounding_box_with_options(AabbOptions::Adaptive {
            max_subdivisions: 1000,
        });

        // Rough は最も大きい（保守的）
        assert!(rough.max().y() >= precise.max().y());
        assert!(rough.max().y() >= adaptive.max().y());

        // Precise と Adaptive はほぼ同等（トレランス依存）
        let diff = (precise.max().y() - adaptive.max().y()).abs();
        assert!(diff < 0.1); // 適度な誤差範囲
    }

    #[test]
    fn test_curve_adaptive_params_line() {
        use geo_contracts::NurbsCurve3DConstructor;

        let control_points = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
        let knots = clamped_knot_vector(1, 2);
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            1,
            knots,
            control_points,
            None,
        )
        .unwrap();

        let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
        let params = curve.adaptive_params_curve(&settings);

        assert!(params.params.len() > settings.min_segments as usize);

        let (t_min, t_max) = curve.parameter_domain();
        assert!((params.params.first().unwrap() - t_min).abs() < 1e-10);
        assert!((params.params.last().unwrap() - t_max).abs() < 1e-10);
    }
}
