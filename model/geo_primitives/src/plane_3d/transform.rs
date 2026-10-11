//! Plane3D の相似変換
//!
//! 原点・法線・U 軸を変換し、V 軸は右手系として法線 × U 軸から再構築する。

use crate::Plane3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Plane3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let origin = self.origin.transform_similarity(transform)?;
        let normal = self.normal.transform_similarity(transform)?;
        let u_axis = self.u_axis.transform_similarity(transform)?;
        Self::from_origin_and_axes(origin, normal.as_vector(), u_axis.as_vector())
            .ok_or_else(|| TransformError::InvalidGeometry("degenerate plane frame".to_string()))
    }
}
