use geo_core::{Point3D, Vector3D, Scalar, ToleranceContext, TolerantEq};
use crate::geometry3d::direction::Direction3D;
use crate::geometry_utils::*;

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
        if t.value() < 0.0 {
            None
        } else {
            let offset = Vector3D::new(
                self.direction.to_vector().x().clone() * t.clone(),
                self.direction.to_vector().y().clone() * t.clone(),
                self.direction.to_vector().z().clone() * t,
            );
            Some(Point3D::new(
                self.origin.x().clone() + offset.x(),
                self.origin.y().clone() + offset.y(),
                self.origin.z().clone() + offset.z(),
            ))
        }
    }

    /// f64パラメータでの点を評価
    pub fn evaluate_f64(&self, t: f64) -> Option<Point3D> {
        self.evaluate(Scalar::new(t))
    }

    /// 点が半直線上にあるか判定
    pub fn contains_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();

        // 外積で直線からの距離をチェック
        let cross = Vector3D::new(
            to_point.y().clone() * direction_vec.z().clone() - to_point.z().clone() * direction_vec.y().clone(),
            to_point.z().clone() * direction_vec.x().clone() - to_point.x().clone() * direction_vec.z().clone(),
            to_point.x().clone() * direction_vec.y().clone() - to_point.y().clone() * direction_vec.x().clone(),
        );

        let cross_magnitude = (cross.x().value() * cross.x().value() +
                              cross.y().value() * cross.y().value() +
                              cross.z().value() * cross.z().value()).sqrt();

        if cross_magnitude >= tolerance.linear {
            return false;
        }

        // 方向チェック（前方かどうか）
        let dot = to_point.x().clone() * direction_vec.x().clone() +
                  to_point.y().clone() * direction_vec.y().clone() +
                  to_point.z().clone() * direction_vec.z().clone();
        dot.value() >= -tolerance.linear
    }

    /// f64での点が半直線上にあるか判定
    pub fn contains_point_f64(&self, point: &Point3D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.contains_point(point, &tolerance)
    }

    /// 点が半直線の前方向にあるか判定
    pub fn is_forward(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();
        let dot = to_point.x().clone() * direction_vec.x().clone() +
                  to_point.y().clone() * direction_vec.y().clone() +
                  to_point.z().clone() * direction_vec.z().clone();

        dot.value() >= -tolerance.linear
    }

    /// f64での点が半直線の前方向にあるか判定
    pub fn is_forward_f64(&self, point: &Point3D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.is_forward(point, &tolerance)
    }

    /// 点から半直線までの距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.x().clone() * direction_vec.x().clone() +
                               to_point.y().clone() * direction_vec.y().clone() +
                               to_point.z().clone() * direction_vec.z().clone();

        if projection_length.value() <= 0.0 {
            // 点が原点より後ろにある場合、原点までの距離
            let dx = point.x().clone() - self.origin.x().clone();
            let dy = point.y().clone() - self.origin.y().clone();
            let dz = point.z().clone() - self.origin.z().clone();
            Scalar::new((dx.value() * dx.value() +
                        dy.value() * dy.value() +
                        dz.value() * dz.value()).sqrt())
        } else {
            // 点を直線に投影した点までの距離
            let closest_on_line = Point3D::new(
                self.origin.x().clone() + direction_vec.x().clone() * projection_length.clone(),
                self.origin.y().clone() + direction_vec.y().clone() * projection_length.clone(),
                self.origin.z().clone() + direction_vec.z().clone() * projection_length,
            );

            let dx = point.x().clone() - closest_on_line.x().clone();
            let dy = point.y().clone() - closest_on_line.y().clone();
            let dz = point.z().clone() - closest_on_line.z().clone();
            Scalar::new((dx.value() * dx.value() +
                        dy.value() * dy.value() +
                        dz.value() * dz.value()).sqrt())
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

        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();
        let param = to_point.x().clone() * direction_vec.x().clone() +
                   to_point.y().clone() * direction_vec.y().clone() +
                   to_point.z().clone() * direction_vec.z().clone();

        if param.value() >= -tolerance.linear {
            Some(param)
        } else {
            None
        }
    }

    /// 半直線上で点に最も近い点を求める
    pub fn closest_point_on_ray(&self, point: &Point3D) -> Point3D {
        let to_point = Vector3D::new(
            point.x().clone() - self.origin.x().clone(),
            point.y().clone() - self.origin.y().clone(),
            point.z().clone() - self.origin.z().clone(),
        );

        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.x().clone() * direction_vec.x().clone() +
                               to_point.y().clone() * direction_vec.y().clone() +
                               to_point.z().clone() * direction_vec.z().clone();

        if projection_length.value() <= 0.0 {
            // 投影が原点より後ろの場合、原点が最近点
            self.origin.clone()
        } else {
            // 投影点が最近点
            self.evaluate(projection_length).unwrap()
        }
    }

    /// 半直線を移動
    pub fn translate(&self, dx: Scalar, dy: Scalar, dz: Scalar) -> Ray3D {
        Self {
            origin: Point3D::new(
                self.origin.x().clone() + dx,
                self.origin.y().clone() + dy,
                self.origin.z().clone() + dz,
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
