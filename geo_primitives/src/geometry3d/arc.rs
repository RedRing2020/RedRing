use geo_core::{Scalar, ToleranceContext};
use crate::geometry3d::{Point3D, Vector3D, Direction3D, Circle3D};

/// 3D円弧：3D円の一部分を表現
#[derive(Debug, Clone)]
pub struct Arc3D {
    circle: Circle3D,
    start_angle: Scalar,
    end_angle: Scalar,
}

impl Arc3D {
    /// 円とアングルから円弧を作成
    pub fn new(circle: Circle3D, start_angle: Scalar, end_angle: Scalar) -> Self {
        // 角度を正規化（0-2π範囲）
        let start_norm = Scalar::new(start_angle.value().rem_euclid(2.0 * std::f64::consts::PI));
        let end_norm = Scalar::new(end_angle.value().rem_euclid(2.0 * std::f64::consts::PI));

        Self {
            circle,
            start_angle: start_norm,
            end_angle: end_norm,
        }
    }

    /// 中心点、半径、法線、開始角度、終了角度から円弧を作成
    pub fn from_center_radius_normal(
        center: Point3D,
        radius: Scalar,
        normal: Direction3D,
        start_angle: Scalar,
        end_angle: Scalar
    ) -> Option<Self> {
        if let Some(circle) = Circle3D::new(center, radius, normal) {
            Some(Self::new(circle, start_angle, end_angle))
        } else {
            None
        }
    }

    /// f64値からの簡易コンストラクタ
    pub fn from_f64(
        center_x: f64, center_y: f64, center_z: f64,
        radius: f64,
        normal_x: f64, normal_y: f64, normal_z: f64,
        start_angle: f64, end_angle: f64
    ) -> Option<Self> {
        let center = Point3D::new(center_x, center_y, center_z);
        let normal = Direction3D::from_f64(normal_x, normal_y, normal_z)?;
        if let Some(circle) = Circle3D::from_f64(center, radius, normal) {
            Some(Self::new(circle, Scalar::new(start_angle), Scalar::new(end_angle)))
        } else {
            None
        }
    }

    /// 3点から円弧を作成
    pub fn from_three_points(start: Point3D, mid: Point3D, end: Point3D) -> Option<Self> {
        // 3点を通る円の中心と法線を計算
        let v1 = Vector3D::from_f64(
            mid.x() - start.x(),
            mid.y() - start.y(),
            mid.z() - start.z(),
        );
        let v2 = Vector3D::from_f64(
            end.x() - start.x(),
            end.y() - start.y(),
            end.z() - start.z(),
        );

        // 法線ベクトル = v1 × v2
        let normal_vec = Vector3D::from_f64(
            v1.y().value() * v2.z().value() - v1.z().value() * v2.y().value(),
            v1.z().value() * v2.x().value() - v1.x().value() * v2.z().value(),
            v1.x().value() * v2.y().value() - v1.y().value() * v2.x().value(),
        );

        let normal = Direction3D::from_f64(
            normal_vec.x().value(),
            normal_vec.y().value(),
            normal_vec.z().value(),
        )?;

        // 3点を通る円の中心を計算（外心計算）
        let center = Self::circumcenter(&start, &mid, &end)?;
        let radius = ((start.x() - center.x()).powi(2) +
                     (start.y() - center.y()).powi(2) +
                     (start.z() - center.z()).powi(2)).sqrt();

        if let Some(circle) = Circle3D::from_f64(center, radius, normal) {
            // 角度計算
            let start_angle = Self::calculate_angle(&circle, &start);
            let end_angle = Self::calculate_angle(&circle, &end);
            Some(Self::new(circle, start_angle, end_angle))
        } else {
            None
        }
    }

    /// 3点の外心を計算
    fn circumcenter(p1: &Point3D, p2: &Point3D, p3: &Point3D) -> Option<Point3D> {
        // 標準的な外心計算: a = p2-p1, b = p3-p1
        let a = Vector3D::from_f64(p2.x() - p1.x(), p2.y() - p1.y(), p2.z() - p1.z());
        let b = Vector3D::from_f64(p3.x() - p1.x(), p3.y() - p1.y(), p3.z() - p1.z());

        let ax = a.x().value(); let ay = a.y().value(); let az = a.z().value();
        let bx = b.x().value(); let by = b.y().value(); let bz = b.z().value();

        // n = a × b
        let nx = ay * bz - az * by;
        let ny = az * bx - ax * bz;
        let nz = ax * by - ay * bx;
        let n_sq = nx*nx + ny*ny + nz*nz;
        if n_sq < 1e-14 { return None; }

        let a_sq = ax*ax + ay*ay + az*az;
        let b_sq = bx*bx + by*by + bz*bz;

        // (a·a)(b×n) + (b·b)(n×a)
        let bxn_x = by * nz - bz * ny;
        let bxn_y = bz * nx - bx * nz;
        let bxn_z = bx * ny - by * nx;

        let nxa_x = ny * az - nz * ay;
        let nxa_y = nz * ax - nx * az;
        let nxa_z = nx * ay - ny * ax;

        let cx = p1.x() + (a_sq * bxn_x + b_sq * nxa_x) / (2.0 * n_sq);
        let cy = p1.y() + (a_sq * bxn_y + b_sq * nxa_y) / (2.0 * n_sq);
        let cz = p1.z() + (a_sq * bxn_z + b_sq * nxa_z) / (2.0 * n_sq);

        Some(Point3D::new(cx, cy, cz))
    }

    /// 円上の点から角度を計算
    fn calculate_angle(circle: &Circle3D, point: &Point3D) -> Scalar {
        let to_point = Vector3D::from_f64(
            point.x() - circle.center().x(),
            point.y() - circle.center().y(),
            point.z() - circle.center().z(),
        );

        let u_proj = to_point.x().value() * circle.u_axis().x() +
                     to_point.y().value() * circle.u_axis().y() +
                     to_point.z().value() * circle.u_axis().z();
        let v_proj = to_point.x().value() * circle.v_axis().x() +
                     to_point.y().value() * circle.v_axis().y() +
                     to_point.z().value() * circle.v_axis().z();

    Scalar::new(v_proj.atan2(u_proj))
    }

    /// 点から中心への角度を計算
    pub fn angle_of(&self, point: &Point3D) -> Scalar {
        Self::calculate_angle(&self.circle, point)
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
    pub fn base_circle(&self) -> &Circle3D {
        &self.circle
    }

    /// 中心点を取得
    pub fn center(&self) -> &Point3D {
        self.circle.center()
    }

    /// 半径を取得
    pub fn radius(&self) -> Scalar {
        self.circle.radius()
    }

    /// 法線を取得
    pub fn normal(&self) -> &Direction3D {
        self.circle.normal()
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
    pub fn evaluate(&self, t: f64) -> Point3D {
        let angle = self.start_angle.value() + t * self.sweep_angle().value();
        self.circle.evaluate(angle)
    }

    /// 開始点
    pub fn start_point(&self) -> Point3D {
        self.evaluate(0.0)
    }

    /// 終了点
    pub fn end_point(&self) -> Point3D {
        self.evaluate(1.0)
    }

    /// 中点
    pub fn midpoint(&self) -> Point3D {
        self.evaluate(0.5)
    }

    /// パラメータtにおける接線方向
    pub fn tangent(&self, t: f64) -> Vector3D {
        let angle = self.start_angle.value() + t * self.sweep_angle().value();
        self.circle.tangent(angle)
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
    pub fn contains_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        // まず円周上にあるかチェック
        if !self.circle.contains_point(point, tolerance) {
            return false;
        }

        // 角度が範囲内にあるかチェック
        let angle = self.angle_of(point);
        self.contains_angle(angle, tolerance)
    }

    /// 点までの距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        if self.contains_point(point, &ToleranceContext::standard()) {
            // 円弧上の場合、距離は0
            Scalar::new(0.0)
        } else {
            // 円弧上にない場合、最も近い点（開始点または終了点）までの距離
            let start_point = self.start_point();
            let end_point = self.end_point();

            let dist_to_start = Scalar::new(((point.x() - start_point.x()).powi(2) +
                                            (point.y() - start_point.y()).powi(2) +
                                            (point.z() - start_point.z()).powi(2)).sqrt());
            let dist_to_end = Scalar::new(((point.x() - end_point.x()).powi(2) +
                                          (point.y() - end_point.y()).powi(2) +
                                          (point.z() - end_point.z()).powi(2)).sqrt());

            if dist_to_start.value() < dist_to_end.value() {
                dist_to_start
            } else {
                dist_to_end
            }
        }
    }

    /// 平行移動
    pub fn translate(&self, translation: &Vector3D) -> Arc3D {
        let translated_circle = self.circle.translate(translation);
        Self::new(translated_circle, self.start_angle, self.end_angle)
    }

    /// 指定軸周りの回転
    pub fn rotate_around_axis(&self, axis: &Direction3D, angle: f64, origin: &Point3D) -> Arc3D {
        let rotated_circle = self.circle.rotate_around_axis(axis, angle, origin);
        Self::new(rotated_circle, self.start_angle, self.end_angle)
    }

    /// スケーリング
    pub fn scale(&self, factor: Scalar) -> Option<Arc3D> {
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
    fn test_arc3d_creation() {
        let arc = Arc3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, std::f64::consts::PI);

        assert!(arc.is_some());
        let arc = arc.unwrap();
        assert_eq!(arc.radius().value(), 1.0);
        assert_eq!(arc.start_angle().value(), 0.0);
        assert_eq!(arc.end_angle().value(), std::f64::consts::PI);
    }

    #[test]
    fn test_arc3d_evaluation() {
        let arc = Arc3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, std::f64::consts::PI).unwrap();

        let start_point = arc.start_point();
        assert!((start_point.x() - 1.0).abs() < 1e-10);
        assert!(start_point.y().abs() < 1e-10);
        assert!(start_point.z().abs() < 1e-10);

        let end_point = arc.end_point();
        assert!((end_point.x() + 1.0).abs() < 1e-10);
        assert!(end_point.y().abs() < 1e-10);
        assert!(end_point.z().abs() < 1e-10);
    }

    #[test]
    fn test_arc3d_length() {
        let arc = Arc3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, std::f64::consts::PI).unwrap();
        let expected_length = std::f64::consts::PI; // 半円の弧長
        assert!((arc.length().value() - expected_length).abs() < 1e-10);
    }

    #[test]
    fn test_contains_point() {
        let arc = Arc3D::from_f64(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, std::f64::consts::PI).unwrap();
        let tolerance = ToleranceContext::standard();

        // 弧上の点
        let point_on_arc = Point3D::new(0.0, 1.0, 0.0);
        assert!(arc.contains_point(&point_on_arc, &tolerance));

        // 弧外の点
        let point_off_arc = Point3D::new(0.0, -1.0, 0.0);
        assert!(!arc.contains_point(&point_off_arc, &tolerance));
    }

    #[test]
    fn test_three_point_arc3d() {
        let start = Point3D::new(1.0, 0.0, 0.0);
        let mid = Point3D::new(0.0, 1.0, 0.0);
        let end = Point3D::new(-1.0, 0.0, 0.0);

        let arc = Arc3D::from_three_points(start, mid, end);
        assert!(arc.is_some());

        let arc = arc.unwrap();
        assert!((arc.radius().value() - 1.0).abs() < 1e-8);
    }
}
