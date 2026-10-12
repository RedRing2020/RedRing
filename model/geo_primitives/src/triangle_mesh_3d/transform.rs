//! TriangleMesh3D の相似変換

use crate::SimilarityTransform3D;
use crate::TriangleMesh3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

/// 頂点を点として変換し、法線（ある場合）はベクトルとして変換して正規化する。
impl<T: Scalar> SimilarityTransformable3D<T> for TriangleMesh3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let vertices = self
            .vertices()
            .iter()
            .map(|v| transform.apply_point(*v))
            .collect::<Vec<_>>();
        let normals = self.normals().map(|normals| {
            normals
                .iter()
                .map(|n| transform.apply_vector(*n).normalize())
                .collect::<Vec<_>>()
        });
        Ok(Self::from_parts(vertices, self.indices().to_vec(), normals))
    }
}
