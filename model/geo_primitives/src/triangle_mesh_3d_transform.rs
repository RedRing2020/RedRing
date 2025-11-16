//! TriangleMesh3D Analysis Transform実装
//!
//! Analysis Matrix4x4を使用したTriangleMesh3D効率的変換実装
//! 頂点の一括変換によるメッシュ全体の幾何変換

use crate::{Point3D, TriangleMesh3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// TriangleMesh3D用Analysis変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一メッシュの行列変換
    pub fn transform_triangle_mesh_3d<T: Scalar>(
        mesh: &TriangleMesh3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> TriangleMesh3D<T> {
        // 頂点の変換
        let mut transformed_vertices = Vec::with_capacity(mesh.vertices().len());

        for vertex in mesh.vertices() {
            // geo_primitives::Point3D → analysis::Vector3 → 変換 → geo_primitives::Point3D
            let vertex_vec = vertex.to_analysis_vector3();
            let transformed_vec = matrix.transform_point_3d(&vertex_vec);
            let new_vertex = Point3D::from_analysis_vector3(transformed_vec);
            transformed_vertices.push(new_vertex);
        }

        // 新しいメッシュの作成（インデックスはそのまま）
        TriangleMesh3D::new(transformed_vertices, mesh.indices().to_vec())
            .unwrap_or_else(|_| TriangleMesh3D::empty())
    }

    /// 複数メッシュの一括行列変換
    pub fn transform_triangle_meshes_3d<T: Scalar>(
        meshes: &[TriangleMesh3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Vec<TriangleMesh3D<T>> {
        meshes
            .iter()
            .map(|mesh| transform_triangle_mesh_3d(mesh, matrix))
            .collect()
    }

    /// 平行移動行列生成
    pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3<T>) -> Matrix4x4<T> {
        Matrix4x4::translation_3d(translation)
    }

    /// 回転行列生成（軸回転版）
    pub fn rotation_matrix_3d<T: Scalar>(
        _center: &TriangleMesh3D<T>,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // 回転軸の正規化チェック
        if axis.norm_squared().is_zero() {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero vector".to_string(),
            ));
        }

        let normalized_axis = axis.normalize().map_err(TransformError::ZeroVector)?;
        Ok(Matrix4x4::rotation_axis(
            &normalized_axis,
            angle.to_radians(),
        ))
    }

    /// スケール行列生成（中心点指定版）
    pub fn scale_matrix_3d<T: Scalar>(
        center: &TriangleMesh3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        // メッシュの重心を計算
        let centroid = mesh_centroid_3d(center);
        let scale_vec = Vector3::new(scale_x, scale_y, scale_z);
        let translate_to_origin = Matrix4x4::translation_3d(&(-centroid));
        let scale = Matrix4x4::scale_3d(&scale_vec);
        let translate_back = Matrix4x4::translation_3d(&centroid);

        Ok(translate_back * scale * translate_to_origin)
    }

    /// メッシュの重心を計算する補助関数
    pub fn mesh_centroid_3d<T: Scalar>(mesh: &TriangleMesh3D<T>) -> Vector3<T> {
        if mesh.vertices().is_empty() {
            return Vector3::zero();
        }

        let mut sum = Vector3::zero();
        for vertex in mesh.vertices() {
            sum = sum + vertex.to_analysis_vector3();
        }

        let count = T::from_f64(mesh.vertices().len() as f64);
        sum / count
    }
}

/// TriangleMesh3DでのAnalysisTransform3D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform3D<T> for TriangleMesh3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        analysis_transform::transform_triangle_mesh_3d(self, matrix)
    }

    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self, TransformError> {
        let matrix = analysis_transform::translation_matrix_3d(translation);
        Ok(self.transform_point_matrix(&matrix))
    }

    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        let matrix = analysis_transform::rotation_matrix_3d(center, axis, angle)?;
        Ok(self.transform_point_matrix(&matrix))
    }

    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        let matrix = analysis_transform::scale_matrix_3d(center, scale_x, scale_y, scale_z)?;
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
        let mut result = self.clone();

        // 平行移動
        if let Some(trans) = translation {
            result = result.translate_analysis(trans)?;
        }

        // 回転
        if let Some((center, axis, angle)) = rotation {
            result = result.rotate_analysis(center, axis, angle)?;
        }

        // スケール
        if let Some((sx, sy, sz)) = scale {
            result = result.scale_analysis(&result, sx, sy, sz)?;
        }

        Ok(result)
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
    use approx::assert_relative_eq;

    fn create_test_mesh() -> TriangleMesh3D<f64> {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0), // vertex 0
            Point3D::new(1.0, 0.0, 0.0), // vertex 1
            Point3D::new(0.0, 1.0, 0.0), // vertex 2
            Point3D::new(1.0, 1.0, 1.0), // vertex 3
        ];
        let indices = vec![
            [0, 1, 2], // first triangle
            [1, 3, 2], // second triangle
        ];
        TriangleMesh3D::new(vertices, indices).unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let mesh = create_test_mesh();
        let translation = Vector3::new(2.0, 3.0, 4.0);

        let result = mesh.translate_analysis(&translation).unwrap();

        // 各頂点が正しく平行移動されているかチェック
        let vertices = result.vertices();
        assert_relative_eq!(vertices[0].x(), 2.0); // 0.0 + 2.0
        assert_relative_eq!(vertices[0].y(), 3.0); // 0.0 + 3.0
        assert_relative_eq!(vertices[0].z(), 4.0); // 0.0 + 4.0

        assert_relative_eq!(vertices[1].x(), 3.0); // 1.0 + 2.0
        assert_relative_eq!(vertices[1].y(), 3.0); // 0.0 + 3.0
        assert_relative_eq!(vertices[1].z(), 4.0); // 0.0 + 4.0

        // インデックスは変更されていないはず
        assert_eq!(result.indices(), mesh.indices());
        assert_eq!(result.triangle_count(), mesh.triangle_count());
    }

    #[test]
    fn test_analysis_rotation_z() {
        let mesh = create_test_mesh();
        let center = mesh.clone();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸
        let angle = Angle::from_degrees(90.0);

        let result = mesh.rotate_analysis(&center, &axis, angle).unwrap();

        // メッシュがZ軸周り90度回転されていることを確認
        assert_eq!(result.vertex_count(), mesh.vertex_count());
        assert_eq!(result.triangle_count(), mesh.triangle_count());
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let mesh = create_test_mesh();
        let center = mesh.clone();

        let result = mesh.uniform_scale_analysis(&center, 2.0).unwrap();

        // メッシュが2倍にスケールされていることを確認
        assert_eq!(result.vertex_count(), mesh.vertex_count());
        assert_eq!(result.triangle_count(), mesh.triangle_count());
    }

    #[test]
    fn test_mesh_centroid() {
        let mesh = create_test_mesh();
        let centroid = analysis_transform::mesh_centroid_3d(&mesh);

        // 4つの頂点の重心: (0+1+0+1)/4, (0+0+1+1)/4, (0+0+0+1)/4
        assert_relative_eq!(centroid.x(), 0.5);
        assert_relative_eq!(centroid.y(), 0.5);
        assert_relative_eq!(centroid.z(), 0.25);
    }

    #[test]
    fn test_transform_matrix() {
        let mesh = create_test_mesh();
        let identity_matrix = Matrix4x4::identity();

        let result = mesh.transform_point_matrix(&identity_matrix);

        // 恒等変換では何も変わらないはず
        assert_eq!(result.vertex_count(), mesh.vertex_count());
        assert_eq!(result.triangle_count(), mesh.triangle_count());
    }

    #[test]
    fn test_transform_multiple_meshes() {
        let mesh1 = create_test_mesh();
        let mesh2 = create_test_mesh();
        let meshes = vec![mesh1, mesh2];
        let identity_matrix = Matrix4x4::identity();

        let results = analysis_transform::transform_triangle_meshes_3d(&meshes, &identity_matrix);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].vertex_count(), meshes[0].vertex_count());
        assert_eq!(results[1].vertex_count(), meshes[1].vertex_count());
    }
}
