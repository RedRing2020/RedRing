//! `NurbsCurve3D` の相似変換

use crate::NurbsCurve3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};
use geo_core::{Point3D, SimilarityTransform3D};

/// 制御点を点として変換する。重みは相似変換で変わらないため保持する。
impl<T: Scalar> SimilarityTransformable3D<T> for NurbsCurve3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        Ok(self.map_control_points(|(x, y, z)| {
            let p = transform.apply_point(Point3D::new(x, y, z));
            (p.x(), p.y(), p.z())
        }))
    }
}
