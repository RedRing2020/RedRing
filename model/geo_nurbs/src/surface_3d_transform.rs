//! `NurbsSurface3D` の相似変換

use crate::surface_3d::NurbsSurface3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

/// 制御点を点として変換する。重みは相似変換で変わらないため保持する。
impl<T: Scalar> SimilarityTransformable3D<T> for NurbsSurface3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(self.map_control_points(|p| transform.apply_point(p)))
    }
}
