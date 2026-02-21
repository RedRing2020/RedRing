//! Rect3D Analysis Matrix/Vector統合変換実装

use crate::{Point3D, Rect3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

pub mod analysis_transform {
    use super::*;

    pub fn transform_rect_3d<T: Scalar>(
        rect: &Rect3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<Rect3D<T>, TransformError> {
        let corners = rect.corners();
        let p0v = Vector3::new(corners[0].x(), corners[0].y(), corners[0].z());
        let p1v = Vector3::new(corners[1].x(), corners[1].y(), corners[1].z());
        let p3v = Vector3::new(corners[3].x(), corners[3].y(), corners[3].z());

        let tp0 = matrix.transform_point_3d(&p0v);
        let tp1 = matrix.transform_point_3d(&p1v);
        let tp3 = matrix.transform_point_3d(&p3v);

        let new_origin = Point3D::new(tp0.x(), tp0.y(), tp0.z());
        let u_vec = Vector3D::new(tp1.x() - tp0.x(), tp1.y() - tp0.y(), tp1.z() - tp0.z());
        let v_vec = Vector3D::new(tp3.x() - tp0.x(), tp3.y() - tp0.y(), tp3.z() - tp0.z());

        let width = u_vec.length();
        let height = v_vec.length();

        Rect3D::new(new_origin, u_vec, v_vec, width, height).ok_or_else(|| {
            TransformError::InvalidGeometry("Transformed rectangle is invalid".to_string())
        })
    }

    pub fn translation_matrix<T: Scalar>(translation: &Vector3<T>) -> Matrix4x4<T> {
        Matrix4x4::translation(translation.x(), translation.y(), translation.z())
    }

    pub fn rotation_matrix<T: Scalar>(
        center: &Point3D<T>,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let axis_len = (axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z()).sqrt();
        if axis_len.is_zero() {
            return Err(TransformError::InvalidRotation(
                "Rotation axis cannot be zero vector".to_string(),
            ));
        }

        let axis_n = Vector3::new(
            axis.x() / axis_len,
            axis.y() / axis_len,
            axis.z() / axis_len,
        );
        let c = Vector3::new(center.x(), center.y(), center.z());
        let to_origin = Matrix4x4::translation(-c.x(), -c.y(), -c.z());
        let rot = Matrix4x4::rotation_axis_3d(axis_n, angle.to_radians());
        let back = Matrix4x4::translation(c.x(), c.y(), c.z());
        Ok(back * rot * to_origin)
    }

    pub fn scale_matrix<T: Scalar>(
        center: &Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let c = Vector3::new(center.x(), center.y(), center.z());
        let to_origin = Matrix4x4::translation(-c.x(), -c.y(), -c.z());
        let scale = Matrix4x4::scale(scale_x, scale_y, scale_z);
        let back = Matrix4x4::translation(c.x(), c.y(), c.z());
        Ok(back * scale * to_origin)
    }
}

impl<T: Scalar> AnalysisTransform3D<T> for Rect3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Rect3D<T>;

    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_rect_3d(self, matrix).unwrap_or(*self)
    }

    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        let m = analysis_transform::translation_matrix(translation);
        analysis_transform::transform_rect_3d(self, &m)
    }

    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let m = analysis_transform::rotation_matrix(&center.center_point(), axis, angle)?;
        analysis_transform::transform_rect_3d(self, &m)
    }

    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        let m =
            analysis_transform::scale_matrix(&center.center_point(), scale_x, scale_y, scale_z)?;
        analysis_transform::transform_rect_3d(self, &m)
    }

    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis(center, scale_factor, scale_factor, scale_factor)
    }

    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, TransformError> {
        let mut matrix = Matrix4x4::identity();

        if let Some((sx, sy, sz)) = scale {
            let scale_center = rotation.as_ref().map_or(self, |(center, _, _)| *center);
            let scale_matrix =
                analysis_transform::scale_matrix(&scale_center.center_point(), sx, sy, sz)?;
            matrix = scale_matrix * matrix;
        }

        if let Some((center, axis, angle)) = rotation {
            let rotation_matrix =
                analysis_transform::rotation_matrix(&center.center_point(), axis, angle)?;
            matrix = rotation_matrix * matrix;
        }

        if let Some(translation_vec) = translation {
            let translation_matrix = analysis_transform::translation_matrix(translation_vec);
            matrix = translation_matrix * matrix;
        }

        analysis_transform::transform_rect_3d(self, &matrix)
    }

    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, TransformError> {
        let scale_tuple = scale.map(|s| (s, s, s));
        self.apply_composite_transform(translation, rotation, scale_tuple)
    }
}
