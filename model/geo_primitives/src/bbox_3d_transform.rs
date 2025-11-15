//! BBox3D Analysis Matrix/Vector統合変換実装
//!
//! Analysis Matrix4x4を直接使用した効率的な3D境界ボックス変換
//! Direction3D実装パターンを踏襲した統一設計
//! BBox3Dの直方体構造を保持した変換処理

use crate::{BBox3D, Point3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// BBox3D用Analysis Matrix4x4変換モジュール
pub mod analysis_transform {
    use super::*;

    /// BBox3Dの4x4行列変換（境界ボックス構造保持）
    pub fn transform_bbox_3d<T: Scalar>(
        bbox: &BBox3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<BBox3D<T>, TransformError> {
        // 境界ボックスの8つの角点を変換
        let min_point = bbox.min();
        let max_point = bbox.max();

        // 8つの角点を定義（直方体の頂点）
        let corners = [
            min_point,                                                 // 左下手前
            Point3D::new(max_point.x(), min_point.y(), min_point.z()), // 右下手前
            Point3D::new(max_point.x(), max_point.y(), min_point.z()), // 右上手前
            Point3D::new(min_point.x(), max_point.y(), min_point.z()), // 左上手前
            Point3D::new(min_point.x(), min_point.y(), max_point.z()), // 左下奥
            Point3D::new(max_point.x(), min_point.y(), max_point.z()), // 右下奥
            max_point,                                                 // 右上奥
            Point3D::new(min_point.x(), max_point.y(), max_point.z()), // 左上奥
        ];

        // 全ての角点を変換
        let mut transformed_points = Vec::with_capacity(8);
        for corner in &corners {
            let corner_vec = Vector3::new(corner.x(), corner.y(), corner.z());
            let transformed_vec = matrix.transform_point_3d(&corner_vec);
            transformed_points.push(Point3D::new(
                transformed_vec.x(),
                transformed_vec.y(),
                transformed_vec.z(),
            ));
        }

        // 変換後の点群から新しい境界ボックスを生成
        create_bbox_from_points(&transformed_points).ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create transformed BBox3D".to_string())
        })
    }

    /// 点群から境界ボックスを生成
    fn create_bbox_from_points<T: Scalar>(points: &[Point3D<T>]) -> Option<BBox3D<T>> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut max_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_y = points[0].y();
        let mut min_z = points[0].z();
        let mut max_z = points[0].z();

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
            min_z = min_z.min(point.z());
            max_z = max_z.max(point.z());
        }

        Some(BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        ))
    }

    /// 複数境界ボックスの一括4x4行列変換
    pub fn transform_bboxes_3d<T: Scalar>(
        bboxes: &[BBox3D<T>],
        matrix: &Matrix4x4<T>,
    ) -> Result<Vec<BBox3D<T>>, TransformError> {
        bboxes
            .iter()
            .map(|bbox| transform_bbox_3d(bbox, matrix))
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
}

// ============================================================================
// AnalysisTransform3D Implementation
// ============================================================================

impl<T: Scalar> AnalysisTransform3D<T> for BBox3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = BBox3D<T>;

    /// Matrix4x4による直接座標変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_bbox_3d(self, matrix).unwrap_or_else(|_| *self)
    }

    /// 平行移動変換（Analysis Vector3使用）
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::translation_matrix_3d(
            translation.x(),
            translation.y(),
            translation.z(),
        );
        analysis_transform::transform_bbox_3d(self, &matrix)
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
        analysis_transform::transform_bbox_3d(self, &combined_matrix)
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
        analysis_transform::transform_bbox_3d(self, &combined_matrix)
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
        let mut result = *self;

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
            let center = result; // 現在の境界ボックスを中心として使用
            result = result.scale_analysis(&center, sx, sy, sz)?;
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
    use crate::Point3D;

    /// テスト用BBox3D生成
    fn create_test_bbox() -> BBox3D<f64> {
        BBox3D::new(Point3D::new(-1.0, -1.0, -1.0), Point3D::new(1.0, 1.0, 1.0))
    }

    #[test]
    fn test_translation_analysis_transform() {
        let bbox = create_test_bbox();
        let translation = Vector3::new(2.0, 3.0, 4.0);

        let result = bbox.translate_analysis(&translation).unwrap();

        assert_eq!(result.min().x(), 1.0);
        assert_eq!(result.min().y(), 2.0);
        assert_eq!(result.min().z(), 3.0);
        assert_eq!(result.max().x(), 3.0);
        assert_eq!(result.max().y(), 4.0);
        assert_eq!(result.max().z(), 5.0);
        assert_eq!(result.width(), 2.0);
        assert_eq!(result.height(), 2.0);
        assert_eq!(result.depth(), 2.0);
    }

    #[test]
    fn test_rotation_analysis_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();
        let x_axis = Vector3::new(1.0, 0.0, 0.0);
        let rotation_angle = Angle::from_degrees(90.0);

        let result = bbox
            .rotate_analysis(&center_bbox, &x_axis, rotation_angle)
            .unwrap();

        // X軸回転後は同じサイズの境界ボックスになる（対称な立方体のため）
        assert!((result.width() - 2.0).abs() < 1e-10);
        assert!((result.height() - 2.0).abs() < 1e-10);
        assert!((result.depth() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_axis_rotation_analysis_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();
        let axis = Vector3::new(1.0, 1.0, 1.0); // (1,1,1)軸回転
        let rotation_angle = Angle::from_degrees(120.0);

        let result = bbox
            .rotate_analysis(&center_bbox, &axis, rotation_angle)
            .unwrap();

        // 任意軸回転により境界ボックスは拡大する（立方体が回転すると外接直方体が大きくなる）
        assert!(result.width() >= 2.0);
        assert!(result.height() >= 2.0);
        assert!(result.depth() >= 2.0);
        // 中心位置は保持される
        let center = result.center();
        assert!((center.x()).abs() < 1e-10);
        assert!((center.y()).abs() < 1e-10);
        assert!((center.z()).abs() < 1e-10);
    }

    #[test]
    fn test_scale_analysis_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.scale_analysis(&center_bbox, 2.0, 3.0, 4.0).unwrap();

        assert_eq!(result.width(), 4.0); // 2.0 * 2.0
        assert_eq!(result.height(), 6.0); // 2.0 * 3.0
        assert_eq!(result.depth(), 8.0); // 2.0 * 4.0
    }

    #[test]
    fn test_uniform_scale_analysis_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.uniform_scale_analysis(&center_bbox, 2.5).unwrap();

        assert_eq!(result.width(), 5.0); // 2.0 * 2.5
        assert_eq!(result.height(), 5.0); // 2.0 * 2.5
        assert_eq!(result.depth(), 5.0); // 2.0 * 2.5
    }

    #[test]
    fn test_matrix_direct_transform() {
        let bbox = create_test_bbox();
        let matrix = analysis_transform::translation_matrix_3d(5.0, -3.0, 2.0);

        let result = bbox.transform_point_matrix(&matrix);

        assert_eq!(result.min().x(), 4.0); // -1.0 + 5.0
        assert_eq!(result.min().y(), -4.0); // -1.0 + (-3.0)
        assert_eq!(result.min().z(), 1.0); // -1.0 + 2.0
        assert_eq!(result.max().x(), 6.0); // 1.0 + 5.0
        assert_eq!(result.max().y(), -2.0); // 1.0 + (-3.0)
        assert_eq!(result.max().z(), 3.0); // 1.0 + 2.0
    }

    #[test]
    fn test_zero_scale_error() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.scale_analysis(&center_bbox, 0.0, 1.0, 1.0);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(_) => {}
            _ => panic!("Expected InvalidScaleFactor error"),
        }
    }

    #[test]
    fn test_batch_transform() {
        let bboxes = vec![create_test_bbox(); 3];
        let matrix = analysis_transform::translation_matrix_3d(10.0, -5.0, 7.0);

        let results = analysis_transform::transform_bboxes_3d(&bboxes, &matrix).unwrap();

        assert_eq!(results.len(), 3);
        for result in results {
            assert_eq!(result.min().x(), 9.0); // -1.0 + 10.0
            assert_eq!(result.min().y(), -6.0); // -1.0 + (-5.0)
            assert_eq!(result.min().z(), 6.0); // -1.0 + 7.0
            assert_eq!(result.max().x(), 11.0); // 1.0 + 10.0
            assert_eq!(result.max().y(), -4.0); // 1.0 + (-5.0)
            assert_eq!(result.max().z(), 8.0); // 1.0 + 7.0
        }
    }

    #[test]
    fn test_composite_transform() {
        let bbox = create_test_bbox();
        let translation = Vector3::new(1.0, 1.0, 1.0);
        let center_bbox = create_test_bbox();
        let axis = Vector3::new(0.0, 0.0, 1.0); // Z軸回転
        let rotation_angle = Angle::from_degrees(90.0);
        let scale = (2.0, 2.0, 2.0);

        let result = bbox
            .apply_composite_transform(
                Some(&translation),
                Some((&center_bbox, &axis, rotation_angle)),
                Some(scale),
            )
            .unwrap();

        // 複合変換後のサイズ確認
        assert_eq!(result.width(), 4.0); // 2.0 * 2.0
        assert_eq!(result.height(), 4.0); // 2.0 * 2.0
        assert_eq!(result.depth(), 4.0); // 2.0 * 2.0
    }

    #[test]
    fn test_negative_scale_transform() {
        let bbox = create_test_bbox();
        let center_bbox = create_test_bbox();

        let result = bbox.scale_analysis(&center_bbox, -1.0, 1.0, 2.0).unwrap();

        // 負のスケールで反転
        assert_eq!(result.width(), 2.0);
        assert_eq!(result.height(), 2.0);
        assert_eq!(result.depth(), 4.0);
        let center = result.center();
        assert!((center.x()).abs() < 1e-10);
        assert!((center.y()).abs() < 1e-10);
        assert!((center.z()).abs() < 1e-10);
    }
}
