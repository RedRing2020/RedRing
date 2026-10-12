//! 3次元楕円弧（EllipseArc3D）のCore実装
//!
//! Core Foundation パターンに基づく EllipseArc3D の必須機能のみ
//! 拡張機能は子モジュール extensions を参照

mod bounds;
mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Ellipse3D, Point3D, Vector3D};
use geo_contracts::{
    Angle, EllipseArc3DConstructor, EllipseArc3DContainment, EllipseArc3DDerived,
    EllipseArc3DEndpoint, EllipseArc3DEvaluation, EllipseArc3DProperties, EllipseArc3DTrimRange,
    Scalar,
};

/// 3次元楕円弧
///
/// 3D空間内の楕円の一部分を表現する楕円弧
/// 開始角度と終了角度で定義される
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipseArc3D<T: Scalar> {
    ellipse: Ellipse3D<T>, // 基底楕円
    start_angle: Angle<T>, // 開始角度
    end_angle: Angle<T>,   // 終了角度
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for EllipseArc3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::Arc
    }
}

impl<T: Scalar> EllipseArc3D<T> {
    /// 新しい3D楕円弧を作成
    pub fn new(ellipse: Ellipse3D<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self {
            ellipse,
            start_angle,
            end_angle,
        }
    }

    /// 基底楕円を取得
    pub fn ellipse(&self) -> &Ellipse3D<T> {
        &self.ellipse
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.ellipse.center()
    }

    /// 長半径を取得
    pub fn semi_major(&self) -> T {
        self.ellipse.semi_major_axis()
    }

    /// 短半径を取得
    pub fn semi_minor(&self) -> T {
        self.ellipse.semi_minor_axis()
    }

    /// 法線方向を取得
    pub fn normal(&self) -> Direction3D<T> {
        self.ellipse.normal()
    }

    /// 長軸方向を取得
    pub fn major_axis_direction(&self) -> Direction3D<T> {
        self.ellipse.major_axis_direction()
    }

    /// 短軸方向を取得
    pub fn minor_axis_direction(&self) -> Direction3D<T> {
        self.ellipse.minor_axis_direction()
    }

    /// 開始点を取得
    pub fn start_point(&self) -> Point3D<T> {
        self.ellipse.point_at_angle(self.start_angle)
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point3D<T> {
        self.ellipse.point_at_angle(self.end_angle)
    }

    /// Arc trim-local parameter `t` (`0 <= t <= 1`) で楕円弧上の点を取得
    ///
    /// 基底楕円の local angle parameter へ線形写像して評価する。
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle_diff = self.end_angle.to_radians() - self.start_angle.to_radians();
        let current_angle = self.start_angle.to_radians() + t * angle_diff;
        self.ellipse
            .point_at_angle(Angle::from_radians(current_angle))
    }

    /// Primitive 局所角度系の角度で楕円弧上の点を取得する convenience API
    pub fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T> {
        self.ellipse.point_at_angle(angle)
    }

    /// 弧の角度スパンを取得
    pub fn angle_span(&self) -> T {
        let diff = self.end_angle.to_radians() - self.start_angle.to_radians();
        if diff >= T::ZERO {
            diff
        } else {
            diff + T::from_f64(2.0 * std::f64::consts::PI)
        }
    }

    /// 楕円弧の有効性を検証
    pub fn is_valid(&self) -> bool {
        self.ellipse.semi_major_axis() > T::ZERO
            && self.ellipse.semi_minor_axis() > T::ZERO
            && self.start_angle.to_radians().is_finite()
            && self.end_angle.to_radians().is_finite()
    }

    /// 平行移動
    pub fn translate(&self, vector: Vector3D<T>) -> Self {
        Self::new(
            self.ellipse.translate(vector),
            self.start_angle,
            self.end_angle,
        )
    }

    /// 向きを反転した楕円弧を取得
    pub fn reverse(&self) -> Self {
        Self::new(self.ellipse, self.end_angle, self.start_angle)
    }

    /// 原点中心の均等スケール
    pub fn scale(&self, factor: T) -> Option<Self> {
        // スケールは基底楕円に適用
        if factor <= T::ZERO {
            return None;
        }

        // 楕円をスケールして新しいEllipseArc3Dを作成
        let scaled_ellipse = Ellipse3D::new(
            self.ellipse.center(),
            self.ellipse.semi_major_axis() * factor,
            self.ellipse.semi_minor_axis() * factor,
            self.ellipse.normal().as_vector(),
            self.ellipse.major_axis_direction().as_vector(),
        )?;

        Some(Self::new(scaled_ellipse, self.start_angle, self.end_angle))
    }

    /// バウンディングボックスを取得（近似）
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
        let start = self.start_point();
        let end = self.end_point();
        let mid = self.point_at_parameter(T::ONE / (T::ONE + T::ONE));

        geo_core::Aabb3D::from_points(&[start, end, mid]).unwrap_or_else(|| {
            // フォールバック: ゼロサイズのボックス
            geo_core::Aabb3D::new(start, start)
        })
    }
}

impl<T: Scalar> Default for EllipseArc3D<T> {
    fn default() -> Self {
        let center = Point3D::origin();
        let normal = Vector3D::unit_z();
        let major_axis = Vector3D::unit_x();
        let ellipse = Ellipse3D::new(center, T::ONE, T::ONE, normal, major_axis)
            .unwrap_or_else(|| panic!("Default ellipse creation failed"));

        Self::new(
            ellipse,
            Angle::from_degrees(T::ZERO),
            Angle::from_degrees(T::from_f64(90.0)),
        )
    }
}

impl<T: Scalar> EllipseArc3DConstructor<T> for EllipseArc3D<T> {
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major: T,
        semi_minor: T,
        major_direction: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        let major_vec = Vector3D::new(major_direction.0, major_direction.1, major_direction.2);

        let ellipse = Ellipse3D::new(center_point, semi_major, semi_minor, normal_vec, major_vec)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }
}

impl<T: Scalar> EllipseArc3DProperties<T> for EllipseArc3D<T> {
    fn center(&self) -> (T, T, T) {
        let c = self.ellipse.center();
        (c.x(), c.y(), c.z())
    }

    fn semi_major_axis(&self) -> T {
        self.ellipse.semi_major_axis()
    }

    fn semi_minor_axis(&self) -> T {
        self.ellipse.semi_minor_axis()
    }

    fn start_angle(&self) -> T {
        self.start_angle.to_radians()
    }

    fn end_angle(&self) -> T {
        self.end_angle.to_radians()
    }

    fn normal(&self) -> (T, T, T) {
        let n = self.ellipse.normal();
        (n.x(), n.y(), n.z())
    }
}

impl<T: Scalar> EllipseArc3DDerived<T> for EllipseArc3D<T> {
    fn length(&self) -> T {
        // 楕円弧の長さの簡易近似: angle_span計算を直接展開
        let full_circumference = self.ellipse.circumference();
        let diff = self.end_angle.to_radians() - self.start_angle.to_radians();
        let angle_span = if diff >= T::ZERO {
            diff
        } else {
            diff + T::from_f64(2.0 * std::f64::consts::PI)
        };
        let angle_ratio = angle_span / T::TAU;
        full_circumference * angle_ratio
    }

    fn bounding_box(&self) -> ((T, T, T), (T, T, T)) {
        let start = <Self as EllipseArc3DEndpoint<T>>::start_point(self);
        let end = <Self as EllipseArc3DEndpoint<T>>::end_point(self);

        let mut min_x = start.0.min(end.0);
        let mut max_x = start.0.max(end.0);
        let mut min_y = start.1.min(end.1);
        let mut max_y = start.1.max(end.1);
        let mut min_z = start.2.min(end.2);
        let mut max_z = start.2.max(end.2);

        let t_values = [
            T::ZERO,
            T::ONE
                / (T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE
                    + T::ONE),
            T::ONE / (T::ONE + T::ONE + T::ONE + T::ONE + T::ONE + T::ONE + T::ONE + T::ONE),
            T::ONE / (T::ONE + T::ONE + T::ONE + T::ONE),
            T::ONE / (T::ONE + T::ONE),
            T::ONE,
        ];
        for &t in &t_values {
            let p = <Self as EllipseArc3DEvaluation<T>>::point_at_parameter(self, t);
            min_x = min_x.min(p.0);
            max_x = max_x.max(p.0);
            min_y = min_y.min(p.1);
            max_y = max_y.max(p.1);
            min_z = min_z.min(p.2);
            max_z = max_z.max(p.2);
        }

        ((min_x, min_y, min_z), (max_x, max_y, max_z))
    }
}

impl<T: Scalar> EllipseArc3DEndpoint<T> for EllipseArc3D<T> {
    fn start_point(&self) -> (T, T, T) {
        let p = self.ellipse.point_at_angle(self.start_angle);
        (p.x(), p.y(), p.z())
    }

    fn end_point(&self) -> (T, T, T) {
        let p = self.ellipse.point_at_angle(self.end_angle);
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar> EllipseArc3DEvaluation<T> for EllipseArc3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let angle_diff = self.end_angle.to_radians() - self.start_angle.to_radians();
        let current_angle = self.start_angle.to_radians() + t * angle_diff;
        let p = self
            .ellipse
            .point_at_angle(Angle::from_radians(current_angle));
        (p.x(), p.y(), p.z())
    }

    fn point_at_angle(&self, angle: T) -> (T, T, T) {
        let p = self.ellipse.point_at_angle(Angle::from_radians(angle));
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar> EllipseArc3DContainment<T> for EllipseArc3D<T> {
    fn contains_point(&self, point: (T, T, T), tolerance: T) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        let center = self.center();
        let vec = Vector3D::from_points(&center, &p);

        let distance = vec.length();
        let expected_radius = self.semi_major();

        if (distance - expected_radius).abs() > tolerance {
            return false;
        }

        let angle = vec.y().atan2(vec.x());
        let start = self.start_angle.to_radians();
        let end = self.end_angle.to_radians();

        if start <= end {
            angle >= start - tolerance && angle <= end + tolerance
        } else {
            angle >= start - tolerance || angle <= end + tolerance
        }
    }
}

impl<T: Scalar> EllipseArc3DTrimRange<T> for EllipseArc3D<T> {
    fn contains_angle(&self, angle: T) -> bool {
        EllipseArc3D::contains_angle(self, Angle::from_radians(angle))
    }
}

impl<T: Scalar> EllipseArc3D<T> {
    /// 開始角から反時計回りに終了角まで進む角度範囲
    ///
    /// 開始角と終了角が等しい場合は `None` を返す。
    pub fn angle_range(&self) -> Option<analysis::AngleRange<T>> {
        analysis::AngleRange::from_ccw_bounds(self.start_angle, self.end_angle)
    }

    /// 角度が角度範囲に含まれるかを判定する（範囲の両端を含み、既定の角度トレランスで判定する）
    pub fn contains_angle(&self, angle: Angle<T>) -> bool {
        let tolerance = geo_contracts::default_angle_tolerance::<T>();
        match self.angle_range() {
            Some(range) => range.contains(angle, tolerance),
            None => angle.is_equivalent(&self.start_angle, tolerance),
        }
    }

    /// 点の角度が角度範囲に含まれるかを判定する
    ///
    /// 点が曲線上にあるかではなく、角度範囲のみを判定する。点の角度は、点に最も近い母楕円上の点のパラメータ角とする（平面外の点は平面へ投影した点の最近点）。
    pub fn contains_point_angle(&self, point: &Point3D<T>) -> bool {
        self.contains_angle(Angle::from_radians(self.ellipse.parameter_for_point(point)))
    }
}
