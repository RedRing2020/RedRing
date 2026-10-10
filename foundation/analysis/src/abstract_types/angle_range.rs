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
