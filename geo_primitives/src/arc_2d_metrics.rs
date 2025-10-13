//! Arc計量演算拡張トレイト実装
//!
//! 弧長・面積・中心角などの計算機能
//! 他の幾何プリミティブでも共通利用可能な抽象化

use crate::Arc2D;
use geo_foundation::{abstract_types::foundation::ArcMetrics, Angle, Scalar};

// ============================================================================
// ArcMetrics Trait Implementation
// ============================================================================

impl<T: Scalar> ArcMetrics<T> for Arc2D<T> {
    /// 弧長を計算
    fn arc_length(&self) -> T {
        self.radius() * self.angle_span().to_radians()
    }

    /// 扇形の面積を計算
    fn sector_area(&self) -> T {
        let half = T::ONE / (T::ONE + T::ONE);
        let radius = self.radius();
        let angle_span = self.angle_span().to_radians();
        half * radius * radius * angle_span
    }

    /// 角度スパンを取得
    fn angle_span(&self) -> Self::Angle {
        self.angle_span()
    }

    /// 中点角度を取得
    fn mid_angle(&self) -> Self::Angle {
        let half = T::ONE / (T::ONE + T::ONE);
        let start = self.start_angle().to_radians();
        let end = self.end_angle().to_radians();
        let mid = if end < start {
            start + (end + T::TAU - start) * half
        } else {
            start + (end - start) * half
        };
        Angle::from_radians(mid)
    }

    /// 中点を取得
    fn mid_point(&self) -> Self::Point {
        let mid_angle = self.mid_angle();
        self.circle().point_at_angle(mid_angle.to_radians())
    }
}

// ============================================================================
// Arc2D用の計量関連ヘルパーメソッド
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    /// 角度範囲を取得
    pub fn angle_span(&self) -> Angle<T> {
        let mut span = self.end_angle() - self.start_angle();
        if span.to_radians() < T::ZERO {
            span += Angle::from_radians(T::TAU);
        }
        span
    }

    /// 扇形の周長を計算（弧長 + 2 × 半径）
    pub fn sector_perimeter(&self) -> T {
        self.arc_length() + (T::ONE + T::ONE) * self.radius()
    }

    /// 弦の長さを計算
    pub fn chord_length(&self) -> T {
        let angle_span = self.angle_span().to_radians();
        let half_angle = angle_span / (T::ONE + T::ONE);
        (T::ONE + T::ONE) * self.radius() * half_angle.sin()
    }

    /// 矢高（sagitta）を計算
    pub fn sagitta(&self) -> T {
        let angle_span = self.angle_span().to_radians();
        let half_angle = angle_span / (T::ONE + T::ONE);
        self.radius() * (T::ONE - half_angle.cos())
    }
}
