//! Arc2D拡張メソッド
//!
//! Core Foundation パターンに基づく Arc2D の拡張機能
//! 基本機能は arc_2d.rs を参照

use crate::{Arc2D, Circle2D, Point2D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Angle, Scalar};

impl<T: Scalar> Arc2D<T> {
    // ========================================================================
    // Extension Construction Methods
    // ========================================================================

    /// 3点を通る円弧を作成
    ///
    /// # 引数
    /// * `start` - 開始点
    /// * `middle` - 中間点
    /// * `end` - 終了点
    pub fn from_three_points(
        start: Point2D<T>,
        middle: Point2D<T>,
        end: Point2D<T>,
    ) -> Option<Self> {
        // 3点から円の中心と半径を計算
        let v1 = start.vector_to(&middle);
        let v2 = start.vector_to(&end);

        // 3点が一直線上にある場合は円弧を作成できない
        let cross = v1.cross(&v2);
        if cross.abs() <= DefaultTolerances::distance::<T>() {
            return None;
        }

        // 外心の計算
        let d = (start.x() * (middle.y() - end.y())
            + middle.x() * (end.y() - start.y())
            + end.x() * (start.y() - middle.y()))
            * (T::ONE + T::ONE);

        if d.abs() <= DefaultTolerances::distance::<T>() {
            return None;
        }

        let ux = ((start.x() * start.x() + start.y() * start.y()) * (middle.y() - end.y())
            + (middle.x() * middle.x() + middle.y() * middle.y()) * (end.y() - start.y())
            + (end.x() * end.x() + end.y() * end.y()) * (start.y() - middle.y()))
            / d;

        let uy = ((start.x() * start.x() + start.y() * start.y()) * (end.x() - middle.x())
            + (middle.x() * middle.x() + middle.y() * middle.y()) * (start.x() - end.x())
            + (end.x() * end.x() + end.y() * end.y()) * (middle.x() - start.x()))
            / d;

        let center = Point2D::new(ux, uy);
        let radius = start.vector_to(&center).length();

        // 各点に対応する角度を計算
        let start_dir = start.vector_to(&center).try_normalize()?;
        let end_dir = end.vector_to(&center).try_normalize()?;

        let start_angle = start_dir.y().atan2(start_dir.x());
        let end_angle = end_dir.y().atan2(end_dir.x());

        let center_f64 = geo_foundation::abstracts::Point2D::new(
            center.x().to_f64().unwrap_or(0.0),
            center.y().to_f64().unwrap_or(0.0),
        );
        let radius_f64 = radius.to_f64().unwrap_or(0.0);
        let circle = Circle2D::new(center_f64, radius_f64)?;
        Self::new(
            circle,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    // ========================================================================
    // Extension Predicate Methods
    // ========================================================================

    /// 完全円かどうかを判定
    pub fn is_full_circle(&self) -> bool {
        let span = self.angle_span();
        let two_pi = Angle::from_radians(T::TAU);
        span.is_equivalent_default(&two_pi)
    }

    /// 退化した円弧かどうかを判定（非常に小さい半径または角度範囲）
    pub fn is_degenerate(&self) -> bool {
        self.radius() <= DefaultTolerances::distance::<T>()
            || self.angle_span().to_radians() <= Angle::<T>::tolerance()
    }

    /// 指定角度が円弧の範囲内にあるかを判定
    pub fn contains_angle(&self, angle: Angle<T>) -> bool {
        let normalized_angle = self.normalize_angle(angle);
        let normalized_start = self.normalize_angle(self.start_angle());
        let normalized_end = self.normalize_angle(self.end_angle());

        if normalized_start <= normalized_end {
            normalized_angle >= normalized_start && normalized_angle <= normalized_end
        } else {
            // 0度をまたぐ場合
            normalized_angle >= normalized_start || normalized_angle <= normalized_end
        }
    }

    // ========================================================================
    // Extension Geometric Methods
    // ========================================================================

    // mid_point is implemented in arc_2d_sampling.rs
    // normalize_angle is implemented in arc_2d_containment.rs

    // ========================================================================
    // Extension Type Conversion Methods
    // ========================================================================

    /// Circle2D に変換（完全円の場合のみ）
    pub fn to_circle(&self) -> Option<Circle2D> {
        if self.is_full_circle() {
            let center_f64 = geo_foundation::abstracts::Point2D::new(
                self.center().x().to_f64().unwrap_or(0.0),
                self.center().y().to_f64().unwrap_or(0.0),
            );
            Circle2D::new(center_f64, self.radius())
        } else {
            None
        }
    }
}
