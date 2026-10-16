//! B-スプライン基底関数計算
//!
//! ノット区間ごとの非ゼロの B-スプライン基底関数と、その 1 階導関数を計算する。

use analysis::Scalar;

/// 指定ノットスパンにおける非ゼロ基底関数を計算
///
/// # 引数
/// * `span` - ノットスパン
/// * `degree` - 次数
/// * `t` - パラメータ値
/// * `knots` - ノット列
///
/// # 戻り値
/// 非ゼロ基底関数値の配列（長さ = degree + 1）。`[i]` は `N_{span-degree+i, degree}(t)`
pub(crate) fn basis_functions<T: Scalar>(span: usize, degree: usize, t: T, knots: &[T]) -> Vec<T> {
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

/// 指定ノットスパンにおける非ゼロ基底関数の 1 階導関数を計算
///
/// `N'_{i,p} = p / (u_{i+p} - u_i) * N_{i,p-1} - p / (u_{i+p+1} - u_{i+1}) * N_{i+1,p-1}`
///
/// # 戻り値
/// 長さ = degree + 1 の配列。`[i]` は `N'_{span-degree+i, degree}(t)`
pub(crate) fn basis_first_derivatives<T: Scalar>(
    span: usize,
    degree: usize,
    t: T,
    knots: &[T],
) -> Vec<T> {
    let mut derivatives = vec![T::ZERO; degree + 1];
    if degree == 0 {
        return derivatives;
    }

    // 1 つ低い次数の基底関数。`[j]` は `N_{span-degree+1+j, degree-1}`
    let lower = basis_functions(span, degree - 1, t, knots);
    let p = T::from_usize(degree);
    for (i, derivative) in derivatives.iter_mut().enumerate() {
        let index = span - degree + i;
        let left = if i > 0 {
            let denominator = knots[index + degree] - knots[index];
            if denominator.is_zero() {
                T::ZERO
            } else {
                lower[i - 1] / denominator
            }
        } else {
            T::ZERO
        };
        let right = if i < degree {
            let denominator = knots[index + degree + 1] - knots[index + 1];
            if denominator.is_zero() {
                T::ZERO
            } else {
                lower[i] / denominator
            }
        } else {
            T::ZERO
        };
        *derivative = p * (left - right);
    }

    derivatives
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basis_functions() {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let basis = basis_functions(2, 2, 0.5, &knots);

        // 2 次 Bézier の基底 (1-t)², 2t(1-t), t²
        let expected = [0.25, 0.5, 0.25];
        for (value, expected) in basis.iter().zip(expected) {
            assert!((value - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn test_basis_first_derivatives() {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let t = 0.3;
        let derivatives = basis_first_derivatives(2, 2, t, &knots);

        // (1-t)², 2t(1-t), t² の導関数 -2(1-t), 2 - 4t, 2t
        let expected = [-2.0 * (1.0 - t), 2.0 - 4.0 * t, 2.0 * t];
        for (value, expected) in derivatives.iter().zip(expected) {
            assert!((value - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn test_basis_first_derivatives_with_interior_knot() {
        // 内部ノットを持つ 2 次の基底関数の導関数を、差分と比べる
        let knots = vec![0.0, 0.0, 0.0, 0.4, 1.0, 1.0, 1.0];
        let (span, t, h) = (3, 0.7, 1e-6);
        let derivatives = basis_first_derivatives(span, 2, t, &knots);
        let ahead = basis_functions(span, 2, t + h, &knots);
        let behind = basis_functions(span, 2, t - h, &knots);
        for i in 0..3 {
            let difference = (ahead[i] - behind[i]) / (2.0 * h);
            assert!((derivatives[i] - difference).abs() < 1e-6);
        }
    }
}
