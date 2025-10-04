/// メッシュプリミティブの定義
///
/// 3D三角形メッシュ要素

use geo_core::Vector3D;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox, geometry_utils::*};
use geo_core::Point3D;

/// 3D三角形メッシュプリミティブ
#[derive(Debug, Clone)]
pub struct TriangleMesh {
    /// 頂点配列
    vertices: Vec<Point3D>,
    /// 三角形インデックス（各要素は3つの頂点インデックス）
    triangles: Vec<[usize; 3]>,
}

impl TriangleMesh {
    /// 新しい三角形メッシュを作成
    pub fn new(vertices: Vec<Point3D>, triangles: Vec<[usize; 3]>) -> Option<Self> {
        // インデックスの有効性をチェック
        for triangle in &triangles {
            for &index in triangle {
                if index >= vertices.len() {
                    return None; // 無効なインデックス
                }
            }
        }
        
        Some(Self { vertices, triangles })
    }

    /// 頂点を取得
    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }

    /// 頂点の可変参照を取得
    pub fn vertices_mut(&mut self) -> &mut Vec<Point3D> {
        &mut self.vertices
    }

    /// 三角形インデックスを取得
    pub fn triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }

    /// 三角形インデックスの可変参照を取得
    pub fn triangles_mut(&mut self) -> &mut Vec<[usize; 3]> {
        &mut self.triangles
    }

    /// 頂点数を取得
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 三角形数を取得
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// 指定された三角形の頂点を取得
    pub fn triangle_vertices(&self, triangle_index: usize) -> Option<[&Point3D; 3]> {
        if triangle_index >= self.triangles.len() {
            return None;
        }
        
        let indices = &self.triangles[triangle_index];
        Some([
            &self.vertices[indices[0]],
            &self.vertices[indices[1]],
            &self.vertices[indices[2]],
        ])
    }

    /// 指定された三角形の法線ベクトルを計算
    pub fn triangle_normal(&self, triangle_index: usize) -> Option<Vector3D> {
        let vertices = self.triangle_vertices(triangle_index)?;
        
        let (x0, y0, z0) = point3d_to_f64(vertices[0]);
        let (x1, y1, z1) = point3d_to_f64(vertices[1]);
        let (x2, y2, z2) = point3d_to_f64(vertices[2]);
        
        let v1_x = x1 - x0;
        let v1_y = y1 - y0;
        let v1_z = z1 - z0;
        
        let v2_x = x2 - x0;
        let v2_y = y2 - y0;
        let v2_z = z2 - z0;
        
        // 外積
        let cross_x = v1_y * v2_z - v1_z * v2_y;
        let cross_y = v1_z * v2_x - v1_x * v2_z;
        let cross_z = v1_x * v2_y - v1_y * v2_x;
        
        // 長さチェック
        let length = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
        if length < 1e-10 {
            return None; // 退化した三角形
        }
        
        Some(Vector3D::from_f64(cross_x, cross_y, cross_z))
    }

    /// 指定された三角形の面積を計算
    pub fn triangle_area(&self, triangle_index: usize) -> Option<f64> {
        let vertices = self.triangle_vertices(triangle_index)?;
        
        let (x0, y0, z0) = point3d_to_f64(vertices[0]);
        let (x1, y1, z1) = point3d_to_f64(vertices[1]);
        let (x2, y2, z2) = point3d_to_f64(vertices[2]);
        
        let v1_x = x1 - x0;
        let v1_y = y1 - y0;
        let v1_z = z1 - z0;
        
        let v2_x = x2 - x0;
        let v2_y = y2 - y0;
        let v2_z = z2 - z0;
        
        // 外積の長さ
        let cross_x = v1_y * v2_z - v1_z * v2_y;
        let cross_y = v1_z * v2_x - v1_x * v2_z;
        let cross_z = v1_x * v2_y - v1_y * v2_x;
        
        let cross_length = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
        Some(0.5 * cross_length)
    }

    /// メッシュ全体の表面積を計算
    pub fn surface_area(&self) -> f64 {
        let mut total_area = 0.0;
        for i in 0..self.triangles.len() {
            if let Some(area) = self.triangle_area(i) {
                total_area += area;
            }
        }
        total_area
    }

    /// メッシュの重心を計算
    pub fn centroid(&self) -> Point3D {
        if self.vertices.is_empty() {
            return point3d_from_f64(0.0, 0.0, 0.0);
        }
        
        point3d_centroid(&self.vertices).unwrap()
    }

    /// 簡単な立方体メッシュを作成
    pub fn create_cube(size: f64) -> Self {
        let half = size / 2.0;
        let vertices = vec![
            point3d_from_f64(-half, -half, half),  // 0
            point3d_from_f64(half, -half, half),   // 1
            point3d_from_f64(half, half, half),    // 2
            point3d_from_f64(-half, half, half),   // 3
            point3d_from_f64(-half, -half, -half), // 4
            point3d_from_f64(half, -half, -half),  // 5
            point3d_from_f64(half, half, -half),   // 6
            point3d_from_f64(-half, half, -half),  // 7
        ];

        let triangles = vec![
            // Front face
            [0, 1, 2], [0, 2, 3],
            // Back face
            [4, 7, 6], [4, 6, 5],
            // Left face
            [0, 3, 7], [0, 7, 4],
            // Right face
            [1, 5, 6], [1, 6, 2],
            // Top face
            [3, 2, 6], [3, 6, 7],
            // Bottom face
            [0, 4, 5], [0, 5, 1],
        ];

        Self::new(vertices, triangles).unwrap()
    }
}

impl GeometricPrimitive for TriangleMesh {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::TriangleMesh
    }

    fn bounding_box(&self) -> BoundingBox {
        if self.vertices.is_empty() {
            return BoundingBox::new(
                point3d_from_f64(0.0, 0.0, 0.0),
                point3d_from_f64(0.0, 0.0, 0.0),
            );
        }
        
        let bbox = point3d_bounding_box(&self.vertices).unwrap();
        BoundingBox::new(
            point3d_from_f64(bbox.0, bbox.1, bbox.2),
            point3d_from_f64(bbox.3, bbox.4, bbox.5),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.surface_area())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_mesh_creation() {
        let vertices = vec![
            point3d_from_f64(0.0, 0.0, 0.0),
            point3d_from_f64(1.0, 0.0, 0.0),
            point3d_from_f64(0.0, 1.0, 0.0),
        ];
        let triangles = vec![[0, 1, 2]];
        
        let mesh = TriangleMesh::new(vertices, triangles).unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn test_triangle_mesh_invalid_indices() {
        let vertices = vec![
            point3d_from_f64(0.0, 0.0, 0.0),
            point3d_from_f64(1.0, 0.0, 0.0),
        ];
        let triangles = vec![[0, 1, 2]]; // インデックス2は無効
        
        let mesh = TriangleMesh::new(vertices, triangles);
        assert!(mesh.is_none());
    }

    #[test]
    fn test_triangle_area() {
        let vertices = vec![
            point3d_from_f64(0.0, 0.0, 0.0),
            point3d_from_f64(1.0, 0.0, 0.0),
            point3d_from_f64(0.0, 1.0, 0.0),
        ];
        let triangles = vec![[0, 1, 2]];
        
        let mesh = TriangleMesh::new(vertices, triangles).unwrap();
        let area = mesh.triangle_area(0).unwrap();
        assert!((area - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_cube_mesh() {
        let cube = TriangleMesh::create_cube(2.0);
        assert_eq!(cube.vertex_count(), 8);
        assert_eq!(cube.triangle_count(), 12);
        
        let surface_area = cube.surface_area();
        assert!((surface_area - 24.0).abs() < 1e-10); // 2x2の6面 = 24
    }

    #[test]
    fn test_mesh_centroid() {
        let cube = TriangleMesh::create_cube(2.0);
        let centroid = cube.centroid();
        let (x, y, z) = point3d_to_f64(&centroid);
        
        assert!(x.abs() < 1e-10);
        assert!(y.abs() < 1e-10);
        assert!(z.abs() < 1e-10);
    }
}