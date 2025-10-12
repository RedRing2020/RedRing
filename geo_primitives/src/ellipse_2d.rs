//! 2次元楕円（Ellipse2D）の Core 実装
//!
//! Core Foundation パターンに基づく Ellipse2D の必須機能のみ
//! 拡張機能は ellipse_2d_extensions.rs を参照

use crate::{BBox2D, Circle2D, Point2D, Vector2D};
use geo_foundation::{
    abstract_types::geometry::core_foundation::{
        BasicContainment, BasicMetrics, BasicParametric, CoreFoundation,
    },
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

    /// 軸が整列した楕円を作成（回転なし）
    pub fn axis_aligned(center: Point2D<T>, semi_major: T, semi_minor: T) -> Option<Self> {
        Self::new(center, semi_major, semi_minor, T::ZERO)
    }

    /// 円から楕円を作成
    pub fn from_circle(circle: &Circle2D<T>) -> Self {
        Self {
            center: circle.center(),
            semi_major: circle.radius(),
            semi_minor: circle.radius(),
            rotation: T::ZERO,
        }
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 長半軸を取得
    pub fn semi_major_axis(&self) -> T {
        self.semi_major
    }

    /// 短半軸を取得
    pub fn semi_minor_axis(&self) -> T {
        self.semi_minor
    }

    /// 回転角を取得
    pub fn rotation(&self) -> T {
        self.rotation
    }

    // ========================================================================
    // Core Metrics Methods
    // ========================================================================

    /// 面積を計算
    pub fn area(&self) -> T {
        T::PI * self.semi_major * self.semi_minor
    }

    /// 周長を近似計算（ラマヌジャンの公式）
    pub fn perimeter(&self) -> T {
        let a = self.semi_major;
        let b = self.semi_minor;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        let three = T::from_f64(3.0);
        let ten = T::from_f64(10.0);
        let four = T::from_f64(4.0);

        T::PI * (a + b) * (T::ONE + (three * h) / (ten + (four - three * h).sqrt()))
    }

    // ========================================================================
    // Core Parametric Methods
    // ========================================================================

    /// パラメータ t での点を取得（0 <= t <= 2π）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        // 楕円上の点（ローカル座標）
        let local_x = self.semi_major * cos_t;
        let local_y = self.semi_minor * sin_t;

        // 回転変換
        let rotated_x = local_x * cos_rot - local_y * sin_rot;
        let rotated_y = local_x * sin_rot + local_y * cos_rot;

        // 中心への平行移動
        Point2D::new(self.center.x() + rotated_x, self.center.y() + rotated_y)
    }

    /// パラメータ t での接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: T) -> Vector2D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        // 接線ベクトル（ローカル座標）
        let local_dx = -self.semi_major * sin_t;
        let local_dy = self.semi_minor * cos_t;

        // 回転変換
        let rotated_dx = local_dx * cos_rot - local_dy * sin_rot;
        let rotated_dy = local_dx * sin_rot + local_dy * cos_rot;

        Vector2D::new(rotated_dx, rotated_dy)
    }

    // ========================================================================
    // Core Containment Methods
    // ========================================================================

    /// 点が楕円内部にあるかを判定
    pub fn contains_point(&self, point: &Point2D<T>) -> bool {
        // 中心からの相対位置
        let rel_x = point.x() - self.center.x();
        let rel_y = point.y() - self.center.y();

        // 回転の逆変換
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        let local_x = rel_x * cos_rot + rel_y * sin_rot;
        let local_y = -rel_x * sin_rot + rel_y * cos_rot;

        // 楕円の方程式: (x/a)² + (y/b)² <= 1
        let term_x = local_x / self.semi_major;
        let term_y = local_y / self.semi_minor;
        term_x * term_x + term_y * term_y <= T::ONE
    }

    /// 点から楕円への最短距離を近似計算
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        // 中心からの相対位置
        let rel_x = point.x() - self.center.x();
        let rel_y = point.y() - self.center.y();

        // 回転の逆変換
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();
        let local_x = rel_x * cos_rot + rel_y * sin_rot;
        let local_y = -rel_x * sin_rot + rel_y * cos_rot;

        // 楕円の標準形での距離計算（近似）
        // 正確な計算は複雑なので、最も近い楕円上の点を求める簡略化
        let t = local_y.atan2(local_x); // 角度の近似
        let ellipse_point = self.point_at_parameter(t);
        point.distance_to(&ellipse_point)
    }

    /// 境界ボックスを計算
    pub fn bounding_box(&self) -> BBox2D<T> {
        // 回転を考慮した境界ボックス計算
        let cos_rot = self.rotation.cos();
        let sin_rot = self.rotation.sin();

        let a_cos = self.semi_major * cos_rot;
        let a_sin = self.semi_major * sin_rot;
        let b_cos = self.semi_minor * cos_rot;
        let b_sin = self.semi_minor * sin_rot;

        let width = (a_cos * a_cos + b_sin * b_sin).sqrt();
        let height = (a_sin * a_sin + b_cos * b_cos).sqrt();

        BBox2D::new(
            Point2D::new(self.center.x() - width, self.center.y() - height),
            Point2D::new(self.center.x() + width, self.center.y() + height),
        )
    }
}

// ============================================================================
// Core Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> CoreFoundation<T> for Ellipse2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
    }
}

impl<T: Scalar> BasicMetrics<T> for Ellipse2D<T> {
    fn length(&self) -> Option<T> {
        Some(self.perimeter())
    }
}

impl<T: Scalar> BasicContainment<T> for Ellipse2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point(point)
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        let distance = self.distance_to_point(point);
        distance <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to_point(point)
    }
}

impl<T: Scalar> BasicParametric<T> for Ellipse2D<T> {
    fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::TAU)
    }

    fn point_at_parameter(&self, t: T) -> Self::Point {
        self.point_at_parameter(t)
    }

    fn tangent_at_parameter(&self, t: T) -> Self::Vector {
        self.tangent_at_parameter(t)
    }
}
