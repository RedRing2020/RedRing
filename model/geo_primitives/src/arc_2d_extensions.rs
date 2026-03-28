//! Arc2D拡張メソッド
//!
//! Core Foundation パターンに基づく Arc2D の拡張機能
//! 基本機能は arc_2d.rs を参照

use crate::{arc_2d::Arc2D, Circle2D, Point2D};
use geo_contracts::{default_angle_tolerance, default_distance_tolerance};
use geo_contracts::{Angle, Scalar};

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
        if cross.abs() <= default_distance_tolerance::<T>() {
            return None;
        }

        // 外心の計算
        let d = (start.x() * (middle.y() - end.y())
            + middle.x() * (end.y() - start.y())
            + end.x() * (start.y() - middle.y()))
            * (T::ONE + T::ONE);

        if d.abs() <= default_distance_tolerance::<T>() {
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

        let circle = Circle2D::new(center, radius)?;
        Self::new(
            circle,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    // ========================================================================
    // Extension Predicate Methods
    // ========================================================================

    // pub fn is_full_circle(&self) -> bool {
    //     let span = self.angular_span();
    //     (span - (T::ONE + T::ONE) * T::PI).abs() <= T::EPSILON
    // }

    /// 退化した円弧かどうかを判定（非常に小さい半径または角度範囲）
    pub fn is_degenerate(&self) -> bool {
        self.radius_internal() <= default_distance_tolerance::<T>()
            || self.angular_span() <= default_angle_tolerance::<T>()
    }

    /// 指定角度が円弧の範囲内にあるかを判定
    pub fn contains_angle(&self, angle: Angle<T>) -> bool {
        // normalize_angle 未実装のため単純比較
        let a = angle.to_radians();
        let start = self.start_angle().to_radians();
        let end = self.end_angle().to_radians();
        let angle_tol = default_angle_tolerance::<T>();
        if start <= end {
            a + angle_tol >= start && a <= end + angle_tol
        } else {
            a + angle_tol >= start || a <= end + angle_tol
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
    pub fn to_circle(&self) -> Option<Circle2D<T>> {
        // 型安全な変換のみ許可
        if (self.angular_span() - (T::ONE + T::ONE) * T::PI).abs() <= default_angle_tolerance::<T>()
        {
            let center = self.center_internal();
            let radius = self.radius_internal();
            Some(Circle2D::new(center, radius)?)
        } else {
            None
        }
    }
}
