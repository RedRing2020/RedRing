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
            direction_x,
            direction_y,
            direction_z,
        ))?;
        Some(Self { origin, direction })
    }

    /// 2つの点を通る無限直線を作成
    pub fn from_points(p1: &Point3D, p2: &Point3D) -> Option<Self> {
        let direction_vector = Vector3D::new(
            p2.x().value() - p1.x().value(),
            p2.y().value() - p1.y().value(),
            p2.z().value() - p1.z().value(),
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

    pub fn evaluate_f64(&self, t: f64) -> Point3D {
        let ox = self.origin.x().value(); let oy = self.origin.y().value(); let oz = self.origin.z().value();
        let dx = self.direction.x(); let dy = self.direction.y(); let dz = self.direction.z();
        Point3D::new(Scalar::new(ox + dx * t), Scalar::new(oy + dy * t), Scalar::new(oz + dz * t))
    }

    #[deprecated(note = "Use evaluate_f64(t: f64) instead")]
    pub fn evaluate(&self, t: Scalar) -> Point3D { self.evaluate_f64(t.value()) }

    /// 点から直線までの距離
    pub fn distance_to_point_f64(&self, point: &Point3D) -> f64 {
        let ox = self.origin.x().value(); let oy = self.origin.y().value(); let oz = self.origin.z().value();
        let px = point.x().value(); let py = point.y().value(); let pz = point.z().value();
        let dx = self.direction.x(); let dy = self.direction.y(); let dz = self.direction.z();
        let vx = px - ox; let vy = py - oy; let vz = pz - oz;
        // |v × d| = sqrt(|v|^2 - (v·d)^2)  (d は単位想定)
        let dot = vx*dx + vy*dy + vz*dz;
        let v_len_sq = vx*vx + vy*vy + vz*vz;
        (v_len_sq - dot*dot).max(0.0).sqrt()
    }

    #[deprecated(note = "Use distance_to_point_f64(&Point3D) -> f64 instead")]
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar { Scalar::new(self.distance_to_point_f64(point)) }

    /// 点が直線上にあるか判定
    pub fn contains_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        self.distance_to_point_f64(point) <= tolerance.linear
    }

    /// f64での点が直線上にあるか判定
    pub fn contains_point_f64(&self, point: &Point3D, epsilon: f64) -> bool {
        self.distance_to_point_f64(point) < epsilon
    }

    /// 直線上で点に最も近い点を求める
    pub fn closest_point_on_line(&self, point: &Point3D) -> Point3D {
        let ox = self.origin.x().value(); let oy = self.origin.y().value(); let oz = self.origin.z().value();
        let px = point.x().value(); let py = point.y().value(); let pz = point.z().value();
        let dx = self.direction.x(); let dy = self.direction.y(); let dz = self.direction.z();
        let vx = px - ox; let vy = py - oy; let vz = pz - oz; let proj = vx*dx + vy*dy + vz*dz;
        self.evaluate_f64(proj)
    }

    /// 直線上での点のパラメータを取得
    pub fn parameter_of_point_f64(&self, point: &Point3D, tolerance: &ToleranceContext) -> Option<f64> {
        if !self.contains_point(point, tolerance) { return None; }
        let ox = self.origin.x().value(); let oy = self.origin.y().value(); let oz = self.origin.z().value();
        let px = point.x().value(); let py = point.y().value(); let pz = point.z().value();
        let dx = self.direction.x(); let dy = self.direction.y(); let dz = self.direction.z();
        Some((px - ox)*dx + (py - oy)*dy + (pz - oz)*dz)
    }

    #[deprecated(note = "Use parameter_of_point_f64(&Point3D, &ToleranceContext) -> Option<f64> instead")]
    pub fn parameter_of_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> Option<Scalar> {
        self.parameter_of_point_f64(point, tolerance).map(Scalar::new)
    }

    /// 他の無限直線との最短距離
    pub fn distance_to_line_f64(&self, other: &InfiniteLine3D) -> f64 {
        let dx1 = self.direction.x(); let dy1 = self.direction.y(); let dz1 = self.direction.z();
        let dx2 = other.direction.x(); let dy2 = other.direction.y(); let dz2 = other.direction.z();
        let ox1 = self.origin.x().value(); let oy1 = self.origin.y().value(); let oz1 = self.origin.z().value();
        let ox2 = other.origin.x().value(); let oy2 = other.origin.y().value(); let oz2 = other.origin.z().value();
        let wx = ox1 - ox2; let wy = oy1 - oy2; let wz = oz1 - oz2;
        // cross(d1,d2)
        let cx = dy1*dz2 - dz1*dy2; let cy = dz1*dx2 - dx1*dz2; let cz = dx1*dy2 - dy1*dx2;
        let cross_len = (cx*cx + cy*cy + cz*cz).sqrt();
        if cross_len < 1e-12 {
            // 平行: |w × d1|
            let wx_cross = wy*dz1 - wz*dy1;
            let wy_cross = wz*dx1 - wx*dz1;
            let wz_cross = wx*dy1 - wy*dx1;
            (wx_cross*wx_cross + wy_cross*wy_cross + wz_cross*wz_cross).sqrt()
        } else {
            // |(w · (d1 × d2))| / |d1 × d2|
            let numerator = (wx*cx + wy*cy + wz*cz).abs();
            numerator / cross_len
        }
    }

    #[deprecated(note = "Use distance_to_line_f64(&InfiniteLine3D) -> f64 instead")]
    pub fn distance_to_line(&self, other: &InfiniteLine3D) -> Scalar { Scalar::new(self.distance_to_line_f64(other)) }

    /// 他の無限直線との交点（3Dでは通常存在しない）
    pub fn intersection_with_line(&self, other: &InfiniteLine3D, tolerance: &ToleranceContext) -> Option<Point3D> {
        let dist = self.distance_to_line_f64(other);
        if dist > tolerance.linear { return None; }
        // 平行または一致の場合
        let dx1 = self.direction.x(); let dy1 = self.direction.y(); let dz1 = self.direction.z();
        let dx2 = other.direction.x(); let dy2 = other.direction.y(); let dz2 = other.direction.z();
        let cx = dy1*dz2 - dz1*dy2; let cy = dz1*dx2 - dx1*dz2; let cz = dx1*dy2 - dy1*dx2;
        let cross_len_sq = cx*cx + cy*cy + cz*cz;
        if cross_len_sq < 1e-12 {
            if self.contains_point(&other.origin, tolerance) { Some(self.origin.clone()) } else { None }
        } else {
            // 交差: 1本目のパラメータ t1 を計算
            // w = O1 - O2
            let ox1 = self.origin.x().value(); let oy1 = self.origin.y().value(); let oz1 = self.origin.z().value();
            let ox2 = other.origin.x().value(); let oy2 = other.origin.y().value(); let oz2 = other.origin.z().value();
            let wx = ox1 - ox2; let wy = oy1 - oy2; let wz = oz1 - oz2;
            // w × d2
            let wxcx = wy*dz2 - wz*dy2; let wxyc = wz*dx2 - wx*dz2; let wxcz = wx*dy2 - wy*dx2;
            // (w × d2) · (d1 × d2)
            let dot = wxcx*cx + wxyc*cy + wxcz*cz;
            let t1 = dot / cross_len_sq;
            Some(self.evaluate_f64(t1))
        }
    }

    /// 直線を移動
    pub fn translate_f64(&self, dx: f64, dy: f64, dz: f64) -> InfiniteLine3D {
        InfiniteLine3D {
            origin: Point3D::new(
                Scalar::new(self.origin.x().value() + dx),
                Scalar::new(self.origin.y().value() + dy),
                Scalar::new(self.origin.z().value() + dz),
            ),
            direction: self.direction.clone(),
        }
    }

    #[deprecated(note = "Use translate_f64(dx, dy, dz) instead")]
    pub fn translate(&self, dx: Scalar, dy: Scalar, dz: Scalar) -> InfiniteLine3D { self.translate_f64(dx.value(), dy.value(), dz.value()) }
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
