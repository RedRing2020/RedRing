use geo_core::{Point3D, Vector3D, Scalar, ToleranceContext, TolerantEq, Vector};
use crate::geometry3d::direction::Direction3D;

/// 3D無限直線
///
/// 原点と方向ベクトルで定義される無限に長い直線
#[derive(Debug, Clone)]
pub struct InfiniteLine3D {
    origin: Point3D,
    direction: Direction3D,
}

impl InfiniteLine3D {
    /// 原点と方向から無限直線を作成
    pub fn new(origin: Point3D, direction: Direction3D) -> Self {
        Self { origin, direction }
    }

    /// f64座標から無限直線を作成
    pub fn from_f64(origin_x: f64, origin_y: f64, origin_z: f64,
                    direction_x: f64, direction_y: f64, direction_z: f64) -> Option<Self> {
        let origin = Point3D::new(Scalar::new(origin_x), Scalar::new(origin_y), Scalar::new(origin_z));
        let direction = Direction3D::from_vector(&Vector3D::new(
            Scalar::new(direction_x),
            Scalar::new(direction_y),
            Scalar::new(direction_z)
        ))?;
        Some(Self { origin, direction })
    }

    /// 2つの点を通る無限直線を作成
    pub fn from_points(p1: &Point3D, p2: &Point3D) -> Option<Self> {
        let direction_vector = Vector3D::new(
            p2.x().clone() - p1.x().clone(),
            p2.y().clone() - p1.y().clone(),
            p2.z().clone() - p1.z().clone(),
        );
        let direction = Direction3D::from_vector(&direction_vector)?;
        Some(Self {
            origin: p1.clone(),
            direction,
        })
    }

    /// 原点を取得
    pub fn origin(&self) -> &Point3D {
        &self.origin
    }

    /// 方向を取得
    pub fn direction(&self) -> &Direction3D {
        &self.direction
    }

    /// パラメータtでの点を評価
    pub fn evaluate(&self, t: Scalar) -> Point3D {
        let direction_vec = self.direction.to_vector();
        let offset = Vector3D::new(
            direction_vec.x() * t.clone(),
            direction_vec.y() * t.clone(),
            direction_vec.z() * t
        );
        Point3D::new(
            self.origin.x().clone() + offset.x().clone(),
            self.origin.y().clone() + offset.y().clone(),
            self.origin.z().clone() + offset.z().clone(),
        )
    }

    /// f64パラメータでの点を評価
    pub fn evaluate_f64(&self, t: f64) -> Point3D {
        self.evaluate(Scalar::new(t))
    }

    /// 点から直線までの距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();

        // 点から直線への最短距離は外積の大きさ
        let cross = to_point.cross(&direction_vec);
        Scalar::new((cross.x().value() * cross.x().value() + cross.y().value() * cross.y().value() + cross.z().value() * cross.z().value()).sqrt())
    }

    /// f64での点から直線までの距離
    pub fn distance_to_point_f64(&self, point: &Point3D) -> f64 {
        self.distance_to_point(point).value()
    }

    /// 点が直線上にあるか判定
    pub fn contains_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        self.distance_to_point(point).tolerant_eq(&Scalar::new(0.0), tolerance)
    }

    /// f64での点が直線上にあるか判定
    pub fn contains_point_f64(&self, point: &Point3D, epsilon: f64) -> bool {
        self.distance_to_point_f64(point) < epsilon
    }

    /// 直線上で点に最も近い点を求める
    pub fn closest_point_on_line(&self, point: &Point3D) -> Point3D {
        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.x() * direction_vec.x() + to_point.y() * direction_vec.y() + to_point.z() * direction_vec.z();

        self.evaluate(projection_length)
    }

    /// 直線上での点のパラメータを取得
    pub fn parameter_of_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> Option<Scalar> {
        if !self.contains_point(point, tolerance) {
            return None;
        }

        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();
        Some(to_point.dot(&direction_vec))
    }

    /// 他の無限直線との最短距離
    pub fn distance_to_line(&self, other: &InfiniteLine3D) -> Scalar {
        let d1 = self.direction.to_vector();
        let d2 = other.direction.to_vector();
        let w = Vector3D::new(
            self.origin.x().clone() - other.origin.x().clone(),
            self.origin.y().clone() - other.origin.y().clone(),
            self.origin.z().clone() - other.origin.z().clone(),
        );

        let cross = d1.cross(&d2);
        let cross_magnitude = Scalar::new((cross.x().value() * cross.x().value() + cross.y().value() * cross.y().value() + cross.z().value() * cross.z().value()).sqrt());

        if cross_magnitude.value() < 1e-12 {
            // 平行線の場合
            let w_cross_d1 = w.cross(&d1);
            Scalar::new((w_cross_d1.x().value() * w_cross_d1.x().value() + w_cross_d1.y().value() * w_cross_d1.y().value() + w_cross_d1.z().value() * w_cross_d1.z().value()).sqrt())
        } else {
            // 非平行線の場合
            let numerator = w.dot(&cross).value().abs();
            Scalar::new(numerator) / cross_magnitude
        }
    }

    /// 他の無限直線との交点（3Dでは通常存在しない）
    pub fn intersection_with_line(&self, other: &InfiniteLine3D, tolerance: &ToleranceContext) -> Option<Point3D> {
        let distance = self.distance_to_line(other);

        if distance.tolerant_eq(&Scalar::new(0.0), tolerance) {
            // 直線が交差または一致する場合
            // 最も近い点を求める
            let d1 = self.direction.to_vector();
            let d2 = other.direction.to_vector();
            let w = Vector3D::new(
                self.origin.x().clone() - other.origin.x().clone(),
                self.origin.y().clone() - other.origin.y().clone(),
                self.origin.z().clone() - other.origin.z().clone(),
            );

            let cross = d1.cross(&d2);
            let cross_magnitude_sq = cross.dot(&cross);

            if cross_magnitude_sq.value() < 1e-12 {
                // 平行または一致
                if self.contains_point(&other.origin, tolerance) {
                    // 一致する場合
                    Some(self.origin.clone())
                } else {
                    // 平行だが一致しない
                    None
                }
            } else {
                // 交差する場合
                let w_cross_d2 = w.cross(&d2);
                let t1 = w_cross_d2.dot(&cross) / cross_magnitude_sq;
                Some(self.evaluate(t1))
            }
        } else {
            None
        }
    }

    /// 直線を移動
    pub fn translate(&self, dx: Scalar, dy: Scalar, dz: Scalar) -> InfiniteLine3D {
        Self {
            origin: Point3D::new(
                self.origin.x().clone() + dx,
                self.origin.y().clone() + dy,
                self.origin.z().clone() + dz,
            ),
            direction: self.direction.clone(),
        }
    }

    /// f64での直線を移動
    pub fn translate_f64(&self, dx: f64, dy: f64, dz: f64) -> InfiniteLine3D {
        self.translate(Scalar::new(dx), Scalar::new(dy), Scalar::new(dz))
    }
}

impl PartialEq for InfiniteLine3D {
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
        let origin = Point3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(3.0));
        let direction = Direction3D::from_vector(&Vector3D::new(
            Scalar::new(1.0), Scalar::new(0.0), Scalar::new(0.0)
        )).unwrap();
        let line = InfiniteLine3D::new(origin.clone(), direction.clone());

        assert!(line.origin().tolerant_eq(&origin, &ToleranceContext::default()));
        assert!(line.direction().tolerant_eq(&direction, &ToleranceContext::default()));
    }

    #[test]
    fn test_infinite_line_from_f64() {
        let line = InfiniteLine3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();

        assert_eq!(line.origin().x().value(), 1.0);
        assert_eq!(line.origin().y().value(), 2.0);
        assert_eq!(line.origin().z().value(), 3.0);
        assert_eq!(line.direction().x(), 1.0);
        assert_eq!(line.direction().y(), 0.0);
        assert_eq!(line.direction().z(), 0.0);
    }

    #[test]
    fn test_infinite_line_from_points() {
        let p1 = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
        let p2 = Point3D::new(Scalar::new(3.0), Scalar::new(4.0), Scalar::new(0.0));
        let line = InfiniteLine3D::from_points(&p1, &p2).unwrap();

        // 方向ベクトルが正規化されていることを確認
        let dir_vec = line.direction().to_vector();
        let magnitude = (dir_vec.x().value() * dir_vec.x().value() + dir_vec.y().value() * dir_vec.y().value() + dir_vec.z().value() * dir_vec.z().value()).sqrt();
        assert!((magnitude - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate() {
        let line = InfiniteLine3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();
        let point = line.evaluate_f64(5.0);

        assert_eq!(point.x().value(), 6.0);
        assert_eq!(point.y().value(), 2.0);
        assert_eq!(point.z().value(), 3.0);
    }

    #[test]
    fn test_distance_to_point() {
        // X軸に平行な直線
        let line = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        let point = Point3D::new(Scalar::new(5.0), Scalar::new(3.0), Scalar::new(4.0));

        let distance = line.distance_to_point_f64(&point);
        assert_eq!(distance, 5.0); // sqrt(3^2 + 4^2) = 5
    }

    #[test]
    fn test_contains_point() {
        let line = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).unwrap();
        let point_on_line = Point3D::new(Scalar::new(2.0), Scalar::new(2.0), Scalar::new(2.0));
        let point_off_line = Point3D::new(Scalar::new(2.0), Scalar::new(3.0), Scalar::new(2.0));

        assert!(line.contains_point_f64(&point_on_line, 1e-10));
        assert!(!line.contains_point_f64(&point_off_line, 1e-10));
    }

    #[test]
    fn test_closest_point_on_line() {
        let line = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        let point = Point3D::new(Scalar::new(5.0), Scalar::new(3.0), Scalar::new(4.0));

        let closest = line.closest_point_on_line(&point);
        assert_eq!(closest.x().value(), 5.0);
        assert_eq!(closest.y().value(), 0.0);
        assert_eq!(closest.z().value(), 0.0);
    }

    #[test]
    fn test_parameter_of_point() {
        let line = InfiniteLine3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();
        let point = Point3D::new(Scalar::new(6.0), Scalar::new(2.0), Scalar::new(3.0));

        let tolerance = ToleranceContext::default();
        let param = line.parameter_of_point(&point, &tolerance).unwrap();
        assert_eq!(param.value(), 5.0);
    }

    #[test]
    fn test_distance_to_parallel_lines() {
        let line1 = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        let line2 = InfiniteLine3D::from_f64(0.0, 1.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // 平行線

        let distance = line1.distance_to_line(&line2);
        assert_eq!(distance.value(), 1.0);
    }

    #[test]
    fn test_intersection_parallel_lines() {
        let line1 = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        let line2 = InfiniteLine3D::from_f64(0.0, 1.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // 平行線

        let tolerance = ToleranceContext::default();
        let intersection = line1.intersection_with_line(&line2, &tolerance);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_intersection_same_line() {
        let line1 = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        let line2 = InfiniteLine3D::from_f64(5.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // 同じ直線

        let tolerance = ToleranceContext::default();
        let intersection = line1.intersection_with_line(&line2, &tolerance);
        assert!(intersection.is_some());
    }

    #[test]
    fn test_translate() {
        let line = InfiniteLine3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();
        let translated = line.translate_f64(1.0, 2.0, 3.0);

        assert_eq!(translated.origin().x().value(), 2.0);
        assert_eq!(translated.origin().y().value(), 4.0);
        assert_eq!(translated.origin().z().value(), 6.0);
        assert_eq!(translated.direction().x(), line.direction().x());
        assert_eq!(translated.direction().y(), line.direction().y());
        assert_eq!(translated.direction().z(), line.direction().z());
    }

    #[test]
    fn test_equality() {
        let line1 = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        let line2 = InfiniteLine3D::from_f64(5.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // 同じ直線
        let line3 = InfiniteLine3D::from_f64(0.0, 1.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // 平行だが異なる直線

        assert_eq!(line1, line2);
        assert_ne!(line1, line3);
    }

    #[test]
    fn test_zero_direction_vector() {
        let result = InfiniteLine3D::from_f64(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(result.is_none()); // ゼロベクトルは方向として無効
    }
}
