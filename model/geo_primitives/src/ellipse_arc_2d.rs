//! 2次元楕円弧（EllipseArc2D）のCore実装
//!
//! Foundation統一システムに基づくEllipseArc2Dの必須機能のみ

use crate::{BBox2D, Ellipse2D, Point2D, Vector2D};
use geo_foundation::{Angle, Scalar};

/// 2次元楕円弧
///
/// 楕円の一部分を表現する楕円弧
/// 開始角度と終了角度で定義される
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipseArc2D<T: Scalar> {
    ellipse: Ellipse2D<T>, // 基底楕円
    start_angle: Angle<T>, // 開始角度
    end_angle: Angle<T>,   // 終了角度
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
        self.ellipse.center()
    }

    /// 長半軸を取得
    pub fn semi_major(&self) -> T {
        self.ellipse.semi_major()
    }

    /// 短半軸を取得
    pub fn semi_minor(&self) -> T {
        self.ellipse.semi_minor()
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
        if self.point_in_angle_range(point, T::EPSILON) {
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
    pub fn bounding_box(&self) -> BBox2D<T> {
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

        BBox2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
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
        let center = self.ellipse.center();
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

// TODO: Foundation実装は後で段階的に追加
