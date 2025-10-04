/// 平面プリミティブの定義
///
/// 3D空間における平面要素

use geo_core::Vector3D;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox, geometry_utils::*};
use geo_core::Point3D;

/// 3D平面プリミティブ
#[derive(Debug, Clone)]
pub struct Plane {
    /// 平面上の一点
    point: Point3D,
    /// 法線ベクトル
    normal: Vector3D,
}

impl Plane {
    /// 新しい平面を作成
    pub fn new(point: Point3D, normal: Vector3D) -> Self {
        Self { point, normal }
    }

    /// 3点から平面を作成
    pub fn from_three_points(p1: &Point3D, p2: &Point3D, p3: &Point3D) -> Option<Self> {
        let (x1, y1, z1) = point3d_to_f64(p1);
        let (x2, y2, z2) = point3d_to_f64(p2);
        let (x3, y3, z3) = point3d_to_f64(p3);
        
        let v1_x = x2 - x1;
        let v1_y = y2 - y1;
        let v1_z = z2 - z1;
        
        let v2_x = x3 - x1;
        let v2_y = y3 - y1;
        let v2_z = z3 - z1;
        
        // 外積で法線を計算
        let normal_x = v1_y * v2_z - v1_z * v2_y;
        let normal_y = v1_z * v2_x - v1_x * v2_z;
        let normal_z = v1_x * v2_y - v1_y * v2_x;
        
        // 法線ベクトルがゼロかチェック
        let normal_length = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).sqrt();
        if normal_length < 1e-10 {
            return None; // 退化した平面（3点が一直線上）
        }
        
        let normal = Vector3D::from_f64(normal_x, normal_y, normal_z);
        
        Some(Self::new(p1.clone(), normal))
    }

    /// 平面上の基準点を取得
    pub fn point(&self) -> &Point3D {
        &self.point
    }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> &Vector3D {
        &self.normal
    }

    /// 平面の方程式 ax + by + cz + d = 0 の係数を取得
    pub fn equation_coefficients(&self) -> (f64, f64, f64, f64) {
        let (px, py, pz) = point3d_to_f64(&self.point);
    let a = self.normal.x();
    let b = self.normal.y();
    let c = self.normal.z();
        let d = -(a * px + b * py + c * pz);
        (a, b, c, d)
    }

    /// 点から平面までの符号付き距離
    pub fn signed_distance_to_point(&self, point: &Point3D) -> f64 {
        let (a, b, c, d) = self.equation_coefficients();
        let (px, py, pz) = point3d_to_f64(point);
        a * px + b * py + c * pz + d
    }

    /// 点から平面までの距離
    pub fn distance_to_point(&self, point: &Point3D) -> f64 {
        self.signed_distance_to_point(point).abs()
    }

    /// 点を平面に投影
    pub fn project_point(&self, point: &Point3D) -> Point3D {
        let distance = self.signed_distance_to_point(point);
        let (px, py, pz) = point3d_to_f64(point);
        point3d_from_f64(
            px - distance * self.normal.x(),
            py - distance * self.normal.y(),
            pz - distance * self.normal.z(),
        )
    }

    /// 点が平面上にあるかをチェック
    pub fn contains_point(&self, point: &Point3D, tolerance: f64) -> bool {
        self.distance_to_point(point) < tolerance
    }
}

impl GeometricPrimitive for Plane {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Plane
    }

    fn bounding_box(&self) -> BoundingBox {
        // 平面は無限に広がるため、適切なバウンディングボックスを定義するのは困難
        // ここでは基準点を中心とした大きなボックスを返す
        let (px, py, pz) = point3d_to_f64(&self.point);
        let extent = 1000.0; // 大きな値
        BoundingBox::new(
            point3d_from_f64(px - extent, py - extent, pz - extent),
            point3d_from_f64(px + extent, py + extent, pz + extent),
        )
    }

    fn measure(&self) -> Option<f64> {
        // 平面の面積は無限大なので、代わりに何らかの代表値を返す
        // ここでは法線ベクトルの長さを返す
        let (nx, ny, nz) = (self.normal.x(), self.normal.y(), self.normal.z());
        Some((nx * nx + ny * ny + nz * nz).sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_creation() {
        let point = Point3D::from_f64(0.0, 0.0, 0.0);
    let normal = Vector3D::from_f64(0.0, 0.0, 1.0);
        let plane = Plane::new(point, normal);
        
        let (px, py, pz) = point3d_to_f64(plane.point());
        assert_eq!(px, 0.0);
        assert_eq!(py, 0.0);
        assert_eq!(pz, 0.0);
    }

    #[test]
    fn test_plane_from_three_points() {
        let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
        let p2 = Point3D::from_f64(1.0, 0.0, 0.0);
        let p3 = Point3D::from_f64(0.0, 1.0, 0.0);
        
        let plane = Plane::from_three_points(&p1, &p2, &p3).unwrap();
        
        // Z=0平面の法線は(0, 0, 1)方向
    assert!((plane.normal().z() - 1.0).abs() < 1e-10 || (plane.normal().z() + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_plane_distance_to_point() {
        let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
        let p2 = Point3D::from_f64(1.0, 0.0, 0.0);
        let p3 = Point3D::from_f64(0.0, 1.0, 0.0);
        
        let plane = Plane::from_three_points(&p1, &p2, &p3).unwrap();
        
        let test_point = Point3D::from_f64(0.0, 0.0, 5.0);
        let distance = plane.distance_to_point(&test_point);
        assert!((distance - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_plane_project_point() {
        let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
        let p2 = Point3D::from_f64(1.0, 0.0, 0.0);
        let p3 = Point3D::from_f64(0.0, 1.0, 0.0);
        
        let plane = Plane::from_three_points(&p1, &p2, &p3).unwrap();
        
        let test_point = Point3D::from_f64(1.0, 1.0, 5.0);
        let projected = plane.project_point(&test_point);
        
        let (px, py, pz) = point3d_to_f64(&projected);
        assert!((px - 1.0).abs() < 1e-10);
        assert!((py - 1.0).abs() < 1e-10);
        assert!(pz.abs() < 1e-10);
    }
}