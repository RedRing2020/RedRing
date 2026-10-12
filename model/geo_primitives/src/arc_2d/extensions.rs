//! Arc2D拡張メソッド
//!
//! Core Foundation パターンに基づく Arc2D の拡張機能
//! 基本機能は親モジュール（arc_2d.rs）を参照

use crate::{arc_2d::Arc2D, Circle2D, Point2D, Vector2D};
use geo_contracts::{default_distance_tolerance, Angle, Circle2DConstructor, Scalar};

impl<T: Scalar> Arc2D<T> {
    /// Arc trim-local parameter `t` (`0 <= t <= 1`) で円弧上の点を計算
    ///
    /// 母円の local angle parameter へ線形写像して評価する。
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = self.start_angle().to_radians() + self.angular_span() * t;
        self.point_at_angle(angle)
    }

    /// Primitive 局所角度系の角度で円弧上の点を計算する convenience API
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let x = self.radius_internal() * angle.cos();
        let y = self.radius_internal() * angle.sin();
        self.center_internal() + Vector2D::new(x, y)
    }

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
        let v1 = start.vector_to(middle);
        let v2 = start.vector_to(end);

        // 3点が一直線上にある場合は円弧を作成できない
        let cross = v1.cross(v2);
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
        let radius = start.vector_to(center).length();

        // 各点に対応する角度を計算
        let start_dir = start.vector_to(center).try_normalize()?;
        let end_dir = end.vector_to(center).try_normalize()?;

        let start_angle = start_dir.y().atan2(start_dir.x());
        let end_angle = end_dir.y().atan2(end_dir.x());

        let circle = Circle2D::new(center, radius)?;
        Self::new(
            circle,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    // pub fn is_full_circle(&self) -> bool {
    //     let span = self.angular_span();
    //     (span - (T::ONE + T::ONE) * T::PI).abs() <= T::EPSILON
    // }

    // normalize_angle is implemented in arc_2d_containment.rs
}
