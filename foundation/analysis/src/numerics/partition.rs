//! 非減少列に対する区間探索ユーティリティ。
//!
//! knot vector のような重複を含み得る非減少列に対し、
//! 指定値が属する span の左 index を返す。

use crate::Scalar;

/// 非減少列上で、`sequence[i] <= value < sequence[i + 1]` を満たす span の左 index を返す。
///
/// `lower_bound` と `upper_bound` は探索対象の有効範囲を表す。
/// `value <= sequence[lower_bound]` のときは `lower_bound`、
/// `value >= sequence[upper_bound]` のときは `upper_bound - 1` を返す。
///
/// # Panics
///
/// - `sequence.len() < 2` の場合
/// - `lower_bound >= upper_bound` の場合
/// - `upper_bound >= sequence.len()` の場合
pub fn find_span_in_non_decreasing_sequence<T: Scalar>(
    value: T,
    sequence: &[T],
    lower_bound: usize,
    upper_bound: usize,
) -> usize {
    assert!(
        sequence.len() >= 2,
        "sequence must contain at least two elements"
    );
    assert!(
        lower_bound < upper_bound,
        "lower_bound must be smaller than upper_bound"
    );
    assert!(
        upper_bound < sequence.len(),
        "upper_bound must be within sequence"
    );

    if value <= sequence[lower_bound] {
        return lower_bound;
    }

    if value >= sequence[upper_bound] {
        return upper_bound - 1;
    }

    let mut low = lower_bound;
    let mut high = upper_bound;

    while low < high {
        let mid = usize::midpoint(low, high);
        if value < sequence[mid] {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    low - 1
}
