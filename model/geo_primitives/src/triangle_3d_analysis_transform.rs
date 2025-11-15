//! Triangle3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な三角形変換
//! 3頂点の一括Matrix4x4変換による最適化実装
//! Point3D/LineSegment3D Analysis Transform パターンを基盤とする統一実装

use crate::{Point3D, Triangle3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// Triangle3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一三角形の行列変換（3頂点を一括変換）
    pub fn transform_triangle_3d<T: Scalar>(
        triangle: &Triangle3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Triangle3D<T> {
        let va_vec = Vector3::new(
            triangle.vertex_a().x(),
            triangle.vertex_a().y(),
            triangle.vertex_a().z(),
        );
        let vb_vec = Vector3::new(
            triangle.vertex_b().x(),
            triangle.vertex_b().y(),
            triangle.vertex_b().z(),
        );
        let vc_vec = Vector3::new(
            triangle.vertex_c().x(),
            triangle.vertex_c().y(),
            triangle.vertex_c().z(),
        );

        // Matrix4x4による一括変換
        let transformed_va = matrix.transform_point_3d(&va_vec);
        let transformed_vb = matrix.transform_point_3d(&vb_vec);
        let transformed_vc = matrix.transform_point_3d(&vc_vec);

        let new_va = Point3D::new(transformed_va.x(), transformed_va.y(), transformed_va.z());
        let new_vb = Point3D::new(transformed_vb.x(), transformed_vb.y(), transformed_vb.z());
        let new_vc = Point3D::new(transformed_vc.x(), transformed_vc.y(), transformed_vc.z());

        Triangle3D::new(new_va, new_vb, new_vc).unwrap()
    }

    /// 複数三角形の一括行列変換
    pub fn transform_triangles_3d<T: Scalar>(
        triangles: &[Triangle3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Vec<Triangle3D<T>> {
        triangles
            .iter()
            .map(|triangle| transform_triangle_3d(triangle, matrix))
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
        // 回転軸の正規化チェック（ベクトルの大きさを手動計算）
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
    pub fn composite_triangle_transform_3d<T: Scalar>(
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

    /// 三角形の重心を計算する補助関数
    pub fn triangle_centroid_3d<T: Scalar>(triangle: &Triangle3D<T>) -> Point3D<T> {
        let vertex_a = triangle.vertex_a();
        let vertex_b = triangle.vertex_b();
        let vertex_c = triangle.vertex_c();

        // 3で除算するため、T::ONEを3つ足す
        let three = T::ONE + T::ONE + T::ONE;
        let centroid_x = (vertex_a.x() + vertex_b.x() + vertex_c.x()) / three;
        let centroid_y = (vertex_a.y() + vertex_b.y() + vertex_c.y()) / three;
        let centroid_z = (vertex_a.z() + vertex_b.z() + vertex_c.z()) / three;

        Point3D::new(centroid_x, centroid_y, centroid_z)
    }
}

/// Triangle3DでのAnalysisTransform3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform3D<T> for Triangle3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        analysis_transform::transform_triangle_3d(self, matrix)
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
        // Triangle3Dから重心を取得（3頂点の平均）
        let center_point = analysis_transform::triangle_centroid_3d(center);
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
        // Triangle3Dから重心を取得（3頂点の平均）
        let center_point = analysis_transform::triangle_centroid_3d(center);
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
            let center_point = analysis_transform::triangle_centroid_3d(rot_center);
            let axis_vector3d = Vector3D::new(axis.x(), axis.y(), axis.z());
            (center_point, axis_vector3d, angle)
        });

        let rotation_ref = rotation_adapted
            .as_ref()
            .map(|(center, axis, angle)| (center, axis, *angle));

        let matrix = analysis_transform::composite_triangle_transform_3d(
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

    fn create_test_triangle() -> Triangle3D<f64> {
        Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.5, 1.0, 0.5),
        )
        .unwrap()
    }

    fn create_center_triangle() -> Triangle3D<f64> {
        // 原点周辺の小さな正三角形
        Triangle3D::new(
            Point3D::new(-0.1, -0.1, 0.0),
            Point3D::new(0.1, -0.1, 0.0),
            Point3D::new(0.0, 0.1, 0.0),
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let triangle = create_test_triangle();
        let translation = Vector3::new(2.0, 3.0, 4.0);

        let result = triangle.translate_analysis(&translation).unwrap();

        assert!((result.vertex_a().x() - 2.0).abs() < 1e-10); // 0.0 + 2.0
        assert!((result.vertex_a().y() - 3.0).abs() < 1e-10); // 0.0 + 3.0
        assert!((result.vertex_a().z() - 4.0).abs() < 1e-10); // 0.0 + 4.0
        assert!((result.vertex_b().x() - 3.0).abs() < 1e-10); // 1.0 + 2.0
        assert!((result.vertex_b().y() - 3.0).abs() < 1e-10); // 0.0 + 3.0
        assert!((result.vertex_b().z() - 4.0).abs() < 1e-10); // 0.0 + 4.0
        assert!((result.vertex_c().x() - 2.5).abs() < 1e-10); // 0.5 + 2.0
        assert!((result.vertex_c().y() - 4.0).abs() < 1e-10); // 1.0 + 3.0
        assert!((result.vertex_c().z() - 4.5).abs() < 1e-10); // 0.5 + 4.0
    }

    #[test]
    fn test_analysis_rotation_z() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸
        let angle = Angle::from_degrees(90.0);

        let result = triangle.rotate_analysis(&center, &axis, angle).unwrap();

        // Z軸周り90度回転: (x, y, z) -> (-y, x, z)
        assert!((result.vertex_a().x() - 0.0).abs() < 1e-10);
        assert!((result.vertex_a().y() - 0.0).abs() < 1e-10);
        assert!((result.vertex_a().z() - 0.0).abs() < 1e-10);
        assert!((result.vertex_b().x() - 0.0).abs() < 1e-10);
        assert!((result.vertex_b().y() - 1.0).abs() < 1e-10);
        assert!((result.vertex_b().z() - 0.0).abs() < 1e-10);
        assert!((result.vertex_c().x() - (-1.0)).abs() < 1e-10);
        assert!((result.vertex_c().y() - 0.5).abs() < 1e-10);
        assert!((result.vertex_c().z() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();

        let result = triangle.scale_analysis(&center, 2.0, 2.0, 2.0).unwrap();

        assert!((result.vertex_a().x() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_a().y() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_a().z() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_b().x() - 2.0).abs() < 1e-10); // 1.0 * 2.0
        assert!((result.vertex_b().y() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_b().z() - 0.0).abs() < 1e-10); // 0.0 * 2.0
        assert!((result.vertex_c().x() - 1.0).abs() < 1e-10); // 0.5 * 2.0
        assert!((result.vertex_c().y() - 2.0).abs() < 1e-10); // 1.0 * 2.0
        assert!((result.vertex_c().z() - 1.0).abs() < 1e-10); // 0.5 * 2.0
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();

        let result = triangle.uniform_scale_analysis(&center, 3.0).unwrap();

        assert!((result.vertex_a().x() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_a().y() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_a().z() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_b().x() - 3.0).abs() < 1e-10); // 1.0 * 3.0
        assert!((result.vertex_b().y() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_b().z() - 0.0).abs() < 1e-10); // 0.0 * 3.0
        assert!((result.vertex_c().x() - 1.5).abs() < 1e-10); // 0.5 * 3.0
        assert!((result.vertex_c().y() - 3.0).abs() < 1e-10); // 1.0 * 3.0
        assert!((result.vertex_c().z() - 1.5).abs() < 1e-10); // 0.5 * 3.0
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let triangle = create_test_triangle();

        // 平行移動行列（1, 1, 1移動）
        let translation_vec = Vector3::new(1.0, 1.0, 1.0);
        let matrix = Matrix4x4::translation_3d(&translation_vec);
        let result = triangle.transform_point_matrix(&matrix);

        assert!((result.vertex_a().x() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_a().y() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_a().z() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_b().x() - 2.0).abs() < 1e-10); // 1.0 + 1.0
        assert!((result.vertex_b().y() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_b().z() - 1.0).abs() < 1e-10); // 0.0 + 1.0
        assert!((result.vertex_c().x() - 1.5).abs() < 1e-10); // 0.5 + 1.0
        assert!((result.vertex_c().y() - 2.0).abs() < 1e-10); // 1.0 + 1.0
        assert!((result.vertex_c().z() - 1.5).abs() < 1e-10); // 0.5 + 1.0
    }

    #[test]
    fn test_analysis_multiple_triangles() {
        let triangles = vec![
            create_test_triangle(),
            Triangle3D::new(
                Point3D::new(2.0, 2.0, 2.0),
                Point3D::new(3.0, 2.0, 2.0),
                Point3D::new(2.5, 3.0, 2.5),
            )
            .unwrap(),
        ];

        let translation_vec = Vector3::new(1.0, 1.0, 1.0);
        let matrix = Matrix4x4::translation_3d(&translation_vec);
        let results = analysis_transform::transform_triangles_3d(&triangles, &matrix);

        assert_eq!(results.len(), 2);

        // 最初の三角形
        assert!((results[0].vertex_a().x() - 1.0).abs() < 1e-10);
        assert!((results[0].vertex_a().y() - 1.0).abs() < 1e-10);
        assert!((results[0].vertex_a().z() - 1.0).abs() < 1e-10);

        // 2番目の三角形
        assert!((results[1].vertex_a().x() - 3.0).abs() < 1e-10);
        assert!((results[1].vertex_a().y() - 3.0).abs() < 1e-10);
        assert!((results[1].vertex_a().z() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();

        let result = triangle.scale_analysis(&center, 0.0, 1.0, 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_error_handling_zero_axis() {
        let triangle = create_test_triangle();
        let center = create_center_triangle();
        let zero_axis = Vector3::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = triangle.rotate_analysis(&center, &zero_axis, angle);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }
}
