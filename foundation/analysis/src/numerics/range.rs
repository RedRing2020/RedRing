//! 数値の範囲判定

use crate::Scalar;

/// 値が閉区間 `[min, max]` に許容誤差付きで含まれるかを判定する。
///
/// `min - tolerance <= value <= max + tolerance` なら `true` を返す。許容誤差は絶対値として扱う。
/// 値・下限・上限のいずれかが有限でない場合と、`min > max` の場合は `false` を返す。
pub fn is_within_closed_range<T: Scalar>(value: T, min: T, max: T, tolerance: T) -> bool {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || min > max {
        return false;
    }
    let tolerance = tolerance.abs();
    value >= min - tolerance && value <= max + tolerance
}
