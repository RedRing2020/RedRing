/// 平面プリミティブの定義
/// 
/// 3D空間における平面要素

use geo_core::{Vector3D, Scalar, Vector, ToleranceContext};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::point::Point3D;

/// 3D平面プリミティブ（点と法線ベクトルで定義）
#[derive(Debug, Clone)]
pub struct Plane3D {
    point: Point3D,  // 平面上の任意の点
    normal: Vector3D, // 正規化された法線ベクトル
}

impl Plane3D {
    /// 点と法線ベクトルから平面を作成
    pub fn new(point: Point3D, normal: Vector3D) -> Option<Self> {
        let mag_squared = normal.x().value() * normal.x().value() 
            + normal.y().value() * normal.y().value() 
            + normal.z().value() * normal.z().value();
        
        if mag_squared > 1e-20 {
            let magnitude = Scalar::new(mag_squared.sqrt());
            let normalized_normal = Vector3D::new(
                normal.x() / magnitude,
                normal.y() / magnitude,
                normal.z() / magnitude,
            );
            Some(Self { 
                point, 
                normal: normalized_normal 
            })
        } else {
            None
        }
    }
    
    /// 3点から平面を作成
    pub fn from_three_points(p1: Point3D, p2: Point3D, p3: Point3D) -> Option<Self> {
        let v1 = Vector3D::new(
            Scalar::new(p2.x() - p1.x()),
            Scalar::new(p2.y() - p1.y()),
            Scalar::new(p2.z() - p1.z()),
        );
        let v2 = Vector3D::new(
            Scalar::new(p3.x() - p1.x()),
            Scalar::new(p3.y() - p1.y()),
            Scalar::new(p3.z() - p1.z()),
        );
        
        let normal = v1.cross(&v2);
        Self::new(p1, normal)
    }
    
    pub fn point(&self) -> &Point3D {
        &self.point
    }
    
    pub fn normal(&self) -> &Vector3D {
        &self.normal
    }
    
    /// 平面の方程式係数 (ax + by + cz + d = 0) を取得
    pub fn equation_coefficients(&self) -> (f64, f64, f64, f64) {
        let a = self.normal.x().value();
        let b = self.normal.y().value();
        let c = self.normal.z().value();
        let d = -(a * self.point.x() + b * self.point.y() + c * self.point.z());
        (a, b, c, d)
    }
    
    /// 点から平面までの距離（符号付き）
    pub fn signed_distance_to_point(&self, point: &Point3D) -> f64 {
        let (a, b, c, d) = self.equation_coefficients();
        a * point.x() + b * point.y() + c * point.z() + d
    }
    
    /// 点から平面までの距離（絶対値）
    pub fn distance_to_point(&self, point: &Point3D) -> f64 {
        self.signed_distance_to_point(point).abs()
    }
    
    /// 点が平面上にあるかチェック
    pub fn contains_point(&self, point: &Point3D, tolerance: Option<&ToleranceContext>) -> bool {
        let distance = self.distance_to_point(point);
        let threshold = tolerance
            .map(|t| t.linear)
            .unwrap_or(1e-6);
        distance < threshold
    }
    
    /// 点の平面への投影
    pub fn project_point(&self, point: &Point3D) -> Point3D {
        let distance = self.signed_distance_to_point(point);
        Point3D::new(
            point.x() - distance * self.normal.x().value(),
            point.y() - distance * self.normal.y().value(),
            point.z() - distance * self.normal.z().value(),
        )
    }
}

impl GeometricPrimitive for Plane3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Plane
    }
    
    fn bounding_box(&self) -> BoundingBox {
        // 平面は無限なので、便宜上大きなボックスを返す
        let large_value = 1e6;
        BoundingBox::new(
            geo_core::Point3D::from_f64(-large_value, -large_value, -large_value),
            geo_core::Point3D::from_f64(large_value, large_value, large_value),
        )
    }
    
    fn measure(&self) -> Option<f64> {
        None // 無限平面に面積はない
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_from_three_points() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(0.0, 1.0, 0.0);
        
        let plane = Plane3D::from_three_points(p1, p2, p3).unwrap();
        
        // XY平面なので法線はZ軸方向
        let normal = plane.normal();
        assert!(normal.x().value().abs() < 1e-10);
        assert!(normal.y().value().abs() < 1e-10);
        assert!((normal.z().value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_plane_distance() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(0.0, 1.0, 0.0);
        let plane = Plane3D::from_three_points(p1, p2, p3).unwrap();
        
        let test_point = Point3D::new(0.0, 0.0, 5.0);
        assert!((plane.distance_to_point(&test_point) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_plane_contains_point() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(0.0, 1.0, 0.0);
        let plane = Plane3D::from_three_points(p1, p2, p3).unwrap();
        
        let on_plane = Point3D::new(0.5, 0.5, 0.0);
        let off_plane = Point3D::new(0.0, 0.0, 1.0);
        
        assert!(plane.contains_point(&on_plane, None));
        assert!(!plane.contains_point(&off_plane, None));
    }

    #[test]
    fn test_plane_project_point() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(0.0, 1.0, 0.0);
        let plane = Plane3D::from_three_points(p1, p2, p3).unwrap();
        
        let point_above = Point3D::new(1.0, 1.0, 5.0);
        let projected = plane.project_point(&point_above);
        
        assert!((projected.z() - 0.0).abs() < 1e-10);
        assert!((projected.x() - 1.0).abs() < 1e-10);
        assert!((projected.y() - 1.0).abs() < 1e-10);
    }
}