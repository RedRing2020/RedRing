use geo_core::{Point3D, Vector3D, Scalar, ToleranceContext, TolerantEq};
use geo_core::vector::Vector; // bring dot() trait into scope
use crate::geometry3d::direction::Direction3D;

/// 3D半直線（レイ）
///
/// 原点から特定方向に無限に伸びる半直線
#[derive(Debug, Clone)]
pub struct Ray3D {
    origin: Point3D,
    direction: Direction3D,
}

impl Ray3D {
    /// 原点と方向から半直線を作成
    pub fn new(origin: Point3D, direction: Direction3D) -> Self {
        Self { origin, direction }
    }

    /// f64座標から半直線を作成
    pub fn from_f64(origin_x: f64, origin_y: f64, origin_z: f64,
                    direction_x: f64, direction_y: f64, direction_z: f64) -> Option<Self> {
        let origin = Point3D::new(Scalar::new(origin_x), Scalar::new(origin_y), Scalar::new(origin_z));
        let direction = Direction3D::from_f64(direction_x, direction_y, direction_z)?;
        Some(Self { origin, direction })
    }

    /// 2つの点を通る半直線を作成（第一点が原点、第二点への方向）
    pub fn from_points(start: &Point3D, through: &Point3D) -> Option<Self> {
        let direction_vector = Vector3D::new(
            through.x().clone() - start.x().clone(),
            through.y().clone() - start.y().clone(),
            through.z().clone() - start.z().clone(),
        );
        let direction = Direction3D::from_vector(&direction_vector)?;
        Some(Self {
            origin: start.clone(),
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

    /// パラメータt (t >= 0) での点を評価
    pub fn evaluate(&self, t: Scalar) -> Option<Point3D> {
        if t.value() < 0.0 { return None; }
        let dir = self.direction.to_vector();
        let offset = dir.clone() * t;
        Some(Point3D::new(
            Scalar::new(self.origin.x().value() + offset.x().value()),
            Scalar::new(self.origin.y().value() + offset.y().value()),
            Scalar::new(self.origin.z().value() + offset.z().value()),
        ))
    }

    /// f64パラメータでの点を評価
    pub fn evaluate_f64(&self, t: f64) -> Option<Point3D> {
        self.evaluate(Scalar::new(t))
    }

    /// 点が半直線上にあるか判定
    pub fn contains_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        let to_point = Vector3D::from_f64(
            point.x().value() - self.origin.x().value(),
            point.y().value() - self.origin.y().value(),
            point.z().value() - self.origin.z().value(),
        );
        let direction_vec = self.direction.to_vector();
        let cross = to_point.cross(&direction_vec);
        let cross_magnitude_sq = cross.x().value().powi(2)
            + cross.y().value().powi(2)
            + cross.z().value().powi(2);
        if cross_magnitude_sq.sqrt() >= tolerance.linear { return false; }
        let dot = to_point.dot(&direction_vec);
        dot.value() >= -tolerance.linear
    }

    /// f64での点が半直線上にあるか判定
    pub fn contains_point_f64(&self, point: &Point3D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.contains_point(point, &tolerance)
    }

    /// 点が半直線の前方向にあるか判定
    pub fn is_forward(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        let to_point = Vector3D::from_f64(
            point.x().value() - self.origin.x().value(),
            point.y().value() - self.origin.y().value(),
            point.z().value() - self.origin.z().value(),
        );
        let direction_vec = self.direction.to_vector();
        to_point.dot(&direction_vec).value() >= -tolerance.linear
    }

    /// f64での点が半直線の前方向にあるか判定
    pub fn is_forward_f64(&self, point: &Point3D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.is_forward(point, &tolerance)
    }

    /// 点から半直線までの距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let to_point = Vector3D::from_f64(
            point.x().value() - self.origin.x().value(),
            point.y().value() - self.origin.y().value(),
            point.z().value() - self.origin.z().value(),
        );
        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.dot(&direction_vec);
        if projection_length.value() <= 0.0 {
            let dx = point.x().value() - self.origin.x().value();
            let dy = point.y().value() - self.origin.y().value();
            let dz = point.z().value() - self.origin.z().value();
            Scalar::new((dx*dx + dy*dy + dz*dz).sqrt())
        } else {
            let closest_on_line = Point3D::new(
                Scalar::new(self.origin.x().value() + direction_vec.x().value() * projection_length.value()),
                Scalar::new(self.origin.y().value() + direction_vec.y().value() * projection_length.value()),
                Scalar::new(self.origin.z().value() + direction_vec.z().value() * projection_length.value()),
            );
            let dx = point.x().value() - closest_on_line.x().value();
            let dy = point.y().value() - closest_on_line.y().value();
            let dz = point.z().value() - closest_on_line.z().value();
            Scalar::new((dx*dx + dy*dy + dz*dz).sqrt())
        }
    }

    /// f64での点から半直線までの距離
    pub fn distance_to_point_f64(&self, point: &Point3D) -> f64 {
        self.distance_to_point(point).value()
    }

    /// 半直線上での点のパラメータを取得
    pub fn parameter_of_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> Option<Scalar> {
        if !self.contains_point(point, tolerance) {
            return None;
        }

        let to_point = Vector3D::from_f64(
            point.x().value() - self.origin.x().value(),
            point.y().value() - self.origin.y().value(),
            point.z().value() - self.origin.z().value(),
        );
        let direction_vec = self.direction.to_vector();
        let param = to_point.dot(&direction_vec);
        if param.value() >= -tolerance.linear { Some(param) } else { None }
    }

    /// 半直線上で点に最も近い点を求める
    pub fn closest_point_on_ray(&self, point: &Point3D) -> Point3D {
        let to_point = Vector3D::from_f64(
            point.x().value() - self.origin.x().value(),
            point.y().value() - self.origin.y().value(),
            point.z().value() - self.origin.z().value(),
        );
        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.dot(&direction_vec);
        if projection_length.value() <= 0.0 { self.origin.clone() } else { self.evaluate(projection_length).unwrap() }
    }

    /// 半直線を移動
    pub fn translate(&self, dx: Scalar, dy: Scalar, dz: Scalar) -> Ray3D {
        Self {
            origin: Point3D::new(
                Scalar::new(self.origin.x().value() + dx.value()),
                Scalar::new(self.origin.y().value() + dy.value()),
                Scalar::new(self.origin.z().value() + dz.value()),
            ),
            direction: self.direction.clone(),
        }
    }

    /// f64での半直線を移動
    pub fn translate_f64(&self, dx: f64, dy: f64, dz: f64) -> Ray3D {
        self.translate(Scalar::new(dx), Scalar::new(dy), Scalar::new(dz))
    }

    /// 指定軸周りの回転
    pub fn rotate_around_axis(&self, axis: &Direction3D, angle_rad: f64) -> Ray3D {
        Self {
            origin: self.origin.clone(),
            direction: self.direction.rotate_around_axis(axis, angle_rad),
        }
    }
}

impl TolerantEq for Ray3D {
    fn tolerant_eq(&self, other: &Self, tolerance: &ToleranceContext) -> bool {
        self.origin.tolerant_eq(&other.origin, tolerance) &&
        self.direction.tolerant_eq(&other.direction, tolerance)
    }
}

impl PartialEq for Ray3D {
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
        let origin = Point3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(3.0));
        let direction = Direction3D::unit_x();
        let ray = Ray3D::new(origin.clone(), direction.clone());

        assert!(ray.origin().tolerant_eq(&origin, &ToleranceContext::default()));
        assert!(ray.direction().tolerant_eq(&direction, &ToleranceContext::default()));
    }

    #[test]
    fn test_ray_from_f64() {
        let ray = Ray3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();

        assert_eq!(ray.origin().x().value(), 1.0);
        assert_eq!(ray.origin().y().value(), 2.0);
        assert_eq!(ray.origin().z().value(), 3.0);
        assert_eq!(ray.direction().x(), 1.0);
        assert_eq!(ray.direction().y(), 0.0);
        assert_eq!(ray.direction().z(), 0.0);
    }

    #[test]
    fn test_ray_from_points() {
        let start = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
        let through = Point3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(2.0));
        let ray = Ray3D::from_points(&start, &through).unwrap();

        // 方向ベクトルが正規化されていることを確認
        let dir = ray.direction();
        let magnitude = (dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z()).sqrt();
        assert!((magnitude - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate() {
        let ray = Ray3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();

        let point = ray.evaluate_f64(5.0).unwrap();
        assert_eq!(point.x().value(), 6.0);
        assert_eq!(point.y().value(), 2.0);
        assert_eq!(point.z().value(), 3.0);

        // 負のパラメータは無効
        assert!(ray.evaluate_f64(-1.0).is_none());
    }

    #[test]
    fn test_contains_point() {
        let ray = Ray3D::from_f64(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).unwrap();

        let point_on_ray = Point3D::new(Scalar::new(2.0), Scalar::new(2.0), Scalar::new(2.0));
        let point_behind = Point3D::new(Scalar::new(-1.0), Scalar::new(-1.0), Scalar::new(-1.0));
        let point_off_ray = Point3D::new(Scalar::new(2.0), Scalar::new(3.0), Scalar::new(2.0));

        assert!(ray.contains_point_f64(&point_on_ray, 1e-10));
        assert!(!ray.contains_point_f64(&point_behind, 1e-10)); // 後方
        assert!(!ray.contains_point_f64(&point_off_ray, 1e-10)); // 直線外
    }

    #[test]
    fn test_is_forward() {
        let ray = Ray3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // X軸方向

        let forward_point = Point3D::new(Scalar::new(5.0), Scalar::new(3.0), Scalar::new(4.0));
        let backward_point = Point3D::new(Scalar::new(-2.0), Scalar::new(1.0), Scalar::new(2.0));

        assert!(ray.is_forward_f64(&forward_point, 1e-10));
        assert!(!ray.is_forward_f64(&backward_point, 1e-10));
    }

    #[test]
    fn test_distance_to_point() {
        let ray = Ray3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // X軸方向

        // 半直線上の点
        let on_ray = Point3D::new(Scalar::new(5.0), Scalar::new(0.0), Scalar::new(0.0));
        assert!((ray.distance_to_point_f64(&on_ray) - 0.0).abs() < 1e-10);

        // 原点より後方の点
        let behind = Point3D::new(Scalar::new(-3.0), Scalar::new(4.0), Scalar::new(0.0));
        assert!((ray.distance_to_point_f64(&behind) - 5.0).abs() < 1e-10); // 原点までの距離

        // 半直線の前方、横の点
        let side = Point3D::new(Scalar::new(5.0), Scalar::new(3.0), Scalar::new(4.0));
        assert!((ray.distance_to_point_f64(&side) - 5.0).abs() < 1e-10); // sqrt(3^2 + 4^2) = 5
    }

    #[test]
    fn test_closest_point_on_ray() {
        let ray = Ray3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // X軸方向

        // 後方の点の場合、原点が最近点
        let behind = Point3D::new(Scalar::new(-3.0), Scalar::new(4.0), Scalar::new(5.0));
        let closest = ray.closest_point_on_ray(&behind);
        assert_eq!(closest.x().value(), 0.0);
        assert_eq!(closest.y().value(), 0.0);
        assert_eq!(closest.z().value(), 0.0);

        // 前方の点の場合、投影点が最近点
        let forward = Point3D::new(Scalar::new(5.0), Scalar::new(3.0), Scalar::new(4.0));
        let closest = ray.closest_point_on_ray(&forward);
        assert_eq!(closest.x().value(), 5.0);
        assert_eq!(closest.y().value(), 0.0);
        assert_eq!(closest.z().value(), 0.0);
    }

    #[test]
    fn test_parameter_of_point() {
        let ray = Ray3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();
        let point = Point3D::new(Scalar::new(6.0), Scalar::new(2.0), Scalar::new(3.0));

        let tolerance = ToleranceContext::standard();
        let param = ray.parameter_of_point(&point, &tolerance).unwrap();
        assert_eq!(param.value(), 5.0);
    }

    #[test]
    fn test_translate() {
        let ray = Ray3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();
        let translated = ray.translate_f64(1.0, 2.0, 3.0);

        assert_eq!(translated.origin().x().value(), 2.0);
        assert_eq!(translated.origin().y().value(), 4.0);
        assert_eq!(translated.origin().z().value(), 6.0);
        assert_eq!(translated.direction().x(), ray.direction().x());
        assert_eq!(translated.direction().y(), ray.direction().y());
        assert_eq!(translated.direction().z(), ray.direction().z());
    }

    #[test]
    fn test_rotate_around_axis() {
        let ray = Ray3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // X軸方向
        let rotated = ray.rotate_around_axis(&Direction3D::unit_z(), std::f64::consts::FRAC_PI_2);

        // X軸をZ軸周りに90度回転するとY軸方向
        assert!((rotated.direction().x() - 0.0).abs() < 1e-10);
        assert!((rotated.direction().y() - 1.0).abs() < 1e-10);
        assert!((rotated.direction().z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_direction_vector() {
        let result = Ray3D::from_f64(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(result.is_none()); // ゼロベクトルは方向として無効
    }

    #[test]
    fn test_equality() {
        let ray1 = Ray3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();
        let ray2 = Ray3D::from_f64(1.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap();
        let ray3 = Ray3D::from_f64(2.0, 2.0, 3.0, 1.0, 0.0, 0.0).unwrap(); // 異なる原点

        assert_eq!(ray1, ray2);
        assert_ne!(ray1, ray3);
    }
}
