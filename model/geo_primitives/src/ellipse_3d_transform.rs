//! Ellipse3D Analysis Matrix統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な楕円変換
//! 中心点・軸ベクトル変換による3D楕円パラメータ更新
//! Triangle3D/LineSegment3D Analysis Transform パターンを基盤とする統一実装

use crate::{Ellipse3D, Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// Ellipse3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一楕円の行列変換（中心点＋軸ベクトル変換）
    pub fn transform_ellipse_3d<T: Scalar>(
        ellipse: &Ellipse3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Ellipse3D<T> {
        // 中心点の変換
        let center_vec = Vector3::new(
            ellipse.center().x(),
            ellipse.center().y(),
            ellipse.center().z(),
        );
        let transformed_center_vec = matrix.transform_point_3d(&center_vec);
        let new_center = Point3D::new(
            transformed_center_vec.x(),
            transformed_center_vec.y(),
            transformed_center_vec.z(),
        );

        // 法線ベクトルと長軸方向ベクトルの変換
        let normal_vec = Vector3::new(
            ellipse.normal().x(),
            ellipse.normal().y(),
            ellipse.normal().z(),
        );
        let major_axis_vec = Vector3::new(
            ellipse.major_axis_direction().x(),
            ellipse.major_axis_direction().y(),
            ellipse.major_axis_direction().z(),
        );

        // Matrix4x4による方向ベクトル変換（平行移動成分を除去）
        let transform_direction = |v: Vector3<T>| -> Vector3<T> {
            let origin = Vector3::new(T::ZERO, T::ZERO, T::ZERO);
            let transformed_point = matrix.transform_point_3d(&v);
            let transformed_origin = matrix.transform_point_3d(&origin);
            Vector3::new(
                transformed_point.x() - transformed_origin.x(),
                transformed_point.y() - transformed_origin.y(),
                transformed_point.z() - transformed_origin.z(),
            )
        };

        let transformed_normal = transform_direction(normal_vec);
        let transformed_major_axis = transform_direction(major_axis_vec);

        // 変換後の軸長計算（スケーリング効果を反映）
        let major_scale_factor = (transformed_major_axis.x() * transformed_major_axis.x()
            + transformed_major_axis.y() * transformed_major_axis.y()
            + transformed_major_axis.z() * transformed_major_axis.z())
        .sqrt();
        let new_semi_major = ellipse.semi_major_axis() * major_scale_factor;

        // 短軸ベクトルを計算（法線と長軸の外積）
        let minor_axis_vec = Vector3::new(
            normal_vec.y() * major_axis_vec.z() - normal_vec.z() * major_axis_vec.y(),
            normal_vec.z() * major_axis_vec.x() - normal_vec.x() * major_axis_vec.z(),
            normal_vec.x() * major_axis_vec.y() - normal_vec.y() * major_axis_vec.x(),
        );
        let transformed_minor_axis = transform_direction(minor_axis_vec);
        let minor_scale_factor = (transformed_minor_axis.x() * transformed_minor_axis.x()
            + transformed_minor_axis.y() * transformed_minor_axis.y()
            + transformed_minor_axis.z() * transformed_minor_axis.z())
        .sqrt();
        let new_semi_minor = ellipse.semi_minor_axis() * minor_scale_factor;

        // 変換後のベクトルをVector3Dに変換（正規化）
        let new_normal_vector3d = Vector3D::new(
            transformed_normal.x(),
            transformed_normal.y(),
            transformed_normal.z(),
        );
        let new_major_axis_vector3d = Vector3D::new(
            transformed_major_axis.x(),
            transformed_major_axis.y(),
            transformed_major_axis.z(),
        );

        // 新しい楕円を作成（軸の順序を確認）
        if new_semi_major >= new_semi_minor {
            Ellipse3D::new(
                new_center,
                new_semi_major,
                new_semi_minor,
                new_normal_vector3d,
                new_major_axis_vector3d,
            )
            .unwrap_or_else(|| {
                Ellipse3D::xy_aligned(Point3D::origin(), T::ONE, T::ONE / (T::ONE + T::ONE))
                    .unwrap()
            })
        } else {
            // 長軸と短軸が逆転した場合、軸を入れ替え
            // 新しい長軸方向 = 元の短軸方向
            let corrected_major_axis = Vector3D::new(
                transformed_minor_axis.x(),
                transformed_minor_axis.y(),
                transformed_minor_axis.z(),
            );
            Ellipse3D::new(
                new_center,
                new_semi_minor,
                new_semi_major,
                new_normal_vector3d,
                corrected_major_axis,
            )
            .unwrap_or_else(|| {
                Ellipse3D::xy_aligned(Point3D::origin(), T::ONE, T::ONE / (T::ONE + T::ONE))
                    .unwrap()
            })
        }
    }

    /// 複数楕円の一括行列変換
    pub fn transform_ellipses_3d<T: Scalar>(
        ellipses: &[Ellipse3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Vec<Ellipse3D<T>> {
        ellipses
            .iter()
            .map(|ellipse| transform_ellipse_3d(ellipse, matrix))
            .collect()
    }

    /// 平行移動行列生成
    pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3D<T>) -> Matrix4x4<T> {
        let translation_vec3 = Vector3::new(translation.x(), translation.y(), translation.z());
        Matrix4x4::translation_3d(&translation_vec3)
    }

    /// 回転行列生成（軸回転版）
    pub fn rotation_matrix_3d<T: Scalar>(
        _center: &Point3D<T>,
        axis: &Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // 回転軸の正規化チェック
        let axis_length_squared = axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z();
        if axis_length_squared.is_zero() {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero vector".to_string(),
            ));
        }

        let axis_length = axis_length_squared.sqrt();
        let normalized_axis = Vector3D::new(
            axis.x() / axis_length,
            axis.y() / axis_length,
            axis.z() / axis_length,
        );
        let axis_vec3 = Vector3::new(
            normalized_axis.x(),
            normalized_axis.y(),
            normalized_axis.z(),
        );

        Ok(Matrix4x4::rotation_axis_3d(axis_vec3, angle.to_radians()))
    }

    /// スケール行列生成
    pub fn scale_matrix_3d<T: Scalar>(
        _center: &Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // スケール倍率のゼロチェック
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let scale_vec3 = Vector3::new(scale_x, scale_y, scale_z);
        Ok(Matrix4x4::scale_3d(&scale_vec3))
    }

    /// 均等スケール行列生成
    pub fn uniform_scale_matrix_3d<T: Scalar>(
        center: &Point3D<T>,
        scale_factor: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        scale_matrix_3d(center, scale_factor, scale_factor, scale_factor)
    }

    /// 複合変換行列生成（平行移動+回転+スケール）
    pub fn composite_ellipse_transform_3d<T: Scalar>(
        translation: Option<&Vector3D<T>>,
        rotation: Option<(&Point3D<T>, &Vector3D<T>, Angle<T>)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        let mut matrix = Matrix4x4::identity();

        // スケール変換（最初に適用）
        if let Some((sx, sy, sz)) = scale {
            let origin = Point3D::origin();
            let scale_mat = scale_matrix_3d(&origin, sx, sy, sz)?;
            matrix = matrix * scale_mat;
        }

        // 回転変換
        if let Some((center, axis, angle)) = rotation {
            let rotation_mat = rotation_matrix_3d(center, axis, angle)?;
            matrix = matrix * rotation_mat;
        }

        // 平行移動変換（最後に適用）
        if let Some(translation_vec) = translation {
            let translation_mat = translation_matrix_3d(translation_vec);
            matrix = matrix * translation_mat;
        }

        Ok(matrix)
    }

    /// 楕円の中心を計算する補助関数
    pub fn ellipse_center_3d<T: Scalar>(ellipse: &Ellipse3D<T>) -> Point3D<T> {
        ellipse.center()
    }
}

/// Ellipse3DでのAnalysisTransform3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform3D<T> for Ellipse3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        analysis_transform::transform_ellipse_3d(self, matrix)
    }

    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self, TransformError> {
        // Vector3からVector3Dへの変換
        let vector3d = Vector3D::new(translation.x(), translation.y(), translation.z());
        let matrix = analysis_transform::translation_matrix_3d(&vector3d);
        Ok(self.transform_point_matrix(&matrix))
    }

    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // Ellipse3Dから中心を取得
        let center_point = analysis_transform::ellipse_center_3d(center);
        // Vector3からVector3Dへの変換
        let axis_vector3d = Vector3D::new(axis.x(), axis.y(), axis.z());
        let matrix = analysis_transform::rotation_matrix_3d(&center_point, &axis_vector3d, angle)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        // Ellipse3Dから中心を取得
        let center_point = analysis_transform::ellipse_center_3d(center);
        let matrix = analysis_transform::scale_matrix_3d(&center_point, scale_x, scale_y, scale_z)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        self.scale_analysis(center, scale_factor, scale_factor, scale_factor)
    }

    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Angle<T>)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self, TransformError> {
        // Vector3をVector3Dに変換（所有権の問題を回避）
        let translation_vector3d = translation.map(|t| Vector3D::new(t.x(), t.y(), t.z()));
        let rotation_adapted = rotation.map(|(rot_center, axis, angle)| {
            let center_point = analysis_transform::ellipse_center_3d(rot_center);
            let axis_vector3d = Vector3D::new(axis.x(), axis.y(), axis.z());
            (center_point, axis_vector3d, angle)
        });

        let rotation_ref = rotation_adapted
            .as_ref()
            .map(|(center, axis, angle)| (center, axis, *angle));

        let matrix = analysis_transform::composite_ellipse_transform_3d(
            translation_vector3d.as_ref(),
            rotation_ref,
            scale,
        )?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Self, &Vector3<T>, Angle<T>)>,
        scale: Option<T>,
    ) -> Result<Self, TransformError> {
        let scale_tuple = scale.map(|s| (s, s, s));
        self.apply_composite_transform(translation, rotation, scale_tuple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ellipse() -> Ellipse3D<f64> {
        Ellipse3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,                          // semi_major_axis
            1.0,                          // semi_minor_axis
            Vector3D::new(0.0, 0.0, 1.0), // Z軸法線
            Vector3D::new(1.0, 0.0, 0.0), // X軸長軸方向
        )
        .unwrap()
    }

    fn create_center_ellipse() -> Ellipse3D<f64> {
        // 原点周辺の小さな楕円
        Ellipse3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            0.1,                          // semi_major_axis
            0.05,                         // semi_minor_axis
            Vector3D::new(0.0, 0.0, 1.0), // Z軸法線
            Vector3D::new(1.0, 0.0, 0.0), // X軸長軸方向
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let ellipse = create_test_ellipse();
        let translation = Vector3::new(3.0, 4.0, 5.0);

        let result = ellipse.translate_analysis(&translation).unwrap();

        assert!((result.center().x() - 3.0).abs() < 1e-10);
        assert!((result.center().y() - 4.0).abs() < 1e-10);
        assert!((result.center().z() - 5.0).abs() < 1e-10);
        // 軸長は変化しない
        assert!((result.semi_major_axis() - 2.0).abs() < 1e-10);
        assert!((result.semi_minor_axis() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_rotation_z() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸周り
        let angle = Angle::from_degrees(90.0);

        let result = ellipse.rotate_analysis(&center, &axis, angle).unwrap();

        // Z軸周り90度回転で長軸方向がY軸方向に変化
        assert!((result.major_axis_direction().x() - 0.0).abs() < 1e-10);
        assert!((result.major_axis_direction().y() - 1.0).abs() < 1e-10);
        assert!((result.major_axis_direction().z() - 0.0).abs() < 1e-10);

        // 軸長は保持される
        assert!((result.semi_major_axis() - 2.0).abs() < 1e-10);
        assert!((result.semi_minor_axis() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();

        let result = ellipse.scale_analysis(&center, 2.0, 3.0, 1.0).unwrap();

        // X軸方向の長軸が2倍、Y軸方向の短軸が3倍にスケール
        assert!((result.semi_major_axis() - 4.0).abs() < 1e-10); // 2.0 * 2.0
        assert!((result.semi_minor_axis() - 3.0).abs() < 1e-10); // 1.0 * 3.0
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();

        let result = ellipse.uniform_scale_analysis(&center, 1.5).unwrap();

        // 両軸とも均等にスケールされる
        assert!((result.semi_major_axis() - 3.0).abs() < 1e-10); // 2.0 * 1.5
        assert!((result.semi_minor_axis() - 1.5).abs() < 1e-10); // 1.0 * 1.5
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let ellipse = create_test_ellipse();

        // 平行移動行列（2, 3, 4移動）
        let translation_vec = Vector3::new(2.0, 3.0, 4.0);
        let matrix = Matrix4x4::translation_3d(&translation_vec);
        let result = ellipse.transform_point_matrix(&matrix);

        assert!((result.center().x() - 2.0).abs() < 1e-10);
        assert!((result.center().y() - 3.0).abs() < 1e-10);
        assert!((result.center().z() - 4.0).abs() < 1e-10);
        // 軸長は変化しない
        assert!((result.semi_major_axis() - 2.0).abs() < 1e-10);
        assert!((result.semi_minor_axis() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_multiple_ellipses() {
        let ellipses = vec![
            create_test_ellipse(),
            Ellipse3D::new(
                Point3D::new(5.0, 5.0, 5.0),
                3.0,                          // semi_major_axis
                2.0,                          // semi_minor_axis
                Vector3D::new(0.0, 1.0, 0.0), // Y軸法線
                Vector3D::new(1.0, 0.0, 0.0), // X軸長軸方向
            )
            .unwrap(),
        ];

        let translation_vec = Vector3::new(1.0, 1.0, 1.0);
        let matrix = Matrix4x4::translation_3d(&translation_vec);
        let results = analysis_transform::transform_ellipses_3d(&ellipses, &matrix);

        assert_eq!(results.len(), 2);

        // 最初の楕円
        assert!((results[0].center().x() - 1.0).abs() < 1e-10);
        assert!((results[0].center().y() - 1.0).abs() < 1e-10);
        assert!((results[0].center().z() - 1.0).abs() < 1e-10);

        // 2番目の楕円
        assert!((results[1].center().x() - 6.0).abs() < 1e-10);
        assert!((results[1].center().y() - 6.0).abs() < 1e-10);
        assert!((results[1].center().z() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();

        let result = ellipse.scale_analysis(&center, 0.0, 1.0, 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_error_handling_zero_axis() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = ellipse.rotate_analysis(&center, &zero_axis, angle);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }
}
