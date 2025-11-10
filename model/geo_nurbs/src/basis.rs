//! B-スプライン基底関数計算
//!
//! Cox-de Boorの再帰公式を使用したB-スプライン基底関数とその導関数の計算を提供します。

use crate::KnotVector;
use analysis::Scalar;

/// B-スプライン基底関数を計算（Cox-de Boor再帰公式）
///
/// # 引数
/// * `i` - 基底関数のインデックス
/// * `degree` - 次数
/// * `t` - パラメータ値
/// * `knots` - ノットベクトル
///
/// # 戻り値
/// B-スプライン基底関数値 N_{i,p}(t)
pub fn basis_function<T: Scalar>(i: usize, degree: usize, t: T, knots: &KnotVector<T>) -> T {
    if degree == 0 {
        // 0次基底関数（特性関数）
        if i < knots.len() - 1 && t >= knots[i] && t < knots[i + 1] {
            T::ONE
        } else {
            T::ZERO
        }
    } else {
        // 高次基底関数の再帰計算
        let mut left_term = T::ZERO;
        let mut right_term = T::ZERO;

        // 左側の項
        let left_denom = knots[i + degree] - knots[i];
        if !left_denom.is_zero() {
            let left_basis = basis_function(i, degree - 1, t, knots);
            left_term = (t - knots[i]) * left_basis / left_denom;
        }

        // 右側の項
        if i + degree + 1 < knots.len() {
            let right_denom = knots[i + degree + 1] - knots[i + 1];
            if !right_denom.is_zero() {
                let right_basis = basis_function(i + 1, degree - 1, t, knots);
                right_term = (knots[i + degree + 1] - t) * right_basis / right_denom;
            }
        }

        left_term + right_term
    }
}

/// 指定スパンでの非ゼロ基底関数群を計算（効率的版）
///
/// # 引数
/// * `span` - ノットスパン
/// * `degree` - 次数
/// * `t` - パラメータ値
/// * `knots` - ノットベクトル
///
/// # 戻り値
/// 非ゼロ基底関数値の配列（長さ = degree + 1）
pub fn basis_functions<T: Scalar>(
    span: usize,
    degree: usize,
    t: T,
    knots: &KnotVector<T>,
) -> Vec<T> {
    let mut basis = vec![T::ZERO; degree + 1];
    let mut left = vec![T::ZERO; degree + 1];
    let mut right = vec![T::ZERO; degree + 1];

    basis[0] = T::ONE;

    for j in 1..=degree {
        left[j] = t - knots[span + 1 - j];
        right[j] = knots[span + j] - t;

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

/// 基底関数の導関数を計算
///
/// # 引数
/// * `span` - ノットスパン
/// * `degree` - 次数
/// * `t` - パラメータ値
/// * `knots` - ノットベクトル
/// * `derivative_order` - 導関数の次数
///
/// # 戻り値
/// `基底関数とその導関数の値（derivative_order次まで`）
/// 戻り値[k][i] = N_{span-degree+i,degree}の k次導関数
pub fn basis_derivatives<T: Scalar>(
    span: usize,
    degree: usize,
    t: T,
    knots: &KnotVector<T>,
    derivative_order: usize,
) -> Vec<Vec<T>> {
    let n = derivative_order;
    let p = degree;

    // 結果の配列を初期化
    let mut derivatives = vec![vec![T::ZERO; p + 1]; n + 1];

    // 0次導関数（基底関数そのもの）
    derivatives[0] = basis_functions(span, degree, t, knots);

    if n == 0 {
        return derivatives;
    }

    // 高次導関数の計算
    for k in 1..=n {
        for i in 0..=p {
            let left_index = span - p + i;
            let right_index = left_index + 1;

            let mut left_term = T::ZERO;
            let mut right_term = T::ZERO;

            // 左側の項
            if left_index + p < knots.len() {
                let denom = knots[left_index + p] - knots[left_index];
                if !denom.is_zero() && i > 0 {
                    left_term = T::from_usize(p) * derivatives[k - 1][i - 1] / denom;
                }
            }

            // 右側の項
            if right_index + p < knots.len() {
                let denom = knots[right_index + p] - knots[right_index];
                if !denom.is_zero() && i < p {
                    right_term = T::from_usize(p) * derivatives[k - 1][i + 1] / denom;
                }
            }

            derivatives[k][i] = left_term - right_term;
        }
    }

    derivatives
}

/// 有理基底関数（NURBS基底関数）を計算
///
/// # 引数
/// * `span` - ノットスパン
/// * `degree` - 次数
/// * `t` - パラメータ値
/// * `knots` - ノットベクトル
/// * `weights` - 重み配列
///
/// # 戻り値
/// 有理基底関数値の配列
pub fn rational_basis_functions<T: Scalar>(
    span: usize,
    degree: usize,
    t: T,
    knots: &KnotVector<T>,
    weights: &[T],
) -> Vec<T> {
    let basis = basis_functions(span, degree, t, knots);
    let mut rational_basis = vec![T::ZERO; degree + 1];

    // 重み付き基底関数の合計を計算
    let mut weight_sum = T::ZERO;
    #[allow(clippy::needless_range_loop)]
    for i in 0..=degree {
        let control_index = span - degree + i;
        if control_index < weights.len() {
            weight_sum += basis[i] * weights[control_index];
        }
    }

    // 正規化された有理基底関数を計算
    if !weight_sum.is_zero() {
        #[allow(clippy::needless_range_loop)]
        for i in 0..=degree {
            let control_index = span - degree + i;
            if control_index < weights.len() {
                rational_basis[i] = basis[i] * weights[control_index] / weight_sum;
            }
        }
    }

    rational_basis
}

/// 有理基底関数の導関数を計算
///
/// # 引数
/// * `span` - ノットスパン
/// * `degree` - 次数
/// * `t` - パラメータ値
/// * `knots` - ノットベクトル
/// * `weights` - 重み配列
/// * `derivative_order` - 導関数の次数
///
/// # 戻り値
/// 有理基底関数の導関数の配列
pub fn rational_basis_derivatives<T: Scalar>(
    span: usize,
    degree: usize,
    t: T,
    knots: &KnotVector<T>,
    weights: &[T],
    derivative_order: usize,
) -> Vec<Vec<T>> {
    let basis_derivs = basis_derivatives(span, degree, t, knots, derivative_order);
    let mut rational_derivs = vec![vec![T::ZERO; degree + 1]; derivative_order + 1];

    // 0次導関数（有理基底関数そのもの）
    rational_derivs[0] = rational_basis_functions(span, degree, t, knots, weights);

    if derivative_order == 0 {
        return rational_derivs;
    }

    // 高次導関数の計算（商の微分公式を使用）
    for k in 1..=derivative_order {
        for i in 0..=degree {
            let control_index = span - degree + i;
            if control_index >= weights.len() {
                continue;
            }

            let weight = weights[control_index];
            let mut numerator = basis_derivs[k][i] * weight;

            // 重み付き基底関数の合計とその導関数
            let mut weighted_sum = T::ZERO;
            let mut weighted_sum_deriv = T::ZERO;

            #[allow(clippy::needless_range_loop)] // NURBS基底関数計算の標準アルゴリズム
            for j in 0..=degree {
                let j_index = span - degree + j;
                if j_index < weights.len() {
                    weighted_sum += basis_derivs[0][j] * weights[j_index];
                    weighted_sum_deriv += basis_derivs[k][j] * weights[j_index];
                }
            }

            if !weighted_sum.is_zero() {
                // 商の微分公式: (f/g)' = (f'g - fg')/g²
                let denominator = weighted_sum * weighted_sum;
                numerator =
                    numerator * weighted_sum - basis_derivs[0][i] * weight * weighted_sum_deriv;
                rational_derivs[k][i] = numerator / denominator;
            }
        }
    }

    rational_derivs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basis_function_degree_0() {
        let knots = vec![0.0, 1.0, 2.0, 3.0];

        // N_0,0(0.5) should be 1
        assert!((basis_function(0, 0, 0.5, &knots) - 1.0).abs() < 1e-10);

        // N_1,0(0.5) should be 0
        assert!(basis_function(1, 0, 0.5, &knots).abs() < 1e-10);
    }

    #[test]
    fn test_basis_functions() {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;
        let span = 2;
        let t = 0.5;

        let basis = basis_functions(span, degree, t, &knots);

        // 基底関数の和は1になるはず
        let sum: f64 = basis.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rational_basis_functions() {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let weights = vec![1.0, 1.0, 1.0];
        let degree = 2;
        let span = 2;
        let t = 0.5;

        let rational_basis = rational_basis_functions(span, degree, t, &knots, &weights);

        // 有理基底関数の和は1になるはず
        let sum: f64 = rational_basis.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }
}
