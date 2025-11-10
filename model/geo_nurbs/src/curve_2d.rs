//! NURBS 2D曲線実装（メモリ最適化版）
//!
//! Non-Uniform Rational B-Spline 2D curves の基本実装です。
//! フラット配列による高効率メモリ配置で制御点、重み、ノットベクトルを管理します。

use crate::{KnotVector, NurbsError, Result};
use analysis::Scalar;
use geo_primitives::{Point2D, Vector2D};

/// 重み配列の効率的管理（2D曲線用）
#[derive(Debug, Clone)]
pub enum WeightStorage<T: Scalar> {
    /// 非有理曲線（全重み = 1.0）
    Uniform,
    /// 有理曲線（個別重み）
    Individual(Vec<T>),
}

/// NURBS曲線 - 2次元（メモリ最適化版）
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
    /// 新しいNURBS曲線を作成
    ///
    /// # 引数
    /// * `control_points` - 制御点配列
    /// * `weights` - 重み配列（Noneの場合は非有理）
    /// * `knot_vector` - ノットベクトル
    /// * `degree` - NURBS次数
    ///
    /// # エラー
    /// 制御点と重みのサイズが一致しない場合など
    ///
    /// # Errors
    /// * 制御点数が次数+1未満の場合
    /// * ノットベクトルが無効な場合
    /// * 重み配列のサイズが制御点数と一致しない場合
    pub fn new(
        control_points: Vec<Point2D<T>>,
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
    pub fn control_point(&self, index: usize) -> Point2D<T> {
        let base = self.control_point_index(index);
        Point2D::new(self.coordinates[base], self.coordinates[base + 1])
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
    pub fn evaluate_at(&self, t: T) -> Point2D<T> {
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

        Point2D::new(numerator_x / denominator, numerator_y / denominator)
    }

    /// 指定パラメータでの1次導関数を計算
    pub fn derivative_at(&self, t: T) -> Vector2D<T> {
        let h = T::from_f64(1e-8);
        let p1 = self.evaluate_at(t - h);
        let p2 = self.evaluate_at(t + h);

        let dx = (p2.x() - p1.x()) / (h + h);
        let dy = (p2.y() - p1.y()) / (h + h);

        Vector2D::new(dx, dy)
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

            let dx = current_point.x() - prev_point.x();
            let dy = current_point.y() - prev_point.y();
            let segment_length = (dx * dx + dy * dy).sqrt();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nurbs_curve_2d_creation() {
        let control_points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ];
        let weights = Some(vec![1.0, 1.0, 1.0]);
        let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let curve = NurbsCurve2D::new(control_points, weights, knot_vector, degree);
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.degree(), 2);
        assert_eq!(curve.num_points(), 3);
    }

    #[test]
    fn test_curve_evaluation() {
        let control_points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ];
        let weights = Some(vec![1.0, 1.0, 1.0]);
        let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let curve = NurbsCurve2D::new(control_points, weights, knot_vector, degree).unwrap();

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
        let control_points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(2.0, 0.0),
        ];
        let weights = Some(vec![1.0, 1.0, 1.0]);
        let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let curve = NurbsCurve2D::new(control_points, weights, knot_vector, degree).unwrap();
        let length = curve.approximate_length(100);

        // 直線に近い曲線なので長さは約2.0
        assert!((length - 2.0).abs() < 0.1);
    }
}
