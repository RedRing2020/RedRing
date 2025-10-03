/// 三角形プリミティブの定義
/// 
/// 2D/3D空間における三角形要素

use geo_core::{Vector3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::point::{Point2D, Point3D};

/// 2D三角形プリミティブ
#[derive(Debug, Clone)]
pub struct Triangle2D {
    vertices: [Point2D; 3],
}

impl Triangle2D {
    pub fn new(v0: Point2D, v1: Point2D, v2: Point2D) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }
    
    pub fn vertices(&self) -> &[Point2D; 3] {
        &self.vertices
    }
    
    pub fn vertex(&self, index: usize) -> Option<&Point2D> {
        self.vertices.get(index)
    }
    
    /// 重心を計算
    pub fn centroid(&self) -> Point2D {
        Point2D::new(
            (self.vertices[0].x() + self.vertices[1].x() + self.vertices[2].x()) / 3.0,
            (self.vertices[0].y() + self.vertices[1].y() + self.vertices[2].y()) / 3.0,
        )
    }
    
    /// 面積を計算（外積使用）
    pub fn area(&self) -> f64 {
        let v1_x = self.vertices[1].x() - self.vertices[0].x();
        let v1_y = self.vertices[1].y() - self.vertices[0].y();
        let v2_x = self.vertices[2].x() - self.vertices[0].x();
        let v2_y = self.vertices[2].y() - self.vertices[0].y();
        
        0.5 * (v1_x * v2_y - v1_y * v2_x).abs()
    }
    
    /// 点が三角形内部にあるかチェック（重心座標使用）
    pub fn contains_point(&self, point: &Point2D) -> bool {
        let (alpha, beta, gamma) = self.barycentric_coordinates(point);
        alpha >= 0.0 && beta >= 0.0 && gamma >= 0.0 && 
        (alpha + beta + gamma - 1.0).abs() < 1e-10
    }
    
    /// 重心座標を計算
    pub fn barycentric_coordinates(&self, point: &Point2D) -> (f64, f64, f64) {
        let v0 = &self.vertices[0];
        let v1 = &self.vertices[1];
        let v2 = &self.vertices[2];
        
        let denom = (v1.y() - v2.y()) * (v0.x() - v2.x()) + (v2.x() - v1.x()) * (v0.y() - v2.y());
        
        if denom.abs() < 1e-10 {
            return (0.0, 0.0, 0.0); // 縮退三角形
        }
        
        let alpha = ((v1.y() - v2.y()) * (point.x() - v2.x()) + (v2.x() - v1.x()) * (point.y() - v2.y())) / denom;
        let beta = ((v2.y() - v0.y()) * (point.x() - v2.x()) + (v0.x() - v2.x()) * (point.y() - v2.y())) / denom;
        let gamma = 1.0 - alpha - beta;
        
        (alpha, beta, gamma)
    }
}

impl GeometricPrimitive for Triangle2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }
    
    fn bounding_box(&self) -> BoundingBox {
        let min_x = self.vertices[0].x().min(self.vertices[1].x()).min(self.vertices[2].x());
        let min_y = self.vertices[0].y().min(self.vertices[1].y()).min(self.vertices[2].y());
        let max_x = self.vertices[0].x().max(self.vertices[1].x()).max(self.vertices[2].x());
        let max_y = self.vertices[0].y().max(self.vertices[1].y()).max(self.vertices[2].y());
        
        BoundingBox::from_2d(
            geo_core::Point2D::from_f64(min_x, min_y),
            geo_core::Point2D::from_f64(max_x, max_y),
        )
    }
    
    fn measure(&self) -> Option<f64> {
        Some(self.area())
    }
}

/// 3D三角形プリミティブ
#[derive(Debug, Clone)]
pub struct Triangle3D {
    vertices: [Point3D; 3],
}

impl Triangle3D {
    pub fn new(v0: Point3D, v1: Point3D, v2: Point3D) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }
    
    pub fn vertices(&self) -> &[Point3D; 3] {
        &self.vertices
    }
    
    pub fn vertex(&self, index: usize) -> Option<&Point3D> {
        self.vertices.get(index)
    }
    
    /// 重心を計算
    pub fn centroid(&self) -> Point3D {
        Point3D::new(
            (self.vertices[0].x() + self.vertices[1].x() + self.vertices[2].x()) / 3.0,
            (self.vertices[0].y() + self.vertices[1].y() + self.vertices[2].y()) / 3.0,
            (self.vertices[0].z() + self.vertices[1].z() + self.vertices[2].z()) / 3.0,
        )
    }
    
    /// 面積を計算（外積使用）
    pub fn area(&self) -> f64 {
        let v1 = Vector3D::new(
            Scalar::new(self.vertices[1].x() - self.vertices[0].x()),
            Scalar::new(self.vertices[1].y() - self.vertices[0].y()),
            Scalar::new(self.vertices[1].z() - self.vertices[0].z()),
        );
        let v2 = Vector3D::new(
            Scalar::new(self.vertices[2].x() - self.vertices[0].x()),
            Scalar::new(self.vertices[2].y() - self.vertices[0].y()),
            Scalar::new(self.vertices[2].z() - self.vertices[0].z()),
        );
        
        let cross = v1.cross(&v2);
        let mag_squared = cross.x().value() * cross.x().value() 
            + cross.y().value() * cross.y().value() 
            + cross.z().value() * cross.z().value();
        0.5 * mag_squared.sqrt()
    }
    
    /// 法線ベクトルを計算
    pub fn normal(&self) -> Option<Vector3D> {
        let v1 = Vector3D::new(
            Scalar::new(self.vertices[1].x() - self.vertices[0].x()),
            Scalar::new(self.vertices[1].y() - self.vertices[0].y()),
            Scalar::new(self.vertices[1].z() - self.vertices[0].z()),
        );
        let v2 = Vector3D::new(
            Scalar::new(self.vertices[2].x() - self.vertices[0].x()),
            Scalar::new(self.vertices[2].y() - self.vertices[0].y()),
            Scalar::new(self.vertices[2].z() - self.vertices[0].z()),
        );
        
        let cross = v1.cross(&v2);
        let mag_squared = cross.x().value() * cross.x().value() 
            + cross.y().value() * cross.y().value() 
            + cross.z().value() * cross.z().value();
        
        if mag_squared > 1e-20 {
            let magnitude = Scalar::new(mag_squared.sqrt());
            Some(Vector3D::new(
                cross.x() / magnitude,
                cross.y() / magnitude,
                cross.z() / magnitude,
            ))
        } else {
            None
        }
    }
    
    /// 重心座標を計算（3D点を平面に投影）
    pub fn barycentric_coordinates_3d(&self, point: &Point3D) -> Option<(f64, f64, f64)> {
        let normal = self.normal()?;
        
        // 主軸を選択（法線の最大成分）
        let abs_x = normal.x().value().abs();
        let abs_y = normal.y().value().abs();
        let abs_z = normal.z().value().abs();
        
        let (u_idx, v_idx) = if abs_x >= abs_y && abs_x >= abs_z {
            (1, 2) // YZ平面に投影
        } else if abs_y >= abs_z {
            (0, 2) // XZ平面に投影
        } else {
            (0, 1) // XY平面に投影
        };
        
        // 2D座標を取得
        let get_coord = |p: &Point3D, idx: usize| match idx {
            0 => p.x(),
            1 => p.y(),
            2 => p.z(),
            _ => unreachable!(),
        };
        
        let v0_u = get_coord(&self.vertices[0], u_idx);
        let v0_v = get_coord(&self.vertices[0], v_idx);
        let v1_u = get_coord(&self.vertices[1], u_idx);
        let v1_v = get_coord(&self.vertices[1], v_idx);
        let v2_u = get_coord(&self.vertices[2], u_idx);
        let v2_v = get_coord(&self.vertices[2], v_idx);
        let p_u = get_coord(point, u_idx);
        let p_v = get_coord(point, v_idx);
        
        // 重心座標を計算
        let denom = (v1_v - v2_v) * (v0_u - v2_u) + (v2_u - v1_u) * (v0_v - v2_v);
        
        if denom.abs() < 1e-10 {
            return None; // 縮退三角形
        }
        
        let alpha = ((v1_v - v2_v) * (p_u - v2_u) + (v2_u - v1_u) * (p_v - v2_v)) / denom;
        let beta = ((v2_v - v0_v) * (p_u - v2_u) + (v0_u - v2_u) * (p_v - v2_v)) / denom;
        let gamma = 1.0 - alpha - beta;
        
        Some((alpha, beta, gamma))
    }
}

impl GeometricPrimitive for Triangle3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }
    
    fn bounding_box(&self) -> BoundingBox {
        let min_x = self.vertices[0].x().min(self.vertices[1].x()).min(self.vertices[2].x());
        let min_y = self.vertices[0].y().min(self.vertices[1].y()).min(self.vertices[2].y());
        let min_z = self.vertices[0].z().min(self.vertices[1].z()).min(self.vertices[2].z());
        let max_x = self.vertices[0].x().max(self.vertices[1].x()).max(self.vertices[2].x());
        let max_y = self.vertices[0].y().max(self.vertices[1].y()).max(self.vertices[2].y());
        let max_z = self.vertices[0].z().max(self.vertices[1].z()).max(self.vertices[2].z());
        
        BoundingBox::new(
            geo_core::Point3D::from_f64(min_x, min_y, min_z),
            geo_core::Point3D::from_f64(max_x, max_y, max_z),
        )
    }
    
    fn measure(&self) -> Option<f64> {
        Some(self.area())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_2d_area() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.0, 1.0),
        );
        
        assert!((triangle.area() - 0.5).abs() < 1e-10);
        assert_eq!(triangle.primitive_kind(), PrimitiveKind::Triangle);
    }

    #[test]
    fn test_triangle_3d_area() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        );
        
        assert!((triangle.area() - 0.5).abs() < 1e-10);
        
        let normal = triangle.normal().unwrap();
        // Z軸方向の法線になるはず
        assert!((normal.z().value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_barycentric_coordinates() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.0, 1.0),
        );
        
        let center = Point2D::new(1.0/3.0, 1.0/3.0);
        let (alpha, beta, gamma) = triangle.barycentric_coordinates(&center);
        
        // 重心での重心座標は (1/3, 1/3, 1/3) になるはず
        assert!((alpha - 1.0/3.0).abs() < 1e-10);
        assert!((beta - 1.0/3.0).abs() < 1e-10);
        assert!((gamma - 1.0/3.0).abs() < 1e-10);
    }
}