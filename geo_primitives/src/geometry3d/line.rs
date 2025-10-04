use geo_core::{Point3D, Vector3D, Scalar, ToleranceContext, TolerantEq};
use crate::geometry3d::direction::Direction3D;
use crate::geometry3d::infinite_line::InfiniteLine3D;

/// 3D線分
///
/// 開始点と終了点を持つ有限線分
/// InfiniteLineをベースとして持つ
#[derive(Debug, Clone)]
pub struct Line3D {
    base: InfiniteLine3D,
    start: Point3D,
    end: Point3D,
}

impl Line3D {
    /// 開始点と終了点から線分を作成
    pub fn new(start: Point3D, end: Point3D) -> Self {
        let dx = end.x().value() - start.x().value();
        let dy = end.y().value() - start.y().value();
        let dz = end.z().value() - start.z().value();
    let direction_vector = Vector3D::new(dx, dy, dz);
        let base = if let Some(direction) = Direction3D::from_vector(&direction_vector) {
            InfiniteLine3D::new(start.clone(), direction)
        } else {
            InfiniteLine3D::new(start.clone(), Direction3D::unit_x())
        };
        Self { base, start, end }
    }

    /// f64座標から線分を作成
    pub fn from_f64(start_x: f64, start_y: f64, start_z: f64,
                    end_x: f64, end_y: f64, end_z: f64) -> Self {
        let start = Point3D::new(Scalar::new(start_x), Scalar::new(start_y), Scalar::new(start_z));
        let end = Point3D::new(Scalar::new(end_x), Scalar::new(end_y), Scalar::new(end_z));
        Self::new(start, end)
    }

    /// 開始点を取得
    pub fn start(&self) -> &Point3D {
        &self.start
    }

    /// 終了点を取得
    pub fn end(&self) -> &Point3D {
        &self.end
    }

    /// 線分の方向ベクトルを取得
    pub fn direction_vector(&self) -> Vector3D {
        Vector3D::new(
            self.end.x().value() - self.start.x().value(),
            self.end.y().value() - self.start.y().value(),
            self.end.z().value() - self.start.z().value(),
        )
    }

    /// 線分の方向を取得
    pub fn direction(&self) -> Option<Direction3D> {
        Direction3D::from_vector(&self.direction_vector())
    }

    /// 線分の長さを取得
    pub fn length(&self) -> Scalar { Scalar::new(self.length_f64()) }
    pub fn length_f64(&self) -> f64 {
        let dx = self.end.x().value() - self.start.x().value();
        let dy = self.end.y().value() - self.start.y().value();
        let dz = self.end.z().value() - self.start.z().value();
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// InfiniteLineとしての表現を取得
    pub fn to_infinite(&self) -> &InfiniteLine3D {
        &self.base
    }

    /// 線分の中点を取得
    pub fn midpoint(&self) -> Point3D {
        let mx = (self.start.x().value() + self.end.x().value()) * 0.5;
        let my = (self.start.y().value() + self.end.y().value()) * 0.5;
        let mz = (self.start.z().value() + self.end.z().value()) * 0.5;
        Point3D::new(Scalar::new(mx), Scalar::new(my), Scalar::new(mz))
    }

    /// パラメータt (0 <= t <= 1) での点を評価
    pub fn evaluate_f64(&self, t: f64) -> Option<Point3D> {
        if !(0.0..=1.0).contains(&t) { return None; }
        let sx = self.start.x().value();
        let sy = self.start.y().value();
        let sz = self.start.z().value();
        let ex = self.end.x().value();
        let ey = self.end.y().value();
        let ez = self.end.z().value();
        let x = sx + (ex - sx) * t;
        let y = sy + (ey - sy) * t;
        let z = sz + (ez - sz) * t;
        Some(Point3D::new(Scalar::new(x), Scalar::new(y), Scalar::new(z)))
    }

    #[deprecated(note = "Use evaluate_f64(t: f64) instead")]
    pub fn evaluate(&self, t: Scalar) -> Option<Point3D> { self.evaluate_f64(t.value()) }

    /// 点が線分上にあるか判定
    pub fn contains_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        let to_point = Vector3D::new(
            point.x().value() - self.start.x().value(),
            point.y().value() - self.start.y().value(),
            point.z().value() - self.start.z().value(),
        );

        let direction_vec = self.direction_vector();

        // 方向ベクトルがゼロベクトルの場合（同じ点）
        let len_sq = direction_vec.x()*direction_vec.x() + direction_vec.y()*direction_vec.y() + direction_vec.z()*direction_vec.z();
        if len_sq < tolerance.linear * tolerance.linear {
            return point.tolerant_eq(&self.start, tolerance);
        }

        // 外積で直線からの距離をチェック
        let cx = to_point.y()*direction_vec.z() - to_point.z()*direction_vec.y();
        let cy = to_point.z()*direction_vec.x() - to_point.x()*direction_vec.z();
        let cz = to_point.x()*direction_vec.y() - to_point.y()*direction_vec.x();
        let cross_len_sq = cx*cx + cy*cy + cz*cz;
        if cross_len_sq > tolerance.linear * tolerance.linear {
            return false;
        }

        // 線分の範囲内かチェック
    let dot = to_point.x()*direction_vec.x() +
          to_point.y()*direction_vec.y() +
          to_point.z()*direction_vec.z();
    let projection_param = dot / len_sq;

        projection_param >= -tolerance.linear && projection_param <= 1.0 + tolerance.linear
    }

    /// f64での点が線分上にあるか判定
    pub fn contains_point_f64(&self, point: &Point3D, _epsilon: f64) -> bool {
        let tolerance = ToleranceContext::standard();
        self.contains_point(point, &tolerance)
    }

    /// 点から線分までの距離
    pub fn distance_to_point_f64(&self, point: &Point3D) -> f64 {
        let sx = self.start.x().value(); let sy = self.start.y().value(); let sz = self.start.z().value();
        let ex = self.end.x().value();   let ey = self.end.y().value();   let ez = self.end.z().value();
        let px = point.x().value(); let py = point.y().value(); let pz = point.z().value();
        let dx = ex - sx; let dy = ey - sy; let dz = ez - sz;
        let len_sq = dx*dx + dy*dy + dz*dz;
        if len_sq < 1e-10 { let dxp = px - sx; let dyp = py - sy; let dzp = pz - sz; return (dxp*dxp + dyp*dyp + dzp*dzp).sqrt(); }
        let t = ((px - sx)*dx + (py - sy)*dy + (pz - sz)*dz) / len_sq;
        if t <= 0.0 { let dxp = px - sx; let dyp = py - sy; let dzp = pz - sz; (dxp*dxp + dyp*dyp + dzp*dzp).sqrt() }
        else if t >= 1.0 { let dxp = px - ex; let dyp = py - ey; let dzp = pz - ez; (dxp*dxp + dyp*dyp + dzp*dzp).sqrt() }
        else { let cx = sx + dx*t; let cy = sy + dy*t; let cz = sz + dz*t; let dxp = px - cx; let dyp = py - cy; let dzp = pz - cz; (dxp*dxp + dyp*dyp + dzp*dzp).sqrt() }
    }

    #[deprecated(note = "Use distance_to_point_f64(&Point3D) -> f64 instead")]
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar { Scalar::new(self.distance_to_point_f64(point)) }

    /// 線分上での点のパラメータを取得
    pub fn parameter_of_point_f64(&self, point: &Point3D, tolerance: &ToleranceContext) -> Option<f64> {
        if !self.contains_point(point, tolerance) { return None; }
        let sx = self.start.x().value(); let sy = self.start.y().value(); let sz = self.start.z().value();
        let ex = self.end.x().value();   let ey = self.end.y().value();   let ez = self.end.z().value();
        let dx = ex - sx; let dy = ey - sy; let dz = ez - sz; let len_sq = dx*dx + dy*dy + dz*dz;
        if len_sq < tolerance.linear * tolerance.linear { return Some(0.0); }
        let px = point.x().value(); let py = point.y().value(); let pz = point.z().value();
        let param = ((px - sx)*dx + (py - sy)*dy + (pz - sz)*dz) / len_sq;
        Some(param.clamp(0.0, 1.0))
    }

    #[deprecated(note = "Use parameter_of_point_f64(&Point3D, &ToleranceContext) -> Option<f64> instead")]
    pub fn parameter_of_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> Option<Scalar> {
        self.parameter_of_point_f64(point, tolerance).map(Scalar::new)
    }

    /// 線分を移動
    pub fn translate_f64(&self, dx: f64, dy: f64, dz: f64) -> Line3D {
        let sx = self.start.x().value() + dx; let sy = self.start.y().value() + dy; let sz = self.start.z().value() + dz;
        let ex = self.end.x().value() + dx;   let ey = self.end.y().value() + dy;   let ez = self.end.z().value() + dz;
        Line3D::new(
            Point3D::new(Scalar::new(sx), Scalar::new(sy), Scalar::new(sz)),
            Point3D::new(Scalar::new(ex), Scalar::new(ey), Scalar::new(ez)),
        )
    }

    #[deprecated(note = "Use translate_f64(dx, dy, dz) instead")]
    pub fn translate(&self, dx: Scalar, dy: Scalar, dz: Scalar) -> Line3D { self.translate_f64(dx.value(), dy.value(), dz.value()) }

    /// 線分をZ軸中心に回転
    pub fn rotate_z_f64(&self, angle: f64) -> Line3D {
        let (sin_angle, cos_angle) = angle.sin_cos();
        let (sx, sy, sz) = (self.start.x().value(), self.start.y().value(), self.start.z().value());
        let (ex, ey, ez) = (self.end.x().value(), self.end.y().value(), self.end.z().value());
        let rsx = sx * cos_angle - sy * sin_angle;
        let rsy = sx * sin_angle + sy * cos_angle;
        let rex = ex * cos_angle - ey * sin_angle;
        let rey = ex * sin_angle + ey * cos_angle;
        Line3D::new(
            Point3D::new(Scalar::new(rsx), Scalar::new(rsy), Scalar::new(sz)),
            Point3D::new(Scalar::new(rex), Scalar::new(rey), Scalar::new(ez)),
        )
    }

    #[deprecated(note = "Use rotate_z_f64(angle) instead")]
    pub fn rotate_z(&self, angle: Scalar) -> Line3D { self.rotate_z_f64(angle.value()) }

    /// 線分を任意軸中心に回転
    pub fn rotate_around_axis(&self, axis: &Vector3D, angle: Scalar) -> Line3D {
        // Rodriguesの回転公式を使用
        let cos_angle = angle.value().cos();
        let sin_angle = angle.value().sin();

        // 軸を正規化
        let axis_length = (axis.x().value() * axis.x().value() +
                          axis.y().value() * axis.y().value() +
                          axis.z().value() * axis.z().value()).sqrt();

        if axis_length < 1e-10 {
            return self.clone(); // 軸がゼロベクトルの場合は回転なし
        }

        let k = Vector3D::new(
            axis.x().clone() / Scalar::new(axis_length),
            axis.y().clone() / Scalar::new(axis_length),
            axis.z().clone() / Scalar::new(axis_length),
        );

        let rotate_point = |point: &Point3D| -> Point3D {
            let v = Vector3D::new(point.x().clone(), point.y().clone(), point.z().clone());

            // v * cos(θ) + (k × v) * sin(θ) + k * (k · v) * (1 - cos(θ))
            let k_dot_v = k.x().clone() * v.x().clone() +
                         k.y().clone() * v.y().clone() +
                         k.z().clone() * v.z().clone();

            let k_cross_v = Vector3D::new(
                k.y().clone() * v.z().clone() - k.z().clone() * v.y().clone(),
                k.z().clone() * v.x().clone() - k.x().clone() * v.z().clone(),
                k.x().clone() * v.y().clone() - k.y().clone() * v.x().clone(),
            );

            let cos_s = cos_angle; let sin_s = sin_angle; let one_minus = 1.0 - cos_s;
            let (kx, ky, kz) = (k.x().value(), k.y().value(), k.z().value());
            let (vx, vy, vz) = (v.x().value(), v.y().value(), v.z().value());
            let kcvx = k_cross_v.x().value();
            let kcvy = k_cross_v.y().value();
            let kcvz = k_cross_v.z().value();
            Point3D::new(
                Scalar::new(vx * cos_s + kcvx * sin_s + kx * k_dot_v.value() * one_minus),
                Scalar::new(vy * cos_s + kcvy * sin_s + ky * k_dot_v.value() * one_minus),
                Scalar::new(vz * cos_s + kcvz * sin_s + kz * k_dot_v.value() * one_minus),
            )
        };

        Self::new(rotate_point(&self.start), rotate_point(&self.end))
    }

    /// 2つの線分が等しいか判定
    pub fn equals(&self, other: &Line3D, tolerance: &ToleranceContext) -> bool {
        (self.start.tolerant_eq(&other.start, tolerance) && self.end.tolerant_eq(&other.end, tolerance)) ||
        (self.start.tolerant_eq(&other.end, tolerance) && self.end.tolerant_eq(&other.start, tolerance))
    }

    /// 線分の境界ボックスを取得
    pub fn bounding_box(&self) -> (Point3D, Point3D) {
        let min_x = if self.start.x().value() < self.end.x().value() {
            self.start.x().clone()
        } else {
            self.end.x().clone()
        };
        let min_y = if self.start.y().value() < self.end.y().value() {
            self.start.y().clone()
        } else {
            self.end.y().clone()
        };
        let min_z = if self.start.z().value() < self.end.z().value() {
            self.start.z().clone()
        } else {
            self.end.z().clone()
        };

        let max_x = if self.start.x().value() > self.end.x().value() {
            self.start.x().clone()
        } else {
            self.end.x().clone()
        };
        let max_y = if self.start.y().value() > self.end.y().value() {
            self.start.y().clone()
        } else {
            self.end.y().clone()
        };
        let max_z = if self.start.z().value() > self.end.z().value() {
            self.start.z().clone()
        } else {
            self.end.z().clone()
        };

        (Point3D::new(min_x, min_y, min_z), Point3D::new(max_x, max_y, max_z))
    }

    /// 線分上の最近点を取得
    pub fn closest_point_on_line(&self, point: &Point3D) -> Point3D {
        let sx = self.start.x().value(); let sy = self.start.y().value(); let sz = self.start.z().value();
        let ex = self.end.x().value();   let ey = self.end.y().value();   let ez = self.end.z().value();
        let px = point.x().value(); let py = point.y().value(); let pz = point.z().value();
        let dx = ex - sx; let dy = ey - sy; let dz = ez - sz; let len_sq = dx*dx + dy*dy + dz*dz;
        if len_sq < 1e-10 { return self.start.clone(); }
        let t = ((px - sx)*dx + (py - sy)*dy + (pz - sz)*dz) / len_sq;
        let tc = t.clamp(0.0, 1.0);
        let cx = sx + dx * tc; let cy = sy + dy * tc; let cz = sz + dz * tc;
        Point3D::new(Scalar::new(cx), Scalar::new(cy), Scalar::new(cz))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_creation() {
        let start = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
        let end = Point3D::new(Scalar::new(1.0), Scalar::new(1.0), Scalar::new(1.0));
        let line = Line3D::new(start.clone(), end.clone());

        assert_eq!(line.start().x().value(), 0.0);
        assert_eq!(line.start().y().value(), 0.0);
        assert_eq!(line.start().z().value(), 0.0);
        assert_eq!(line.end().x().value(), 1.0);
        assert_eq!(line.end().y().value(), 1.0);
        assert_eq!(line.end().z().value(), 1.0);
    }

    #[test]
    fn test_line_from_f64() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 3.0, 4.0, 5.0);

        assert_eq!(line.start().x().value(), 0.0);
        assert_eq!(line.start().y().value(), 0.0);
        assert_eq!(line.start().z().value(), 0.0);
        assert_eq!(line.end().x().value(), 3.0);
        assert_eq!(line.end().y().value(), 4.0);
        assert_eq!(line.end().z().value(), 5.0);
    }

    #[test]
    fn test_length() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 3.0, 4.0, 5.0);
        let length = line.length();

        // sqrt(3^2 + 4^2 + 5^2) = sqrt(50) ≈ 7.071
        assert!((length.value() - (50.0_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_midpoint() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 2.0, 4.0, 6.0);
        let midpoint = line.midpoint();

        assert_eq!(midpoint.x().value(), 1.0);
        assert_eq!(midpoint.y().value(), 2.0);
        assert_eq!(midpoint.z().value(), 3.0);
    }

    #[test]
    fn test_evaluate() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 2.0, 4.0, 6.0);

        // t = 0.0で開始点
        let point = line.evaluate_f64(0.0).unwrap();
        assert_eq!(point.x().value(), 0.0);
        assert_eq!(point.y().value(), 0.0);
        assert_eq!(point.z().value(), 0.0);

        // t = 1.0で終了点
        let point = line.evaluate_f64(1.0).unwrap();
        assert_eq!(point.x().value(), 2.0);
        assert_eq!(point.y().value(), 4.0);
        assert_eq!(point.z().value(), 6.0);

        // t = 0.5で中点
        let point = line.evaluate_f64(0.5).unwrap();
        assert_eq!(point.x().value(), 1.0);
        assert_eq!(point.y().value(), 2.0);
        assert_eq!(point.z().value(), 3.0);

        // 範囲外
        assert!(line.evaluate_f64(-0.1).is_none());
        assert!(line.evaluate_f64(1.1).is_none());
    }

    #[test]
    fn test_contains_point() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 2.0, 4.0, 6.0);
        let tolerance = ToleranceContext::standard();

        // 線分上の点
        let point = Point3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(3.0));
        assert!(line.contains_point(&point, &tolerance));

        // 開始点
        let point = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
        assert!(line.contains_point(&point, &tolerance));

        // 終了点
        let point = Point3D::new(Scalar::new(2.0), Scalar::new(4.0), Scalar::new(6.0));
        assert!(line.contains_point(&point, &tolerance));

        // 線分上にない点
        let point = Point3D::new(Scalar::new(1.0), Scalar::new(1.0), Scalar::new(1.0));
        assert!(!line.contains_point(&point, &tolerance));

        // 直線の延長上にある点
        let point = Point3D::new(Scalar::new(-1.0), Scalar::new(-2.0), Scalar::new(-3.0));
        assert!(!line.contains_point(&point, &tolerance));
    }

    #[test]
    fn test_distance_to_point() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 2.0, 0.0, 0.0); // X軸上の線分

        // 線分上の点
        let point = Point3D::new(Scalar::new(1.0), Scalar::new(0.0), Scalar::new(0.0));
    assert!(line.distance_to_point_f64(&point) < 1e-10);

        // 線分に垂直な点
        let point = Point3D::new(Scalar::new(1.0), Scalar::new(1.0), Scalar::new(0.0));
    assert!((line.distance_to_point_f64(&point) - 1.0).abs() < 1e-10);

        // 開始点の延長上
        let point = Point3D::new(Scalar::new(-1.0), Scalar::new(0.0), Scalar::new(0.0));
    assert!((line.distance_to_point_f64(&point) - 1.0).abs() < 1e-10);

        // 終了点の延長上
        let point = Point3D::new(Scalar::new(3.0), Scalar::new(0.0), Scalar::new(0.0));
        assert!((line.distance_to_point_f64(&point) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_of_point() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 2.0, 4.0, 6.0);
        let tolerance = ToleranceContext::standard();

        // 開始点
        let point = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
    let param = line.parameter_of_point_f64(&point, &tolerance).unwrap();
    assert!((param - 0.0).abs() < 1e-10);

        // 終了点
        let point = Point3D::new(Scalar::new(2.0), Scalar::new(4.0), Scalar::new(6.0));
    let param = line.parameter_of_point_f64(&point, &tolerance).unwrap();
    assert!((param - 1.0).abs() < 1e-10);

        // 中点
        let point = Point3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(3.0));
    let param = line.parameter_of_point_f64(&point, &tolerance).unwrap();
    assert!((param - 0.5).abs() < 1e-10);

        // 線分上にない点
        let point = Point3D::new(Scalar::new(1.0), Scalar::new(1.0), Scalar::new(1.0));
        assert!(line.parameter_of_point_f64(&point, &tolerance).is_none());
    }

    #[test]
    fn test_translate() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    let translated = line.translate_f64(2.0, 3.0, 4.0);

        assert_eq!(translated.start().x().value(), 2.0);
        assert_eq!(translated.start().y().value(), 3.0);
        assert_eq!(translated.start().z().value(), 4.0);
        assert_eq!(translated.end().x().value(), 3.0);
        assert_eq!(translated.end().y().value(), 4.0);
        assert_eq!(translated.end().z().value(), 5.0);
    }

    #[test]
    fn test_rotate_z() {
        let line = Line3D::from_f64(1.0, 0.0, 0.0, 2.0, 0.0, 0.0);
    let rotated = line.rotate_z_f64(std::f64::consts::PI / 2.0); // 90度回転

        assert!((rotated.start().x().value() - 0.0).abs() < 1e-10);
        assert!((rotated.start().y().value() - 1.0).abs() < 1e-10);
        assert_eq!(rotated.start().z().value(), 0.0);
        assert!((rotated.end().x().value() - 0.0).abs() < 1e-10);
        assert!((rotated.end().y().value() - 2.0).abs() < 1e-10);
        assert_eq!(rotated.end().z().value(), 0.0);
    }

    #[test]
    fn test_rotate_around_axis() {
        let line = Line3D::from_f64(1.0, 0.0, 0.0, 2.0, 0.0, 0.0);
        let z_axis = Vector3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(1.0));
        let rotated = line.rotate_around_axis(&z_axis, Scalar::new(std::f64::consts::PI / 2.0));

        assert!((rotated.start().x().value() - 0.0).abs() < 1e-10);
        assert!((rotated.start().y().value() - 1.0).abs() < 1e-10);
        assert_eq!(rotated.start().z().value(), 0.0);
        assert!((rotated.end().x().value() - 0.0).abs() < 1e-10);
        assert!((rotated.end().y().value() - 2.0).abs() < 1e-10);
        assert_eq!(rotated.end().z().value(), 0.0);
    }

    #[test]
    fn test_equals() {
        let line1 = Line3D::from_f64(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let line2 = Line3D::from_f64(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let line3 = Line3D::from_f64(1.0, 1.0, 1.0, 0.0, 0.0, 0.0); // 反対方向
        let line4 = Line3D::from_f64(0.0, 0.0, 0.0, 2.0, 2.0, 2.0);

        let tolerance = ToleranceContext::standard();

        assert!(line1.equals(&line2, &tolerance));
        assert!(line1.equals(&line3, &tolerance)); // 方向は関係なし
        assert!(!line1.equals(&line4, &tolerance));
    }

    #[test]
    fn test_bounding_box() {
        let line = Line3D::from_f64(1.0, 3.0, 5.0, 2.0, 1.0, 4.0);
        let (min_point, max_point) = line.bounding_box();

        assert_eq!(min_point.x().value(), 1.0);
        assert_eq!(min_point.y().value(), 1.0);
        assert_eq!(min_point.z().value(), 4.0);
        assert_eq!(max_point.x().value(), 2.0);
        assert_eq!(max_point.y().value(), 3.0);
        assert_eq!(max_point.z().value(), 5.0);
    }

    #[test]
    fn test_closest_point_on_line() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 2.0, 0.0, 0.0); // X軸上の線分

        // 線分上の点
        let test_point = Point3D::new(Scalar::new(1.0), Scalar::new(1.0), Scalar::new(1.0));
        let closest = line.closest_point_on_line(&test_point);
        assert_eq!(closest.x().value(), 1.0);
        assert_eq!(closest.y().value(), 0.0);
        assert_eq!(closest.z().value(), 0.0);

        // 開始点より前の場合
        let test_point = Point3D::new(Scalar::new(-1.0), Scalar::new(1.0), Scalar::new(1.0));
        let closest = line.closest_point_on_line(&test_point);
        assert_eq!(closest.x().value(), 0.0);
        assert_eq!(closest.y().value(), 0.0);
        assert_eq!(closest.z().value(), 0.0);

        // 終了点より後の場合
        let test_point = Point3D::new(Scalar::new(3.0), Scalar::new(1.0), Scalar::new(1.0));
        let closest = line.closest_point_on_line(&test_point);
        assert_eq!(closest.x().value(), 2.0);
        assert_eq!(closest.y().value(), 0.0);
        assert_eq!(closest.z().value(), 0.0);
    }

    #[test]
    fn test_degenerate_line() {
        // 退化した線分（同じ点）
        let line = Line3D::from_f64(1.0, 2.0, 3.0, 1.0, 2.0, 3.0);
        let tolerance = ToleranceContext::standard();

        assert_eq!(line.length().value(), 0.0);

        let point = Point3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(3.0));
        assert!(line.contains_point(&point, &tolerance));

        let other_point = Point3D::new(Scalar::new(2.0), Scalar::new(3.0), Scalar::new(4.0));
        assert!(!line.contains_point(&other_point, &tolerance));
    }

    #[test]
    fn test_direction_vector() {
        let line = Line3D::from_f64(1.0, 2.0, 3.0, 4.0, 6.0, 8.0);
        let direction = line.direction_vector();

        assert_eq!(direction.x().value(), 3.0);
        assert_eq!(direction.y().value(), 4.0);
        assert_eq!(direction.z().value(), 5.0);
    }

    #[test]
    fn test_direction() {
        let line = Line3D::from_f64(0.0, 0.0, 0.0, 3.0, 4.0, 5.0);
        let direction = line.direction().unwrap();

        // 正規化された方向ベクトル
        let magnitude = (3.0_f64 * 3.0 + 4.0 * 4.0 + 5.0 * 5.0).sqrt();
        assert!((direction.to_vector().x().value() - 3.0 / magnitude).abs() < 1e-10);
        assert!((direction.to_vector().y().value() - 4.0 / magnitude).abs() < 1e-10);
        assert!((direction.to_vector().z().value() - 5.0 / magnitude).abs() < 1e-10);

        // 退化した線分では方向が取得できない
        let degenerate_line = Line3D::from_f64(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        assert!(degenerate_line.direction().is_none());
    }
}
