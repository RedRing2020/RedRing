//! Circle3D Analysis Transform実装
//!
//! Analysis Matrix4x4/Vector3を使用したCircle3D効率的変換実装
//! 中心点変換、法線ベクトル変換、半径スケーリングの統合処理

use crate::{Circle3D, Direction3D, Point3D, Vector3D};
use analysis::linalg::{matrix::Matrix4x4, vector::Vector3};
use geo_foundation::{AnalysisTransform3D, Angle, Scalar, TransformError};

/// Circle3D用Analysis Matrix/Vector変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一円の行列変換
    ///
    /// Matrix4x4による中心点・法線ベクトル・半径の統合変換
    /// 半径のスケーリングはmatrixからスケール成分を抽出して適用
    pub fn transform_circle_3d<T: Scalar>(
        circle: &Circle3D<T>,
        matrix: &Matrix4x4<T>,
    ) -> Result<Circle3D<T>, TransformError> {
        // 中心点の変換
        let center_vec: Vector3<T> = Vector3::new(
            circle.center().x(),
            circle.center().y(),
            circle.center().z(),
        );
        let transformed_center_vec = matrix.transform_point_3d(&center_vec);
        let new_center = Point3D::new(
            transformed_center_vec.x(),
            transformed_center_vec.y(),
            transformed_center_vec.z(),
        );

        // 法線ベクトルの変換（方向のみ、長さは保持）
        let normal_vec: Vector3<T> = Vector3::new(
            circle.normal().x(),
            circle.normal().y(),
            circle.normal().z(),
        );
        let transformed_normal_vec = matrix.transform_vector_3d(&normal_vec);

        // 変換後の法線ベクトルを正規化してDirection3Dに変換
        let normalized_normal = transformed_normal_vec.normalize().map_err(|_| {
            TransformError::InvalidGeometry("Failed to normalize normal vector".to_string())
        })?;
        let normal_vector = Vector3D::new(
            normalized_normal.x(),
            normalized_normal.y(),
            normalized_normal.z(),
        );
        let new_normal = Direction3D::from_vector(normal_vector).ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to normalize normal vector".to_string())
        })?;

        // 半径のスケール変換（X軸ベースでスケール係数を計算）
        let unit_x = Vector3::new(T::ONE, T::ZERO, T::ZERO);
        let scaled_x = matrix.transform_vector_3d(&unit_x);
        let scale_factor = scaled_x.norm();

        if scale_factor <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "Invalid scale factor for radius".to_string(),
            ));
        }

        let new_radius = circle.radius() * scale_factor;

        Circle3D::new(new_center, new_normal, new_radius).ok_or_else(|| {
            TransformError::InvalidGeometry("Invalid transformed circle".to_string())
        })
    }

    /// 平行移動行列の生成
    pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3<T>) -> Matrix4x4<T> {
        Matrix4x4::translation_3d(translation)
    }

    /// 軸回転行列の生成（原点中心）
    pub fn rotation_matrix_3d<T: Scalar>(
        axis: &Vector3<T>,
        angle: Angle<T>,
    ) -> Result<Matrix4x4<T>, TransformError> {
        // 軸ベクトルの正規化確認
        let axis_norm = axis.norm();
        if axis_norm <= T::ZERO {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero".to_string(),
            ));
        }

        let normalized_axis = *axis / axis_norm;

        Ok(Matrix4x4::rotation_axis(
            &normalized_axis,
            angle.to_radians(),
        ))
    }

    /// 非均等スケール行列の生成
    pub fn scale_matrix_3d<T: Scalar>(
        center: &Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_x <= T::ZERO || scale_y <= T::ZERO || scale_z <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors must be positive".to_string(),
            ));
        }

        let center_vec: Vector3<T> = Vector3::new(center.x(), center.y(), center.z());
        let scale_vec: Vector3<T> = Vector3::new(scale_x, scale_y, scale_z);

        // T * S * T^-1 の組み合わせで中心点周りのスケール
        let to_origin = Matrix4x4::translation_3d(&(-center_vec));
        let scale_matrix = Matrix4x4::scale_3d(&scale_vec);
        let back_to_center = Matrix4x4::translation_3d(&center_vec);

        Ok(back_to_center * scale_matrix * to_origin)
    }

    /// 均等スケール行列の生成
    pub fn uniform_scale_matrix_3d<T: Scalar>(
        center: &Point3D<T>,
        scale_factor: T,
    ) -> Result<Matrix4x4<T>, TransformError> {
        if scale_factor <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor must be positive".to_string(),
            ));
        }

        let center_vec: Vector3<T> = Vector3::new(center.x(), center.y(), center.z());

        // T * S * T^-1 の組み合わせで中心点周りの均等スケール
        let to_origin = Matrix4x4::translation_3d(&(-center_vec));
        let scale_matrix = Matrix4x4::uniform_scale_3d(scale_factor);
        let back_to_center = Matrix4x4::translation_3d(&center_vec);

        Ok(back_to_center * scale_matrix * to_origin)
    }
}

// AnalysisTransform3D トレイト実装
impl<T: Scalar> AnalysisTransform3D<T> for Circle3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Circle3D<T>;

    /// Matrix4x4による直接座標変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output {
        analysis_transform::transform_circle_3d(self, matrix).unwrap_or_else(|_| self.clone())
        // エラー時は元の円を返す
    }

    /// 平行移動変換
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::translation_matrix_3d(translation);
        analysis_transform::transform_circle_3d(self, &matrix)
    }

    /// 軸回転変換
    fn rotate_analysis(
        &self,
        _center: &Self, // 現在の実装では原点中心回転のみサポート
        axis: &Vector3<T>,
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::rotation_matrix_3d(axis, angle)?;
        analysis_transform::transform_circle_3d(self, &matrix)
    }

    /// 非均等スケール変換
    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, TransformError> {
        // 円の非均等スケールは楕円になるため、均等スケールのみ許可
        if !((scale_x - scale_y).abs() < T::EPSILON && (scale_y - scale_z).abs() < T::EPSILON) {
            return Err(TransformError::InvalidGeometry(
                "Non-uniform scale not supported for Circle3D".to_string(),
            ));
        }

        let matrix = analysis_transform::uniform_scale_matrix_3d(&center.center(), scale_x)?;
        analysis_transform::transform_circle_3d(self, &matrix)
    }

    /// 均等スケール変換
    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, TransformError> {
        let matrix = analysis_transform::uniform_scale_matrix_3d(&center.center(), scale_factor)?;
        analysis_transform::transform_circle_3d(self, &matrix)
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

        // スケール（均等スケールのみ）
        if let Some((sx, sy, sz)) = scale {
            if !((sx - sy).abs() < T::EPSILON && (sy - sz).abs() < T::EPSILON) {
                return Err(TransformError::InvalidGeometry(
                    "Non-uniform scale not supported for Circle3D".to_string(),
                ));
            }
            let center_circle = Circle3D::new(result.center(), result.normal(), T::ONE)
                .ok_or_else(|| {
                    TransformError::InvalidGeometry("Failed to create center circle".to_string())
                })?;
            result = result.uniform_scale_analysis(&center_circle, sx)?;
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
        let mut result = self.clone();

        // 平行移動
        if let Some(trans) = translation {
            result = result.translate_analysis(trans)?;
        }

        // 回転
        if let Some((center, axis, angle)) = rotation {
            result = result.rotate_analysis(center, axis, angle)?;
        }

        // 均等スケール
        if let Some(scale_factor) = scale {
            let center_circle = Circle3D::new(result.center(), result.normal(), T::ONE)
                .ok_or_else(|| {
                    TransformError::InvalidGeometry("Failed to create center circle".to_string())
                })?;
            result = result.uniform_scale_analysis(&center_circle, scale_factor)?;
        }

        Ok(result)
    }
}

impl<T: Scalar> Circle3D<T> {
    /// 原点中心のAnalysis平行移動
    ///
    /// Analysis Vector3による効率的な平行移動
    pub fn translate_analysis_origin(
        &self,
        translation: &Vector3<T>,
    ) -> Result<Self, TransformError> {
        self.translate_analysis(translation)
    }

    /// Z軸中心のAnalysis回転
    ///
    /// Z軸を中心とした回転の最適化版
    pub fn rotate_z_analysis_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        let origin = Point3D::origin();
        let z_axis = Vector3::new(T::ZERO, T::ZERO, T::ONE);
        let center_circle = Circle3D::new(
            origin,
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            T::ONE,
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create center circle".to_string())
        })?;
        self.rotate_analysis(&center_circle, &z_axis, angle)
    }

    /// 半径のみのスケール変換
    ///
    /// 中心点と法線を保持して半径のみをスケールする特殊変換
    pub fn scale_radius_analysis(&self, scale_factor: T) -> Result<Self, TransformError> {
        if scale_factor <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor must be positive".to_string(),
            ));
        }

        let new_radius = self.radius() * scale_factor;
        Circle3D::new(self.center(), self.normal(), new_radius)
            .ok_or_else(|| TransformError::InvalidGeometry("Invalid scaled radius".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_analysis_translation() {
        let circle = Circle3D::new(
            Point3D::new(1.0_f64, 2.0, 3.0),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            2.0,
        )
        .unwrap();
        let translation = Vector3::new(5.0_f64, 7.0, 11.0);

        let transformed = circle.translate_analysis(&translation).unwrap();

        assert!((transformed.center().x() - 6.0).abs() < 1e-10);
        assert!((transformed.center().y() - 9.0).abs() < 1e-10);
        assert!((transformed.center().z() - 14.0).abs() < 1e-10);
        assert!((transformed.radius() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_rotation() {
        let circle = Circle3D::new(
            Point3D::new(1.0_f64, 0.0, 0.0),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            2.0,
        )
        .unwrap();
        let center_circle = Circle3D::new(
            Point3D::<f64>::origin(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            1.0,
        )
        .unwrap();
        let z_axis = Vector3::new(0.0_f64, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0_f64);

        let transformed = circle
            .rotate_analysis(&center_circle, &z_axis, angle)
            .unwrap();

        // 90度回転で (1,0,0) -> (0,1,0)
        assert!((transformed.center().x() - 0.0).abs() < 1e-10);
        assert!((transformed.center().y() - 1.0).abs() < 1e-10);
        assert!((transformed.center().z() - 0.0).abs() < 1e-10);
        assert!((transformed.radius() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let circle = Circle3D::new(
            Point3D::new(2.0_f64, 4.0, 6.0),
            Direction3D::from_vector(Vector3D::unit_y()).unwrap(),
            3.0,
        )
        .unwrap();
        let center_circle = Circle3D::new(
            Point3D::<f64>::origin(),
            Direction3D::from_vector(Vector3D::unit_y()).unwrap(),
            1.0,
        )
        .unwrap();
        let scale_factor = 2.0_f64;

        let transformed = circle
            .uniform_scale_analysis(&center_circle, scale_factor)
            .unwrap();

        assert!((transformed.center().x() - 4.0).abs() < 1e-10);
        assert!((transformed.center().y() - 8.0).abs() < 1e-10);
        assert!((transformed.center().z() - 12.0).abs() < 1e-10);
        assert!((transformed.radius() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale_radius_only() {
        let circle = Circle3D::new(
            Point3D::new(1.0_f64, 2.0, 3.0),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            4.0,
        )
        .unwrap();
        let scale_factor = 1.5_f64;

        let transformed = circle.scale_radius_analysis(scale_factor).unwrap();

        // 中心点と法線は変化しない
        assert!((transformed.center().x() - 1.0).abs() < 1e-10);
        assert!((transformed.center().y() - 2.0).abs() < 1e-10);
        assert!((transformed.center().z() - 3.0).abs() < 1e-10);
        assert!((transformed.normal().x() - 1.0).abs() < 1e-10);
        // 半径のみスケール
        assert!((transformed.radius() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_multiple_circles() {
        let circles = vec![
            Circle3D::new(
                Point3D::new(0.0_f64, 0.0, 0.0),
                Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
                1.0,
            )
            .unwrap(),
            Circle3D::new(
                Point3D::new(1.0_f64, 1.0, 0.0),
                Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
                2.0,
            )
            .unwrap(),
        ];

        let translation = Vector3::new(10.0_f64, 20.0, 0.0);

        for circle in circles {
            let transformed = circle.translate_analysis(&translation).unwrap();
            assert!((transformed.center().x() - (circle.center().x() + 10.0)).abs() < 1e-10);
            assert!((transformed.center().y() - (circle.center().y() + 20.0)).abs() < 1e-10);
            assert!((transformed.radius() - circle.radius()).abs() < 1e-10);
        }
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let circle = Circle3D::new(
            Point3D::new(1.0_f64, 1.0, 1.0),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            2.0,
        )
        .unwrap();

        // Matrix変換のテスト
        let matrix = Matrix4x4::uniform_scale_3d(1.5_f64);
        let transformed = circle.transform_point_matrix(&matrix);

        assert!((transformed.center().x() - 1.5).abs() < 1e-10);
        assert!((transformed.center().y() - 1.5).abs() < 1e-10);
        assert!((transformed.center().z() - 1.5).abs() < 1e-10);
        assert!((transformed.radius() - 3.0).abs() < 1e-10); // 2.0 * 1.5
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let circle = Circle3D::new(
            Point3D::new(1.0_f64, 1.0, 1.0),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            2.0,
        )
        .unwrap();
        let center_circle = Circle3D::new(
            Point3D::<f64>::origin(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            1.0,
        )
        .unwrap();

        let result = circle.uniform_scale_analysis(&center_circle, 0.0);

        assert!(result.is_err());
        match result {
            Err(TransformError::InvalidScaleFactor(_)) => {}
            _ => panic!("Expected InvalidScaleFactor error for zero scale factor"),
        }
    }

    #[test]
    fn test_error_handling_non_uniform_scale() {
        let circle = Circle3D::new(
            Point3D::new(1.0_f64, 1.0, 1.0),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            2.0,
        )
        .unwrap();
        let center_circle = Circle3D::new(
            Point3D::<f64>::origin(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            1.0,
        )
        .unwrap();

        let result = circle.scale_analysis(&center_circle, 2.0, 3.0, 2.0); // 非均等スケール

        assert!(result.is_err());
        match result {
            Err(TransformError::InvalidGeometry(_)) => {}
            _ => panic!("Expected InvalidGeometry error for non-uniform scale"),
        }
    }
}
