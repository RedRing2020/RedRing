//! NURBS 曲線の評価（2D・3D 共通）
//!
//! 制御点の座標を `[x0, y0, (z0,) x1, ...]` と並べた配列から、点・1 階導関数・長さを求める。

use crate::basis::{basis_first_derivatives, basis_functions};
use crate::{constants, WeightStorage};
use geo_contracts::{default_distance_tolerance, NurbsError, Scalar};

/// 定義域 `(start, end)` から距離トレランス分を超えて外れない `t` を定義域に収めて返す（外れる・有限でない場合は `None`）
pub(crate) fn clamp_to_domain<T: Scalar>(t: T, (start, end): (T, T)) -> Option<T> {
    let tolerance = default_distance_tolerance::<T>();
    if !t.is_finite() || t < start - tolerance || t > end + tolerance {
        return None;
    }
    Some(t.max(start).min(end))
}

/// 曲線の生成の入力を検証し、重みの格納方式を返す
pub(crate) fn validate_curve<T: Scalar>(
    degree: usize,
    knots: &[T],
    control_point_count: usize,
    weights: Option<Vec<T>>,
) -> Result<WeightStorage<T>, NurbsError> {
    if control_point_count < degree + 1 {
        return Err(NurbsError::InsufficientControlPoints {
            actual: control_point_count,
            required: degree + 1,
            degree,
        });
    }

    let weights = match weights {
        Some(weights) => {
            if weights.len() != control_point_count {
                return Err(NurbsError::WeightCountMismatch {
                    actual: weights.len(),
                    expected: control_point_count,
                });
            }
            if let Some(index) = weights.iter().position(|&weight| weight <= T::ZERO) {
                return Err(NurbsError::NonPositiveWeight {
                    index,
                    weight: weights[index].to_f64(),
                });
            }
            WeightStorage::Individual(weights)
        }
        None => WeightStorage::Uniform,
    };

    crate::knot::validate_knot_vector(knots, degree, control_point_count)?;
    Ok(weights)
}

/// 曲線の定義（次元 `N` の制御点の座標・重み・ノットベクトル・次数）
pub(crate) struct CurveDefinition<'a, T: Scalar, const N: usize> {
    pub(crate) coordinates: &'a [T],
    pub(crate) weights: &'a WeightStorage<T>,
    pub(crate) knots: &'a [T],
    pub(crate) degree: usize,
}

impl<T: Scalar, const N: usize> CurveDefinition<'_, T, N> {
    /// パラメータの定義域 `(開始, 終了)`
    pub(crate) fn parameter_domain(&self) -> (T, T) {
        crate::knot::get_parameter_domain(self.knots, self.degree)
    }

    /// 定義域から距離トレランス分を超えて外れない `t` を定義域に収めて返す（外れる・有限でない場合は `None`）
    pub(crate) fn clamp_to_domain(&self, t: T) -> Option<T> {
        clamp_to_domain(t, self.parameter_domain())
    }

    /// 定義域内の `t` での点と 1 階導関数
    ///
    /// 同次座標 `A(t) = Σ N_i w_i P_i`、`w(t) = Σ N_i w_i` から、`C = A / w`、`C' = (A' - C w') / w` とする。
    pub(crate) fn point_and_derivative(&self, t: T) -> ([T; N], [T; N]) {
        let span = crate::knot::find_knot_span(t, self.knots, self.degree);
        let basis = basis_functions(span, self.degree, t, self.knots);
        let basis_derivatives = basis_first_derivatives(span, self.degree, t, self.knots);

        let mut numerator = [T::ZERO; N];
        let mut numerator_derivative = [T::ZERO; N];
        let mut denominator = T::ZERO;
        let mut denominator_derivative = T::ZERO;
        for i in 0..=self.degree {
            let index = span - self.degree + i;
            let weight = self.weights.weight(index);
            let value = basis[i] * weight;
            let derivative = basis_derivatives[i] * weight;
            for k in 0..N {
                let coordinate = self.coordinates[index * N + k];
                numerator[k] += value * coordinate;
                numerator_derivative[k] += derivative * coordinate;
            }
            denominator += value;
            denominator_derivative += derivative;
        }

        let mut point = [T::ZERO; N];
        let mut tangent = [T::ZERO; N];
        for k in 0..N {
            point[k] = numerator[k] / denominator;
            tangent[k] =
                (numerator_derivative[k] - point[k] * denominator_derivative) / denominator;
        }
        (point, tangent)
    }

    /// 定義域全体の長さ
    ///
    /// 導関数の大きさを、ノット区間ごとに 5 点のガウス・ルジャンドル求積で積分する。
    /// 区間を 2 等分した結果との差が既定の距離トレランスに収まるまで分割を繰り返す。
    pub(crate) fn length(&self) -> T {
        let speed = |t: T| {
            let (_, tangent) = self.point_and_derivative(t);
            tangent
                .iter()
                .map(|&c| c * c)
                .fold(T::ZERO, |sum, c| sum + c)
                .sqrt()
        };
        let (start, end) = self.parameter_domain();
        let tolerance = default_distance_tolerance::<T>();
        let mut total = T::ZERO;
        for window in self.knots.windows(2) {
            let (a, b) = (window[0].max(start), window[1].min(end));
            if b > a {
                total += analysis::adaptive_gauss_legendre(
                    speed,
                    a,
                    b,
                    tolerance,
                    constants::LENGTH_MAX_BISECTION_DEPTH,
                );
            }
        }
        total
    }
}
