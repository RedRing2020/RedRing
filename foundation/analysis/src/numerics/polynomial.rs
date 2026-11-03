//! 多項式の実根
//!
//! 導関数の根で区間を単調な小区間に分け、符号が変わる小区間を二分法で解く。

use crate::Scalar;

/// 二分法の反復回数の上限
///
/// 二分法は区間の中点が端点と一致した時点（浮動小数点で区間をこれ以上分割できない時点）で終了する。
/// 上限は、f64 の仮数部と指数部の範囲を区間の分割で使い切る回数を上回る値とする。
const POLYNOMIAL_ROOT_MAX_BISECTION_ITERATIONS: usize = 1100;

/// 閉区間 `[lower, upper]` にある多項式の実根を昇順で返す
///
/// `coefficients` は低次の項から並べた係数（`c0 + c1 x + c2 x² + ...`）。
/// 根は、多項式の値の符号が変わる点と、値がちょうど 0 になる区間の端・導関数の根として求める。
/// 符号が変わらない重根（値が正確に 0 にならない接点）は返さない。
/// すべての係数が 0 の多項式は根を返さない。
pub fn polynomial_real_roots_in_interval<T: Scalar>(
    coefficients: &[T],
    lower: T,
    upper: T,
) -> Vec<T> {
    let degree = match coefficients.iter().rposition(|c| *c != T::ZERO) {
        Some(degree) => degree,
        None => return Vec::new(),
    };
    let coefficients = &coefficients[..=degree];
    if degree == 0 || lower > upper {
        return Vec::new();
    }

    let derivative: Vec<T> = coefficients
        .iter()
        .enumerate()
        .skip(1)
        .map(|(power, c)| *c * T::from_usize(power))
        .collect();
    let mut breakpoints = vec![lower];
    breakpoints.extend(polynomial_real_roots_in_interval(&derivative, lower, upper));
    breakpoints.push(upper);

    let mut roots: Vec<T> = Vec::new();
    for window in breakpoints.windows(2) {
        let (left, right) = (window[0], window[1]);
        let (left_value, right_value) = (
            evaluate_polynomial(coefficients, left),
            evaluate_polynomial(coefficients, right),
        );
        let root = if left_value == T::ZERO {
            Some(left)
        } else if right_value == T::ZERO {
            Some(right)
        } else if (left_value < T::ZERO) != (right_value < T::ZERO) {
            Some(bisect_monotone(
                coefficients,
                left,
                right,
                left_value < T::ZERO,
            ))
        } else {
            None
        };
        if let Some(root) = root {
            if roots.last() != Some(&root) {
                roots.push(root);
            }
        }
    }
    roots
}

/// 多項式の値（Horner 法）
fn evaluate_polynomial<T: Scalar>(coefficients: &[T], x: T) -> T {
    coefficients
        .iter()
        .rev()
        .fold(T::ZERO, |value, c| value * x + *c)
}

/// 単調で、`left` と `right` で値の符号が異なる区間の根を二分法で求める
fn bisect_monotone<T: Scalar>(
    coefficients: &[T],
    mut left: T,
    mut right: T,
    left_is_negative: bool,
) -> T {
    let two = T::ONE + T::ONE;
    let mut middle = (left + right) / two;
    for _ in 0..POLYNOMIAL_ROOT_MAX_BISECTION_ITERATIONS {
        middle = (left + right) / two;
        if middle == left || middle == right {
            break;
        }
        let value = evaluate_polynomial(coefficients, middle);
        if value == T::ZERO {
            break;
        }
        if (value < T::ZERO) == left_is_negative {
            left = middle;
        } else {
            right = middle;
        }
    }
    middle
}

#[cfg(test)]
mod tests {
    use super::polynomial_real_roots_in_interval;

    fn assert_roots(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len(), "roots: {actual:?}");
        for (a, e) in actual.iter().zip(expected) {
            assert!(
                (a - e).abs() < 1e-12,
                "roots: {actual:?}, expected: {expected:?}"
            );
        }
    }

    #[test]
    fn linear_and_quadratic_roots() {
        // 2x - 1
        assert_roots(
            &polynomial_real_roots_in_interval(&[-1.0, 2.0], -10.0, 10.0),
            &[0.5],
        );
        // x² - 2
        assert_roots(
            &polynomial_real_roots_in_interval(&[-2.0, 0.0, 1.0], -10.0, 10.0),
            &[-(2.0_f64.sqrt()), 2.0_f64.sqrt()],
        );
        // x² + 1 は実根を持たない
        assert!(polynomial_real_roots_in_interval(&[1.0, 0.0, 1.0], -10.0, 10.0).is_empty());
    }

    #[test]
    fn quartic_with_four_roots() {
        // (x + 2)(x + 0.5)(x - 1)(x - 3) = x⁴ - 1.5x³ - 6x² + 3.5x + 3
        assert_roots(
            &polynomial_real_roots_in_interval(&[3.0, 3.5, -6.0, -1.5, 1.0], -10.0, 10.0),
            &[-2.0, -0.5, 1.0, 3.0],
        );
    }

    #[test]
    fn roots_outside_the_interval_are_excluded() {
        // (x + 2)(x + 0.5)(x - 1)(x - 3)
        assert_roots(
            &polynomial_real_roots_in_interval(&[3.0, 3.5, -6.0, -1.5, 1.0], -1.0, 1.0),
            &[-0.5, 1.0],
        );
    }

    #[test]
    fn exact_double_root_and_degenerate_leading_coefficients() {
        // (x - 1)² は符号が変わらないが、値がちょうど 0 になる導関数の根として返す
        assert_roots(
            &polynomial_real_roots_in_interval(&[1.0, -2.0, 1.0], -10.0, 10.0),
            &[1.0],
        );
        // 最高次の係数が 0 の 4 次式は 1 次式として扱う
        assert_roots(
            &polynomial_real_roots_in_interval(&[-1.0, 2.0, 0.0, 0.0, 0.0], -10.0, 10.0),
            &[0.5],
        );
        assert!(polynomial_real_roots_in_interval(&[0.0, 0.0], -1.0, 1.0).is_empty());
    }
}
