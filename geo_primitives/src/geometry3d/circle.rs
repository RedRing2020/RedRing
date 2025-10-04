use geo_core::{Scalar, ToleranceContext};
use crate::geometry3d::{Point3D, Vector3D, Direction3D};

/// 3D円：中心点、半径、法線ベクトルを持つ円
#[derive(Debug, Clone)]
pub struct Circle3D {
    center: Point3D,
    radius: Scalar,
    normal: Direction3D,
    u_axis: Direction3D, // 円の局所X軸
    v_axis: Direction3D, // 円の局所Y軸
}

impl Circle3D {
    /// 中心点、半径、法線ベクトルから円を作成
    pub fn new(center: Point3D, radius: Scalar, normal: Direction3D) -> Option<Self> {
        if radius.value() <= 0.0 {
            None
        } else {
            // 円の局所座標系を構築
            let (u_axis, v_axis) = Self::build_local_axes(&normal)?;

            Some(Self {
                center,
                radius,
                normal,
                u_axis,
                v_axis,
            })
        }
    }

    /// f64値から円を作成
    pub fn from_f64(center: Point3D, radius: f64, normal: Direction3D) -> Option<Self> {
        if radius <= 0.0 {
            None
        } else {
            Self::new(center, Scalar::new(radius), normal)
        }
    }

    /// 法線ベクトルから局所座標系を構築
    fn build_local_axes(normal: &Direction3D) -> Option<(Direction3D, Direction3D)> {
        // ワールドX軸またはY軸との外積で局所U軸を作成
        let world_x = Direction3D::unit_x();
        let world_y = Direction3D::unit_y();

        let u_axis = if normal.dot(&world_x).value().abs() < 0.9 {
            // 法線がX軸と平行でない場合、X軸との外積を使用
            let cross_vec = normal.cross(&world_x);
            Direction3D::from_vector(&cross_vec)?
        } else {
            // 法線がX軸とほぼ平行な場合、Y軸との外積を使用
            let cross_vec = normal.cross(&world_y);
            Direction3D::from_vector(&cross_vec)?
        };

        // V軸 = 法線 × U軸
        let cross_vec = normal.cross(&u_axis);
        let v_axis = Direction3D::from_vector(&cross_vec)?;

        Some((u_axis, v_axis))
    }

    /// 中心点を取得
    pub fn center(&self) -> &Point3D {
        &self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> Scalar {
        self.radius
    }

    /// 半径をf64で取得
    pub fn radius_f64(&self) -> f64 {
        self.radius.value()
    }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> &Direction3D {
        &self.normal
    }

    /// U軸（局所X軸）を取得
    pub fn u_axis(&self) -> &Direction3D {
        &self.u_axis
    }

    /// V軸（局所Y軸）を取得
    pub fn v_axis(&self) -> &Direction3D {
        &self.v_axis
    }

    /// θ における円周上の点を計算
    pub fn evaluate(&self, theta: f64) -> Point3D {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let radius_val = self.radius.value();

        Point3D::new(
            self.center.x() + radius_val * (cos_theta * self.u_axis.x() + sin_theta * self.v_axis.x()),
            self.center.y() + radius_val * (cos_theta * self.u_axis.y() + sin_theta * self.v_axis.y()),
            self.center.z() + radius_val * (cos_theta * self.u_axis.z() + sin_theta * self.v_axis.z()),
        )
    }

    /// θ における接線方向ベクトル
    pub fn tangent(&self, theta: f64) -> Vector3D {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        Vector3D::new(
            -sin_theta * self.u_axis.x() + cos_theta * self.v_axis.x(),
            -sin_theta * self.u_axis.y() + cos_theta * self.v_axis.y(),
            -sin_theta * self.u_axis.z() + cos_theta * self.v_axis.z(),
        )
    }

    /// 点が円周上にあるかどうか（誤差を考慮）
    pub fn contains_point(&self, point: &Point3D, tolerance: &ToleranceContext) -> bool {
        // まず円の平面上にあるかチェック
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 法線との内積で平面上にあるかチェック
        let plane_distance = to_point.x() * self.normal.x() +
                           to_point.y() * self.normal.y() +
                           to_point.z() * self.normal.z();

        if plane_distance.abs() > tolerance.linear {
            return false;
        }

        // 中心からの距離が半径と等しいかチェック
        let distance_sq = to_point.x() * to_point.x() +
                         to_point.y() * to_point.y() +
                         to_point.z() * to_point.z();
        let radius_sq = self.radius.value() * self.radius.value();

        (distance_sq - radius_sq).abs() < tolerance.linear
    }

    /// 円を平行移動
    pub fn translate(&self, translation: &Vector3D) -> Self {
        Self {
            center: Point3D::new(
                self.center.x() + translation.x(),
                self.center.y() + translation.y(),
                self.center.z() + translation.z(),
            ),
            radius: self.radius,
            normal: self.normal.clone(),
            u_axis: self.u_axis.clone(),
            v_axis: self.v_axis.clone(),
        }
    }

    /// 円を拡大・縮小
    pub fn scale(&self, factor: Scalar) -> Option<Self> {
        if factor.value() <= 0.0 {
            None
        } else {
            Some(Self {
                center: self.center.clone(),
                radius: self.radius * factor,
                normal: self.normal.clone(),
                u_axis: self.u_axis.clone(),
                v_axis: self.v_axis.clone(),
            })
        }
    }

    /// 指定軸周りで回転
    pub fn rotate_around_axis(&self, axis: &Direction3D, angle: f64, origin: &Point3D) -> Self {
        // 簡易実装：ロドリゲスの回転公式を使用
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let one_minus_cos = 1.0 - cos_angle;

        // 中心点の回転
        let to_center = Vector3D::new(
            self.center.x() - origin.x(),
            self.center.y() - origin.y(),
            self.center.z() - origin.z(),
        );

        let rotated_center = Self::rodrigues_rotation(&to_center, axis, cos_angle, sin_angle, one_minus_cos);
        let new_center = Point3D::new(
            origin.x() + rotated_center.x(),
            origin.y() + rotated_center.y(),
            origin.z() + rotated_center.z(),
        );

        // 法線ベクトルの回転
        let normal_vec = Vector3D::new(self.normal.x(), self.normal.y(), self.normal.z());
        let rotated_normal_vec = Self::rodrigues_rotation(&normal_vec, axis, cos_angle, sin_angle, one_minus_cos);
        let new_normal = Direction3D::from_f64(rotated_normal_vec.x(), rotated_normal_vec.y(), rotated_normal_vec.z()).unwrap();

        Self::new(new_center, self.radius, new_normal).unwrap()
    }

    /// ロドリゲスの回転公式
    fn rodrigues_rotation(v: &Vector3D, k: &Direction3D, cos_angle: f64, sin_angle: f64, one_minus_cos: f64) -> Vector3D {
        let k_dot_v = k.x() * v.x() + k.y() * v.y() + k.z() * v.z();
        let k_cross_v = Vector3D::new(
            k.y() * v.z() - k.z() * v.y(),
            k.z() * v.x() - k.x() * v.z(),
            k.x() * v.y() - k.y() * v.x(),
        );

        Vector3D::new(
            v.x() * cos_angle + k_cross_v.x() * sin_angle + k.x() * k_dot_v * one_minus_cos,
            v.y() * cos_angle + k_cross_v.y() * sin_angle + k.y() * k_dot_v * one_minus_cos,
            v.z() * cos_angle + k_cross_v.z() * sin_angle + k.z() * k_dot_v * one_minus_cos,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::ToleranceContext;

    #[test]
    fn test_circle3d_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::unit_z();
        let circle = Circle3D::from_f64(center, 1.0, normal);

        assert!(circle.is_some());
        let circle = circle.unwrap();
        assert_eq!(circle.radius().value(), 1.0);
    }

    #[test]
    fn test_circle3d_evaluation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::unit_z();
        let circle = Circle3D::from_f64(center, 1.0, normal).unwrap();

        let point_0 = circle.evaluate(0.0);
        assert!((point_0.x() - 1.0).abs() < 1e-10);
        assert!(point_0.y().abs() < 1e-10);
        assert!(point_0.z().abs() < 1e-10);

        let point_pi_2 = circle.evaluate(std::f64::consts::PI / 2.0);
        assert!(point_pi_2.x().abs() < 1e-10);
        assert!((point_pi_2.y() - 1.0).abs() < 1e-10);
        assert!(point_pi_2.z().abs() < 1e-10);
    }

    #[test]
    fn test_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::unit_z();
        let circle = Circle3D::from_f64(center, 1.0, normal).unwrap();
        let tolerance = ToleranceContext::standard();

        let point_on_circle = Point3D::new(1.0, 0.0, 0.0);
        assert!(circle.contains_point(&point_on_circle, &tolerance));

        let point_off_circle = Point3D::new(2.0, 0.0, 0.0);
        assert!(!circle.contains_point(&point_off_circle, &tolerance));

        let point_off_plane = Point3D::new(1.0, 0.0, 1.0);
        assert!(!circle.contains_point(&point_off_plane, &tolerance));
    }
}
