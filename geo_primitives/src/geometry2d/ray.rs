use geo_core::{Point2D, Vector2D, Scalar, ToleranceContext, TolerantEq};
use crate::geometry2d::Direction2D;

/// 2D半直線（レイ）
///
/// 原点から特定方向に無限に伸びる半直線
#[derive(Debug, Clone)]
pub struct Ray2D {
    origin: Point2D,
    direction: Direction2D,
}

impl Ray2D {
    /// 原点と方向から半直線を作成
    pub fn new(origin: Point2D, direction: Direction2D) -> Self {
        Self { origin, direction }
    }

    /// f64座標から半直線を作成
    pub fn from_f64(origin_x: f64, origin_y: f64, direction_x: f64, direction_y: f64) -> Option<Self> {
        let origin = Point2D::new(origin_x, origin_y);
        let direction = Direction2D::from_f64(direction_x, direction_y)?;
        Some(Self { origin, direction })
    }

    /// 2つの点を通る半直線を作成（第一点が原点、第二点への方向）
    pub fn from_points(start: &Point2D, through: &Point2D) -> Option<Self> {
        let direction_vector = Vector2D::new(
            through.x() - start.x(),
            through.y() - start.y(),
        );
        let direction = Direction2D::from_vector(&direction_vector)?;
        Some(Self {
            origin: start.clone(),
            direction,
        })
    }

    /// 原点を取得
    pub fn origin(&self) -> &Point2D {
        &self.origin
    }

    /// 方向を取得
    pub fn direction(&self) -> &Direction2D {
        &self.direction
    }

    /// パラメータt (t >= 0) での点を評価
    pub fn evaluate_f64(&self, t: f64) -> Option<Point2D> {
        if t < 0.0 { return None; }
    let ox = self.origin.x();
    let oy = self.origin.y();
        let dx = self.direction.x();
        let dy = self.direction.y();
        Some(Point2D::new(ox + dx * t, oy + dy * t))
    }

    #[deprecated(note = "Use evaluate_f64(t: f64) instead")]
    pub fn evaluate(&self, t: Scalar) -> Option<Point2D> { self.evaluate_f64(t.value()) }

    /// 点が半直線上にあるか判定
    pub fn contains_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> bool {
    let ox = self.origin.x();
    let oy = self.origin.y();
    let px = point.x();
    let py = point.y();
        let dx = self.direction.x();
        let dy = self.direction.y();
        let vx = px - ox;
        let vy = py - oy;
        // 外積 (2D) 相当 vx*dy - vy*dx
        let cross = vx * dy - vy * dx;
        if cross.abs() > tolerance.linear { return false; }
        let dot = vx * dx + vy * dy;
        dot >= -tolerance.linear
    }

    /// f64での点が半直線上にあるか判定
    pub fn contains_point_f64(&self, point: &Point2D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.contains_point(point, &tolerance)
    }

    /// 点が半直線の前方向にあるか判定
    pub fn is_forward(&self, point: &Point2D, tolerance: &ToleranceContext) -> bool {
    let ox = self.origin.x();
    let oy = self.origin.y();
    let px = point.x();
    let py = point.y();
        let dx = self.direction.x();
        let dy = self.direction.y();
        let vx = px - ox; let vy = py - oy;
        (vx * dx + vy * dy) >= -tolerance.linear
    }

    /// f64での点が半直線の前方向にあるか判定
    pub fn is_forward_f64(&self, point: &Point2D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.is_forward(point, &tolerance)
    }

    /// 点から半直線までの距離
    pub fn distance_to_point_f64(&self, point: &Point2D) -> f64 {
    let ox = self.origin.x();
    let oy = self.origin.y();
    let px = point.x();
    let py = point.y();
        let dx = self.direction.x();
        let dy = self.direction.y();
        let vx = px - ox; let vy = py - oy;
        let proj = vx * dx + vy * dy; // 方向は正規化済み想定
        if proj <= 0.0 { (vx*vx + vy*vy).sqrt() } else {
            // 垂直距離 = |v × d| (d は単位) = |vx*dy - vy*dx|
            (vx * dy - vy * dx).abs()
        }
    }

    #[deprecated(note = "Use distance_to_point_f64(&Point2D) -> f64 instead")]
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar { Scalar::new(self.distance_to_point_f64(point)) }

    /// 半直線上での点のパラメータを取得
    pub fn parameter_of_point_f64(&self, point: &Point2D, tolerance: &ToleranceContext) -> Option<f64> {
        if !self.contains_point(point, tolerance) { return None; }
    let ox = self.origin.x(); let oy = self.origin.y();
    let px = point.x(); let py = point.y();
        let dx = self.direction.x(); let dy = self.direction.y();
        let vx = px - ox; let vy = py - oy; let param = vx * dx + vy * dy; // d は単位
        if param >= -tolerance.linear { Some(param) } else { None }
    }

    #[deprecated(note = "Use parameter_of_point_f64(&Point2D, &ToleranceContext) -> Option<f64> instead")]
    pub fn parameter_of_point(&self, point: &Point2D, tolerance: &ToleranceContext) -> Option<Scalar> {
        self.parameter_of_point_f64(point, tolerance).map(Scalar::new)
    }

    /// 半直線を移動
    pub fn translate_f64(&self, dx: f64, dy: f64) -> Ray2D {
        let ox = self.origin.x() + dx;
        let oy = self.origin.y() + dy;
        Ray2D { origin: Point2D::new(ox, oy), direction: self.direction.clone() }
    }

    #[deprecated(note = "Use translate_f64(dx, dy) instead")]
    pub fn translate(&self, dx: Scalar, dy: Scalar) -> Ray2D { self.translate_f64(dx.value(), dy.value()) }

    /// 半直線を回転
    pub fn rotate(&self, angle_rad: f64) -> Ray2D {
        Self {
            origin: self.origin.clone(),
            direction: self.direction.rotate(angle_rad),
        }
    }
}

impl TolerantEq for Ray2D {
    fn tolerant_eq(&self, other: &Self, tolerance: &ToleranceContext) -> bool {
        self.origin.tolerant_eq(&other.origin, tolerance) &&
        self.direction.tolerant_eq(&other.direction, tolerance)
    }
}

impl PartialEq for Ray2D {
    fn eq(&self, other: &Self) -> bool {
        let tolerance = ToleranceContext::default();
        self.tolerant_eq(other, &tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_creation() {
    let origin = Point2D::new(1.0, 2.0);
        let direction = Direction2D::unit_x();
        let ray = Ray2D::new(origin.clone(), direction.clone());

        assert!(ray.origin().tolerant_eq(&origin, &ToleranceContext::default()));
        assert!(ray.direction().tolerant_eq(&direction, &ToleranceContext::default()));
    }

    #[test]
    fn test_ray_from_f64() {
        let ray = Ray2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();

    assert_eq!(ray.origin().x(), 1.0);
    assert_eq!(ray.origin().y(), 2.0);
        assert_eq!(ray.direction().x(), 1.0);
        assert_eq!(ray.direction().y(), 0.0);
    }

    #[test]
    fn test_ray_from_points() {
    let start = Point2D::new(0.0, 0.0);
    let through = Point2D::new(3.0, 4.0);
        let ray = Ray2D::from_points(&start, &through).unwrap();

        // 方向ベクトルが正規化されていることを確認
        let dir = ray.direction();
        assert!((dir.x() - 0.6).abs() < 1e-10);
        assert!((dir.y() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate() {
        let ray = Ray2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();

        let point = ray.evaluate_f64(3.0).unwrap();
    assert_eq!(point.x(), 4.0);
    assert_eq!(point.y(), 2.0);

        // 負のパラメータは無効
        assert!(ray.evaluate_f64(-1.0).is_none());
    }

    #[test]
    fn test_contains_point() {
        let ray = Ray2D::from_f64(0.0, 0.0, 1.0, 1.0).unwrap(); // 45度方向

    let point_on_ray = Point2D::new(2.0, 2.0);
    let point_behind = Point2D::new(-1.0, -1.0);
    let point_off_ray = Point2D::new(2.0, 3.0);

        assert!(ray.contains_point_f64(&point_on_ray, 1e-10));
        assert!(!ray.contains_point_f64(&point_behind, 1e-10)); // 後方
        assert!(!ray.contains_point_f64(&point_off_ray, 1e-10)); // 直線外
    }

    #[test]
    fn test_is_forward() {
        let ray = Ray2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap(); // X軸方向

    let forward_point = Point2D::new(5.0, 3.0);
    let backward_point = Point2D::new(-2.0, 1.0);

        assert!(ray.is_forward_f64(&forward_point, 1e-10));
        assert!(!ray.is_forward_f64(&backward_point, 1e-10));
    }

    #[test]
    fn test_distance_to_point() {
        let ray = Ray2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap(); // X軸方向

        // 半直線上の点
    let on_ray = Point2D::new(5.0, 0.0);
        assert!((ray.distance_to_point_f64(&on_ray) - 0.0).abs() < 1e-10);

        // 原点より後方の点
    let behind = Point2D::new(-3.0, 4.0);
        assert!((ray.distance_to_point_f64(&behind) - 5.0).abs() < 1e-10); // 原点までの距離

        // 半直線の前方、横の点
    let side = Point2D::new(5.0, 3.0);
        assert!((ray.distance_to_point_f64(&side) - 3.0).abs() < 1e-10); // 垂直距離
    }

    #[test]
    fn test_parameter_of_point() {
        let ray = Ray2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();
    let point = Point2D::new(6.0, 2.0);

        let tolerance = ToleranceContext::standard();
        let param = ray.parameter_of_point(&point, &tolerance).unwrap();
        assert_eq!(param.value(), 5.0);
    }

    #[test]
    fn test_translate() {
        let ray = Ray2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();
        let translated = ray.translate_f64(3.0, 4.0);

    assert_eq!(translated.origin().x(), 4.0);
    assert_eq!(translated.origin().y(), 6.0);
        assert_eq!(translated.direction().x(), ray.direction().x());
        assert_eq!(translated.direction().y(), ray.direction().y());
    }

    #[test]
    fn test_rotate() {
        let ray = Ray2D::from_f64(0.0, 0.0, 1.0, 0.0).unwrap(); // X軸方向
        let rotated = ray.rotate(std::f64::consts::FRAC_PI_2); // 90度回転

        assert!((rotated.direction().x() - 0.0).abs() < 1e-10);
        assert!((rotated.direction().y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_direction_vector() {
        let result = Ray2D::from_f64(0.0, 0.0, 0.0, 0.0);
        assert!(result.is_none()); // ゼロベクトルは方向として無効
    }

    #[test]
    fn test_equality() {
        let ray1 = Ray2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();
        let ray2 = Ray2D::from_f64(1.0, 2.0, 1.0, 0.0).unwrap();
        let ray3 = Ray2D::from_f64(2.0, 2.0, 1.0, 0.0).unwrap(); // 異なる原点

        assert_eq!(ray1, ray2);
        assert_ne!(ray1, ray3);
    }
}
