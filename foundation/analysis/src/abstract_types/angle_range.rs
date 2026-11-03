//! AngleRange - 周期的な角度範囲

use crate::abstract_types::{Angle, Scalar};

/// 開始角から反時計回りに `span` だけ進む角度範囲
///
/// `span` は `0 < span <= 2π` とし、0° 跨ぎ（開始角 + `span` が 2π を超える範囲）と全周（`span = 2π`）を
/// 特別な場合分けなしに表す。開始角は `[0, 2π)` に正規化して保持する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AngleRange<T: Scalar> {
    start: Angle<T>,
    span: Angle<T>,
}

impl<T: Scalar> AngleRange<T> {
    /// 開始角と反時計回りの幅から角度範囲を作成する。
    ///
    /// 幅が 0 以下の場合と、開始角・幅が有限でない場合は `None` を返す。2π を超える幅は全周とする。
    pub fn new(start: Angle<T>, span: Angle<T>) -> Option<Self> {
        let span_radians = span.to_radians();
        if !start.to_radians().is_finite() || !span_radians.is_finite() || span_radians <= T::ZERO {
            return None;
        }
        Some(Self {
            start: start.normalize(),
            span: Angle::from_radians(span_radians.min(T::TAU)),
        })
    }

    /// 開始角から反時計回りに終了角まで進む角度範囲を作成する。
    ///
    /// 終了角が開始角より小さい場合は 0° を跨ぐ範囲とする。開始角と終了角の差が 2π の正の整数倍の
    /// 場合は全周とし、開始角と終了角が等しい場合は `None` を返す。
    pub fn from_ccw_bounds(start: Angle<T>, end: Angle<T>) -> Option<Self> {
        let difference = end.to_radians() - start.to_radians();
        if difference == T::ZERO {
            return None;
        }
        let span = Angle::from_radians(difference).normalize().to_radians();
        let span = if span == T::ZERO { T::TAU } else { span };
        Self::new(start, Angle::from_radians(span))
    }

    /// 全周の角度範囲を作成する。
    pub fn full(start: Angle<T>) -> Option<Self> {
        Self::new(start, Angle::from_radians(T::TAU))
    }

    /// 開始角（`[0, 2π)` に正規化済み）
    pub fn start(&self) -> Angle<T> {
        self.start
    }

    /// 反時計回りの幅（`0 < span <= 2π`）
    pub fn span(&self) -> Angle<T> {
        self.span
    }

    /// 終了角（`[0, 2π)` に正規化済み）
    pub fn end(&self) -> Angle<T> {
        (self.start + self.span).normalize()
    }

    /// 全周の範囲かを判定する。
    pub fn is_full(&self) -> bool {
        self.span.to_radians() >= T::TAU
    }

    /// 角度が範囲に含まれるかを許容誤差付きで判定する。
    ///
    /// 範囲の両端を含む。許容誤差は角度（ラジアン）の絶対値として扱い、開始角の手前・終了角の先へ
    /// それぞれ許容誤差まで含める。角度が有限でない場合は `false` を返す。
    pub fn contains(&self, angle: Angle<T>, tolerance: T) -> bool {
        if !angle.to_radians().is_finite() {
            return false;
        }
        if self.is_full() {
            return true;
        }
        let tolerance = tolerance.abs();
        let offset = (angle - self.start).normalize().to_radians();
        offset <= self.span.to_radians() + tolerance || offset >= T::TAU - tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::super::angle::Angle;
    use super::AngleRange;

    use crate::test_constants::TOLERANCE_F64;

    fn deg(value: f64) -> Angle<f64> {
        Angle::from_degrees(value)
    }

    #[test]
    fn contains_angles_in_simple_range() {
        let range = AngleRange::from_ccw_bounds(deg(30.0), deg(120.0)).unwrap();

        assert!(range.contains(deg(30.0), 0.0));
        assert!(range.contains(deg(75.0), 0.0));
        assert!(range.contains(deg(120.0), 0.0));
        assert!(!range.contains(deg(150.0), 0.0));
        assert!(!range.contains(deg(10.0), 0.0));

        // 周期の違う同じ角度も含む
        assert!(range.contains(deg(75.0 + 360.0), 0.0));
        assert!(range.contains(deg(75.0 - 720.0), 0.0));
    }

    #[test]
    fn contains_angles_in_range_crossing_zero() {
        let range = AngleRange::from_ccw_bounds(deg(300.0), deg(60.0)).unwrap();

        assert!((range.span().to_degrees() - 120.0).abs() < TOLERANCE_F64);
        assert!(range.contains(deg(330.0), 0.0));
        assert!(range.contains(deg(0.0), 0.0));
        assert!(range.contains(deg(360.0), 0.0));
        assert!(range.contains(deg(-30.0), 0.0));
        assert!(range.contains(deg(60.0), 0.0));
        assert!(!range.contains(deg(90.0), 0.0));
        assert!(!range.contains(deg(270.0), 0.0));
    }

    #[test]
    fn tolerance_extends_both_ends() {
        let range = AngleRange::from_ccw_bounds(deg(30.0), deg(120.0)).unwrap();
        let tolerance = 1.0_f64.to_radians();

        assert!(range.contains(deg(29.5), tolerance));
        assert!(range.contains(deg(120.5), tolerance));
        assert!(!range.contains(deg(28.0), tolerance));
        assert!(!range.contains(deg(122.0), tolerance));

        // 0° を跨いで開始角の手前を判定する
        let range = AngleRange::from_ccw_bounds(deg(0.0), deg(90.0)).unwrap();
        assert!(range.contains(deg(359.5), tolerance));
        assert!(!range.contains(deg(358.0), tolerance));
    }

    #[test]
    fn full_range_contains_every_angle() {
        let full = AngleRange::full(deg(45.0)).unwrap();
        assert!(full.is_full());
        assert!(full.contains(deg(44.0), 0.0));
        assert!(full.contains(deg(-1000.0), 0.0));

        // 差が 2π の整数倍の上下限は全周
        let full = AngleRange::from_ccw_bounds(deg(0.0), deg(360.0)).unwrap();
        assert!(full.is_full());
        assert!(full.contains(deg(180.0), 0.0));

        // 2π を超える幅は全周
        assert!(AngleRange::new(deg(0.0), deg(720.0)).unwrap().is_full());
    }

    #[test]
    fn rejects_empty_and_non_finite_ranges() {
        assert!(AngleRange::from_ccw_bounds(deg(30.0), deg(30.0)).is_none());
        assert!(AngleRange::new(deg(0.0), deg(0.0)).is_none());
        assert!(AngleRange::new(deg(0.0), deg(-10.0)).is_none());
        assert!(AngleRange::new(Angle::from_radians(f64::NAN), deg(10.0)).is_none());
        assert!(AngleRange::new(deg(0.0), Angle::from_radians(f64::INFINITY)).is_none());

        let range = AngleRange::from_ccw_bounds(deg(0.0), deg(90.0)).unwrap();
        assert!(!range.contains(Angle::from_radians(f64::NAN), 0.0));
    }

    #[test]
    fn start_and_end_are_normalized() {
        let range = AngleRange::from_ccw_bounds(deg(-60.0), deg(420.0)).unwrap();
        assert!((range.start().to_degrees() - 300.0).abs() < TOLERANCE_F64);
        assert!((range.span().to_degrees() - 120.0).abs() < TOLERANCE_F64);
        assert!((range.end().to_degrees() - 60.0).abs() < TOLERANCE_F64);
    }
}
