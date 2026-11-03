//! TriangleMesh3D 実装
//!
//! 共有頂点と三角形の頂点番号で表す三角形メッシュ

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Point3D, Triangle3D, Vector3D};
use geo_contracts::{
    PrimitiveKind, PrimitiveMetadata, Scalar, TolerantEq, Triangle3DConstructor,
    TriangleMesh3DConstructor, TriangleMesh3DProperties, TriangleMesh3DTriangleAccess,
    TriangleMeshError,
};

/// 3次元三角形メッシュ
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleMesh3D<T: Scalar> {
    /// 共有頂点配列
    vertices: Vec<Point3D<T>>,

    /// 三角形インデックス配列（各要素は3つの頂点インデックス）
    indices: Vec<[usize; 3]>,

    /// 頂点法線（オプション）
    normals: Option<Vec<Vector3D<T>>>,
}

impl<T: Scalar> PrimitiveMetadata for TriangleMesh3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Mesh
    }
}

impl<T: Scalar> TriangleMesh3D<T> {
    /// 三角形の面積の合計
    fn total_area(&self) -> T {
        self.indices
            .iter()
            .map(|&[a, b, c]| {
                let (v0, v1, v2) = (self.vertices[a], self.vertices[b], self.vertices[c]);
                Vector3D::from_points(v0, v1)
                    .cross(Vector3D::from_points(v0, v2))
                    .length()
                    / (T::ONE + T::ONE)
            })
            .fold(T::ZERO, |sum, area| sum + area)
    }
}

impl<T: Scalar> TolerantEq<T> for TriangleMesh3D<T> {
    /// 頂点数・三角形数が等しく、面積の合計の差が許容誤差以内
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        self.vertex_count() == other.vertex_count()
            && self.triangle_count() == other.triangle_count()
            && (self.total_area() - other.total_area()).abs() <= tolerance
    }
}

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

impl<T: Scalar> TriangleMesh3DConstructor<T> for TriangleMesh3D<T> {
    fn new(vertices: Vec<Point3D<T>>, indices: Vec<[usize; 3]>) -> Result<Self, TriangleMeshError> {
        let vertex_count = vertices.len();
        for (triangle, triangle_indices) in indices.iter().enumerate() {
            for (corner, &index) in triangle_indices.iter().enumerate() {
                if index >= vertex_count {
                    return Err(TriangleMeshError::InvalidVertexIndex {
                        triangle,
                        corner,
                        index,
                        vertex_count,
                    });
                }
            }
        }

        Ok(Self {
            vertices,
            indices,
            normals: None,
        })
    }

    fn with_normals(
        vertices: Vec<Point3D<T>>,
        indices: Vec<[usize; 3]>,
        normals: Vec<Vector3D<T>>,
    ) -> Result<Self, TriangleMeshError> {
        if normals.len() != vertices.len() {
            return Err(TriangleMeshError::NormalCountMismatch {
                normal_count: normals.len(),
                vertex_count: vertices.len(),
            });
        }
        let mesh = Self::new(vertices, indices)?;
        Ok(Self {
            normals: Some(normals),
            ..mesh
        })
    }
}

impl<T: Scalar> TriangleMesh3DProperties<T> for TriangleMesh3D<T> {
    fn vertices(&self) -> &[Point3D<T>] {
        &self.vertices
    }

    fn indices(&self) -> &[[usize; 3]] {
        &self.indices
    }

    fn normals(&self) -> Option<&[Vector3D<T>]> {
        self.normals.as_deref()
    }

    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    fn triangle_count(&self) -> usize {
        self.indices.len()
    }

    fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    fn is_valid(&self) -> bool {
        let vertex_count = self.vertices.len();
        let indices_valid = self
            .indices
            .iter()
            .all(|triangle| triangle.iter().all(|&index| index < vertex_count));
        let normals_valid = self
            .normals
            .as_ref()
            .is_none_or(|normals| normals.len() == vertex_count);
        indices_valid && normals_valid
    }
}

impl<T: Scalar> TriangleMesh3DTriangleAccess<T> for TriangleMesh3D<T> {
    type Triangle = Triangle3D<T>;

    fn triangle(&self, index: usize) -> Option<Triangle3D<T>> {
        let [a, b, c] = *self.indices.get(index)?;
        Triangle3D::new(
            *self.vertices.get(a)?,
            *self.vertices.get(b)?,
            *self.vertices.get(c)?,
        )
    }
}
