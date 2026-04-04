//! 2次元楕円弧（EllipseArc2D）のCore実装
//!
//! Foundation統一システムに基づくEllipseArc2Dの必須機能のみ

use crate::{Ellipse2D, Point2D, Vector2D};
use geo_contracts::{
    default_angle_tolerance, Angle, EllipseArc2DConstructor, EllipseArc2DContainment,
    EllipseArc2DDerived, EllipseArc2DEndpoint, EllipseArc2DEvaluation, EllipseArc2DProperties,
    Scalar,
};

/// 2次元楕円弧
///
/// 楕円の一部分を表現する楕円弧
/// 開始角度と終了角度で定義される
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipseArc2D<T: Scalar> {
    pub(crate) ellipse: Ellipse2D<T>, // 基底楕円
    pub(crate) start_angle: Angle<T>, // 開始角度
    pub(crate) end_angle: Angle<T>,   // 終了角度
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> EllipseArc2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい楕円弧を作成
    ///
    /// # 引数
    /// * `ellipse` - 基底楕円
    /// * `start_angle` - 開始角度
    /// * `end_angle` - 終了角度
    pub fn new(ellipse: Ellipse2D<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self {
            ellipse,
            start_angle,
            end_angle,
        }
    }

    /// 円弧から楕円弧を作成
    // 一時的にコメントアウト: Arc2Dはトレイトなので具象型が必要
    // pub fn from_arc(arc: Arc2D<T>) -> Self {
    //     let ellipse = Ellipse2D::from_circle(*arc.circle());
    //     Self::new(ellipse, arc.start_angle(), arc.end_angle())
    // }
    // ========================================================================
    // Core Accessor Methods
    // ========================================================================
    /// 基底楕円を取得
    pub fn ellipse(&self) -> &Ellipse2D<T> {
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
    pub fn center(&self) -> Point2D<T> {
        self.ellipse.center_internal()
    }

    /// 長半軸を取得
    pub fn semi_major(&self) -> T {
        self.ellipse.semi_major_internal()
    }

    /// 短半軸を取得
    pub fn semi_minor(&self) -> T {
        self.ellipse.semi_minor_internal()
    }

    /// 回転角を取得
    pub fn rotation(&self) -> T {
        self.ellipse.rotation()
    }

    /// 角度スパンを取得
    pub fn angle_span(&self) -> Angle<T> {
        self.end_angle - self.start_angle
    }

    // ========================================================================
    // Core Geometric Methods
    // ========================================================================

    /// 開始点を取得
    pub fn start_point(&self) -> Point2D<T> {
        self.ellipse
            .point_at_parameter(self.start_angle.to_radians())
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point2D<T> {
        self.ellipse.point_at_parameter(self.end_angle.to_radians())
    }

    /// 中点を取得
    pub fn midpoint(&self) -> Point2D<T> {
        let half = T::ONE / (T::ONE + T::ONE); // 1/2
        let mid_angle = (self.start_angle + self.end_angle) * half;
        self.ellipse.point_at_parameter(mid_angle.to_radians())
    }

    /// パラメータ t での点を取得（0 <= t <= 1）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = self.start_angle.to_radians()
            + (self.end_angle.to_radians() - self.start_angle.to_radians()) * t;
        self.ellipse.point_at_parameter(angle)
    }

    /// パラメータ t での接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: T) -> Vector2D<T> {
        let angle = self.start_angle.to_radians()
            + (self.end_angle.to_radians() - self.start_angle.to_radians()) * t;
        self.ellipse.tangent_at_parameter(angle)
    }

    /// 弧長を取得（近似値）
    pub fn arc_length(&self) -> T {
        // 簡易近似：楕円周囲長に角度比率を掛ける
        let full_perimeter = self.ellipse.perimeter();
        let angle_ratio = self.angle_span().to_radians().abs() / T::TAU;
        full_perimeter * angle_ratio
    }

    /// 点が楕円弧上にあるかを判定
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 1. 点が基底楕円上にあるか
        if !self.ellipse.on_boundary(point, tolerance) {
            return false;
        }

        // 2. 点が角度範囲内にあるか
        self.point_in_angle_range(point, tolerance)
    }

    /// 点から楕円弧への最短距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        // 点が角度範囲内にある場合
        if self.point_in_angle_range(point, default_angle_tolerance::<T>()) {
            return self.ellipse.distance_to_point(point);
        }

        // 角度範囲外の場合は端点への距離
        let start_point = self.start_point();
        let end_point = self.end_point();

        let dist_to_start = point.distance_to(&start_point);
        let dist_to_end = point.distance_to(&end_point);

        dist_to_start.min(dist_to_end)
    }

    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> geo_core::Aabb2D<T> {
        use geo_core::Point2D;
        // 開始点と終了点
        let start = self.start_point();
        let end = self.end_point();

        let mut min_x = start.x().min(end.x());
        let mut max_x = start.x().max(end.x());
        let mut min_y = start.y().min(end.y());
        let mut max_y = start.y().max(end.y());

        // 楕円の極値点が角度範囲内にある場合を考慮
        // 角度範囲チェック
        let half_pi = T::PI / (T::ONE + T::ONE); // π/2
        let three_half_pi = T::PI + half_pi; // 3π/2
        let critical_angles = [T::ZERO, half_pi, T::PI, three_half_pi];

        for &angle in &critical_angles {
            if self.angle_in_range(angle) {
                let point = self.ellipse.point_at_parameter(angle);
                min_x = min_x.min(point.x());
                max_x = max_x.max(point.x());
                min_y = min_y.min(point.y());
                max_y = max_y.max(point.y());
            }
        }

        geo_core::Aabb2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// 角度が楕円弧の範囲内にあるかを判定
    pub fn angle_in_range(&self, angle: T) -> bool {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();

        if start_rad <= end_rad {
            angle >= start_rad && angle <= end_rad
        } else {
            // 角度が0を跨ぐ場合
            angle >= start_rad || angle <= end_rad
        }
    }

    /// 点が楕円弧の角度範囲内にあるかを判定
    pub fn point_in_angle_range(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let center = self.ellipse.center_internal();
        let to_point = Vector2D::new(point.x() - center.x(), point.y() - center.y());

        if to_point.magnitude() <= tolerance {
            return true; // 中心点の場合
        }

        let angle = to_point.angle();
        self.angle_in_range(angle.to_radians())
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    /// 境界上の点かどうかを判定
    pub fn on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }
}

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

impl<T: Scalar> EllipseArc2DConstructor<T> for EllipseArc2D<T> {
    fn new(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self> {
        let center_point = Point2D::new(center.0, center.1);
        let ellipse = Ellipse2D::new(center_point, semi_major, semi_minor, rotation)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }

    fn unit_ellipse_arc() -> Self {
        let center = Point2D::origin();
        let ellipse = Ellipse2D::new(center, T::ONE, T::ONE, T::ZERO).unwrap();
        let start = Angle::from_radians(T::ZERO);
        let end = Angle::from_radians(T::PI / (T::ONE + T::ONE)); // π/2
        Self::new(ellipse, start, end)
    }

    fn from_ellipse_and_angles(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self> {
        // 回転なし（0度）で楕円弧を作成
        let center_point = Point2D::new(center.0, center.1);
        let ellipse = Ellipse2D::new(center_point, semi_major, semi_minor, T::ZERO)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }

    // ========== Phase 2: 追加コンストラクタ ==========

    fn from_circle_arc(center: (T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self> {
        // 円弧は楕円弧の特殊ケース（a = b = radius）
        let center_point = Point2D::new(center.0, center.1);
        let ellipse = Ellipse2D::new(center_point, radius, radius, T::ZERO)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }

    fn from_three_points(start: (T, T), mid: (T, T), end: (T, T)) -> Option<Self> {
        // 3点を通る円弧として解釈
        let p1 = Point2D::new(start.0, start.1);
        let p2 = Point2D::new(mid.0, mid.1);
        let p3 = Point2D::new(end.0, end.1);

        let v1 = Vector2D::from_points(p1, p2);
        let v2 = Vector2D::from_points(p2, p3);

        let mid1 = Point2D::new(
            (p1.x() + p2.x()) / (T::ONE + T::ONE),
            (p1.y() + p2.y()) / (T::ONE + T::ONE),
        );
        let mid2 = Point2D::new(
            (p2.x() + p3.x()) / (T::ONE + T::ONE),
            (p2.y() + p3.y()) / (T::ONE + T::ONE),
        );

        let perp1 = Vector2D::new(-v1.y(), v1.x());
        let perp2 = Vector2D::new(-v2.y(), v2.x());

        let det = perp1.x() * perp2.y() - perp1.y() * perp2.x();
        if det.abs() < T::EPSILON {
            return None;
        }

        let diff_x = mid2.x() - mid1.x();
        let diff_y = mid2.y() - mid1.y();
        let t = (diff_x * perp2.y() - diff_y * perp2.x()) / det;

        let center = Point2D::new(mid1.x() + perp1.x() * t, mid1.y() + perp1.y() * t);
        let radius = Vector2D::from_points(center, p1).length();

        let start_vec = Vector2D::from_points(center, p1);
        let end_vec = Vector2D::from_points(center, p3);
        let start_angle = start_vec.y().atan2(start_vec.x());
        let end_angle = end_vec.y().atan2(end_vec.x());

        Self::from_circle_arc((center.x(), center.y()), radius, start_angle, end_angle)
    }

    fn from_center_and_points(center: (T, T), start: (T, T), end: (T, T)) -> Option<Self> {
        let c = Point2D::new(center.0, center.1);
        let p1 = Point2D::new(start.0, start.1);
        let p2 = Point2D::new(end.0, end.1);

        let r1 = Vector2D::from_points(c, p1).length();
        let r2 = Vector2D::from_points(c, p2).length();

        if (r1 - r2).abs() < T::EPSILON {
            let start_vec = Vector2D::from_points(c, p1);
            let end_vec = Vector2D::from_points(c, p2);
            let start_angle = start_vec.y().atan2(start_vec.x());
            let end_angle = end_vec.y().atan2(end_vec.x());
            return Self::from_circle_arc(center, r1, start_angle, end_angle);
        }

        let semi_major = r1.max(r2);
        let semi_minor = r1.min(r2);
        let start_vec = Vector2D::from_points(c, p1);
        let end_vec = Vector2D::from_points(c, p2);
        let start_angle = start_vec.y().atan2(start_vec.x());
        let end_angle = end_vec.y().atan2(end_vec.x());

        let center_point = Point2D::new(center.0, center.1);
        let ellipse = Ellipse2D::new(center_point, semi_major, semi_minor, T::ZERO)?;
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Some(Self::new(ellipse, start, end))
    }
}

impl<T: Scalar> EllipseArc2DProperties<T> for EllipseArc2D<T> {
    fn center(&self) -> (T, T) {
        let c = self.ellipse.center_internal();
        (c.x(), c.y())
    }

    fn semi_major_axis(&self) -> T {
        self.ellipse.semi_major_internal()
    }

    fn semi_minor_axis(&self) -> T {
        self.ellipse.semi_minor_internal()
    }

    fn start_angle(&self) -> T {
        self.start_angle.to_radians()
    }

    fn end_angle(&self) -> T {
        self.end_angle.to_radians()
    }

    // ========== Phase 2: 追加プロパティ ==========

    fn rotation(&self) -> T {
        self.ellipse.rotation()
    }

    fn sweep_angle(&self) -> T {
        // 角度範囲を計算（正規化）
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

impl<T: Scalar> EllipseArc2DDerived<T> for EllipseArc2D<T> {
    fn measure(&self) -> T {
        // arc_length の計算を直接展開: 楕円周囲長に角度比率を掛ける
        let full_perimeter = self.ellipse.perimeter();
        let angle_ratio =
            (self.end_angle.to_radians() - self.start_angle.to_radians()).abs() / T::TAU;
        full_perimeter * angle_ratio
    }

    fn bounding_box(&self) -> ((T, T), (T, T)) {
        // 開始点と終了点を取得
        let start = self.start_point();
        let end = self.end_point();

        let mut min_x = start.x().min(end.x());
        let mut max_x = start.x().max(end.x());
        let mut min_y = start.y().min(end.y());
        let mut max_y = start.y().max(end.y());

        // 極値を含むかチェック（0, π/2, π, 3π/2）
        let a = self.semi_major();
        let b = self.semi_minor();
        let rot = self.ellipse.rotation();
        let center = self.center();
        let start_angle = self.start_angle.to_radians();
        let end_angle = self.end_angle.to_radians();

        let two = T::ONE + T::ONE;
        let critical_angles = [T::ZERO, T::PI / two, T::PI, T::PI + T::PI / two];

        for &angle in &critical_angles {
            let in_range = if start_angle <= end_angle {
                angle >= start_angle && angle <= end_angle
            } else {
                angle >= start_angle || angle <= end_angle
            };

            if in_range {
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let cos_r = rot.cos();
                let sin_r = rot.sin();

                let x_local = a * cos_a;
                let y_local = b * sin_a;

                let x = center.x() + x_local * cos_r - y_local * sin_r;
                let y = center.y() + x_local * sin_r + y_local * cos_r;

                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }

        ((min_x, min_y), (max_x, max_y))
    }
}

impl<T: Scalar> EllipseArc2DEndpoint<T> for EllipseArc2D<T> {
    fn start_point(&self) -> (T, T) {
        let p = self
            .ellipse
            .point_at_parameter(self.start_angle.to_radians());
        (p.x(), p.y())
    }

    fn end_point(&self) -> (T, T) {
        let p = self.ellipse.point_at_parameter(self.end_angle.to_radians());
        (p.x(), p.y())
    }

    fn mid_point(&self) -> (T, T) {
        let p = self.point_at_parameter(T::ONE / (T::ONE + T::ONE));
        (p.x(), p.y())
    }
}

impl<T: Scalar> EllipseArc2DEvaluation<T> for EllipseArc2D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T) {
        let angle = self.start_angle.to_radians()
            + (self.end_angle.to_radians() - self.start_angle.to_radians()) * t;
        let p = self.ellipse.point_at_parameter(angle);
        (p.x(), p.y())
    }

    fn point_at_angle(&self, angle: T) -> Option<(T, T)> {
        // 角度が範囲内かチェック
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

        // 楕円上の点を計算
        let a = self.semi_major();
        let b = self.semi_minor();
        let rot = self.ellipse.rotation();
        let center = self.center();

        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let cos_r = rot.cos();
        let sin_r = rot.sin();

        let x_local = a * cos_a;
        let y_local = b * sin_a;

        let x = center.x() + x_local * cos_r - y_local * sin_r;
        let y = center.y() + x_local * sin_r + y_local * cos_r;

        Some((x, y))
    }
}

impl<T: Scalar> EllipseArc2DContainment<T> for EllipseArc2D<T> {
    fn contains_point(&self, point: (T, T), tolerance: T) -> bool {
        let p = Point2D::new(point.0, point.1);

        // まず楕円上にあるかチェック
        if !self.ellipse.contains_point(&p, tolerance) {
            return false;
        }

        // 次に角度範囲内かチェック
        let center = self.center();
        let vec = Vector2D::from_points(center, p);
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
