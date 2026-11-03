//! 1 変数の関数の最小化

use crate::Scalar;

/// 黄金分割探索の反復回数の上限
///
/// 探索は区間の内点が端点と一致した時点（浮動小数点で区間をこれ以上縮められない時点）で終了する。
/// 区間は 1 回の反復で約 0.618 倍になるため、上限は f64 の仮数部と指数部の範囲を使い切る回数を上回る値とする。
const GOLDEN_SECTION_MAX_ITERATIONS: usize = 2300;

/// 閉区間 `[lower, upper]` で単峰（最小点の左で単調に減少し、右で単調に増加する）な関数の最小点
///
/// 黄金分割探索で区間を縮め、最後に残った区間の中点を返す。凸関数は単峰なので、この関数で最小点を求められる。
/// 最小値をとる点が区間をなす場合は、その区間のいずれかの点を返す。
pub fn minimize_unimodal<T: Scalar>(f: impl Fn(T) -> T, lower: T, upper: T) -> T {
    let two = T::ONE + T::ONE;
    let five = two + two + T::ONE;
    // 黄金比の逆数 (√5 - 1) / 2
    let ratio = (five.sqrt() - T::ONE) / two;

    let (mut left, mut right) = (lower, upper);
    let mut inner_left = right - (right - left) * ratio;
    let mut inner_right = left + (right - left) * ratio;
    let mut value_left = f(inner_left);
    let mut value_right = f(inner_right);
    for _ in 0..GOLDEN_SECTION_MAX_ITERATIONS {
        if !(left < inner_left && inner_left < inner_right && inner_right < right) {
            break;
        }
        if value_left <= value_right {
            right = inner_right;
            inner_right = inner_left;
            value_right = value_left;
            inner_left = right - (right - left) * ratio;
            value_left = f(inner_left);
        } else {
            left = inner_left;
            inner_left = inner_right;
            value_left = value_right;
            inner_right = left + (right - left) * ratio;
            value_right = f(inner_right);
        }
    }
    (left + right) / two
}

#[cfg(test)]
mod tests {
    use super::minimize_unimodal;

    #[test]
    fn minimum_of_a_parabola() {
        let minimum = minimize_unimodal(|x: f64| (x - 1.25) * (x - 1.25) + 3.0, -10.0, 10.0);
        assert!((minimum - 1.25).abs() < 1e-7);
    }

    #[test]
    fn minimum_at_the_interval_end() {
        let minimum = minimize_unimodal(|x: f64| x, 2.0, 5.0);
        assert!((minimum - 2.0).abs() < 1e-7);
    }

    #[test]
    fn non_smooth_convex_function() {
        let minimum = minimize_unimodal(|x: f64| (x + 0.5).abs(), -3.0, 4.0);
        assert!((minimum + 0.5).abs() < 1e-7);
    }
}
