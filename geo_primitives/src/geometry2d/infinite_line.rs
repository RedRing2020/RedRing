use geo_core::{Point2D, Vector2D, Scalar, ToleranceContext, TolerantEq};
use crate::geometry2d::Direction2D;

/// 2D無限直線
///
/// 原点と方向ベクトルで定義される無限に長い直線
#[derive(Debug, Clone)]
pub struct InfiniteLine2D {
    origin: Point2D,
    direction: Direction2D,
}

impl InfiniteLine2D {
    /// 原点と方向から無限直線を作成
    pub fn new(origin: Point2D, direction: Direction2D) -> Self {
        Self { origin, direction }
    }

    /// f64座標から無限直線を作成
    pub fn from_f64(origin_x: f64, origin_y: f64, direction_x: f64, direction_y: f64) -> Option<Self> {
        let origin = Point2D::new(Scalar::new(origin_x), Scalar::new(origin_y));
        // 直接 f64 ベクトルを構築
        let dir_vec = Vector2D::new(direction_x, direction_y);
        let direction = Direction2D::from_vector(&dir_vec)?;
        Some(Self { origin, direction })
    }

    /// 2つの点を通る無限直線を作成
    pub fn from_points(p1: &Point2D, p2: &Point2D) -> Option<Self> {
        let direction_vector = Vector2D::new(
            p2.x().value() - p1.x().value(),
            p2.y().value() - p1.y().value(),
        );
        let direction = Direction2D::from_vector(&direction_vector)?;
        Some(Self { origin: p1.clone(), direction })
    }

    /// 原点を取得
    pub fn origin(&self) -> &Point2D {
        &self.origin
    }

    /// 方向を取得
    pub fn direction(&self) -> &Direction2D {
        &self.direction
    }

    /// f64パラメータでの点を評価（コア）
    pub fn evaluate_f64(&self, t: f64) -> Point2D {
        let ox = self.origin.x().value();
        let oy = self.origin.y().value();
        let dx = self.direction.x();
        let dy = self.direction.y();
        Point2D::new(Scalar::new(ox + dx * t), Scalar::new(oy + dy * t))
    }

    #[deprecated(note = "Use evaluate_f64(t: f64) instead")]
    pub fn evaluate(&self, t: Scalar) -> Point2D { self.evaluate_f64(t.value()) }

    /// 点から直線までの距離
    pub fn distance_to_point_f64(&self, point: &Point2D) -> f64 {
        let ox = self.origin.x().value(); let oy = self.origin.y().value();
        let px = point.x().value(); let py = point.y().value();
        let dx = self.direction.x(); let dy = self.direction.y();
        let vx = px - ox; let vy = py - oy;
        (vx * dy - vy * dx).abs()
    }

    #[deprecated(note = "Use distance_to_point_f64(&Point2D) -> f64 instead")]
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar { Scalar::new(self.distance_to_point_f64(point)) }

    /// 点が直線上にあるか判定
    pub fn contains_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> bool {
        self.distance_to_point_f64(point) <= tolerance.linear
    }

    /// f64での点が直線上にあるか判定
    pub fn contains_point_f64(&self, point: &Point2D, epsilon: f64) -> bool {
        self.distance_to_point_f64(point) < epsilon
    }

    /// 右手座標系での法線ベクトル
    pub fn right_hand_normal(&self) -> Direction2D {
        let dir_vec = self.direction.to_vector();
        let normal_vec = Vector2D::new(-dir_vec.y(), dir_vec.x());
        Direction2D::from_vector(&normal_vec).expect("法線ベクトルは非ゼロ")
    }

    /// 他の無限直線との交点
    pub fn intersection_with_infinite_line(&self, other: &InfiniteLine2D, tolerance: &ToleranceContext) -> Option<Point2D> {
        let dx1 = self.direction.x(); let dy1 = self.direction.y();
        let dx2 = other.direction.x(); let dy2 = other.direction.y();
        let cross = dx1 * dy2 - dy1 * dx2; // 2D 外積
        if cross.abs() <= tolerance.linear {
            if self.contains_point(&other.origin, tolerance) { Some(self.origin.clone()) } else { None }
        } else {
            let ox1 = self.origin.x().value(); let oy1 = self.origin.y().value();
            let ox2 = other.origin.x().value(); let oy2 = other.origin.y().value();
            let dx = ox2 - ox1; let dy = oy2 - oy1;
            let t = (dx * dy2 - dy * dx2) / cross;
            Some(self.evaluate_f64(t))
        }
    }

    /// f64での他の無限直線との交点
    pub fn intersection_with_infinite_line_f64(&self, other: &InfiniteLine2D, _epsilon: f64) -> Option<Point2D> {
        let tolerance = ToleranceContext::standard();
        self.intersection_with_infinite_line(other, &tolerance)
    }

    /// 直線を移動
    pub fn translate_f64(&self, dx: f64, dy: f64) -> InfiniteLine2D {
        InfiniteLine2D {
            origin: Point2D::new(
                Scalar::new(self.origin.x().value() + dx),
                Scalar::new(self.origin.y().value() + dy),
            ),
            direction: self.direction.clone(),
        }
    }

    #[deprecated(note = "Use translate_f64(dx, dy) instead")]
    pub fn translate(&self, dx: Scalar, dy: Scalar) -> InfiniteLine2D { self.translate_f64(dx.value(), dy.value()) }
}

impl PartialEq for InfiniteLine2D {
    fn eq(&self, other: &Self) -> bool {
        let tolerance = ToleranceContext::default();

        // 原点が他方の直線上にあり、方向が同じ（または逆）かを確認
        other.contains_point(&self.origin, &tolerance) &&
        (self.direction.tolerant_eq(&other.direction, &tolerance) ||
         self.direction.tolerant_eq(&other.direction.negate(), &tolerance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_line_creation() {
        let origin = Point2D::new(Scalar::new(1.0), Scalar::new(2.0));
    let direction = Direction2D::from_vector(&Vector2D::new(1.0, 0.0)).unwrap();
        let line = InfiniteLine2D::new(origin.clone(), direction.clone());

        assert!(line.origin().tolerant_eq(&origin, &ToleranceContext::default()));
        assert!(line.direction().tolerant_eq(&direction, &ToleranceContext::default()));
    }

    #[test]
    fn test_infinite_line_from_f64() {
    let line = InfiniteLine2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();

        assert_eq!(line.origin().x().value(), 1.0);
        assert_eq!(line.origin().y().value(), 2.0);
        assert_eq!(line.direction().x(), 1.0);
        assert_eq!(line.direction().y(), 0.0);
    }

    #[test]
    fn test_infinite_line_from_points() {
    let p1 = Point2D::new(Scalar::new(0.0), Scalar::new(0.0));
    let p2 = Point2D::new(Scalar::new(3.0), Scalar::new(4.0));
        let line = InfiniteLine2D::from_points(&p1, &p2).unwrap();

        // 方向ベクトルが正規化されていることを確認
        let dir_vec = line.direction().to_vector();
    let magnitude = (dir_vec.x() * dir_vec.x() + dir_vec.y() * dir_vec.y()).sqrt();
        assert!((magnitude - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate() {
        let line = InfiniteLine2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();
        let point = line.evaluate_f64(3.0);

        assert_eq!(point.x().value(), 4.0);
        assert_eq!(point.y().value(), 2.0);
    }

    #[test]
    fn test_distance_to_point() {
        // y = 0の直線
        let line = InfiniteLine2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap();
        let point = Point2D::new(Scalar::new(5.0), Scalar::new(3.0));

        let distance = line.distance_to_point_f64(&point);
        assert_eq!(distance, 3.0);
    }

    #[test]
    fn test_contains_point() {
        let line = InfiniteLine2D::from_f64(0.0, 0.0, 1.0, 1.0).unwrap(); // y = x
        let point_on_line = Point2D::new(Scalar::new(5.0), Scalar::new(5.0));
        let point_off_line = Point2D::new(Scalar::new(5.0), Scalar::new(3.0));

        assert!(line.contains_point_f64(&point_on_line, 1e-10));
        assert!(!line.contains_point_f64(&point_off_line, 1e-10));
    }

    #[test]
    fn test_right_hand_normal() {
        let line = InfiniteLine2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap(); // 水平線
        let normal = line.right_hand_normal();

        // 水平線の右手法線は上向き
        assert_eq!(normal.x(), 0.0);
        assert_eq!(normal.y(), 1.0);
    }

    #[test]
    fn test_intersection_parallel_lines() {
        let line1 = InfiniteLine2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap();
        let line2 = InfiniteLine2D::from_f64(0.0, 1.0, 1.0, 0.0).unwrap(); // 平行線

        let intersection = line1.intersection_with_infinite_line_f64(&line2, 1e-10);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_intersection_same_line() {
        let line1 = InfiniteLine2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap();
        let line2 = InfiniteLine2D::from_f64(5.0, 0.0, 1.0, 0.0).unwrap(); // 同じ直線

        let intersection = line1.intersection_with_infinite_line_f64(&line2, 1e-10);
        assert!(intersection.is_some());
    }

    #[test]
    fn test_intersection_crossing_lines() {
        let line1 = InfiniteLine2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap(); // y = 0
        let line2 = InfiniteLine2D::from_f64(0.0, 0.0, 0.0, 1.0).unwrap(); // x = 0

        let intersection = line1.intersection_with_infinite_line_f64(&line2, 1e-10).unwrap();
        assert_eq!(intersection.x().value(), 0.0);
        assert_eq!(intersection.y().value(), 0.0);
    }

    #[test]
    fn test_translate() {
        let line = InfiniteLine2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();
        let translated = line.translate_f64(3.0, 4.0);

        assert_eq!(translated.origin().x().value(), 4.0);
        assert_eq!(translated.origin().y().value(), 6.0);
        assert_eq!(translated.direction().x(), line.direction().x());
        assert_eq!(translated.direction().y(), line.direction().y());
    }

    #[test]
    fn test_equality() {
        let line1 = InfiniteLine2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap();
        let line2 = InfiniteLine2D::from_f64(5.0, 0.0, 1.0, 0.0).unwrap(); // 同じ直線、異なる原点
        let line3 = InfiniteLine2D::from_f64(0.0, 1.0, 1.0, 0.0).unwrap(); // 平行だが異なる直線

        assert_eq!(line1, line2);
        assert_ne!(line1, line3);
    }

    #[test]
    fn test_zero_direction_vector() {
        let result = InfiniteLine2D::from_f64(0.0, 0.0, 0.0, 0.0);
        assert!(result.is_none()); // ゼロベクトルは方向として無効
    }
}
