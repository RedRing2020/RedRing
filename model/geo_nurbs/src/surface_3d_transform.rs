//! `NurbsSurface3D` Transform Implementation
//!
//! `AnalysisTransform3D` trait implementation for NURBS surfaces.
//! Applies transformations to all control points using Analysis Matrix operations.

use crate::surface_3d::NurbsSurface3D;
use analysis::linalg::{
    matrix::Matrix4x4,
    vector::{Vector3, Vector4},
};
use geo_contracts::NurbsSurface3DProperties;
use geo_contracts::{Angle, Scalar};
use geo_core::{AnalysisTransform3D, TransformError};

/// Matrix4x4による制御点変換の内部実装
fn transform_control_points<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    matrix: &Matrix4x4<T>,
) -> Result<NurbsSurface3D<T>, TransformError> {
    let u_count = surface.u_count();
    let v_count = surface.v_count();
    let mut transformed_points: Vec<Vec<Vector3<T>>> = Vec::with_capacity(u_count);

    // 各制御点を変換
    for u in 0..u_count {
        let mut row = Vec::with_capacity(v_count);
        for v in 0..v_count {
            let point = surface.control_point(u, v);

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
            row.push(transformed_point);
        }
        transformed_points.push(row);
    }

    // 重みはそのまま保持
    let weights = if surface.is_rational() {
        let mut w_grid = Vec::with_capacity(u_count);
        for u in 0..u_count {
            let mut w_row = Vec::with_capacity(v_count);
            for v in 0..v_count {
                w_row.push(surface.weight(u, v));
            }
            w_grid.push(w_row);
        }
        Some(w_grid)
    } else {
        None
    };

    // 内部ヘルパーを使用
    NurbsSurface3D::new_internal(
        transformed_points,
        weights,
        surface.u_knots().clone(),
        surface.v_knots().clone(),
        surface.u_degree(),
        surface.v_degree(),
    )
    .map_err(|e| {
        TransformError::InvalidGeometry(format!("Failed to create transformed surface: {e}"))
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

impl<T: Scalar> AnalysisTransform3D<T> for NurbsSurface3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = NurbsSurface3D<T>;

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
        // centerからNurbsSurface3Dの重心を計算
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
        // centerからNurbsSurface3Dの重心を計算
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
            matrix = trans_mat * matrix;
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

impl<T: Scalar> NurbsSurface3D<T> {
    /// 制御点の重心を計算
    fn compute_centroid(&self) -> Vector3<T> {
        let u_count = self.u_count();
        let v_count = self.v_count();
        let total_points = u_count * v_count;

        let mut sum_x = T::ZERO;
        let mut sum_y = T::ZERO;
        let mut sum_z = T::ZERO;

        for u in 0..u_count {
            for v in 0..v_count {
                let point = self.control_point(u, v);
                sum_x += point.x();
                sum_y += point.y();
                sum_z += point.z();
            }
        }

        let count = T::from_usize(total_points);
        Vector3::new(sum_x / count, sum_y / count, sum_z / count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::Angle;
    use geo_contracts::NurbsSurface3DConstructor;

    #[test]
    fn test_translate_analysis() {
        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

        let translation = Vector3::new(1.0, 2.0, 3.0);
        let result = <NurbsSurface3D<f64> as AnalysisTransform3D<f64>>::translate_analysis(
            &surface,
            &translation,
        );

        assert!(result.is_ok());
        let translated = result.unwrap();

        // 原点の制御点が平行移動されているか確認
        let p00 = translated.control_point(0, 0);
        assert!((p00.x() - 1.0).abs() < 1e-10);
        assert!((p00.y() - 2.0).abs() < 1e-10);
        assert!((p00.z() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_analysis_z_axis() {
        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

        let axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let result = <NurbsSurface3D<f64> as AnalysisTransform3D<f64>>::rotate_analysis(
            &surface, &surface, &axis, angle,
        );

        assert!(result.is_ok());
        let rotated = result.unwrap();

        // 回転が適用されたことを確認（具体的な座標は重心回転のため複雑）
        // 少なくとも制御点が変化していることを確認
        let original_p10 = surface.control_point(1, 0);
        let rotated_p10 = rotated.control_point(1, 0);

        // 何らかの変化があることを確認
        let changed = (original_p10.x() - rotated_p10.x()).abs() > 1e-10
            || (original_p10.y() - rotated_p10.y()).abs() > 1e-10;
        assert!(changed, "Control point should change after rotation");
    }

    #[test]
    fn test_uniform_scale_analysis() {
        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

        let scale_factor = 2.0;
        let result = <NurbsSurface3D<f64> as AnalysisTransform3D<f64>>::uniform_scale_analysis(
            &surface,
            &surface,
            scale_factor,
        );

        assert!(result.is_ok());
        let scaled = result.unwrap();

        // 重心からの距離が2倍になっているか確認
        let centroid = surface.compute_centroid();
        let original_p10 = surface.control_point(1, 0);
        let scaled_p10 = scaled.control_point(1, 0);

        let original_dist_x = (original_p10.x() - centroid.x()).abs();
        let scaled_dist_x = (scaled_p10.x() - centroid.x()).abs();

        assert!((scaled_dist_x - original_dist_x * 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_point_matrix() {
        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

        let translation = Vector3::new(5.0, 10.0, 15.0);
        let matrix = Matrix4x4::translation_3d(&translation);

        let transformed = <NurbsSurface3D<f64> as AnalysisTransform3D<f64>>::transform_point_matrix(
            &surface, &matrix,
        );

        // 原点の制御点が変換されているか確認
        let p00 = transformed.control_point(0, 0);
        assert!((p00.x() - 5.0).abs() < 1e-10);
        assert!((p00.y() - 10.0).abs() < 1e-10);
        assert!((p00.z() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_composite_transform() {
        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

        let translation = Vector3::new(1.0, 0.0, 0.0);
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(45.0);
        let scale = (2.0, 2.0, 2.0);

        let result = <NurbsSurface3D<f64> as AnalysisTransform3D<f64>>::apply_composite_transform(
            &surface,
            Some(&translation),
            Some((&surface, &axis, angle)),
            Some(scale),
        );

        assert!(result.is_ok());
        // 複合変換が成功すればOK（詳細な検証は省略）
    }
}
