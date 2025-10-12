/// 多角形プリミティブ�E定義
///
/// 2D/3D空間における多角形要素

// geo_core参照を削除 - ローカルVector3Dを使用
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox, geometry_utils::*};
use crate::{Point2D, Point3D};

/// 2D多角形プリミティチE#[derive(Debug, Clone)]
pub struct Polygon2D {
    vertices: Vec<Point2D>,
}

impl Polygon2D {
    /// 新しい2D多角形を作�E
    pub fn new(vertices: Vec<Point2D>) -> Option<Self> {
        if vertices.len() < 3 {
            return None; // 多角形は最佁Eつの頂点が忁E��E        }
        Some(Self { vertices })
    }

    /// 頂点を取征E    pub fn vertices(&self) -> &[Point2D] {
        &self.vertices
    }

    /// 頂点の可変参照を取征E    pub fn vertices_mut(&mut self) -> &mut Vec<Point2D> {
        &mut self.vertices
    }

    /// 頂点数を取征E    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 重忁E��計箁E    pub fn centroid(&self) -> Point2D {
        point2d_centroid(&self.vertices).unwrap()
    }

    /// 面積を計算！Ehoelace公式を使用�E�E    pub fn area(&self) -> f64 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }

        let mut area: f64 = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            let (xi, yi) = point2d_to_f64(&self.vertices[i]);
            let (xj, yj) = point2d_to_f64(&self.vertices[j]);
            area += xi * yj;
            area -= xj * yi;
        }

        (area / 2.0).abs()
    }

    /// 周囲長を計箁E    pub fn perimeter(&self) -> f64 {
        let n = self.vertices.len();
        if n < 2 {
            return 0.0;
        }

        let mut perimeter = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            let (xi, yi) = point2d_to_f64(&self.vertices[i]);
            let (xj, yj) = point2d_to_f64(&self.vertices[j]);
            let dx = xj - xi;
            let dy = yj - yi;
            perimeter += (dx * dx + dy * dy).sqrt();
        }

        perimeter
    }

    /// 点が多角形冁E��にあるかを判定！Eay casting algorithm�E�E    pub fn contains_point(&self, point: &Point2D) -> bool {
        let n = self.vertices.len();
        if n < 3 {
            return false;
        }

        let (px, py) = point2d_to_f64(point);
        let mut inside = false;

        for i in 0..n {
            let j = (i + 1) % n;
            let (xi, yi) = point2d_to_f64(&self.vertices[i]);
            let (xj, yj) = point2d_to_f64(&self.vertices[j]);

            if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
        }

        inside
    }
}

impl GeometricPrimitive for Polygon2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Polygon
    }

    fn bounding_box(&self) -> BoundingBox {
        let bbox = point2d_bounding_box(&self.vertices).unwrap();
        BoundingBox::from_2d(
            point2d_from_f64(bbox.0, bbox.1),
            point2d_from_f64(bbox.2, bbox.3),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.area())
    }
}

/// 3D多角形プリミティブ（平面多角形�E�E#[derive(Debug, Clone)]
pub struct Polygon3D {
    vertices: Vec<Point3D>,
}

impl Polygon3D {
    /// 新しい3D多角形を作�E
    pub fn new(vertices: Vec<Point3D>) -> Option<Self> {
        if vertices.len() < 3 {
            return None; // 多角形は最佁Eつの頂点が忁E��E        }
        Some(Self { vertices })
    }

    /// 頂点を取征E    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }

    /// 頂点の可変参照を取征E    pub fn vertices_mut(&mut self) -> &mut Vec<Point3D> {
        &mut self.vertices
    }

    /// 頂点数を取征E    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 重忁E��計箁E    pub fn centroid(&self) -> Point3D {
        point3d_centroid(&self.vertices).unwrap()
    }

    /// 法線�Eクトルを計算（最初�E3点から�E�E    pub fn normal(&self) -> Option<Vector3D> {
        if self.vertices.len() < 3 {
            return None;
        }

        let (x0, y0, z0) = point3d_to_f64(&self.vertices[0]);
        let (x1, y1, z1) = point3d_to_f64(&self.vertices[1]);
        let (x2, y2, z2) = point3d_to_f64(&self.vertices[2]);

        let v1_x = x1 - x0;
        let v1_y = y1 - y0;
        let v1_z = z1 - z0;

        let v2_x = x2 - x0;
        let v2_y = y2 - y0;
        let v2_z = z2 - z0;

        // 外穁E        let cross_x = v1_y * v2_z - v1_z * v2_y;
        let cross_y = v1_z * v2_x - v1_x * v2_z;
        let cross_z = v1_x * v2_y - v1_y * v2_x;

        // 長さチェチE��
        let length = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
        if length < 1e-10 {
            return None;
        }

        Some(Vector3D::new(
            Scalar::new(cross_x),
            Scalar::new(cross_y),
            Scalar::new(cross_z),
        ))
    }

    /// 面積を計算（三角形刁E��を使用�E�E    pub fn area(&self) -> f64 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }

        let mut total_area = 0.0;
        let (x0, y0, z0) = point3d_to_f64(&self.vertices[0]);

        for i in 1..n - 1 {
            let (x1, y1, z1) = point3d_to_f64(&self.vertices[i]);
            let (x2, y2, z2) = point3d_to_f64(&self.vertices[i + 1]);

            let v1_x = x1 - x0;
            let v1_y = y1 - y0;
            let v1_z = z1 - z0;

            let v2_x = x2 - x0;
            let v2_y = y2 - y0;
            let v2_z = z2 - z0;

            // 外穁E            let cross_x = v1_y * v2_z - v1_z * v2_y;
            let cross_y = v1_z * v2_x - v1_x * v2_z;
            let cross_z = v1_x * v2_y - v1_y * v2_x;

            let triangle_area = 0.5 * (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
            total_area += triangle_area;
        }

        total_area
    }
}

impl GeometricPrimitive for Polygon3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Polygon
    }

    fn bounding_box(&self) -> BoundingBox {
        let bbox = point3d_bounding_box(&self.vertices).unwrap();
        BoundingBox::new(
            point3d_from_f64(bbox.0, bbox.1, bbox.2),
            point3d_from_f64(bbox.3, bbox.4, bbox.5),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.area())
    }
}


