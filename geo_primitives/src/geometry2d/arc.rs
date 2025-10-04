use geo_core::{Scalar, ToleranceContext, Vector2D};
use crate::geometry2d::{Point2D, Circle2D, Direction2D};

/// 2D円弧：円の一部分を表現
#[derive(Debug, Clone)]
pub struct Arc2D {
    circle: Circle2D,
    start_angle: Scalar,
    end_angle: Scalar,
}

impl Arc2D {
    /// 円とアングルから円弧を作成
    pub fn new(circle: Circle2D, start_angle: Scalar, end_angle: Scalar) -> Self {
        // 角度を正規化（0-2π範囲）
        let start_norm = Scalar::new(start_angle.value().rem_euclid(2.0 * std::f64::consts::PI));
        let end_norm = Scalar::new(end_angle.value().rem_euclid(2.0 * std::f64::consts::PI));

        Self {
            circle,
            start_angle: start_norm,
            end_angle: end_norm,
        }
    }

    /// 中心点、半径、開始角度、終了角度から円弧を作成
    pub fn from_center_radius(center: Point2D, radius: Scalar, start_angle: Scalar, end_angle: Scalar) -> Option<Self> {
        if let Some(circle) = Circle2D::new(center, radius, Direction2D::unit_x()) {
            Some(Self::new(circle, start_angle, end_angle))
        } else {
            None
        }
    }

    /// f64値からの簡易コンストラクタ
    pub fn from_f64(center_x: f64, center_y: f64, radius: f64, start_angle: f64, end_angle: f64) -> Option<Self> {
        let center = Point2D::new(center_x, center_y);
        if let Some(circle) = Circle2D::from_f64(center, radius) {
            Some(Self::new(circle, Scalar::new(start_angle), Scalar::new(end_angle)))
        } else {
            None
        }
    }

    /// 3点から円弧を作成
    pub fn from_three_points(start: Point2D, mid: Point2D, end: Point2D) -> Option<Self> {
        // 3点を通る円の中心を計算
        let d = 2.0 * (start.x() * (mid.y() - end.y()) +
                      mid.x() * (end.y() - start.y()) +
                      end.x() * (start.y() - mid.y()));

        if d.abs() < 1e-10 {
            return None; // 3点が一直線上
        }

        let ux = ((start.x() * start.x() + start.y() * start.y()) * (mid.y() - end.y()) +
                  (mid.x() * mid.x() + mid.y() * mid.y()) * (end.y() - start.y()) +
                  (end.x() * end.x() + end.y() * end.y()) * (start.y() - mid.y())) / d;

        let uy = ((start.x() * start.x() + start.y() * start.y()) * (end.x() - mid.x()) +
                  (mid.x() * mid.x() + mid.y() * mid.y()) * (start.x() - end.x()) +
                  (end.x() * end.x() + end.y() * end.y()) * (mid.x() - start.x())) / d;

        let center = Point2D::new(ux, uy);
        let radius = ((start.x() - ux).powi(2) + (start.y() - uy).powi(2)).sqrt();

        if let Some(circle) = Circle2D::from_f64(center, radius) {
            let start_angle = Scalar::new((start.y() - uy).atan2(start.x() - ux));
            let end_angle = Scalar::new((end.y() - uy).atan2(end.x() - ux));
            Some(Self::new(circle, start_angle, end_angle))
        } else {
            None
        }
    }

    /// 点から中心への角度を計算
    pub fn angle_of(&self, point: &Point2D) -> Scalar {
        let dx = point.x() - self.center().x();
        let dy = point.y() - self.center().y();
        Scalar::new(dy.atan2(dx))
    }

    /// 掃引角度（sweep angle）を計算
    pub fn sweep_angle(&self) -> Scalar {
        let start = self.start_angle.value();
        let end = self.end_angle.value();

        let sweep = if end >= start {
            end - start
        } else {
            (2.0 * std::f64::consts::PI) - start + end
        };

        Scalar::new(sweep)
    }

    /// ベース円へのアクセス
    pub fn base_circle(&self) -> &Circle2D {
        &self.circle
    }

    /// 中心点を取得
    pub fn center(&self) -> &Point2D {
        self.circle.center()
    }

    /// 半径を取得
    pub fn radius(&self) -> Scalar {
        self.circle.radius()
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Scalar {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Scalar {
        self.end_angle
    }

    /// 弧長を計算
    pub fn length(&self) -> Scalar {
        self.radius() * self.sweep_angle()
    }

    /// パラメータt（0.0～1.0）に対応する円弧上の点を返す
    pub fn evaluate(&self, t: f64) -> Point2D {
        let angle = self.start_angle.value() + t * self.sweep_angle().value();
        self.circle.evaluate(angle)
    }

    /// 開始点
    pub fn start_point(&self) -> Point2D {
        self.evaluate(0.0)
    }

    /// 終了点
    pub fn end_point(&self) -> Point2D {
        self.evaluate(1.0)
    }

    /// 中点
    pub fn midpoint(&self) -> Point2D {
        self.evaluate(0.5)
    }

    /// パラメータtにおける接線方向
    pub fn tangent(&self, t: f64) -> Option<Direction2D> {
        let angle = self.start_angle.value() + t * self.sweep_angle().value();
        let tangent_vector = self.circle.tangent(angle);
        Direction2D::from_vector(&tangent_vector)
    }

    /// パラメータtにおける法線方向
    pub fn normal(&self, t: f64) -> Option<Direction2D> {
        let angle = self.start_angle.value() + t * self.sweep_angle().value();
        let normal_vector = self.circle.normal(angle);
        Direction2D::from_vector(&normal_vector)
    }

    /// 角度がこの円弧の範囲内にあるかチェック
    pub fn contains_angle(&self, angle: Scalar, tolerance: &ToleranceContext) -> bool {
        let normalized_angle = angle.value().rem_euclid(2.0 * std::f64::consts::PI);
        let start = self.start_angle.value();
        let end = self.end_angle.value();

        if start <= end {
            normalized_angle >= start - tolerance.angular && normalized_angle <= end + tolerance.angular
        } else {
            normalized_angle >= start - tolerance.angular || normalized_angle <= end + tolerance.angular
        }
    }

    /// 点が円弧上にあるかどうか
    pub fn contains_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> bool {
        // まず円周上にあるかチェック
        if !self.circle.contains_point(point, tolerance) {
            return false;
        }

        // 角度が範囲内にあるかチェック
        let angle = self.angle_of(point);
        self.contains_angle(angle, tolerance)
    }

    /// 点までの距離
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar {
        if self.contains_point(point, &ToleranceContext::standard()) {
            // 円弧上の場合、距離は0
            Scalar::new(0.0)
        } else {
            // 円弧上にない場合、最も近い点（開始点または終了点）までの距離
            let start_point = self.start_point();
            let end_point = self.end_point();

            let dist_to_start = Scalar::new(((point.x() - start_point.x()).powi(2) +
                                            (point.y() - start_point.y()).powi(2)).sqrt());
            let dist_to_end = Scalar::new(((point.x() - end_point.x()).powi(2) +
                                          (point.y() - end_point.y()).powi(2)).sqrt());

            if dist_to_start.value() < dist_to_end.value() {
                dist_to_start
            } else {
                dist_to_end
            }
        }
    }

    /// 平行移動
    pub fn translate(&self, translation: &Vector2D) -> Arc2D {
        let translated_circle = self.circle.translate(translation);
        Self::new(translated_circle, self.start_angle, self.end_angle)
    }

    /// 原点周りの回転
    pub fn rotate(&self, angle: f64) -> Arc2D {
        let rotated_circle = self.circle.rotate(angle);
        let new_start_angle = Scalar::new(self.start_angle.value() + angle);
        let new_end_angle = Scalar::new(self.end_angle.value() + angle);
        Self::new(rotated_circle, new_start_angle, new_end_angle)
    }

    /// スケーリング
    pub fn scale(&self, factor: Scalar) -> Option<Arc2D> {
        if let Some(scaled_circle) = self.circle.scale(factor) {
            Some(Self::new(scaled_circle, self.start_angle, self.end_angle))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::ToleranceContext;

    #[test]
    fn test_arc_creation() {
        let arc = Arc2D::from_f64(0.0, 0.0, 1.0, 0.0, std::f64::consts::PI);
        assert!(arc.is_some());

        let arc = arc.unwrap();
        assert_eq!(arc.radius().value(), 1.0);
        assert_eq!(arc.start_angle().value(), 0.0);
        assert_eq!(arc.end_angle().value(), std::f64::consts::PI);
    }

    #[test]
    fn test_arc_evaluation() {
        let arc = Arc2D::from_f64(0.0, 0.0, 1.0, 0.0, std::f64::consts::PI).unwrap();

        let start_point = arc.start_point();
        assert!((start_point.x() - 1.0).abs() < 1e-10);
        assert!(start_point.y().abs() < 1e-10);

        let end_point = arc.end_point();
        assert!((end_point.x() + 1.0).abs() < 1e-10);
        assert!(end_point.y().abs() < 1e-10);
    }

    #[test]
    fn test_arc_length() {
        let arc = Arc2D::from_f64(0.0, 0.0, 1.0, 0.0, std::f64::consts::PI).unwrap();
        let expected_length = std::f64::consts::PI; // 半円の弧長
        assert!((arc.length().value() - expected_length).abs() < 1e-10);
    }

    #[test]
    fn test_contains_point() {
        let arc = Arc2D::from_f64(0.0, 0.0, 1.0, 0.0, std::f64::consts::PI).unwrap();
        let tolerance = ToleranceContext::standard();

        // 弧上の点
        let point_on_arc = Point2D::new(0.0, 1.0);
        assert!(arc.contains_point(&point_on_arc, &tolerance));

        // 弧外の点
        let point_off_arc = Point2D::new(0.0, -1.0);
        assert!(!arc.contains_point(&point_off_arc, &tolerance));
    }

    #[test]
    fn test_three_point_arc() {
        let start = Point2D::new(1.0, 0.0);
        let mid = Point2D::new(0.0, 1.0);
        let end = Point2D::new(-1.0, 0.0);

        let arc = Arc2D::from_three_points(start, mid, end);
        assert!(arc.is_some());

        let arc = arc.unwrap();
        assert!((arc.radius().value() - 1.0).abs() < 1e-10);
        assert!((arc.center().x().abs()).abs() < 1e-10);
        assert!((arc.center().y().abs()).abs() < 1e-10);
    }
}
