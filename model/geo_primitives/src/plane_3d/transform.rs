//! Plane3D の相似変換
//!
//! 原点・法線・U 軸を変換し、V 軸は右手系として法線 × U 軸から再構築する。

use crate::Plane3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Plane3DConstructor, Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Plane3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let origin = transform.apply_point(self.origin);
        let normal = transform.apply_direction(self.normal);
        let u_axis = transform.apply_direction(self.u_axis);
        Self::from_origin_and_axes(origin, normal, u_axis)
            .ok_or_else(|| TransformError::InvalidGeometry("degenerate plane frame".to_string()))
    }
}
