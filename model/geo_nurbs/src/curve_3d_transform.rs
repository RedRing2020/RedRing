//! `NurbsCurve3D` Transform Implementation
//!
//! `AnalysisTransform3D` trait implementation for NURBS curves.
//! Applies transformations to all control points using Analysis Matrix operations.

use crate::curve_3d::NurbsCurve3D;
use analysis::linalg::{
    matrix::Matrix4x4,
    vector::{Vector3, Vector4},
};
use geo_contracts::default_kernel_numerical_zero_tolerance;
#[cfg(test)]
use geo_contracts::Bounded;
use geo_contracts::NurbsCurve3DProperties;
use geo_contracts::{Angle, Scalar};
use geo_core::{AnalysisTransform3D, TransformError};

/// Matrix4x4による制御点変換の内部実装
fn transform_control_points<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    matrix: &Matrix4x4<T>,
) -> Result<NurbsCurve3D<T>, TransformError> {
    let num_points = curve.num_points();
    let mut transformed_points = Vec::with_capacity(num_points);

    // 各制御点を変換
    for i in 0..num_points {
        let point = curve.control_point(i);

        // 同次座標に変換して変換
        let homogeneous = Vector4::new(point.x(), point.y(), point.z(), T::ONE);
        let transformed = (*matrix) * homogeneous;

        // w成分で除算（透視投影対応）
        let w = transformed.w();
        if w.abs() < default_kernel_numerical_zero_tolerance::<T>() {
            return Err(TransformError::InvalidGeometry(
                "Transform resulted in zero w component".to_string(),
            ));
        }

        let transformed_point = Vector3::new(
            transformed.x() / w,
            transformed.y() / w,
            transformed.z() / w,
        );
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
    NurbsCurve3D::new_internal(
        transformed_points,
        weights,
        curve.knot_vector().clone(),
        curve.degree(),
    )
    .map_err(|e| {
        TransformError::InvalidGeometry(format!("Failed to create transformed curve: {e}"))
    })
}

/// 平行移動行列を生成
fn translation_matrix_3d<T: Scalar>(tx: T, ty: T, tz: T) -> Matrix4x4<T> {
    let translation = Vector3::new(tx, ty, tz);
    Matrix4x4::translation_3d(&translation)
}

/// 軸回転行列を生成（Rodriguesの回転公式）
fn rotation_matrix_3d<T: Scalar>(
    axis: &Vector3<T>,
    angle: Angle<T>,
) -> Result<Matrix4x4<T>, TransformError> {
    let axis_norm_sq = axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z();
    let zero_tol = default_kernel_numerical_zero_tolerance::<T>();
    if axis_norm_sq < zero_tol * zero_tol {
        return Err(TransformError::ZeroVector(
            "Rotation axis cannot be zero".to_string(),
        ));
    }

    let len = axis_norm_sq.sqrt();
    let normalized = Vector3::new(axis.x() / len, axis.y() / len, axis.z() / len);

    Ok(Matrix4x4::rotation_axis_3d(normalized, angle.to_radians()))
}

/// スケール行列を生成
fn scale_matrix_3d<T: Scalar>(sx: T, sy: T, sz: T) -> Result<Matrix4x4<T>, TransformError> {
    if sx.abs() < default_kernel_numerical_zero_tolerance::<T>()
        || sy.abs() < default_kernel_numerical_zero_tolerance::<T>()
        || sz.abs() < default_kernel_numerical_zero_tolerance::<T>()
    {
        return Err(TransformError::InvalidScaleFactor(
            "Scale factors cannot be zero".to_string(),
        ));
    }

    let scale = Vector3::new(sx, sy, sz);
    Ok(Matrix4x4::scale_3d(&scale))
}

#[cfg(test)]
fn pivot_from_curve_aabb<T: Scalar>(curve: &NurbsCurve3D<T>) -> Result<Vector3<T>, TransformError> {
    let center = curve
        .aabb()
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Center curve has no bounding box".to_string())
        })?
        .center();
    Ok(Vector3::new(center.x(), center.y(), center.z()))
}

impl<T: Scalar> AnalysisTransform3D<T> for NurbsCurve3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = NurbsCurve3D<T>;

    /// Matrix4x4による直接座標変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        transform_control_points(self, matrix).unwrap_or_else(|_| self.clone())
    }

    /// 平行移動変換（Analysis Vector3使用）
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        let matrix = translation_matrix_3d(translation.x(), translation.y(), translation.z());
        transform_control_points(self, &matrix)
    }

    /// 軸回転変換（Analysis Matrix4x4使用）
    fn rotate_analysis(
        &self,
        center: &Vector3<T>,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let rotation = rotation_matrix_3d(axis, angle)?;

        let to_origin = translation_matrix_3d(-center.x(), -center.y(), -center.z());
        let from_origin = translation_matrix_3d(center.x(), center.y(), center.z());

        let combined = from_origin * rotation * to_origin;
        transform_control_points(self, &combined)
    }

    /// スケール変換（Analysis Matrix4x4使用）
    fn scale_analysis(
        &self,
        center: &Vector3<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        let scale = scale_matrix_3d(scale_x, scale_y, scale_z)?;

        let to_origin = translation_matrix_3d(-center.x(), -center.y(), -center.z());
        let from_origin = translation_matrix_3d(center.x(), center.y(), center.z());

        let combined = from_origin * scale * to_origin;
        transform_control_points(self, &combined)
    }

    /// 均等スケール変換（Analysis Matrix4x4使用）
    fn uniform_scale_analysis(
        &self,
        center: &Vector3<T>,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis(center, scale_factor, scale_factor, scale_factor)
    }

    /// 複合変換（平行移動+回転+スケール）
    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Vector3<T>, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, TransformError> {
        let mut matrix = Matrix4x4::identity();

        // スケール適用
        if let Some((sx, sy, sz)) = scale {
            let scale_mat = scale_matrix_3d(sx, sy, sz)?;
            matrix = matrix * scale_mat;
        }

        // 回転適用
        if let Some((center, axis, angle)) = rotation {
            let rotation_mat = rotation_matrix_3d(axis, angle)?;
            let to_origin = translation_matrix_3d(-center.x(), -center.y(), -center.z());
            let from_origin = translation_matrix_3d(center.x(), center.y(), center.z());
            matrix = matrix * from_origin * rotation_mat * to_origin;
        }

        // 平行移動適用
        if let Some(trans) = translation {
            let trans_mat = translation_matrix_3d(trans.x(), trans.y(), trans.z());
            matrix = matrix * trans_mat;
        }

        transform_control_points(self, &matrix)
    }

    /// 複合変換（均等スケール版）
    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Vector3<T>, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, TransformError> {
        let scale_tuple = scale.map(|s| (s, s, s));
        self.apply_composite_transform(translation, rotation, scale_tuple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::{NurbsCurve3DConstructor, NurbsCurve3DProperties};

    #[test]
    fn test_translate_analysis() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
        )
        .unwrap();

        let translation = Vector3::new(5.0, 3.0, 2.0);
        let result = curve.translate_analysis(&translation);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.num_points(), 2);

        // 始点確認（0,0,0 → 5,3,2）
        let p0 = transformed.control_point(0);
        assert!((p0.x() - 5.0).abs() < 1e-10);
        assert!((p0.y() - 3.0).abs() < 1e-10);
        assert!((p0.z() - 2.0).abs() < 1e-10);

        // 終点確認（1,0,0 → 6,3,2）
        let p1 = transformed.control_point(1);
        assert!((p1.x() - 6.0).abs() < 1e-10);
        assert!((p1.y() - 3.0).abs() < 1e-10);
        assert!((p1.z() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_uniform_scale_analysis() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (2.0, 0.0, 0.0),
        )
        .unwrap();

        let center_curve = curve
            .translate_analysis(&Vector3::new(10.0, 0.0, 0.0))
            .expect("center curve should translate");
        let center = pivot_from_curve_aabb(&center_curve).expect("center curve should have pivot");
        let result = curve.uniform_scale_analysis(&center, 2.0);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        let p0 = transformed.control_point(0);
        let p1 = transformed.control_point(1);
        assert_eq!(transformed.num_points(), 2);
        assert!((p0.x() + 11.0).abs() < 1e-10);
        assert!((p1.x() + 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_analysis_z_axis() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (1.0, 0.0, 0.0),
            (2.0, 0.0, 0.0),
        )
        .unwrap();

        let center_curve = curve
            .translate_analysis(&Vector3::new(10.0, 0.0, 0.0))
            .expect("center curve should translate");
        let center = pivot_from_curve_aabb(&center_curve).expect("center curve should have pivot");
        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let result = curve.rotate_analysis(&center, &z_axis, angle);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        let p0 = transformed.control_point(0);
        let p1 = transformed.control_point(1);
        assert_eq!(transformed.control_points_count(), 2);
        assert!((p0.x() - 11.5).abs() < 1e-10);
        assert!((p0.y() + 10.5).abs() < 1e-10);
        assert!((p1.x() - 11.5).abs() < 1e-10);
        assert!((p1.y() + 9.5).abs() < 1e-10);
    }

    #[test]
    fn test_apply_composite_transform() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
        )
        .unwrap();

        let translation = Vector3::new(5.0, 0.0, 0.0);
        let center_curve = curve
            .translate_analysis(&Vector3::new(10.0, 0.0, 0.0))
            .expect("center curve should translate");
        let center = pivot_from_curve_aabb(&center_curve).expect("center curve should have pivot");
        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let result = curve.apply_composite_transform(
            Some(&translation),
            Some((&center, &z_axis, angle)),
            Some((2.0, 2.0, 2.0)),
        );
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.num_points(), 2);
        let p0 = transformed.control_point(0);
        assert!((p0.x() - 21.0).abs() < 1e-10);
        assert!((p0.y() + 11.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_point_matrix() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
        )
        .unwrap();

        // 単位行列による変換（変化なし）
        let identity = Matrix4x4::identity();
        let transformed = curve.transform_point_matrix(&identity);

        assert_eq!(transformed.num_points(), 2);
        let p0 = transformed.control_point(0);
        let p1 = transformed.control_point(1);
        assert!((p0.x() - 0.0).abs() < 1e-10);
        assert!((p1.x() - 1.0).abs() < 1e-10);
    }
}
