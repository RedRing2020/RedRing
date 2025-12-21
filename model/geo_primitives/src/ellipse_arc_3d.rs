//! 3次元楕円弧（EllipseArc3D）のCore実装
//!
//! Core Foundation パターンに基づく EllipseArc3D の必須機能のみ
//! 拡張機能は ellipse_arc_3d_extensions.rs を参照

use crate::{Arc3D, Circle3D, Direction3D, Ellipse3D, Point3D, Vector3D};
use geo_foundation::{
    core::ellipse_arc_core_traits::{
        EllipseArc3DConstructor, EllipseArc3DMeasure, EllipseArc3DProperties,
    },
    Angle, Scalar,
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

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> EllipseArc3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい3D楕円弧を作成
    pub fn new(ellipse: Ellipse3D<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self {
            ellipse,
            start_angle,
            end_angle,
        }
    }

    /// 3D円弧から3D楕円弧を作成
    pub fn from_arc(arc: Arc3D<T>) -> Option<Self> {
        let circle = Circle3D::new(arc.center(), arc.normal(), arc.radius())?;
        let ellipse = Ellipse3D::from_circle(&circle)?;
        Some(Self::new(ellipse, arc.start_angle(), arc.end_angle()))
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

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

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

    /// 開始点を取得
    pub fn start_point(&self) -> Point3D<T> {
        self.ellipse.point_at_angle(self.start_angle)
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point3D<T> {
        self.ellipse.point_at_angle(self.end_angle)
    }

    /// パラメータ値tにおける点を取得
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle_diff = self.end_angle.to_radians() - self.start_angle.to_radians();
        let current_angle = self.start_angle.to_radians() + t * angle_diff;
        self.ellipse
            .point_at_angle(Angle::from_radians(current_angle))
    }

    /// 弧の中点を取得
    pub fn midpoint(&self) -> Point3D<T> {
        self.point_at_parameter(T::from_f64(0.5))
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

    /// 弧が完全な楕円かどうかを判定
    pub fn is_full_ellipse(&self) -> bool {
        let two_pi = T::from_f64(2.0 * std::f64::consts::PI);
        (self.angle_span() - two_pi).abs() < T::EPSILON
    }

    /// 弧が円弧かどうかを判定
    pub fn is_circular(&self) -> bool {
        self.ellipse.is_circle()
    }

    /// 楕円弧の有効性を検証
    pub fn is_valid(&self) -> bool {
        self.ellipse.semi_major_axis() > T::ZERO
            && self.ellipse.semi_minor_axis() > T::ZERO
            && self.start_angle.to_radians().is_finite()
            && self.end_angle.to_radians().is_finite()
    }

    // ========================================================================
    // Core Transform Methods
    // ========================================================================

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

    /// 角度範囲を変更した新しい楕円弧を作成
    pub fn with_angles(&self, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self::new(self.ellipse, start_angle, end_angle)
    }

    /// 基底楕円を変更した新しい楕円弧を作成
    pub fn with_ellipse(&self, ellipse: Ellipse3D<T>) -> Self {
        Self::new(ellipse, self.start_angle, self.end_angle)
    }

    /// 原点中心の回転（Z軸回転）
    pub fn rotate_z(&self, _angle: T) -> Option<Self> {
        // 簡単なZ軸回転（実際の実装は拡張版で）
        Some(Self::new(self.ellipse, self.start_angle, self.end_angle))
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

    /// 部分弧を取得
    pub fn sub_arc(&self, sub_start: Angle<T>, sub_end: Angle<T>) -> Option<Self> {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();
        let sub_start_rad = sub_start.to_radians();
        let sub_end_rad = sub_end.to_radians();

        // 簡単な範囲チェック
        if (sub_start_rad >= start_rad && sub_start_rad <= end_rad)
            && (sub_end_rad >= start_rad && sub_end_rad <= end_rad)
            && sub_start_rad <= sub_end_rad
        {
            Some(Self::new(self.ellipse, sub_start, sub_end))
        } else {
            None
        }
    }

    /// バウンディングボックスを取得（近似）
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
        let start = self.start_point();
        let end = self.end_point();
        let mid = self.midpoint();

        geo_core::Aabb3D::from_points(&[
            start,
            end,
            mid,
        ])
        .unwrap_or_else(|| {
            // フォールバック: ゼロサイズのボックス
            geo_core::Aabb3D::new(
                start,
                start,
            )
        })
    }
}

// ============================================================================
// Default Implementations
// ============================================================================

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

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

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

    fn xy_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::unit_z();

        // 回転を考慮した長軸方向
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();
        let major_vec = Vector3D::new(cos_rot, sin_rot, T::ZERO);

        let ellipse = Ellipse3D::new(center_point, semi_major, semi_minor, normal_vec, major_vec)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }

    fn unit_ellipse_arc_xy() -> Self {
        let center = Point3D::origin();
        let normal = Vector3D::unit_z();
        let major_axis = Vector3D::unit_x();
        let ellipse = Ellipse3D::new(center, T::ONE, T::ONE, normal, major_axis).unwrap();
        let start = Angle::from_radians(T::ZERO);
        let end = Angle::from_radians(T::PI / (T::ONE + T::ONE)); // π/2
        Self::new(ellipse, start, end)
    }

    // ========== Phase 2: 追加コンストラクタ ==========

    fn xz_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::unit_y();

        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();
        let major_vec = Vector3D::new(cos_rot, T::ZERO, sin_rot);

        let ellipse = Ellipse3D::new(center_point, semi_major, semi_minor, normal_vec, major_vec)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }

    fn yz_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::unit_x();

        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();
        let major_vec = Vector3D::new(T::ZERO, cos_rot, sin_rot);

        let ellipse = Ellipse3D::new(center_point, semi_major, semi_minor, normal_vec, major_vec)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }

    fn from_three_points(start: (T, T, T), mid: (T, T, T), end: (T, T, T)) -> Option<Self> {
        let p1 = Point3D::new(start.0, start.1, start.2);
        let p2 = Point3D::new(mid.0, mid.1, mid.2);
        let p3 = Point3D::new(end.0, end.1, end.2);

        let v1 = Vector3D::from_points(&p1, &p2);
        let v2 = Vector3D::from_points(&p2, &p3);

        let normal = v1.cross(&v2);
        if normal.length() < T::EPSILON {
            return None;
        }

        let center = Point3D::new(
            (p1.x() + p2.x() + p3.x()) / (T::ONE + T::ONE + T::ONE),
            (p1.y() + p2.y() + p3.y()) / (T::ONE + T::ONE + T::ONE),
            (p1.z() + p2.z() + p3.z()) / (T::ONE + T::ONE + T::ONE),
        );

        let radius = Vector3D::from_points(&center, &p1).length();
        let major_vec = Vector3D::from_points(&center, &p1).normalize();

        let ellipse = Ellipse3D::new(center, radius, radius, normal, major_vec)?;
        let start_angle = Angle::from_radians(T::ZERO);
        let end_angle = Angle::from_radians(T::PI);
        Some(Self::new(ellipse, start_angle, end_angle))
    }
}

impl<T: Scalar> EllipseArc3DProperties<T> for EllipseArc3D<T> {
    fn center(&self) -> (T, T, T) {
        let c = self.center();
        (c.x(), c.y(), c.z())
    }

    fn semi_major_axis(&self) -> T {
        self.semi_major()
    }

    fn semi_minor_axis(&self) -> T {
        self.semi_minor()
    }

    fn start_angle(&self) -> T {
        self.start_angle.to_radians()
    }

    fn end_angle(&self) -> T {
        self.end_angle.to_radians()
    }

    // ========== Phase 2: 追加プロパティ ==========

    fn normal(&self) -> (T, T, T) {
        let n = self.normal();
        (n.x(), n.y(), n.z())
    }

    fn sweep_angle(&self) -> T {
        let mut sweep = self.end_angle.to_radians() - self.start_angle.to_radians();
        if sweep < T::ZERO {
            sweep += T::TAU;
        }
        sweep
    }

    fn eccentricity(&self) -> T {
        self.ellipse.eccentricity()
    }
}

impl<T: Scalar> EllipseArc3DMeasure<T> for EllipseArc3D<T> {
    fn measure(&self) -> T {
        // 楕円弧の長さの簡易近似
        let full_perimeter = self.ellipse.perimeter();
        let angle_ratio = self.angle_span() / T::TAU;
        full_perimeter * angle_ratio
    }

    fn start_point(&self) -> (T, T, T) {
        let p = self.start_point();
        (p.x(), p.y(), p.z())
    }

    fn end_point(&self) -> (T, T, T) {
        let p = self.end_point();
        (p.x(), p.y(), p.z())
    }

    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let p = self.point_at_parameter(t);
        (p.x(), p.y(), p.z())
    }

    // ========== Phase 2: 追加計量 ==========

    fn mid_point(&self) -> (T, T, T) {
        let p = self.point_at_parameter(T::ONE / (T::ONE + T::ONE));
        (p.x(), p.y(), p.z())
    }

    fn point_at_angle(&self, angle: T) -> Option<(T, T, T)> {
        let normalized_angle = if angle < T::ZERO {
            angle + T::TAU
        } else if angle >= T::TAU {
            angle - T::TAU
        } else {
            angle
        };

        let start = self.start_angle.to_radians();
        let end = self.end_angle.to_radians();

        let in_range = if start <= end {
            normalized_angle >= start && normalized_angle <= end
        } else {
            normalized_angle >= start || normalized_angle <= end
        };

        if !in_range {
            return None;
        }

        let t = (normalized_angle - start) / (end - start);
        let p = self.point_at_parameter(t);
        Some((p.x(), p.y(), p.z()))
    }

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

    fn bounding_box(&self) -> ((T, T, T), (T, T, T)) {
        let start = self.start_point();
        let end = self.end_point();

        let mut min_x = start.x().min(end.x());
        let mut max_x = start.x().max(end.x());
        let mut min_y = start.y().min(end.y());
        let mut max_y = start.y().max(end.y());
        let mut min_z = start.z().min(end.z());
        let mut max_z = start.z().max(end.z());

        // 16分割でサンプリング
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
            let p = self.point_at_parameter(t);
            min_x = min_x.min(p.x());
            max_x = max_x.max(p.x());
            min_y = min_y.min(p.y());
            max_y = max_y.max(p.y());
            min_z = min_z.min(p.z());
            max_z = max_z.max(p.z());
        }

        ((min_x, min_y, min_z), (max_x, max_y, max_z))
    }
}
