//! NURBS 2D曲線実装（メモリ最適化版）
//!
//! Non-Uniform Rational B-Spline 2D curves の基本実装です。
//! フラット配列による高効率メモリ配置で制御点、重み、ノットベクトルを管理します。

use crate::{constants, KnotVector, NurbsError, Result, Scalar};
use analysis::linalg::vector::Vector2;
use geo_contracts::{
    NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DMeasure, NurbsCurve2DProperties,
};

/// 重み配列の効率的管理（2D曲線用）
#[derive(Debug, Clone)]
pub enum WeightStorage<T: Scalar> {
    /// 非有理曲線（全重み = 1.0）
    Uniform,
    /// 有理曲線（個別重み）
    Individual(Vec<T>),
}

/// NURBS曲線 - 2次元（Foundation パターン準拠・ジェネリック）
///
/// # 特徴
/// - フラット配列による高効率メモリ配置
/// - 有理ベーシス関数による滑らかな補間
/// - 局所的制御性（制御点の変更が局所的影響のみ）
/// - アフィン変換の不変性
///
/// # メモリ構造
/// - 座標: `[x0,y0, x1,y1, x2,y2, ...]` (フラット配列)
/// - インデックス: `i * 2 + coord_offset`
#[derive(Debug, Clone)]
pub struct NurbsCurve2D<T: Scalar> {
    /// フラット座標配列 [x0,y0, x1,y1, ...]
    coordinates: Vec<T>,
    /// 効率的重み管理
    weights: WeightStorage<T>,
    /// ノットベクトル
    knot_vector: KnotVector<T>,
    /// NURBS次数
    degree: usize,
    /// 制御点数（高速化のため保持）
    num_points: usize,
}

impl<T: Scalar> NurbsCurve2D<T> {
    /// 内部用コンストラクタ（クレート内専用）
    ///
    /// # 引数
    /// * `control_points` - 制御点座標配列 [(x, y), ...]
    /// * `weights` - 重み配列（Noneの場合は非有理）
    /// * `knot_vector` - ノットベクトル
    /// * `degree` - NURBS次数
    ///
    /// # Errors
    /// * 制御点数が次数+1未満の場合
    /// * ノットベクトルが無効な場合
    /// * 重み配列のサイズが制御点数と一致しない場合
    pub(crate) fn new_internal(
        control_points: &[Vector2<T>],
        weights: Option<Vec<T>>,
        knot_vector: KnotVector<T>,
        degree: usize,
    ) -> Result<Self> {
        let num_points = control_points.len();

        // 基本的なバリデーション
        if num_points < degree + 1 {
            return Err(NurbsError::InsufficientControlPoints {
                actual: num_points,
                required: degree + 1,
                degree,
            });
        }

        // フラット座標配列を構築
        let mut coordinates = Vec::with_capacity(num_points * 2);
        for point in control_points {
            coordinates.push(point.x());
            coordinates.push(point.y());
        }

        // 重み配列を処理
        let weight_storage = if let Some(weight_vec) = weights {
            if weight_vec.len() != num_points {
                return Err(NurbsError::WeightCountMismatch {
                    actual: weight_vec.len(),
                    expected: num_points,
                });
            }

            // 重みの検証
            for &weight in &weight_vec {
                if weight <= T::ZERO {
                    return Err(NurbsError::InvalidWeight {
                        weight: weight.to_f64(),
                    });
                }
            }

            WeightStorage::Individual(weight_vec)
        } else {
            WeightStorage::Uniform
        };

        // ノットベクトルの検証
        crate::knot::validate_knot_vector(&knot_vector, degree, num_points)?;

        Ok(NurbsCurve2D {
            coordinates,
            weights: weight_storage,
            knot_vector,
            degree,
            num_points,
        })
    }

    /// 制御点アクセス用インデックス計算
    #[inline]
    fn control_point_index(&self, index: usize) -> usize {
        debug_assert!(index < self.num_points);
        index * 2
    }

    /// 制御点取得
    #[must_use]
    pub fn control_point(&self, index: usize) -> Vector2<T> {
        let base = self.control_point_index(index);
        Vector2::new(self.coordinates[base], self.coordinates[base + 1])
    }

    /// 重み取得
    #[must_use]
    pub fn weight(&self, index: usize) -> T {
        match &self.weights {
            WeightStorage::Uniform => T::ONE,
            WeightStorage::Individual(weights) => weights[index],
        }
    }

    /// 制御点数を取得
    #[must_use]
    pub fn num_points(&self) -> usize {
        self.num_points
    }

    /// フラット座標配列への参照を取得
    #[must_use]
    pub fn coordinates(&self) -> &[T] {
        &self.coordinates
    }

    /// 重みストレージへの参照を取得
    #[must_use]
    pub fn weights(&self) -> &WeightStorage<T> {
        &self.weights
    }

    /// ノットベクトルを取得
    #[must_use]
    pub fn knot_vector(&self) -> &KnotVector<T> {
        &self.knot_vector
    }

    /// 次数を取得
    #[must_use]
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// パラメータ定義域を取得
    #[must_use]
    pub fn parameter_domain(&self) -> (T, T) {
        crate::knot::get_parameter_domain(&self.knot_vector, self.degree)
    }

    /// 指定パラメータでの曲線上の点を計算
    pub fn evaluate_at(&self, t: T) -> Vector2<T> {
        let span = crate::knot::find_knot_span(t, &self.knot_vector, self.degree);
        let basis = self.compute_basis_functions(t, span);

        let mut numerator_x = T::ZERO;
        let mut numerator_y = T::ZERO;
        let mut denominator = T::ZERO;

        for (i, &basis_value) in basis.iter().enumerate().take(self.degree + 1) {
            let control_index = span - self.degree + i;
            if control_index < self.num_points {
                let control_point = self.control_point(control_index);
                let weight = self.weight(control_index);
                let basis_weight = basis_value * weight;

                numerator_x += control_point.x() * basis_weight;
                numerator_y += control_point.y() * basis_weight;
                denominator += basis_weight;
            }
        }

        Vector2::new(numerator_x / denominator, numerator_y / denominator)
    }

    /// 指定パラメータでの1次導関数を計算
    pub fn derivative_at(&self, t: T) -> Vector2<T> {
        let h = T::from_f64(constants::DERIVATIVE_STEP);
        let p1 = self.evaluate_at(t - h);
        let p2 = self.evaluate_at(t + h);

        let dx = (p2.x() - p1.x()) / (h + h);
        let dy = (p2.y() - p1.y()) / (h + h);

        Vector2::new(dx, dy)
    }

    /// 曲線の長さを近似計算
    #[must_use]
    pub fn approximate_length(&self, subdivisions: usize) -> T {
        if subdivisions == 0 {
            return T::ZERO;
        }

        let (t_min, t_max) = self.parameter_domain();
        let dt = (t_max - t_min) / T::from_usize(subdivisions);

        let mut total_length = T::ZERO;
        let mut prev_point = self.evaluate_at(t_min);

        for i in 1..=subdivisions {
            let t = t_min + dt * T::from_usize(i);
            let current_point = self.evaluate_at(t);

            let segment_length = (current_point - prev_point).norm();

            total_length += segment_length;
            prev_point = current_point;
        }

        total_length
    }

    /// B-スプライン基底関数を計算
    fn compute_basis_functions(&self, t: T, span: usize) -> Vec<T> {
        let mut basis = vec![T::ZERO; self.degree + 1];
        let mut left = vec![T::ZERO; self.degree + 1];
        let mut right = vec![T::ZERO; self.degree + 1];

        basis[0] = T::ONE;

        for j in 1..=self.degree {
            left[j] = t - self.knot_vector[span + 1 - j];
            right[j] = self.knot_vector[span + j] - t;

            let mut saved = T::ZERO;
            for r in 0..j {
                let temp = basis[r] / (right[r + 1] + left[j - r]);
                basis[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }
            basis[j] = saved;
        }

        basis
    }
}

// ============================================================================
// Core Traits 実装
// ============================================================================

impl<T: Scalar> NurbsCurve2DConstructor<T> for NurbsCurve2D<T> {
    fn new(
        control_points: &[(T, T)],
        weights: Option<Vec<T>>,
        knot_vector: Vec<T>,
        degree: usize,
    ) -> std::result::Result<Self, String> {
        // (x,y)タプルをVector2に変換
        let points: Vec<Vector2<T>> = control_points
            .iter()
            .map(|&(x, y)| Vector2::new(x, y))
            .collect();

        Self::new_internal(&points, weights, knot_vector, degree).map_err(|e| e.to_string())
    }

    fn from_control_points(
        control_points: &[(T, T)],
        degree: usize,
    ) -> std::result::Result<Self, String> {
        if control_points.len() < degree + 1 {
            return Err(format!(
                "Insufficient control points: {} < {}",
                control_points.len(),
                degree + 1
            ));
        }

        // クランプド・ノットベクトルを生成
        let knots = crate::knot::clamped_knot_vector(degree, control_points.len());

        <Self as NurbsCurve2DConstructor<T>>::new(control_points, None, knots, degree)
    }

    fn unit_line() -> Self {
        let control_points = &[(T::ZERO, T::ZERO), (T::ONE, T::ZERO)];
        let degree = 1;
        let knots = vec![T::ZERO, T::ZERO, T::ONE, T::ONE];

        <Self as NurbsCurve2DConstructor<T>>::new(control_points, None, knots, degree)
            .expect("Unit line creation should never fail")
    }
}

impl<T: Scalar> NurbsCurve2DProperties<T> for NurbsCurve2D<T> {
    fn degree(&self) -> usize {
        self.degree
    }

    fn num_control_points(&self) -> usize {
        self.num_points
    }

    fn knot_vector(&self) -> &[T] {
        &self.knot_vector
    }

    fn is_rational(&self) -> bool {
        matches!(self.weights, WeightStorage::Individual(_))
    }
}

impl<T: Scalar> NurbsCurve2DMeasure<T> for NurbsCurve2D<T> {
    fn point_at(&self, t: T) -> (T, T) {
        let point = self.evaluate_at(t);
        (point.x(), point.y())
    }

    fn tangent_at(&self, t: T) -> (T, T) {
        let derivative = self.derivative_at(t);
        let len_sq = derivative.x() * derivative.x() + derivative.y() * derivative.y();
        let len = len_sq.sqrt();

        if len.is_zero() {
            (T::ZERO, T::ZERO)
        } else {
            (derivative.x() / len, derivative.y() / len)
        }
    }

    fn length(&self) -> T {
        self.approximate_length(constants::CURVE_LENGTH_SUBDIVISIONS)
    }

    #[allow(clippy::similar_names)]
    fn curvature_at(&self, t: T) -> T {
        let h = T::from_f64(constants::DERIVATIVE_STEP);
        let d1 = self.derivative_at(t);
        let d2_plus = self.derivative_at(t + h);
        let d2_minus = self.derivative_at(t - h);

        // 2次導関数の中央差分近似
        let d2x = (d2_plus.x() - d2_minus.x()) / (h + h);
        let d2y = (d2_plus.y() - d2_minus.y()) / (h + h);

        // 曲率 = |x'y'' - y'x''| / (x'^2 + y'^2)^(3/2)
        let numerator = (d1.x() * d2y - d1.y() * d2x).abs();
        let denominator = (d1.x() * d1.x() + d1.y() * d1.y()).powf(T::from_f64(1.5));

        if denominator.is_zero() {
            T::ZERO
        } else {
            numerator / denominator
        }
    }
}

impl<T: Scalar> NurbsCurve2DCore<T> for NurbsCurve2D<T> {}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nurbs_curve_2d_creation() {
        use geo_contracts::NurbsCurve2DConstructor;
        let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let weights = Some(vec![1.0, 1.0, 1.0]);
        let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
            control_points,
            weights,
            knot_vector,
            degree,
        );
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.degree(), 2);
        assert_eq!(curve.num_points(), 3);
    }

    #[test]
    fn test_curve_evaluation() {
        use geo_contracts::NurbsCurve2DConstructor;
        let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let weights = Some(vec![1.0, 1.0, 1.0]);
        let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
            control_points,
            weights,
            knot_vector,
            degree,
        )
        .unwrap();

        // 開始点と終了点のテスト
        let start_point = curve.evaluate_at(0.0);
        let end_point = curve.evaluate_at(1.0);

        // 開始点は最初の制御点に近い
        assert!((start_point.x() - 0.0).abs() < 1e-10);
        assert!((start_point.y() - 0.0).abs() < 1e-10);

        // 終了点は最後の制御点に近い
        assert!((end_point.x() - 2.0).abs() < 1e-10);
        assert!((end_point.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_approximate_length() {
        use geo_contracts::NurbsCurve2DConstructor;
        let control_points = &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
        let weights = Some(vec![1.0, 1.0, 1.0]);
        let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
            control_points,
            weights,
            knot_vector,
            degree,
        )
        .unwrap();
        let length = curve.approximate_length(100);

        // 直線に近い曲線なので長さは約2.0
        assert!((length - 2.0).abs() < 0.1);
    }
}
