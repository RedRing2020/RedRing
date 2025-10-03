/// 三角形メッシュプリミティブの定義
/// 
/// 3D空間における三角形メッシュ要素

use geo_core::{Vector3D, Scalar, Vector};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::point::Point3D;
use crate::triangle::Triangle3D;

/// 頂点インデックス（三角形の頂点を示す）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexIndex(pub usize);

impl VertexIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
    
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

/// 三角形面（3つの頂点インデックスからなる）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Face {
    pub vertices: [VertexIndex; 3],
}

impl Face {
    pub fn new(v0: VertexIndex, v1: VertexIndex, v2: VertexIndex) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }
    
    pub fn vertex_indices(&self) -> [usize; 3] {
        [
            self.vertices[0].as_usize(),
            self.vertices[1].as_usize(),
            self.vertices[2].as_usize(),
        ]
    }
}

/// 3D三角形メッシュプリミティブ
#[derive(Debug, Clone)]
pub struct TriangleMesh3D {
    vertices: Vec<Point3D>,
    faces: Vec<Face>,
    normals: Option<Vec<Vector3D>>, // 頂点法線（オプション）
}

impl TriangleMesh3D {
    /// 頂点と面から新しいメッシュを作成
    pub fn new(vertices: Vec<Point3D>, faces: Vec<Face>) -> Option<Self> {
        // 全ての面の頂点インデックスが有効かチェック
        for face in &faces {
            for vertex_idx in &face.vertices {
                if vertex_idx.as_usize() >= vertices.len() {
                    return None; // 無効なインデックス
                }
            }
        }
        
        Some(Self {
            vertices,
            faces,
            normals: None,
        })
    }
    
    /// 頂点法線付きでメッシュを作成
    pub fn with_normals(vertices: Vec<Point3D>, faces: Vec<Face>, normals: Vec<Vector3D>) -> Option<Self> {
        if normals.len() != vertices.len() {
            return None; // 法線数が頂点数と一致しない
        }
        
        let mut mesh = Self::new(vertices, faces)?;
        mesh.normals = Some(normals);
        Some(mesh)
    }
    
    pub fn vertices(&self) -> &Vec<Point3D> {
        &self.vertices
    }
    
    pub fn faces(&self) -> &Vec<Face> {
        &self.faces
    }
    
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }
    
    pub fn has_normals(&self) -> bool {
        self.normals.is_some()
    }
    
    pub fn normals(&self) -> Option<&Vec<Vector3D>> {
        self.normals.as_ref()
    }
    
    /// 指定した面の三角形を取得
    pub fn get_triangle(&self, face_index: usize) -> Option<Triangle3D> {
        if face_index >= self.faces.len() {
            return None;
        }
        
        let face = &self.faces[face_index];
        let indices = face.vertex_indices();
        
        if indices[0] >= self.vertices.len() 
            || indices[1] >= self.vertices.len() 
            || indices[2] >= self.vertices.len() {
            return None;
        }
        
        Some(Triangle3D::new(
            self.vertices[indices[0]].clone(),
            self.vertices[indices[1]].clone(),
            self.vertices[indices[2]].clone(),
        ))
    }
    
    /// 面法線を計算
    pub fn compute_face_normals(&self) -> Vec<Option<Vector3D>> {
        self.faces.iter().map(|face| {
            let indices = face.vertex_indices();
            if let (Some(v0), Some(v1), Some(v2)) = (
                self.vertices.get(indices[0]),
                self.vertices.get(indices[1]),
                self.vertices.get(indices[2]),
            ) {
                // 辺ベクトルを計算
                let edge1 = Vector3D::new(
                    Scalar::new(v1.x() - v0.x()),
                    Scalar::new(v1.y() - v0.y()),
                    Scalar::new(v1.z() - v0.z()),
                );
                let edge2 = Vector3D::new(
                    Scalar::new(v2.x() - v0.x()),
                    Scalar::new(v2.y() - v0.y()),
                    Scalar::new(v2.z() - v0.z()),
                );
                
                // 法線を外積で計算し正規化
                let normal = edge1.cross(&edge2);
                let mag_squared = normal.x().value() * normal.x().value() 
                    + normal.y().value() * normal.y().value() 
                    + normal.z().value() * normal.z().value();
                
                if mag_squared > 1e-20 {
                    let magnitude = Scalar::new(mag_squared.sqrt());
                    Some(Vector3D::new(
                        normal.x() / magnitude,
                        normal.y() / magnitude,
                        normal.z() / magnitude,
                    ))
                } else {
                    None // 縮退三角形
                }
            } else {
                None
            }
        }).collect()
    }
    
    /// 頂点法線を計算（面積重み付き平均）
    pub fn compute_vertex_normals(&mut self) {
        let face_normals = self.compute_face_normals();
        let mut vertex_normals = vec![Vector3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0)); self.vertices.len()];
        let mut vertex_weights = vec![0.0f64; self.vertices.len()];
        
        // 各面の法線を対応する頂点に加算
        for (face_idx, face) in self.faces.iter().enumerate() {
            if let Some(normal) = &face_normals[face_idx] {
                if let Some(triangle) = self.get_triangle(face_idx) {
                    let area = triangle.area();
                    let indices = face.vertex_indices();
                    
                    for &idx in &indices {
                        vertex_normals[idx] = Vector3D::new(
                            vertex_normals[idx].x() + normal.x() * Scalar::new(area),
                            vertex_normals[idx].y() + normal.y() * Scalar::new(area),
                            vertex_normals[idx].z() + normal.z() * Scalar::new(area),
                        );
                        vertex_weights[idx] += area;
                    }
                }
            }
        }
        
        // 正規化
        for i in 0..vertex_normals.len() {
            if vertex_weights[i] > 1e-10 {
                let weight = Scalar::new(vertex_weights[i]);
                vertex_normals[i] = Vector3D::new(
                    vertex_normals[i].x() / weight,
                    vertex_normals[i].y() / weight,
                    vertex_normals[i].z() / weight,
                );
                
                let mag_squared = vertex_normals[i].x().value() * vertex_normals[i].x().value() 
                    + vertex_normals[i].y().value() * vertex_normals[i].y().value() 
                    + vertex_normals[i].z().value() * vertex_normals[i].z().value();
                if mag_squared > 1e-20 {
                    let magnitude = Scalar::new(mag_squared.sqrt());
                    vertex_normals[i] = Vector3D::new(
                        vertex_normals[i].x() / magnitude,
                        vertex_normals[i].y() / magnitude,
                        vertex_normals[i].z() / magnitude,
                    );
                }
            }
        }
        
        self.normals = Some(vertex_normals);
    }
    
    /// メッシュの総表面積を計算
    pub fn surface_area(&self) -> f64 {
        self.faces.iter().enumerate().map(|(face_idx, _face)| {
            if let Some(triangle) = self.get_triangle(face_idx) {
                triangle.area()
            } else {
                0.0
            }
        }).sum()
    }
    
    /// メッシュの重心を計算
    pub fn centroid(&self) -> Point3D {
        if self.vertices.is_empty() {
            return Point3D::new(0.0, 0.0, 0.0);
        }
        
        let sum_x: f64 = self.vertices.iter().map(|v| v.x()).sum();
        let sum_y: f64 = self.vertices.iter().map(|v| v.y()).sum();
        let sum_z: f64 = self.vertices.iter().map(|v| v.z()).sum();
        let count = self.vertices.len() as f64;
        
        Point3D::new(sum_x / count, sum_y / count, sum_z / count)
    }
    
    /// エッジの隣接情報を構築（簡易版）
    pub fn build_edge_adjacency(&self) -> Vec<Vec<usize>> {
        let mut adjacency = vec![Vec::new(); self.vertices.len()];
        
        for face in &self.faces {
            let indices = face.vertex_indices();
            
            // 各辺について隣接関係を追加
            for i in 0..3 {
                let v1 = indices[i];
                let v2 = indices[(i + 1) % 3];
                
                if !adjacency[v1].contains(&v2) {
                    adjacency[v1].push(v2);
                }
                if !adjacency[v2].contains(&v1) {
                    adjacency[v2].push(v1);
                }
            }
        }
        
        adjacency
    }
}

impl GeometricPrimitive for TriangleMesh3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::TriangleMesh
    }
    
    fn bounding_box(&self) -> BoundingBox {
        if self.vertices.is_empty() {
            return BoundingBox::new(
                geo_core::Point3D::from_f64(0.0, 0.0, 0.0),
                geo_core::Point3D::from_f64(0.0, 0.0, 0.0),
            );
        }
        
        let min_x = self.vertices.iter().map(|v| v.x()).fold(f64::INFINITY, f64::min);
        let min_y = self.vertices.iter().map(|v| v.y()).fold(f64::INFINITY, f64::min);
        let min_z = self.vertices.iter().map(|v| v.z()).fold(f64::INFINITY, f64::min);
        let max_x = self.vertices.iter().map(|v| v.x()).fold(f64::NEG_INFINITY, f64::max);
        let max_y = self.vertices.iter().map(|v| v.y()).fold(f64::NEG_INFINITY, f64::max);
        let max_z = self.vertices.iter().map(|v| v.z()).fold(f64::NEG_INFINITY, f64::max);
        
        BoundingBox::new(
            geo_core::Point3D::from_f64(min_x, min_y, min_z),
            geo_core::Point3D::from_f64(max_x, max_y, max_z),
        )
    }
    
    fn measure(&self) -> Option<f64> {
        Some(self.surface_area())
    }
}

/// 基本的なメッシュ生成ユーティリティ
impl TriangleMesh3D {
    /// 立方体メッシュを生成
    pub fn cube(size: f64) -> Self {
        let half = size * 0.5;
        
        let vertices = vec![
            // 前面
            Point3D::new(-half, -half, half),  // 0
            Point3D::new(half, -half, half),   // 1
            Point3D::new(half, half, half),    // 2
            Point3D::new(-half, half, half),   // 3
            // 背面
            Point3D::new(-half, -half, -half), // 4
            Point3D::new(half, -half, -half),  // 5
            Point3D::new(half, half, -half),   // 6
            Point3D::new(-half, half, -half),  // 7
        ];
        
        let faces = vec![
            // 前面
            Face::new(VertexIndex(0), VertexIndex(1), VertexIndex(2)),
            Face::new(VertexIndex(0), VertexIndex(2), VertexIndex(3)),
            // 背面
            Face::new(VertexIndex(5), VertexIndex(4), VertexIndex(7)),
            Face::new(VertexIndex(5), VertexIndex(7), VertexIndex(6)),
            // 左面
            Face::new(VertexIndex(4), VertexIndex(0), VertexIndex(3)),
            Face::new(VertexIndex(4), VertexIndex(3), VertexIndex(7)),
            // 右面
            Face::new(VertexIndex(1), VertexIndex(5), VertexIndex(6)),
            Face::new(VertexIndex(1), VertexIndex(6), VertexIndex(2)),
            // 上面
            Face::new(VertexIndex(3), VertexIndex(2), VertexIndex(6)),
            Face::new(VertexIndex(3), VertexIndex(6), VertexIndex(7)),
            // 下面
            Face::new(VertexIndex(4), VertexIndex(5), VertexIndex(1)),
            Face::new(VertexIndex(4), VertexIndex(1), VertexIndex(0)),
        ];
        
        Self::new(vertices, faces).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_mesh_creation() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.5, 1.0, 0.0),
            Point3D::new(0.5, 0.5, 1.0),
        ];
        
        let faces = vec![
            Face::new(VertexIndex(0), VertexIndex(1), VertexIndex(2)),
            Face::new(VertexIndex(0), VertexIndex(2), VertexIndex(3)),
        ];
        
        let mesh = TriangleMesh3D::new(vertices, faces).unwrap();
        
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.face_count(), 2);
        assert_eq!(mesh.primitive_kind(), PrimitiveKind::TriangleMesh);
    }

    #[test]
    fn test_cube_mesh() {
        let cube = TriangleMesh3D::cube(2.0);
        
        assert_eq!(cube.vertex_count(), 8);
        assert_eq!(cube.face_count(), 12); // 立方体は12個の三角形
        
        let area = cube.surface_area();
        assert!((area - 24.0).abs() < 1e-10); // 2x2の6面 = 24
    }

    #[test]
    fn test_face_normals() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        
        let faces = vec![
            Face::new(VertexIndex(0), VertexIndex(1), VertexIndex(2)),
        ];
        
        let mesh = TriangleMesh3D::new(vertices, faces).unwrap();
        let normals = mesh.compute_face_normals();
        
        assert_eq!(normals.len(), 1);
        if let Some(normal) = &normals[0] {
            // Z軸正方向の法線になるはず
            assert!((normal.z().value() - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_vertex_normals_computation() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        
        let faces = vec![
            Face::new(VertexIndex(0), VertexIndex(1), VertexIndex(2)),
        ];
        
        let mut mesh = TriangleMesh3D::new(vertices, faces).unwrap();
        mesh.compute_vertex_normals();
        
        assert!(mesh.has_normals());
        let normals = mesh.normals().unwrap();
        assert_eq!(normals.len(), 3);
        
        // 全ての頂点法線がZ軸方向を向くはず
        for normal in normals {
            assert!((normal.z().value() - 1.0).abs() < 1e-10);
        }
    }
}