//! Arc3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D円弧変換
//! Direction3D実装パターンを踏襲した統一設計
//! Arc3Dの円弧構造（中心、半径、法線、方向）を保持した変換処理

use crate::{Angle, Arc3D, Direction3D, Point3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Scalar, TransformError};

/// Arc3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// Arc3Dの4x4行列変換（円弧構造保持）
    pub fn transform_arc_3d<T: Scalar>(
        arc: &Arc3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<Arc3D<T>, TransformError> {
        // 中心点の変換
        let center_vec = Vector3::new(arc.center().x(), arc.center().y(), arc.center().z());
        let transformed_center_vec = matrix.transform_point_3d(&center_vec);
        let transformed_center = Point3D::new(
            transformed_center_vec.x(),
            transformed_center_vec.y(),
            transformed_center_vec.z(),
        );

        // 法線ベクトルの変換（回転のみを適用）
        let normal_vec = arc.normal().as_vector();
        let transformed_normal_vec = matrix.transform_vector_3d(&Vector3::new(
            normal_vec.x(),
            normal_vec.y(),
            normal_vec.z(),
        ));
        let transformed_normal = Direction3D::from_vector(crate::Vector3D::new(
            transformed_normal_vec.x(),
            transformed_normal_vec.y(),
            transformed_normal_vec.z(),
        ))
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to normalize transformed normal".to_string())
        })?;

        // 開始方向ベクトルの変換（回転のみを適用）
        let start_dir_vec = arc.start_direction().as_vector();
        let transformed_start_dir_vec = matrix.transform_vector_3d(&Vector3::new(
            start_dir_vec.x(),
            start_dir_vec.y(),
            start_dir_vec.z(),
        ));
        let transformed_start_dir = Direction3D::from_vector(crate::Vector3D::new(
            transformed_start_dir_vec.x(),
            transformed_start_dir_vec.y(),
            transformed_start_dir_vec.z(),
        ))
        .ok_or_else(|| {
            TransformError::InvalidGeometry(
                "Failed to normalize transformed start direction".to_string(),
            )
        })?;

        // スケール因子の計算（均等スケールを前提）
        // Matrix4x4からX軸スケールを取得
        let unit_x = Vector3::new(T::ONE, T::ZERO, T::ZERO);
        let transformed_x = matrix.transform_vector_3d(&unit_x);
        let scale_factor = transformed_x.norm();
        let transformed_radius = arc.radius() * scale_factor;

        if transformed_radius <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "Transformed radius is invalid".to_string(),
            ));
        }

        // 変換後の円弧を作成（角度は保持）
        Arc3D::new(
            transformed_center,
            transformed_radius,
            transformed_normal,
            transformed_start_dir,
            arc.start_angle(),
            arc.end_angle(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create transformed Arc3D".to_string())
        })
    }

    /// 複数円弧の一括4x4行列変換
    pub fn transform_arcs_3d<T: Scalar>(
        arcs: &[Arc3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Result<Vec<Arc3D<T>>, TransformError> {
        arcs.iter()
            .map(|arc| transform_arc_3d(arc, matrix))
            .collect()
    }

    /// 平行移動行列生成（3D用）
    pub fn translation_matrix_3d<T: Scalar>(dx: T, dy: T, dz: T) -> Matrix4x4<T> {
        let translation = Vector3::new(dx, dy, dz);
        Matrix4x4::translation_3d(&translation)
    }

    /// 軸回転行列生成（任意軸）
    pub fn axis_rotation_matrix_3d<T: Scalar>(axis: &Vector3<T>, angle: Angle<T>) -> Matrix4x4<T> {
        Matrix4x4::rotation_axis(axis, angle.to_radians())
    }

    /// スケール行列生成（3D用）
    pub fn scale_matrix_3d<T: Scalar>(sx: T, sy: T, sz: T) -> Matrix4x4<T> {
        let scale = Vector3::new(sx, sy, sz);
        Matrix4x4::scale_3d(&scale)
    }

    /// 均等スケール行列生成
    pub fn uniform_scale_matrix_3d<T: Scalar>(scale: T) -> Matrix4x4<T> {
        scale_matrix_3d(scale, scale, scale)
    }
}

// ============================================================================
// AnalysisTransform3D Implementation
// ============================================================================

impl<T: Scalar> AnalysisTransform3D<T> for Arc3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Arc3D<T>;

    /// Matrix4x4による直接座標変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_arc_3d(self, matrix).unwrap_or_else(|_| self.clone())
    }

    /// 平行移動変換（Analysis Vector3使用）
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::translation_matrix_3d(
            translation.x(),
            translation.y(),
            translation.z(),
        );
        analysis_transform::transform_arc_3d(self, &matrix)
    }

    /// 軸回転変換（Analysis Matrix4x4使用）
    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let center_point = center.center();
        let to_origin = analysis_transform::translation_matrix_3d(
            -center_point.x(),
            -center_point.y(),
            -center_point.z(),
        );
        let rotation = analysis_transform::axis_rotation_matrix_3d(axis, angle);
        let from_origin = analysis_transform::translation_matrix_3d(
            center_point.x(),
            center_point.y(),
            center_point.z(),
        );

        let combined_matrix = from_origin.mul_matrix(&rotation.mul_matrix(&to_origin));
        analysis_transform::transform_arc_3d(self, &combined_matrix)
    }

    /// スケール変換
    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        // 非均等スケールの場合は警告（円弧が楕円弧に変形）
        let uniform_scale = scale_x == scale_y && scale_y == scale_z;
        if !uniform_scale {
            return Err(TransformError::InvalidGeometry(
                "Non-uniform scaling would distort Arc3D into ellipse".to_string(),
            ));
        }

        let center_point = center.center();
        let to_origin = analysis_transform::translation_matrix_3d(
            -center_point.x(),
            -center_point.y(),
            -center_point.z(),
        );
        let scale = analysis_transform::scale_matrix_3d(scale_x, scale_y, scale_z);
        let from_origin = analysis_transform::translation_matrix_3d(
            center_point.x(),
            center_point.y(),
            center_point.z(),
        );

        let combined_matrix = from_origin.mul_matrix(&scale.mul_matrix(&to_origin));
        analysis_transform::transform_arc_3d(self, &combined_matrix)
    }

    /// 均等スケール変換
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
        let mut result = self.clone();

        // 平行移動
        if let Some(trans) = translation {
            result = result.translate_analysis(trans)?;
        }

        // 回転
        if let Some((center, axis, angle)) = rotation {
            result = result.rotate_analysis(center, axis, angle)?;
        }

        // スケール（均等スケールのみ許可）
        if let Some((sx, sy, sz)) = scale {
            if sx != sy || sy != sz {
                return Err(TransformError::InvalidGeometry(
                    "Arc3D requires uniform scaling".to_string(),
                ));
            }
            let center = result.clone(); // 現在の円弧を中心として使用
            result = result.uniform_scale_analysis(&center, sx)?;
        }

        Ok(result)
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};

    /// テスト用Arc3D生成（XY平面上の90度円弧）
    fn create_test_arc() -> Arc3D<f64> {
        Arc3D::xy_arc(
            Point3D::origin(),
            2.0,
            Angle::from_degrees(0.0),
            Angle::from_degrees(90.0),
        )
        .unwrap()
    }

    /// テスト用Arc3D生成（XZ平面上の180度円弧）
    fn create_xz_arc() -> Arc3D<f64> {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Direction3D::from_vector(Vector3D::unit_y()).unwrap();
        let start_dir = Direction3D::from_vector(Vector3D::unit_x()).unwrap();
        Arc3D::new(
            center,
            1.5,
            normal,
            start_dir,
            Angle::from_degrees(0.0),
            Angle::from_degrees(180.0),
        )
        .unwrap()
    }

    #[test]
    fn test_translation_analysis_transform() {
        let arc = create_test_arc();
        let translation = Vector3::new(3.0, -2.0, 5.0);

        let result = arc.translate_analysis(&translation).unwrap();

        assert_eq!(result.center().x(), 3.0);
        assert_eq!(result.center().y(), -2.0);
        assert_eq!(result.center().z(), 5.0);
        assert_eq!(result.radius(), 2.0);
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 90.0);
    }

    #[test]
    fn test_rotation_analysis_transform() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();
        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let rotation_angle = Angle::from_degrees(45.0);

        let result = arc
            .rotate_analysis(&center_arc, &z_axis, rotation_angle)
            .unwrap();

        // Z軸回転後も半径と角度範囲は保持
        assert_eq!(result.radius(), 2.0);
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 90.0);
        // 中心は回転しない（原点中心）
        assert!((result.center().x()).abs() < 1e-10);
        assert!((result.center().y()).abs() < 1e-10);
        assert!((result.center().z()).abs() < 1e-10);
    }

    #[test]
    fn test_arbitrary_axis_rotation() {
        let arc = create_xz_arc();
        let center_arc = create_xz_arc();
        let axis = Vector3::new(1.0, 0.0, 0.0); // X軸回転
        let rotation_angle = Angle::from_degrees(90.0);

        let result = arc
            .rotate_analysis(&center_arc, &axis, rotation_angle)
            .unwrap();

        // 半径と角度は保持
        assert_eq!(result.radius(), 1.5);
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 180.0);
    }

    #[test]
    fn test_uniform_scale_analysis_transform() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();

        let result = arc.uniform_scale_analysis(&center_arc, 2.5).unwrap();

        assert_eq!(result.radius(), 5.0); // 2.0 * 2.5
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 90.0);
        // 中心は原点なのでスケール後も原点
        assert!((result.center().x()).abs() < 1e-10);
        assert!((result.center().y()).abs() < 1e-10);
        assert!((result.center().z()).abs() < 1e-10);
    }

    #[test]
    fn test_non_uniform_scale_error() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();

        let result = arc.scale_analysis(&center_arc, 2.0, 1.5, 2.0);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => {}
            _ => panic!("Expected InvalidGeometry error for non-uniform scaling"),
        }
    }

    #[test]
    fn test_zero_scale_error() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();

        let result = arc.scale_analysis(&center_arc, 0.0, 1.0, 1.0);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => {}
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_matrix_direct_transform() {
        let arc = create_test_arc();
        let matrix = analysis_transform::translation_matrix_3d(-1.0, 4.0, -2.0);

        let result = arc.transform_point_matrix(&matrix);

        assert_eq!(result.center().x(), -1.0);
        assert_eq!(result.center().y(), 4.0);
        assert_eq!(result.center().z(), -2.0);
        assert_eq!(result.radius(), 2.0);
    }

    #[test]
    fn test_batch_transform() {
        let arcs = vec![create_test_arc(), create_xz_arc()];
        let matrix = analysis_transform::uniform_scale_matrix_3d(3.0);

        let results = analysis_transform::transform_arcs_3d(&arcs, &matrix).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].radius(), 6.0); // 2.0 * 3.0
        assert_eq!(results[1].radius(), 4.5); // 1.5 * 3.0
    }

    #[test]
    fn test_composite_transform() {
        let arc = create_test_arc();
        let translation = Vector3::new(2.0, 1.0, -1.0);
        let center_arc = create_test_arc();
        let axis = Vector3::new(0.0, 1.0, 0.0); // Y軸回転
        let rotation_angle = Angle::from_degrees(45.0);
        let scale = 1.5;

        let result = arc
            .apply_composite_transform_uniform(
                Some(&translation),
                Some((&center_arc, &axis, rotation_angle)),
                Some(scale),
            )
            .unwrap();

        // 最終的な半径はスケール済み
        assert!((result.radius() - 3.0).abs() < 1e-10); // 2.0 * 1.5
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 90.0);
    }

    #[test]
    fn test_negative_scale_transform() {
        let arc = create_test_arc();
        let center_arc = create_test_arc();

        let result = arc.uniform_scale_analysis(&center_arc, -2.0).unwrap();

        // 負のスケールでも半径は正
        assert_eq!(result.radius(), 4.0); // abs(-2.0) * 2.0
        assert_eq!(result.start_angle().to_degrees(), 0.0);
        assert_eq!(result.end_angle().to_degrees(), 90.0);
    }

    #[test]
    fn test_composite_non_uniform_scale_error() {
        let arc = create_test_arc();
        let translation = Vector3::new(1.0, 1.0, 1.0);

        let result = arc.apply_composite_transform(
            Some(&translation),
            None,
            Some((2.0, 1.0, 2.0)), // 非均等スケール
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(_) => {}
            _ => panic!("Expected InvalidGeometry error for non-uniform scaling"),
        }
    }
}
