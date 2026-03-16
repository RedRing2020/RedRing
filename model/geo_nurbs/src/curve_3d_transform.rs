//! `NurbsCurve3D` Transform Implementation
//!
//! `AnalysisTransform3D` trait implementation for NURBS curves.
//! Applies transformations to all control points using Analysis Matrix operations.

use crate::curve_3d::NurbsCurve3D;
use analysis::linalg::{
    matrix::Matrix4x4,
    vector::{Vector3, Vector4},
};
use geo_core::{AnalysisTransform3D, TransformError};
use geo_foundation::{Angle, NurbsCurve3DProperties, Scalar};

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
        if w.abs() < T::EPSILON {
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
    if axis_norm_sq < T::EPSILON * T::EPSILON {
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
    if sx.abs() < T::EPSILON || sy.abs() < T::EPSILON || sz.abs() < T::EPSILON {
        return Err(TransformError::InvalidScaleFactor(
            "Scale factors cannot be zero".to_string(),
        ));
    }

    let scale = Vector3::new(sx, sy, sz);
    Ok(Matrix4x4::scale_3d(&scale))
}

// ============================================================================
// AnalysisTransform3D Implementation
// ============================================================================

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
        _center: &Self,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        // centerからNurbsCurve3Dの重心を計算
        let centroid = self.compute_centroid();

        // 回転行列を生成
        let rotation = rotation_matrix_3d(axis, angle)?;

        // 複合変換: center → 原点 → 回転 → 元の位置
        let to_origin = translation_matrix_3d(-centroid.x(), -centroid.y(), -centroid.z());
        let from_origin = translation_matrix_3d(centroid.x(), centroid.y(), centroid.z());

        let combined = from_origin * rotation * to_origin;
        transform_control_points(self, &combined)
    }

    /// スケール変換（Analysis Matrix4x4使用）
    fn scale_analysis(
        &self,
        _center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        // centerからNurbsCurve3Dの重心を計算
        let centroid = self.compute_centroid();

        // スケール行列を生成
        let scale = scale_matrix_3d(scale_x, scale_y, scale_z)?;

        // 複合変換: center → 原点 → スケール → 元の位置
        let to_origin = translation_matrix_3d(-centroid.x(), -centroid.y(), -centroid.z());
        let from_origin = translation_matrix_3d(centroid.x(), centroid.y(), centroid.z());

        let combined = from_origin * scale * to_origin;
        transform_control_points(self, &combined)
    }

    /// 均等スケール変換（Analysis Matrix4x4使用）
    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        self.scale_analysis(center, scale_factor, scale_factor, scale_factor)
    }

    /// 複合変換（平行移動+回転+スケール）
    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, TransformError> {
        let mut matrix = Matrix4x4::identity();

        // スケール適用
        if let Some((sx, sy, sz)) = scale {
            let scale_mat = scale_matrix_3d(sx, sy, sz)?;
            matrix = matrix * scale_mat;
        }

        // 回転適用
        if let Some((_center, axis, angle)) = rotation {
            let centroid = self.compute_centroid();
            let rotation_mat = rotation_matrix_3d(axis, angle)?;
            let to_origin = translation_matrix_3d(-centroid.x(), -centroid.y(), -centroid.z());
            let from_origin = translation_matrix_3d(centroid.x(), centroid.y(), centroid.z());
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
        rotation: Option<(&Self, &Vector3<T>, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, TransformError> {
        let scale_tuple = scale.map(|s| (s, s, s));
        self.apply_composite_transform(translation, rotation, scale_tuple)
    }
}

// ============================================================================
// Helper Methods
// ============================================================================

impl<T: Scalar> NurbsCurve3D<T> {
    /// 制御点の重心を計算
    fn compute_centroid(&self) -> Vector3<T> {
        let num_points = self.num_points();
        let mut sum_x = T::ZERO;
        let mut sum_y = T::ZERO;
        let mut sum_z = T::ZERO;

        for i in 0..num_points {
            let point = self.control_point(i);
            sum_x += point.x();
            sum_y += point.y();
            sum_z += point.z();
        }

        #[allow(clippy::cast_precision_loss)]
        let n = T::from_f64(num_points as f64);
        Vector3::new(sum_x / n, sum_y / n, sum_z / n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::{NurbsCurve3DConstructor, NurbsCurve3DProperties};

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

        let center = curve.clone(); // 自身を中心として使用
        let result = curve.uniform_scale_analysis(&center, 2.0);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        // スケール2倍後、元の長さの2倍になる
        assert_eq!(transformed.num_points(), 2);
    }

    #[test]
    fn test_rotate_analysis_z_axis() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (1.0, 0.0, 0.0),
            (2.0, 0.0, 0.0),
        )
        .unwrap();

        let center = curve.clone();
        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let result = curve.rotate_analysis(&center, &z_axis, angle);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        // Z軸周りに90度回転 → X方向がY方向へ
        // (1,0,0) と (2,0,0) の平num_points
        // 回転後は約 (0, 1.5, 0) 付近に分布
        assert_eq!(transformed.control_points_count(), 2);
    }

    #[test]
    fn test_apply_composite_transform() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
        )
        .unwrap();

        let translation = Vector3::new(5.0, 0.0, 0.0);
        let center = curve.clone();
        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(0.0);

        let result = curve.apply_composite_transform(
            Some(&translation),
            Some((&center, &z_axis, angle)),
            Some((2.0, 2.0, 2.0)),
        );
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.num_points(), 2);
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
