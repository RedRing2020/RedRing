//! Rect2D Analysis Matrix/Vector統合変換実装

use crate::{Point2D, Rect2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_contracts::{Angle, Scalar};
use geo_core::{AnalysisTransform2D, TransformError};

pub mod analysis_transform {
    use super::*;

    pub fn transform_rect_2d<T: Scalar>(
        rect: &Rect2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Result<Rect2D<T>, TransformError> {
        let transformed: Vec<Point2D<T>> = rect
            .corners()
            .iter()
            .map(|p| {
                let v = Vector2::new(p.x(), p.y());
                let tv = matrix.transform_point_2d(&v);
                Point2D::new(tv.x(), tv.y())
            })
            .collect();

        let mut min_x = transformed[0].x();
        let mut max_x = transformed[0].x();
        let mut min_y = transformed[0].y();
        let mut max_y = transformed[0].y();

        for p in transformed.iter().skip(1) {
            min_x = min_x.min(p.x());
            max_x = max_x.max(p.x());
            min_y = min_y.min(p.y());
            max_y = max_y.max(p.y());
        }

        Rect2D::from_corners(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y)).ok_or_else(
            || TransformError::InvalidGeometry("Transformed rectangle is invalid".to_string()),
        )
    }

    pub fn translation_matrix_2d<T: Scalar>(translation: &Vector2<T>) -> Matrix3x3<T> {
        Matrix3x3::translation_2d(translation)
    }

    pub fn rotation_matrix_2d<T: Scalar>(center: &Point2D<T>, angle: Angle<T>) -> Matrix3x3<T> {
        let c = Vector2::new(center.x(), center.y());
        Matrix3x3::rotation_around_point_2d(&c, angle.to_radians())
    }

    pub fn scale_matrix_2d<T: Scalar>(
        center: &Point2D<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let c = Vector2::new(center.x(), center.y());
        let to_origin = Matrix3x3::translation_2d(&(-c));
        let scale = Matrix3x3::scale_2d(&Vector2::new(scale_x, scale_y));
        let back = Matrix3x3::translation_2d(&c);
        Ok(back * scale * to_origin)
    }
}

impl<T: Scalar> AnalysisTransform2D<T> for Rect2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Rect2D<T>;

    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        analysis_transform::transform_rect_2d(self, matrix).unwrap_or(*self)
    }

    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError> {
        let m = analysis_transform::translation_matrix_2d(translation);
        analysis_transform::transform_rect_2d(self, &m)
    }

    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let c = center.center_point();
        let m = analysis_transform::rotation_matrix_2d(&c, angle);
        analysis_transform::transform_rect_2d(self, &m)
    }

    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, TransformError> {
        let c = center.center_point();
        let m = analysis_transform::scale_matrix_2d(&c, scale_x, scale_y)?;
        analysis_transform::transform_rect_2d(self, &m)
    }

    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis_2d(center, scale_factor, scale_factor)
    }
}
