//! TriangleMesh3D の相似変換

use crate::TriangleMesh3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

/// 頂点を点として変換し、法線（ある場合）はベクトルとして変換して正規化する。
impl<T: Scalar> SimilarityTransformable3D<T> for TriangleMesh3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let vertices = self
            .vertices()
            .iter()
            .map(|v| v.transform_similarity(transform))
            .collect::<Result<Vec<_>, _>>()?;
        let normals = match self.normals() {
            Some(normals) => Some(
                normals
                    .iter()
                    .map(|n| n.transform_similarity(transform).map(|v| v.normalize()))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            None => None,
        };
        Ok(Self::from_parts(vertices, self.indices().to_vec(), normals))
    }
}
