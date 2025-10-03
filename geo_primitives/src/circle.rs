/// 円プリミティブの定義
/// 
/// 2D/3D空間における円要素

use geo_core::{Vector3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::point::{Point2D, Point3D};

/// 2D円プリミティブ
#[derive(Debug, Clone)]
pub struct Circle2D {
    center: Point2D,
    radius: f64,
}

impl Circle2D {
    pub fn new(center: Point2D, radius: f64) -> Option<Self> {
        if radius > 0.0 {
            Some(Self { center, radius })
        } else {
            None
        }
    }
    
    pub fn center(&self) -> &Point2D {
        &self.center
    }
    
    pub fn radius(&self) -> f64 {
        self.radius
    }
    
    /// 円周上の点を取得（角度指定）
    pub fn point_at_angle(&self, angle: f64) -> Point2D {
        Point2D::new(
            self.center.x() + self.radius * angle.cos(),
            self.center.y() + self.radius * angle.sin(),
        )
    }
    
    /// 面積を計算
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    
    /// 円周を計算
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
    
    /// 点が円内部にあるかチェック
    pub fn contains_point(&self, point: &Point2D) -> bool {
        self.center.distance_to(point) <= self.radius
    }
}

impl GeometricPrimitive for Circle2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Circle
    }
    
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_2d(
            geo_core::Point2D::from_f64(self.center.x() - self.radius, self.center.y() - self.radius),
            geo_core::Point2D::from_f64(self.center.x() + self.radius, self.center.y() + self.radius),
        )
    }
    
    fn measure(&self) -> Option<f64> {
        Some(self.area())
    }
}

/// 3D円プリミティブ
#[derive(Debug, Clone)]
pub struct Circle3D {
    center: Point3D,
    radius: f64,
    normal: Option<Vector3D>, // 円の法線ベクトル
}

impl Circle3D {
    pub fn new(center: Point3D, radius: f64, normal: Vector3D) -> Option<Self> {
        if radius > 0.0 {
            // 法線を正規化
            let mag_squared = normal.x().value() * normal.x().value() 
                + normal.y().value() * normal.y().value() 
                + normal.z().value() * normal.z().value();
            
            if mag_squared > 1e-20 {
                let magnitude = mag_squared.sqrt();
                let normalized_normal = Vector3D::new(
                    Scalar::new(normal.x().value() / magnitude),
                    Scalar::new(normal.y().value() / magnitude),
                    Scalar::new(normal.z().value() / magnitude),
                );
                Some(Self { 
                    center, 
                    radius, 
                    normal: Some(normalized_normal) 
                })
            } else {
                None // 無効な法線
            }
        } else {
            None
        }
    }
    
    /// XY平面上の円を作成
    pub fn new_xy_plane(center: Point3D, radius: f64) -> Option<Self> {
        let z_normal = Vector3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(1.0));
        Self::new(center, radius, z_normal)
    }
    
    pub fn center(&self) -> &Point3D {
        &self.center
    }
    
    pub fn radius(&self) -> f64 {
        self.radius
    }
    
    pub fn normal(&self) -> Option<&Vector3D> {
        self.normal.as_ref()
    }
    
    /// 面積を計算
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    
    /// 円周を計算
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
    
    /// パラメトリック評価（t ∈ [0, 2π]）
    pub fn evaluate(&self, t: f64) -> Option<Point3D> {
        if let Some(normal) = &self.normal {
            // 円の平面内で正交する2つのベクトルを生成
            let (u, v) = self.orthonormal_basis();
            
            let cos_t = t.cos();
            let sin_t = t.sin();
            
            Some(Point3D::new(
                self.center.x() + self.radius * (cos_t * u.x().value() + sin_t * v.x().value()),
                self.center.y() + self.radius * (cos_t * u.y().value() + sin_t * v.y().value()),
                self.center.z() + self.radius * (cos_t * u.z().value() + sin_t * v.z().value()),
            ))
        } else {
            None
        }
    }
    
    /// 法線に垂直な正規直交基底を生成
    fn orthonormal_basis(&self) -> (Vector3D, Vector3D) {
        if let Some(normal) = &self.normal {
            // 法線と最も角度の大きい軸を選択
            let abs_x = normal.x().value().abs();
            let abs_y = normal.y().value().abs();
            let abs_z = normal.z().value().abs();
            
            let u = if abs_x <= abs_y && abs_x <= abs_z {
                Vector3D::new(Scalar::new(1.0), Scalar::new(0.0), Scalar::new(0.0))
            } else if abs_y <= abs_z {
                Vector3D::new(Scalar::new(0.0), Scalar::new(1.0), Scalar::new(0.0))
            } else {
                Vector3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(1.0))
            };
            
            // u と normal の外積で v を計算
            let v_cross = u.cross(normal);
            // v を正規化
            let v_mag_squared = v_cross.x().value() * v_cross.x().value() 
                + v_cross.y().value() * v_cross.y().value() 
                + v_cross.z().value() * v_cross.z().value();
            let v_magnitude = v_mag_squared.sqrt();
            let v = Vector3D::new(
                Scalar::new(v_cross.x().value() / v_magnitude),
                Scalar::new(v_cross.y().value() / v_magnitude),
                Scalar::new(v_cross.z().value() / v_magnitude),
            );
            
            // u を v と normal の外積で再計算（正規直交化）
            let u_cross = v.cross(normal);
            let u_mag_squared = u_cross.x().value() * u_cross.x().value() 
                + u_cross.y().value() * u_cross.y().value() 
                + u_cross.z().value() * u_cross.z().value();
            let u_magnitude = u_mag_squared.sqrt();
            let u_final = Vector3D::new(
                Scalar::new(u_cross.x().value() / u_magnitude),
                Scalar::new(u_cross.y().value() / u_magnitude),
                Scalar::new(u_cross.z().value() / u_magnitude),
            );
            
            (u_final, v)
        } else {
            // デフォルトの基底
            let x_axis = Vector3D::new(Scalar::new(1.0), Scalar::new(0.0), Scalar::new(0.0));
            let y_axis = Vector3D::new(Scalar::new(0.0), Scalar::new(1.0), Scalar::new(0.0));
            (x_axis, y_axis)
        }
    }
}

impl GeometricPrimitive for Circle3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Circle
    }
    
    fn bounding_box(&self) -> BoundingBox {
        // 簡易的なバウンディングボックス（立方体）
        BoundingBox::new(
            geo_core::Point3D::from_f64(
                self.center.x() - self.radius,
                self.center.y() - self.radius,
                self.center.z() - self.radius,
            ),
            geo_core::Point3D::from_f64(
                self.center.x() + self.radius,
                self.center.y() + self.radius,
                self.center.z() + self.radius,
            ),
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
    fn test_circle_2d_area() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        
        assert!((circle.area() - std::f64::consts::PI).abs() < 1e-10);
        assert_eq!(circle.primitive_kind(), PrimitiveKind::Circle);
    }

    #[test]
    fn test_circle_3d_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 2.0;
        let normal = Vector3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(1.0));
        
        let circle = Circle3D::new(center, radius, normal).unwrap();
        
        assert_eq!(circle.radius(), 2.0);
        assert!((circle.area() - 4.0 * std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_circle_2d_contains_point() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        
        assert!(circle.contains_point(&Point2D::new(0.5, 0.5)));
        assert!(!circle.contains_point(&Point2D::new(2.0, 0.0)));
    }

    #[test]
    fn test_circle_3d_parametric_evaluation() {
        let circle = Circle3D::new_xy_plane(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();
        
        let point_0 = circle.evaluate(0.0).unwrap();
        let point_pi_2 = circle.evaluate(std::f64::consts::PI / 2.0).unwrap();
        
        // t=0で(1,0,0)、t=π/2で(0,1,0)になるはず
        assert!((point_0.x() - 1.0).abs() < 1e-10);
        assert!(point_0.y().abs() < 1e-10);
        assert!((point_pi_2.y() - 1.0).abs() < 1e-10);
        assert!(point_pi_2.x().abs() < 1e-10);
    }
}