//! 2次元楕円（Ellipse2D）の新実装
//!
//! 新しいtraitsシステムに対応したEllipse2Dの実装

use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::prelude::{
    EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation,
};
use geo_foundation::{
    core::ellipse_core_traits::{Ellipse2DConstructor, Ellipse2DMeasure, Ellipse2DProperties},
    Scalar,
};

/// 2次元楕円
///
/// 長軸・短軸を持つ楕円を表現
/// 円は半径が等しい特殊な楕円として扱える
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse2D<T: Scalar> {
    center: Point2D<T>, // 中心点
    semi_major: T,      // 長半軸（a）
    semi_minor: T,      // 短半軸（b）
    rotation: T,        // 回転角（ラジアン、X軸からの回転）
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Ellipse2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい楕円を作成
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `semi_major` - 長半軸（a >= b である必要がある）
    /// * `semi_minor` - 短半軸
    /// * `rotation` - 回転角（ラジアン）
    pub fn new(center: Point2D<T>, semi_major: T, semi_minor: T, rotation: T) -> Option<Self> {
        if semi_major > T::ZERO && semi_minor > T::ZERO && semi_major >= semi_minor {
            Some(Self {
                center,
                semi_major,
                semi_minor,
                rotation,
            })
        } else {
            None
        }
    }

    /// 円から楕円を作成
    pub fn from_circle(circle: Circle2D<T>) -> Self {
        Self {
            center: circle.center_internal(),
            semi_major: circle.radius_internal(),
            semi_minor: circle.radius_internal(),
            rotation: T::ZERO,
        }
    }

    /// 軸に平行な楕円を作成（回転なし）
    pub fn axis_aligned(center: Point2D<T>, semi_major: T, semi_minor: T) -> Option<Self> {
        Self::new(center, semi_major, semi_minor, T::ZERO)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 長半軸を取得
    pub fn semi_major(&self) -> T {
        self.semi_major
    }

    /// 短半軸を取得
    pub fn semi_minor(&self) -> T {
        self.semi_minor
    }

    /// 回転角を取得（ラジアン）
    pub fn rotation(&self) -> T {
        self.rotation
    }

    /// 長軸の長さを取得
    pub fn major_axis(&self) -> T {
        self.semi_major * (T::ONE + T::ONE)
    }

    /// 短軸の長さを取得
    pub fn minor_axis(&self) -> T {
        self.semi_minor * (T::ONE + T::ONE)
    }

    /// 離心率を取得
    pub fn eccentricity(&self) -> T {
        if self.semi_major == T::ZERO {
            return T::ZERO;
        }

        let e_squared =
            T::ONE - (self.semi_minor * self.semi_minor) / (self.semi_major * self.semi_major);
        if e_squared <= T::ZERO {
            T::ZERO
        } else {
            e_squared.sqrt()
        }
    }

    /// 面積を取得
    pub fn area(&self) -> T {
        T::PI * self.semi_major * self.semi_minor
    }

    /// 周囲長を取得（ラマヌジャンの近似式）
    pub fn perimeter(&self) -> T {
        let a = self.semi_major;
        let b = self.semi_minor;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));

        // ラマヌジャンの第二近似式の簡易版
        let three = T::ONE + T::ONE + T::ONE;
        let ten = three + three + three + T::ONE;
        let four = T::ONE + T::ONE + T::ONE + T::ONE;

        T::PI * (a + b) * (T::ONE + (three * h) / (ten + (four - three * h).sqrt()))
    }

    // ========================================================================
    // Core Geometric Methods
    // ========================================================================

    /// 点が楕円内に含まれるかを判定
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let distance_to_boundary = self.distance_to_point(point);
        distance_to_boundary <= tolerance
    }

    /// 点から楕円境界への最短距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        // 楕円の中心を原点とする座標系に変換
        let translated = Point2D::new(point.x() - self.center.x(), point.y() - self.center.y());

        // 回転を考慮した座標変換
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let x_rot = translated.x() * cos_theta + translated.y() * sin_theta;
        let y_rot = -translated.x() * sin_theta + translated.y() * cos_theta;

        // geo_commonsの共通実装を使用
        geo_foundation::commons::ellipse_2d_distance_to_point(
            x_rot,
            y_rot,
            self.semi_major,
            self.semi_minor,
        )
    }

    /// パラメータ t での点を取得（0 <= t < 2π）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        // 楕円上の点（回転前）
        let x_local = self.semi_major * cos_t;
        let y_local = self.semi_minor * sin_t;

        // 回転を適用
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let x_rotated = x_local * cos_theta - y_local * sin_theta;
        let y_rotated = x_local * sin_theta + y_local * cos_theta;

        // 中心を考慮した最終座標
        Point2D::new(self.center.x() + x_rotated, self.center.y() + y_rotated)
    }

    /// パラメータ t での接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: T) -> Vector2D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        // 楕円の接線ベクトル（回転前）
        let dx_local = -self.semi_major * sin_t;
        let dy_local = self.semi_minor * cos_t;

        // 回転を適用
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let dx_rotated = dx_local * cos_theta - dy_local * sin_theta;
        let dy_rotated = dx_local * sin_theta + dy_local * cos_theta;

        Vector2D::new(dx_rotated, dy_rotated)
    }

    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> geo_core::Aabb2D<T> {
        use analysis::Point2;
        // 回転を考慮した楕円の境界ボックス計算
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let a = self.semi_major;
        let b = self.semi_minor;

        // 回転楕円の幅と高さ
        let width = ((a * cos_theta) * (a * cos_theta) + (b * sin_theta) * (b * sin_theta)).sqrt();
        let height = ((a * sin_theta) * (a * sin_theta) + (b * cos_theta) * (b * cos_theta)).sqrt();

        geo_core::Aabb2D::new(
            Point2::new(self.center.x() - width, self.center.y() - height),
            Point2::new(self.center.x() + width, self.center.y() + height),
        )
    }

    // ========================================================================
    // Conversion Methods
    // ========================================================================

    /// 円に変換（可能な場合）
    pub fn to_circle(&self) -> Option<Circle2D<T>> {
        if (self.semi_major - self.semi_minor).abs() <= T::EPSILON {
            Circle2D::new(self.center, self.semi_major)
        } else {
            None
        }
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self, tolerance: T) -> bool {
        (self.semi_major - self.semi_minor).abs() <= tolerance
    }
}

// ============================================================================
// Helper Methods
// ============================================================================

impl<T: Scalar> Ellipse2D<T> {
    /// 境界上の点かどうかを判定
    pub fn on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point(point);
        distance <= tolerance
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::TAU) // 0 から 2π
    }

    /// 長軸方向の単位ベクトルを取得
    pub fn major_axis_direction(&self) -> Vector2D<T> {
        Vector2D::new(self.rotation.cos(), self.rotation.sin())
    }

    /// 短軸方向の単位ベクトルを取得
    pub fn minor_axis_direction(&self) -> Vector2D<T> {
        Vector2D::new(-self.rotation.sin(), self.rotation.cos())
    }
}

// ============================================================================
// Advanced Calculation Traits Implementation
// ============================================================================

impl<T: Scalar> EllipseCalculation<T> for Ellipse2D<T> {
    type Point = Point2D<T>;

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T {
        self.semi_major
    }

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T {
        self.semi_minor
    }

    /// ラマヌジャン近似式I（標準版）による周長計算
    fn perimeter_ramanujan_i(&self) -> T {
        let a = self.semi_major;
        let b = self.semi_minor;
        let h = ((a - b) / (a + b)).powi(2);
        T::PI
            * (a + b)
            * (T::ONE
                + (T::from_f64(3.0) * h)
                    / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
    }

    /// ラマヌジャン近似式II（高精度版）による周長計算
    fn perimeter_ramanujan_ii(&self) -> T {
        geo_foundation::prelude::commons::ellipse_perimeter_ramanujan_ii(
            self.semi_major,
            self.semi_minor,
        )
    }

    /// パダン近似による周長計算（中程度精度）
    fn perimeter_pade(&self) -> T {
        geo_foundation::prelude::commons::ellipse_perimeter_padé(self.semi_major, self.semi_minor)
    }

    /// カントレル近似による周長計算（高精度）
    fn perimeter_cantrell(&self) -> T {
        geo_foundation::prelude::commons::ellipse_perimeter_cantrell(
            self.semi_major,
            self.semi_minor,
        )
    }

    /// 級数展開による周長計算（最高精度）
    fn perimeter_series(&self, terms: usize) -> T {
        let a = self.semi_major;
        let b = self.semi_minor;
        let m = ((a - b) / (a + b)).powi(2);

        let mut result = T::ONE;
        let mut coefficient = T::ONE;
        let mut m_power = m;

        for n in 1..=terms {
            coefficient *= T::from_f64((2.0 * n as f64 - 1.0) / (2.0 * n as f64));
            result += coefficient.powi(2) * m_power / T::from_f64(2.0 * n as f64 - 1.0);
            m_power *= m;
        }

        T::PI * (a + b) * result
    }

    /// 数値積分による周長計算（最高精度版）
    fn perimeter_numerical(&self, n_points: usize) -> T {
        let a = self.semi_major;
        let b = self.semi_minor;
        let dt = T::PI / T::from_f64(2.0 * n_points as f64);
        let mut sum = T::ZERO;

        for i in 0..n_points {
            let t = T::from_f64(i as f64) * dt;
            let sin_t = t.sin();
            let cos_t = t.cos();
            let dx_dt = -a * sin_t;
            let dy_dt = b * cos_t;
            let ds = (dx_dt * dx_dt + dy_dt * dy_dt).sqrt();
            sum += ds;
        }

        T::from_f64(4.0) * sum * dt
    }

    /// 楕円の離心率計算
    fn eccentricity(&self) -> T {
        geo_foundation::prelude::commons::ellipse_eccentricity(self.semi_major, self.semi_minor)
    }

    /// 楕円の焦点距離計算
    fn focal_distance(&self) -> T {
        geo_foundation::prelude::commons::ellipse_focal_distance(self.semi_major, self.semi_minor)
    }

    /// 楕円の面積計算
    fn area(&self) -> T {
        T::PI * self.semi_major * self.semi_minor
    }

    /// 楕円の焦点座標を計算
    fn foci(&self) -> (Point2D<T>, Point2D<T>) {
        let a = self.semi_major;
        let b = self.semi_minor;

        // 焦点間距離の半分
        let c = if a >= b {
            (a * a - b * b).sqrt()
        } else {
            T::ZERO // 円の場合、焦点は中心
        };

        // 回転と中心移動を考慮した焦点座標
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let f1_local = Point2D::new(c, T::ZERO);
        let f2_local = Point2D::new(-c, T::ZERO);

        // 回転変換
        let f1_rotated = Point2D::new(
            f1_local.x() * cos_rot - f1_local.y() * sin_rot,
            f1_local.x() * sin_rot + f1_local.y() * cos_rot,
        );
        let f2_rotated = Point2D::new(
            f2_local.x() * cos_rot - f2_local.y() * sin_rot,
            f2_local.x() * sin_rot + f2_local.y() * cos_rot,
        );

        // 中心移動
        let f1_final = Point2D::new(
            self.center.x() + f1_rotated.x(),
            self.center.y() + f1_rotated.y(),
        );
        let f2_final = Point2D::new(
            self.center.x() + f2_rotated.x(),
            self.center.y() + f2_rotated.y(),
        );

        (f1_final, f2_final)
    }
}

impl<T: Scalar> EllipseAdaptiveCalculation<T> for Ellipse2D<T> {
    /// 適応的周長計算（精度パラメータに基づく自動選択）
    fn perimeter_adaptive(&self, tolerance: T, _max_computation_cost: T) -> T {
        let eccentricity = self.eccentricity();

        if tolerance > T::from_f64(1e-3) {
            // 低精度の場合はラマヌジャンI
            self.perimeter_ramanujan_i()
        } else if tolerance > T::from_f64(1e-6) {
            // 中精度の場合はラマヌジャンII
            self.perimeter_ramanujan_ii()
        } else if eccentricity < T::from_f64(0.8) {
            // 高精度で離心率が低い場合はカントレル
            self.perimeter_cantrell()
        } else {
            // 最高精度の場合は級数展開
            self.perimeter_series(50)
        }
    }
}

impl<T: Scalar> EllipseAccuracyAnalysis<T> for Ellipse2D<T> {}

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

impl<T: Scalar> Ellipse2DConstructor<T> for Ellipse2D<T> {
    // ========== Phase 1 実装 ==========

    fn new(center: (T, T), semi_major: T, semi_minor: T, rotation: T) -> Option<Self> {
        let center_point = Point2D::new(center.0, center.1);
        Self::new(center_point, semi_major, semi_minor, rotation)
    }

    fn unit_ellipse() -> Self {
        Self::new(Point2D::origin(), T::ONE, T::ONE, T::ZERO)
            .expect("Unit ellipse should always be valid")
    }

    fn axis_aligned(center: (T, T), semi_major: T, semi_minor: T) -> Option<Self> {
        let c = Point2D::new(center.0, center.1);
        Self::new(c, semi_major, semi_minor, T::ZERO)
    }

    // ========== Phase 2 実装 ==========

    fn from_circle(center: (T, T), radius: T) -> Self {
        let c = Point2D::new(center.0, center.1);
        Self::new(c, radius, radius, T::ZERO).expect("Circle should be a valid ellipse")
    }

    fn from_foci_and_semi_major(focus1: (T, T), focus2: (T, T), semi_major: T) -> Option<Self> {
        let f1 = Point2D::new(focus1.0, focus1.1);
        let f2 = Point2D::new(focus2.0, focus2.1);

        // 中心点は焦点の中点
        let center = Point2D::new(
            (f1.x() + f2.x()) / (T::ONE + T::ONE),
            (f1.y() + f2.y()) / (T::ONE + T::ONE),
        );

        // 焦点間距離の半分 = c
        let focal_vec = Vector2D::from_points(f1, f2);
        let two_c = focal_vec.length();
        let c = two_c / (T::ONE + T::ONE);

        // b = sqrt(a^2 - c^2)
        if c > semi_major {
            return None; // 無効な楕円
        }
        let semi_minor = (semi_major * semi_major - c * c).sqrt();

        // 回転角を算出（f1 -> f2 の方向）
        let rotation = focal_vec.y().atan2(focal_vec.x());

        Self::new(center, semi_major, semi_minor, rotation)
    }

    fn centered_at_origin(semi_major: T, semi_minor: T, rotation: T) -> Option<Self> {
        Self::new(Point2D::origin(), semi_major, semi_minor, rotation)
    }
}

impl<T: Scalar> Ellipse2DProperties<T> for Ellipse2D<T> {
    // ========== Phase 1 実装 ==========

    fn center(&self) -> (T, T) {
        (self.center.x(), self.center.y())
    }

    fn semi_major_axis(&self) -> T {
        self.semi_major
    }

    fn semi_minor_axis(&self) -> T {
        self.semi_minor
    }

    fn rotation(&self) -> T {
        self.rotation
    }

    fn eccentricity(&self) -> T {
        self.eccentricity()
    }

    // ========== Phase 2 実装 ==========

    fn focal_distance(&self) -> T {
        geo_foundation::commons::EllipseCalculation::focal_distance(self)
    }

    fn focus1(&self) -> (T, T) {
        let (f1, _) = self.foci();
        (f1.x(), f1.y())
    }

    fn focus2(&self) -> (T, T) {
        let (_, f2) = self.foci();
        (f2.x(), f2.y())
    }
}

impl<T: Scalar + From<f64>> Ellipse2DMeasure<T> for Ellipse2D<T> {
    // ========== Phase 1 実装 ==========

    fn measure(&self) -> T {
        self.area()
    }

    fn perimeter(&self) -> T {
        self.perimeter_ramanujan_ii()
    }

    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        let tolerance = geo_foundation::GEOMETRIC_DISTANCE_TOLERANCE.into();
        self.contains_point(&p, tolerance)
    }

    fn is_circle(&self) -> bool {
        let tolerance = geo_foundation::GEOMETRIC_DISTANCE_TOLERANCE.into();
        self.is_circle(tolerance)
    }

    // ========== Phase 2 実装 ==========

    fn point_at_parameter(&self, t: T) -> (T, T) {
        // パラメトリック方程式で楕円上の点を計算
        let cos_t = t.cos();
        let sin_t = t.sin();
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        // ローカル座標系での点
        let x_local = self.semi_major * cos_t;
        let y_local = self.semi_minor * sin_t;

        // 回転変換
        let x_rotated = x_local * cos_rot - y_local * sin_rot;
        let y_rotated = x_local * sin_rot + y_local * cos_rot;

        // 中心移動
        (self.center.x() + x_rotated, self.center.y() + y_rotated)
    }

    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        self.distance_to_point(&p)
    }

    fn point_on_boundary(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        let tolerance = geo_foundation::GEOMETRIC_DISTANCE_TOLERANCE.into();
        // 点が楕円上にあるか判定：点から楕円周への距離が許容誤差以内
        let dist = self.distance_to_point(&p);
        dist <= tolerance
    }

    fn linear_eccentricity(&self) -> T {
        // 線形離心率 c = sqrt(a^2 - b^2)
        let a = self.semi_major;
        let b = self.semi_minor;
        (a * a - b * b).sqrt()
    }
}
