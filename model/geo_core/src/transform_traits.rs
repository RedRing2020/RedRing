//! Analysis-based transform traits for geo_core.

use crate::transform_error::TransformError;
use analysis::abstract_types::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

/// 3D point transform with Matrix4x4.
pub trait AnalysisTransform3D<T: Scalar> {
    type Matrix4x4;
    type Angle;
    type Output;

    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;

    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn rotate_analysis(
        &self,
        center: &Vector3<T>,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn scale_analysis(
        &self,
        center: &Vector3<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn uniform_scale_analysis(
        &self,
        center: &Vector3<T>,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Vector3<T>, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Vector3<T>, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;
}

/// 2D point transform with Matrix3x3.
pub trait AnalysisTransform2D<T: Scalar> {
    type Matrix3x3;
    type Angle;
    type Output;

    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output;

    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn rotate_analysis_2d(
        &self,
        center: &Vector2<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn scale_analysis_2d(
        &self,
        center: &Vector2<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;

    fn uniform_scale_analysis_2d(
        &self,
        center: &Vector2<T>,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError>
    where
        Self: Sized;
}

/// Marker trait to indicate analysis transform support.
pub trait AnalysisTransformSupport {
    const HAS_ANALYSIS_INTEGRATION: bool = true;
    const PERFORMANCE_OPTIMIZED: bool = true;
}
