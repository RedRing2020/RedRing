//! TriangleMesh 形状の契約定義
//!
//! 共有頂点と三角形の頂点番号で表す三角形メッシュの公開 trait。

use crate::{Point3D, Scalar, Vector3D};
use std::fmt;

/// 三角形メッシュを作成できない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleMeshError {
    /// 三角形の頂点番号が頂点の範囲外
    InvalidVertexIndex {
        /// 三角形の番号
        triangle: usize,
        /// 三角形内の頂点の位置（0〜2）
        corner: usize,
        /// 範囲外の頂点番号
        index: usize,
        /// 頂点の数
        vertex_count: usize,
    },
    /// 頂点法線の数が頂点の数と異なる
    NormalCountMismatch {
        /// 頂点法線の数
        normal_count: usize,
        /// 頂点の数
        vertex_count: usize,
    },
}

impl fmt::Display for TriangleMeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVertexIndex {
                triangle,
                corner,
                index,
                vertex_count,
            } => write!(
                f,
                "Triangle {triangle} vertex {corner} has invalid index {index} (vertex count: {vertex_count})"
            ),
            Self::NormalCountMismatch {
                normal_count,
                vertex_count,
            } => write!(
                f,
                "Normal count {normal_count} does not match vertex count {vertex_count}"
            ),
        }
    }
}

impl std::error::Error for TriangleMeshError {}

pub trait TriangleMesh3DConstructor<T: Scalar>: Sized {
    /// 共有頂点と三角形の頂点番号から作成する（頂点番号が範囲外の場合はエラー）
    fn new(vertices: Vec<Point3D<T>>, indices: Vec<[usize; 3]>) -> Result<Self, TriangleMeshError>;

    /// 共有頂点・三角形の頂点番号・頂点法線から作成する（頂点番号が範囲外の場合と、頂点法線の数が頂点の数と
    /// 異なる場合はエラー）
    fn with_normals(
        vertices: Vec<Point3D<T>>,
        indices: Vec<[usize; 3]>,
        normals: Vec<Vector3D<T>>,
    ) -> Result<Self, TriangleMeshError>;
}

pub trait TriangleMesh3DProperties<T: Scalar> {
    /// 共有頂点
    fn vertices(&self) -> &[Point3D<T>];
    /// 三角形ごとの頂点番号
    fn indices(&self) -> &[[usize; 3]];
    /// 頂点法線（頂点ごと。持たない場合は `None`）
    fn normals(&self) -> Option<&[Vector3D<T>]>;
    fn vertex_count(&self) -> usize;
    fn triangle_count(&self) -> usize;
    /// 三角形を持たないか
    fn is_empty(&self) -> bool;
    /// 頂点番号が範囲内で、頂点法線（ある場合）の数が頂点数と一致するか
    fn is_valid(&self) -> bool;
}

/// 三角形メッシュの三角形の取り出し
pub trait TriangleMesh3DTriangleAccess<T: Scalar> {
    /// 三角形の型
    type Triangle;

    /// 指定の番号の三角形（番号が範囲外の場合と、三角形が退化している場合は `None`）
    fn triangle(&self, index: usize) -> Option<Self::Triangle>;
}
