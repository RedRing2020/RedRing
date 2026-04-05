//! `NurbsCurve2D` Transform Implementation
//!
//! `AnalysisTransform2D` trait implementation for NURBS curves.
//! Applies transformations to all control points using Analysis Matrix operations.

use crate::curve_2d::NurbsCurve2D;
use analysis::linalg::{
    matrix::Matrix3x3,
    vector::{Vector2, Vector3},
};
use geo_contracts::default_kernel_numerical_zero_tolerance;
use geo_contracts::NurbsCurve2DProperties;
use geo_contracts::{Angle, Scalar};
use geo_core::{AnalysisTransform2D, TransformError};

/// Matrix3x3による制御点変換の内部実装
fn transform_control_points<T: Scalar>(
    curve: &NurbsCurve2D<T>,
    matrix: &Matrix3x3<T>,
) -> Result<NurbsCurve2D<T>, TransformError> {
    let num_points = curve.num_points();
    let mut transformed_points = Vec::with_capacity(num_points);

    // 各制御点を変換
    for i in 0..num_points {
        let point = curve.control_point(i);

        // 同次座標に変換して変換
        let homogeneous = Vector3::new(point.x(), point.y(), T::ONE);
        let transformed = (*matrix) * homogeneous;

        // w成分で除算（透視投影対応）
        let w = transformed.z();
        if w.abs() < default_kernel_numerical_zero_tolerance::<T>() {
            return Err(TransformError::InvalidGeometry(
                "Transform resulted in zero w component".to_string(),
            ));
        }

        let transformed_point = Vector2::new(transformed.x() / w, transformed.y() / w);
        transformed_points.push(transformed_point);
    }

    // 重みはそのまま保持
    let weights = if curve.is_rational() {
        let mut w = Vec::with_capacity(num_points);
        for i in 0..num_points {
            w.push(curve.weight(i));
        }
        Some(w)
    } else {
        None
    };

    // 内部ヘルパーを使用
    NurbsCurve2D::new_internal(
        &transformed_points,
        weights,
        curve.knot_vector().clone(),
        curve.degree(),
    )
    .map_err(|e| {
        TransformError::InvalidGeometry(format!("Failed to create transformed curve: {e}"))
    })
}

/// 平行移動行列を生成
fn translation_matrix_2d<T: Scalar>(tx: T, ty: T) -> Matrix3x3<T> {
    let translation = Vector2::new(tx, ty);
    Matrix3x3::translation_2d(&translation)
}

/// 回転行列を生成
fn rotation_matrix_2d<T: Scalar>(angle: Angle<T>) -> Matrix3x3<T> {
    Matrix3x3::rotation_2d(angle.to_radians())
}

/// スケール行列を生成
fn scale_matrix_2d<T: Scalar>(sx: T, sy: T) -> Result<Matrix3x3<T>, TransformError> {
    if sx.abs() < default_kernel_numerical_zero_tolerance::<T>()
        || sy.abs() < default_kernel_numerical_zero_tolerance::<T>()
    {
        return Err(TransformError::InvalidScaleFactor(
            "Scale factors cannot be zero".to_string(),
        ));
    }

    let scale = Vector2::new(sx, sy);
    Ok(Matrix3x3::scale_2d(&scale))
}

impl<T: Scalar> AnalysisTransform2D<T> for NurbsCurve2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = NurbsCurve2D<T>;

    /// Matrix3x3による直接座標変換
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output {
        transform_control_points(self, matrix).unwrap_or_else(|_| self.clone())
    }

    /// 平行移動変換（Analysis Vector2使用）
    fn translate_analysis_2d(
        &self,
        translation: &Vector2<T>,
    ) -> Result<Self::Output, TransformError> {
        let matrix = translation_matrix_2d(translation.x(), translation.y());
        transform_control_points(self, &matrix)
    }

    /// 回転変換（Analysis Matrix3x3使用）
    fn rotate_analysis_2d(
        &self,
        center: &Vector2<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let rotation = rotation_matrix_2d(angle);

        let to_origin = translation_matrix_2d(-center.x(), -center.y());
        let from_origin = translation_matrix_2d(center.x(), center.y());

        let combined = from_origin * rotation * to_origin;
        transform_control_points(self, &combined)
    }

    /// スケール変換（Analysis Matrix3x3使用）
    fn scale_analysis_2d(
        &self,
        center: &Vector2<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, TransformError> {
        let scale = scale_matrix_2d(scale_x, scale_y)?;

        let to_origin = translation_matrix_2d(-center.x(), -center.y());
        let from_origin = translation_matrix_2d(center.x(), center.y());

        let combined = from_origin * scale * to_origin;
        transform_control_points(self, &combined)
    }

    /// 均等スケール変換（Analysis Matrix3x3使用）
    fn uniform_scale_analysis_2d(
        &self,
        center: &Vector2<T>,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis_2d(center, scale_factor, scale_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::NurbsCurve2DConstructor;

    #[test]
    fn test_translate_analysis_2d() {
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::unit_line();

        let translation = Vector2::new(2.0, 3.0);
        let result = <NurbsCurve2D<f64> as AnalysisTransform2D<f64>>::translate_analysis_2d(
            &curve,
            &translation,
        );

        assert!(result.is_ok());
        let translated = result.unwrap();

        // 始点が平行移動されているか確認
        let p0 = translated.control_point(0);
        assert!((p0.x() - 2.0).abs() < 1e-10);
        assert!((p0.y() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_analysis_2d() {
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::unit_line();
        let center = Vector2::new(10.0, 0.0);

        let angle = Angle::from_degrees(90.0);
        let result = <NurbsCurve2D<f64> as AnalysisTransform2D<f64>>::rotate_analysis_2d(
            &curve, &center, angle,
        );

        assert!(result.is_ok());
        let rotated = result.unwrap();

        let p0 = rotated.control_point(0);
        let p1 = rotated.control_point(1);
        assert!((p0.x() - 10.0).abs() < 1e-10);
        assert!((p0.y() + 10.0).abs() < 1e-10);
        assert!((p1.x() - 10.0).abs() < 1e-10);
        assert!((p1.y() + 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_uniform_scale_analysis_2d() {
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::unit_line();
        let center = Vector2::new(10.0, 0.0);

        let scale_factor = 2.0;
        let result = <NurbsCurve2D<f64> as AnalysisTransform2D<f64>>::uniform_scale_analysis_2d(
            &curve,
            &center,
            scale_factor,
        );

        assert!(result.is_ok());
        let scaled = result.unwrap();

        let scaled_p0 = scaled.control_point(0);
        let scaled_p1 = scaled.control_point(1);

        assert!((scaled_p0.x() + 10.0).abs() < 1e-10);
        assert!((scaled_p0.y() - 0.0).abs() < 1e-10);
        assert!((scaled_p1.x() + 8.0).abs() < 1e-10);
        assert!((scaled_p1.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_point_matrix_2d() {
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::unit_line();

        let translation = Vector2::new(5.0, 10.0);
        let matrix = Matrix3x3::translation_2d(&translation);

        let transformed =
            <NurbsCurve2D<f64> as AnalysisTransform2D<f64>>::transform_point_matrix_2d(
                &curve, &matrix,
            );

        // 始点が変換されているか確認
        let p0 = transformed.control_point(0);
        assert!((p0.x() - 5.0).abs() < 1e-10);
        assert!((p0.y() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_analysis_2d() {
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::unit_line();
        let center = Vector2::new(0.0, 0.0);

        let result = <NurbsCurve2D<f64> as AnalysisTransform2D<f64>>::scale_analysis_2d(
            &curve, &center, 2.0, 3.0,
        );

        assert!(result.is_ok());
        // スケール変換が成功すればOK（詳細な検証は省略）
    }
}
