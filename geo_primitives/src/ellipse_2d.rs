//! 2次元楕円（Ellipse2D）の新実装
//!
//! 新しいtraitsシステムに対応したEllipse2Dの実装

use crate::{BBox2D, Circle2D, Point2D, Vector2D};
use geo_foundation::Scalar;

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
            center: circle.center(),
            semi_major: circle.radius(),
            semi_minor: circle.radius(),
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

        // 正規化された楕円座標での距離計算
        let x_norm = x_rot / self.semi_major;
        let y_norm = y_rot / self.semi_minor;
        let normalized_distance = (x_norm * x_norm + y_norm * y_norm).sqrt();

        if normalized_distance <= T::ONE {
            // 点が楕円内部にある場合
            T::ZERO
        } else {
            // 点が楕円外部にある場合の近似距離
            // より正確な計算には数値的手法が必要
            let scale = T::ONE / normalized_distance;
            let boundary_x = x_rot * scale;
            let boundary_y = y_rot * scale;

            ((x_rot - boundary_x) * (x_rot - boundary_x)
                + (y_rot - boundary_y) * (y_rot - boundary_y))
                .sqrt()
        }
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
    pub fn bounding_box(&self) -> BBox2D<T> {
        // 回転を考慮した楕円の境界ボックス計算
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let a = self.semi_major;
        let b = self.semi_minor;

        // 回転楕円の幅と高さ
        let width = ((a * cos_theta) * (a * cos_theta) + (b * sin_theta) * (b * sin_theta)).sqrt();
        let height = ((a * sin_theta) * (a * sin_theta) + (b * cos_theta) * (b * cos_theta)).sqrt();

        BBox2D::from_center_size(
            self.center,
            width * (T::ONE + T::ONE),
            height * (T::ONE + T::ONE),
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
