//! TriangleMesh3D Core 実装
//!
//! Foundation統一システムに基づくTriangleMesh3Dの必須機能のみ

use crate::{Point3D, Triangle3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元三角形メッシュ（Core実装）
///
/// Core機能のみ：
/// - 頂点配列による共有頂点管理
/// - 三角形インデックス配列
/// - 基本的なアクセサメソッド
/// - 法線計算（オプション）
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleMesh3D<T: Scalar> {
    /// 共有頂点配列
    vertices: Vec<Point3D<T>>,

    /// 三角形インデックス配列（各要素は3つの頂点インデックス）
    indices: Vec<[usize; 3]>,

    /// 頂点法線（オプション）
    normals: Option<Vec<Vector3D<T>>>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> TriangleMesh3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しいメッシュを作成
    pub fn new(vertices: Vec<Point3D<T>>, indices: Vec<[usize; 3]>) -> Result<Self, String> {
        // インデックスの有効性を検証
        let vertex_count = vertices.len();
        for (tri_idx, triangle_indices) in indices.iter().enumerate() {
            for (i, &vertex_idx) in triangle_indices.iter().enumerate() {
                if vertex_idx >= vertex_count {
                    return Err(format!(
                        "Triangle {} vertex {} has invalid index {} (vertex count: {})",
                        tri_idx, i, vertex_idx, vertex_count
                    ));
                }
            }
        }

        Ok(Self {
            vertices,
            indices,
            normals: None,
        })
    }

    /// 空のメッシュを作成
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: None,
        }
    }

    // ========================================================================
    // Core Access Methods
    // ========================================================================

    /// 頂点数を取得
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 三角形数を取得
    pub fn triangle_count(&self) -> usize {
        self.indices.len()
    }

    /// 頂点配列への参照を取得
    pub fn vertices(&self) -> &[Point3D<T>] {
        &self.vertices
    }

    /// インデックス配列への参照を取得
    pub fn indices(&self) -> &[[usize; 3]] {
        &self.indices
    }

    /// 法線配列への参照を取得（存在する場合）
    pub fn normals(&self) -> Option<&[Vector3D<T>]> {
        self.normals.as_deref()
    }

    /// 指定したインデックスの頂点を取得
    pub fn vertex(&self, index: usize) -> Option<Point3D<T>> {
        self.vertices.get(index).copied()
    }

    /// 指定したインデックスの三角形インデックスを取得
    pub fn triangle_indices(&self, index: usize) -> Option<[usize; 3]> {
        self.indices.get(index).copied()
    }

    /// 指定したインデックスの三角形を取得
    pub fn triangle(&self, index: usize) -> Option<Triangle3D<T>> {
        let indices = self.triangle_indices(index)?;
        let a = self.vertex(indices[0])?;
        let b = self.vertex(indices[1])?;
        let c = self.vertex(indices[2])?;

        Triangle3D::new(a, b, c)
    }

    // ========================================================================
    // Core Validation Methods
    // ========================================================================

    /// メッシュが有効かどうかを判定
    pub fn is_valid(&self) -> bool {
        // 空のメッシュは有効
        if self.indices.is_empty() {
            return true;
        }

        // 頂点が存在しないのに三角形がある場合は無効
        if self.vertices.is_empty() {
            return false;
        }

        // すべてのインデックスが有効範囲内かチェック
        let vertex_count = self.vertices.len();
        for indices in &self.indices {
            for &vertex_idx in indices {
                if vertex_idx >= vertex_count {
                    return false;
                }
            }
        }

        // 法線が存在する場合、頂点数と一致するかチェック
        if let Some(ref normals) = self.normals {
            if normals.len() != self.vertices.len() {
                return false;
            }
        }

        true
    }

    /// 退化した三角形の数を取得
    pub fn degenerate_triangle_count(&self) -> usize {
        self.indices
            .iter()
            .filter(|&&indices| {
                // 同じ頂点を参照している場合は退化
                indices[0] == indices[1] || indices[1] == indices[2] || indices[2] == indices[0]
            })
            .count()
    }

    // ========================================================================
    // Core Utility Methods
    // ========================================================================

    /// メッシュが空かどうかを判定
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// 境界ボックスを計算
    pub fn bounding_box(&self) -> Option<(Point3D<T>, Point3D<T>)> {
        if self.vertices.is_empty() {
            return None;
        }

        let first = self.vertices[0];
        let mut min_point = first;
        let mut max_point = first;

        for &vertex in &self.vertices[1..] {
            // X座標
            if vertex.x() < min_point.x() {
                min_point = Point3D::new(vertex.x(), min_point.y(), min_point.z());
            }
            if vertex.x() > max_point.x() {
                max_point = Point3D::new(vertex.x(), max_point.y(), max_point.z());
            }

            // Y座標
            if vertex.y() < min_point.y() {
                min_point = Point3D::new(min_point.x(), vertex.y(), min_point.z());
            }
            if vertex.y() > max_point.y() {
                max_point = Point3D::new(max_point.x(), vertex.y(), max_point.z());
            }

            // Z座標
            if vertex.z() < min_point.z() {
                min_point = Point3D::new(min_point.x(), min_point.y(), vertex.z());
            }
            if vertex.z() > max_point.z() {
                max_point = Point3D::new(max_point.x(), max_point.y(), vertex.z());
            }
        }

        Some((min_point, max_point))
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for TriangleMesh3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TriangleMesh3D({} vertices, {} triangles)",
            self.vertex_count(),
            self.triangle_count()
        )
    }
}
