//! B-スプライン基底関数計算
//!
//! ノット区間ごとの非ゼロの B-スプライン基底関数と、その導関数を計算する。

use analysis::Scalar;

/// 指定ノットスパンにおける非ゼロ基底関数と、その `order` 階までの導関数（The NURBS Book の A2.3）
///
/// # 引数
/// * `span` - ノットスパン
/// * `degree` - 次数
/// * `t` - パラメータ値
/// * `knots` - ノット列
/// * `order` - 求める導関数の最大の階数
///
/// # 戻り値
/// `[k][i]` は `N_{span-degree+i, degree}(t)` の `k` 階導関数（`k = 0..=order`。`[0]` は基底関数の値）。
/// 次数を超える階の導関数は 0
pub(crate) fn basis_derivatives<T: Scalar>(
    span: usize,
    degree: usize,
    t: T,
    knots: &[T],
    order: usize,
) -> Vec<Vec<T>> {
    // 上三角 `table[r][j]`（r <= j）は j 次の基底関数の値、下三角 `table[j][r]`（r < j）はノットの差
    let mut table = vec![vec![T::ZERO; degree + 1]; degree + 1];
    let mut left = vec![T::ZERO; degree + 1];
    let mut right = vec![T::ZERO; degree + 1];
    table[0][0] = T::ONE;
    for j in 1..=degree {
        left[j] = t - knots[span + 1 - j];
        right[j] = knots[span + j] - t;
        let mut saved = T::ZERO;
        for r in 0..j {
            table[j][r] = right[r + 1] + left[j - r];
            let temp = table[r][j - 1] / table[j][r];
            table[r][j] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        table[j][j] = saved;
    }

    let mut derivatives = vec![vec![T::ZERO; degree + 1]; order + 1];
    for (r, value) in derivatives[0].iter_mut().enumerate() {
        *value = table[r][degree];
    }

    // 低い次数の基底関数の線形結合の係数。k - 1 階と k 階の係数を交互に保持する
    let highest = order.min(degree);
    let mut coefficients = [vec![T::ZERO; degree + 1], vec![T::ZERO; degree + 1]];
    for r in 0..=degree {
        let (mut previous, mut current) = (0, 1);
        coefficients[previous][0] = T::ONE;
        for k in 1..=highest {
            let lower_degree = degree - k;
            let mut derivative = T::ZERO;
            if r >= k {
                coefficients[current][0] =
                    coefficients[previous][0] / table[lower_degree + 1][r - k];
                derivative = coefficients[current][0] * table[r - k][lower_degree];
            }
            let first = if r + 1 >= k { 1 } else { k - r };
            let last = if r <= lower_degree + 1 {
                k - 1
            } else {
                degree - r
            };
            for j in first..=last {
                let index = r + j - k;
                coefficients[current][j] = (coefficients[previous][j]
                    - coefficients[previous][j - 1])
                    / table[lower_degree + 1][index];
                derivative += coefficients[current][j] * table[index][lower_degree];
            }
            if r <= lower_degree {
                coefficients[current][k] =
                    -coefficients[previous][k - 1] / table[lower_degree + 1][r];
                derivative += coefficients[current][k] * table[r][lower_degree];
            }
            derivatives[k][r] = derivative;
            std::mem::swap(&mut previous, &mut current);
        }
    }

    // k 階の導関数に degree! / (degree - k)! を掛ける
    let mut factor = T::from_usize(degree);
    for (k, row) in derivatives.iter_mut().enumerate().take(highest + 1).skip(1) {
        for value in row.iter_mut() {
            *value *= factor;
        }
        factor *= T::from_usize(degree - k);
    }
    derivatives
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basis_derivatives_of_bezier() {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let t = 0.3;
        let derivatives = basis_derivatives(2, 2, t, &knots, 3);

        // 2 次 Bézier の基底 (1-t)², 2t(1-t), t² と、その 1 階・2 階・3 階の導関数
        let expected = [
            [(1.0 - t).powi(2), 2.0 * t * (1.0 - t), t * t],
            [-2.0 * (1.0 - t), 2.0 - 4.0 * t, 2.0 * t],
            [2.0, -4.0, 2.0],
            [0.0, 0.0, 0.0],
        ];
        for (row, expected_row) in derivatives.iter().zip(expected) {
            for (value, expected) in row.iter().zip(expected_row) {
                assert!((value - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn test_basis_derivatives_with_interior_knot() {
        // 内部ノットを持つ 3 次の基底関数の 1 階・2 階の導関数を、1 つ低い階の中心差分と比べる
        let knots = vec![0.0, 0.0, 0.0, 0.0, 0.4, 1.0, 1.0, 1.0, 1.0];
        let (span, t, h) = (4, 0.7, 1e-5);
        let derivatives = basis_derivatives(span, 3, t, &knots, 2);
        let ahead = basis_derivatives(span, 3, t + h, &knots, 1);
        let behind = basis_derivatives(span, 3, t - h, &knots, 1);
        for k in 1..=2 {
            for i in 0..4 {
                let difference = (ahead[k - 1][i] - behind[k - 1][i]) / (2.0 * h);
                assert!(
                    (derivatives[k][i] - difference).abs() < 1e-6,
                    "k = {k}, i = {i}"
                );
            }
        }
        // 基底関数の和は 1、導関数の和は 0
        assert!((derivatives[0].iter().sum::<f64>() - 1.0).abs() < 1e-12);
        for row in &derivatives[1..] {
            assert!(row.iter().sum::<f64>().abs() < 1e-12);
        }
    }
}
