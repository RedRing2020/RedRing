/// 円錐サーフェスプリミティブ

use crate::geometry3d::Point3D;
use geo_core::{Vector3D, Scalar};
use geo_core::vector::Vector;
use geo_core::tolerance::ToleranceContext;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};

/// 3D円錐
#[derive(Debug, Clone)]
pub struct Cone {
    base: Point3D,      // 底面中心点
    apex: Point3D,      // 頂点
    radius: f64,        // 底面半径
}

impl Cone {
    /// 新しい円錐を作成
    pub fn new(base: Point3D, apex: Point3D, radius: f64) -> Option<Self> {
        if radius <= 0.0 {
            None
        } else if base.distance_to(&apex) < 1e-10 {
            None // 高さが0の円錐は無効
        } else {
            Some(Self { base, apex, radius })
        }
    }

    /// 底面中心点を取得
    pub fn base(&self) -> &Point3D {
        &self.base
    }

    /// 頂点を取得
    pub fn apex(&self) -> &Point3D {
        &self.apex
    }

    /// 底面半径を取得
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// 軸方向ベクトル（底面から頂点）
    pub fn axis_vector(&self) -> Vector3D {
        self.base.vector_to(&self.apex)
    }

    /// 軸方向の単位ベクトル
    pub fn axis_direction(&self) -> Vector3D {
        self.axis_vector().normalize_default().unwrap_or(Vector3D::z_axis())
    }

    /// 円錐の高さ
    pub fn height(&self) -> f64 {
        self.base.distance_to(&self.apex)
    }

    /// パラメトリック評価 (u: [0, 2π], v: [0, 1])
    /// v=0で底面、v=1で頂点
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        let axis = self.axis_vector();

        // v=1の場合は頂点
        if v >= 1.0 - 1e-10 {
            return self.apex.clone();
        }

        // 軸に垂直な2つのベクトルを計算（任意の直交基底）
        let temp = if axis.x().abs() < 0.9 {
            Vector3D::unit_x()
        } else {
            Vector3D::unit_y()
        };

        let u_vec = axis.cross(&temp).normalize().unwrap();
        let v_vec = axis.cross(&u_vec).normalize().unwrap();

        // v位置での半径（線形補間）
        let current_radius = self.radius * (1.0 - v);

        // 円形断面上の点
        let circle_point = u_vec * (current_radius * u.cos()) + v_vec * (current_radius * u.sin());

        // 軸方向に移動
        let height_offset = axis * v;

        Point3D::new(
            self.base.x() + circle_point.x() + height_offset.x(),
            self.base.y() + circle_point.y() + height_offset.y(),
            self.base.z() + circle_point.z() + height_offset.z(),
        )
    }

    /// 指定パラメータでの法線ベクトル
    pub fn normal(&self, u: f64, _v: f64) -> Vector3D {
        let axis = self.axis_vector();
        let height = self.height();

        // 軸に垂直な2つのベクトルを計算
        let temp = if axis.x().abs() < 0.9 {
            Vector3D::unit_x()
        } else {
            Vector3D::unit_y()
        };

        let u_vec = axis.cross(&temp).normalize().unwrap();
        let v_vec = axis.cross(&u_vec).normalize().unwrap();

        // 半径方向単位ベクトル
        let radial = u_vec * u.cos() + v_vec * u.sin();

        // 円錐の斜面角度を考慮した法線
        let slope = self.radius / height;
        let normal = radial * slope + axis.normalize().unwrap() * (1.0 - slope);

        normal.normalize().unwrap_or(Vector3D::unit_z())
    }

    /// 円錐の表面積（底面含む）
    pub fn surface_area(&self) -> f64 {
        let height = self.height();
        let slant_height = (height * height + self.radius * self.radius).sqrt();
        let base_area = std::f64::consts::PI * self.radius * self.radius;
        let side_area = std::f64::consts::PI * self.radius * slant_height;
        base_area + side_area
    }

    /// 円錐の体積
    pub fn volume(&self) -> f64 {
        let height = self.height();
        (1.0 / 3.0) * std::f64::consts::PI * self.radius * self.radius * height
    }

    /// 母線（generatrix）の長さ
    pub fn slant_height(&self) -> f64 {
        let height = self.height();
        (height * height + self.radius * self.radius).sqrt()
    }

    /// 点が円錐表面上にあるかを判定
    pub fn contains_point(&self, point: &Point3D, tolerance: f64) -> bool {
        // 軸への投影を計算
        let axis = self.axis_direction();
        let to_point = self.base.vector_to(point);
        let projection_length = to_point.dot(&axis);

        // 高さの範囲内か確認
        if projection_length < -tolerance || projection_length > self.height() + tolerance {
            return false;
        }

        // その高さでの理論半径
        let v = projection_length / self.height();
        let expected_radius = self.radius * (1.0 - v);

        // 軸からの距離を計算
        let projection = axis * projection_length;
        let radial_vector = to_point - projection;
        let radial_distance = radial_vector.norm();

        (radial_distance - expected_radius).abs() < tolerance
    }

    /// 点が円錐内部にあるかを判定
    pub fn contains_point_inside(&self, point: &Point3D) -> bool {
        // 軸への投影を計算
        let axis = self.axis_direction();
        let to_point = self.base.vector_to(point);
        let projection_length = to_point.dot(&axis);

        // 高さの範囲内か確認
        if projection_length < 0.0 || projection_length > self.height() {
            return false;
        }

        // その高さでの理論半径
        let v = projection_length / self.height();
        let max_radius = self.radius * (1.0 - v);

        // 軸からの距離を計算
        let projection = axis * projection_length;
        let radial_vector = to_point - projection;
        let radial_distance = radial_vector.norm();

        radial_distance < max_radius
    }

    /// 移動した新しい円錐を作成
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Cone {
        Self {
            base: self.base.translate(dx, dy, dz),
            apex: self.apex.translate(dx, dy, dz),
            radius: self.radius,
        }
    }

    /// 指定点を中心にスケール
    pub fn scale(&self, factor: f64, center: &Point3D) -> Option<Cone> {
        if factor <= 0.0 {
            return None;
        }

        let new_base = self.base.scale(factor, center);
        let new_apex = self.apex.scale(factor, center);
        let new_radius = self.radius * factor;

        Self::new(new_base, new_apex, new_radius)
    }
}

impl GeometricPrimitive for Cone {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Cone
    }

    fn bounding_box(&self) -> BoundingBox {
        // 簡易実装：軸に平行なバウンディングボックス
        let min_x = (self.base.x() - self.radius).min(self.apex.x());
        let max_x = (self.base.x() + self.radius).max(self.apex.x());
        let min_y = (self.base.y() - self.radius).min(self.apex.y());
        let max_y = (self.base.y() + self.radius).max(self.apex.y());
        let min_z = (self.base.z() - self.radius).min(self.apex.z());
        let max_z = (self.base.z() + self.radius).max(self.apex.z());

        BoundingBox::new(
            Point3D::new(min_x, min_y, min_z).to_geo_core(),
            Point3D::new(max_x, max_y, max_z).to_geo_core(),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.surface_area())
    }
}

impl PartialEq for Cone {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
            && self.apex == other.apex
            && (self.radius - other.radius).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cone_creation() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 3.0);
        let cone = Cone::new(base, apex, 2.0).unwrap();
        assert_eq!(cone.radius(), 2.0);
        assert_eq!(cone.height(), 3.0);
    }

    #[test]
    fn test_cone_invalid_radius() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 3.0);
        assert!(Cone::new(base.clone(), apex.clone(), -1.0).is_none());
        assert!(Cone::new(base, apex, 0.0).is_none());
    }

    #[test]
    fn test_cone_zero_height() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 0.0);
        assert!(Cone::new(base, apex, 1.0).is_none());
    }

    #[test]
    fn test_cone_volume_and_surface_area() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 3.0);
        let cone = Cone::new(base, apex, 1.0).unwrap();

        let expected_volume = (1.0 / 3.0) * std::f64::consts::PI * 3.0; // (1/3)πr²h = (1/3)π * 1 * 3

        assert!((cone.volume() - expected_volume).abs() < 1e-10);
        assert!(cone.surface_area() > 0.0);
    }

    #[test]
    fn test_cone_slant_height() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 4.0);
        let cone = Cone::new(base, apex, 3.0).unwrap();

        let expected_slant = 5.0; // sqrt(3² + 4²) = 5
        assert!((cone.slant_height() - expected_slant).abs() < 1e-10);
    }

    #[test]
    fn test_cone_contains_point() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 2.0);
        let cone = Cone::new(base, apex, 1.0).unwrap();

        // 底面近くの内部点
        let inside_base = Point3D::new(0.3, 0.0, 0.1);
        // 頂点近くの内部点（頂点付近では半径が小さくなることを考慮）
        let inside_apex = Point3D::new(0.05, 0.0, 1.8); // 半径を小さく調整
        // 外部点
        let outside = Point3D::new(2.0, 0.0, 1.0);

        assert!(cone.contains_point_inside(&inside_base));
        assert!(cone.contains_point_inside(&inside_apex));
        assert!(!cone.contains_point_inside(&outside));
    }

    #[test]
    fn test_cone_translate() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 2.0);
        let cone = Cone::new(base, apex, 1.0).unwrap();
        let translated = cone.translate(1.0, 1.0, 1.0);

        assert_eq!(translated.base().x(), 1.0);
        assert_eq!(translated.base().y(), 1.0);
        assert_eq!(translated.base().z(), 1.0);
        assert_eq!(translated.apex().z(), 3.0);
        assert_eq!(translated.radius(), 1.0);
    }

    #[test]
    fn test_cone_scale() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let apex = Point3D::new(0.0, 0.0, 2.0);
        let cone = Cone::new(base, apex, 1.0).unwrap();
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scaled = cone.scale(2.0, &scale_center).unwrap();

        assert_eq!(scaled.base().z(), 0.0);
        assert_eq!(scaled.apex().z(), 4.0);
        assert_eq!(scaled.radius(), 2.0);
    }
}
