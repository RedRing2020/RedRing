/// 円柱サーフェスプリミティブ

use crate::geometry3d::Point3D;
use geo_core::{Vector3D, Scalar};
use geo_core::vector::Vector;
use geo_core::tolerance::ToleranceContext;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};

/// 3D円柱
#[derive(Debug, Clone)]
pub struct Cylinder {
    base: Point3D,      // 底面中心点
    top: Point3D,       // 上面中心点
    radius: f64,        // 半径
}

impl Cylinder {
    /// 新しい円柱を作成
    pub fn new(base: Point3D, top: Point3D, radius: f64) -> Option<Self> {
        if radius <= 0.0 {
            None
        } else if base.distance_to(&top) < 1e-10 {
            None // 高さが0の円柱は無効
        } else {
            Some(Self { base, top, radius })
        }
    }

    /// 底面中心点を取得
    pub fn base(&self) -> &Point3D {
        &self.base
    }

    /// 上面中心点を取得
    pub fn top(&self) -> &Point3D {
        &self.top
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// 軸方向ベクトル（底面から上面）
    pub fn axis_vector(&self) -> Vector3D {
        self.base.vector_to(&self.top)
    }

    /// 軸方向の単位ベクトル
    pub fn axis_direction(&self) -> Vector3D {
        let tolerance = ToleranceContext::default();
        self.axis_vector().normalize(&tolerance).unwrap_or(Vector3D::z_axis())
    }

    /// 円柱の高さ
    pub fn height(&self) -> f64 {
        self.base.distance_to(&self.top)
    }

    /// パラメトリック評価 (u: [0, 2π], v: [0, 1])
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        let axis = self.axis_vector();
        let tolerance = ToleranceContext::default();

        // 軸に垂直な2つのベクトルを計算（任意の直交基底）
        let temp = if axis.x().value().abs() < 0.9 {
            Vector3D::x_axis()
        } else {
            Vector3D::y_axis()
        };

        let u_vec = axis.cross(&temp).normalize(&tolerance).unwrap();
        let v_vec = axis.cross(&u_vec).normalize(&tolerance).unwrap();

        // 円形断面上の点
        let circle_point = u_vec * Scalar::from_f64(self.radius * u.cos()) + v_vec * Scalar::from_f64(self.radius * u.sin());

        // 軸方向に移動
        let height_offset = axis * Scalar::from_f64(v);

        Point3D::new(
            self.base.x() + circle_point.x().value() + height_offset.x().value(),
            self.base.y() + circle_point.y().value() + height_offset.y().value(),
            self.base.z() + circle_point.z().value() + height_offset.z().value(),
        )
    }

    /// 指定パラメータでの法線ベクトル
    pub fn normal(&self, u: f64, _v: f64) -> Vector3D {
        let axis = self.axis_vector();
        let tolerance = ToleranceContext::default();

        // 軸に垂直な2つのベクトルを計算
        let temp = if axis.x().value().abs() < 0.9 {
            Vector3D::x_axis()
        } else {
            Vector3D::y_axis()
        };

        let u_vec = axis.cross(&temp).normalize(&tolerance).unwrap();
        let v_vec = axis.cross(&u_vec).normalize(&tolerance).unwrap();

        // 法線は半径方向
        let normal = u_vec * Scalar::from_f64(u.cos()) + v_vec * Scalar::from_f64(u.sin());
        normal.normalize(&tolerance).unwrap_or(Vector3D::z_axis())
    }

    /// 円柱の表面積
    pub fn surface_area(&self) -> f64 {
        let height = self.height();
        let base_area = std::f64::consts::PI * self.radius * self.radius;
        let side_area = 2.0 * std::f64::consts::PI * self.radius * height;
        2.0 * base_area + side_area // 上面 + 底面 + 側面
    }

    /// 円柱の体積
    pub fn volume(&self) -> f64 {
        let height = self.height();
        std::f64::consts::PI * self.radius * self.radius * height
    }

    /// 点が円柱表面上にあるかを判定
    pub fn contains_point(&self, point: &Point3D, tolerance: f64) -> bool {
        // 軸への投影を計算
        let axis = self.axis_direction();
        let to_point = self.base.vector_to(point);
        let projection_length = to_point.dot(&axis).value();

        // 高さの範囲内か確認
        if projection_length < -tolerance || projection_length > self.height() + tolerance {
            return false;
        }

        // 軸からの距離を計算
        let projection = axis * Scalar::from_f64(projection_length);
        let radial_vector = to_point - projection;
        let radial_distance = radial_vector.norm().value();

        (radial_distance - self.radius).abs() < tolerance
    }

    /// 点が円柱内部にあるかを判定
    pub fn contains_point_inside(&self, point: &Point3D) -> bool {
        // 軸への投影を計算
        let axis = self.axis_direction();
        let to_point = self.base.vector_to(point);
        let projection_length = to_point.dot(&axis).value();

        // 高さの範囲内か確認
        if projection_length < 0.0 || projection_length > self.height() {
            return false;
        }

        // 軸からの距離を計算
        let projection = axis * Scalar::from_f64(projection_length);
        let radial_vector = to_point - projection;
        let radial_distance = radial_vector.norm().value();

        radial_distance < self.radius
    }

    /// 移動した新しい円柱を作成
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Cylinder {
        Self {
            base: self.base.translate(dx, dy, dz),
            top: self.top.translate(dx, dy, dz),
            radius: self.radius,
        }
    }

    /// 指定点を中心にスケール
    pub fn scale(&self, factor: f64, center: &Point3D) -> Option<Cylinder> {
        if factor <= 0.0 {
            return None;
        }

        let new_base = self.base.scale(factor, center);
        let new_top = self.top.scale(factor, center);
        let new_radius = self.radius * factor;

        Self::new(new_base, new_top, new_radius)
    }
}

impl GeometricPrimitive for Cylinder {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Cylinder
    }

    fn bounding_box(&self) -> BoundingBox {
        // 簡易実装：軸に平行なバウンディングボックス
        let min_x = self.base.x().min(self.top.x()) - self.radius;
        let max_x = self.base.x().max(self.top.x()) + self.radius;
        let min_y = self.base.y().min(self.top.y()) - self.radius;
        let max_y = self.base.y().max(self.top.y()) + self.radius;
        let min_z = self.base.z().min(self.top.z()) - self.radius;
        let max_z = self.base.z().max(self.top.z()) + self.radius;

        BoundingBox::new(
            Point3D::new(min_x, min_y, min_z).to_geo_core(),
            Point3D::new(max_x, max_y, max_z).to_geo_core(),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.surface_area())
    }
}

impl PartialEq for Cylinder {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
            && self.top == other.top
            && (self.radius - other.radius).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cylinder_creation() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let top = Point3D::new(0.0, 0.0, 5.0);
        let cylinder = Cylinder::new(base, top, 2.0).unwrap();
        assert_eq!(cylinder.radius(), 2.0);
        assert_eq!(cylinder.height(), 5.0);
    }

    #[test]
    fn test_cylinder_invalid_radius() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let top = Point3D::new(0.0, 0.0, 5.0);
        assert!(Cylinder::new(base.clone(), top.clone(), -1.0).is_none());
        assert!(Cylinder::new(base, top, 0.0).is_none());
    }

    #[test]
    fn test_cylinder_zero_height() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let top = Point3D::new(0.0, 0.0, 0.0);
        assert!(Cylinder::new(base, top, 1.0).is_none());
    }

    #[test]
    fn test_cylinder_volume_and_surface_area() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let top = Point3D::new(0.0, 0.0, 2.0);
        let cylinder = Cylinder::new(base, top, 1.0).unwrap();

        let expected_volume = std::f64::consts::PI * 2.0; // π * r² * h = π * 1 * 2
        let expected_surface_area = 2.0 * std::f64::consts::PI + 4.0 * std::f64::consts::PI; // 2πr² + 2πrh

        assert!((cylinder.volume() - expected_volume).abs() < 1e-10);
        assert!((cylinder.surface_area() - expected_surface_area).abs() < 1e-10);
    }

    #[test]
    fn test_cylinder_contains_point() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let top = Point3D::new(0.0, 0.0, 2.0);
        let cylinder = Cylinder::new(base, top, 1.0).unwrap();

        let inside = Point3D::new(0.5, 0.0, 1.0);
        let outside_radius = Point3D::new(2.0, 0.0, 1.0);
        let outside_height = Point3D::new(0.0, 0.0, 3.0);

        assert!(cylinder.contains_point_inside(&inside));
        assert!(!cylinder.contains_point_inside(&outside_radius));
        assert!(!cylinder.contains_point_inside(&outside_height));
    }

    #[test]
    fn test_cylinder_translate() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let top = Point3D::new(0.0, 0.0, 2.0);
        let cylinder = Cylinder::new(base, top, 1.0).unwrap();
        let translated = cylinder.translate(1.0, 1.0, 1.0);

        assert_eq!(translated.base().x(), 1.0);
        assert_eq!(translated.base().y(), 1.0);
        assert_eq!(translated.base().z(), 1.0);
        assert_eq!(translated.top().z(), 3.0);
        assert_eq!(translated.radius(), 1.0);
    }

    #[test]
    fn test_cylinder_scale() {
        let base = Point3D::new(0.0, 0.0, 0.0);
        let top = Point3D::new(0.0, 0.0, 2.0);
        let cylinder = Cylinder::new(base, top, 1.0).unwrap();
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scaled = cylinder.scale(2.0, &scale_center).unwrap();

        assert_eq!(scaled.base().z(), 0.0);
        assert_eq!(scaled.top().z(), 4.0);
        assert_eq!(scaled.radius(), 2.0);
    }
}
