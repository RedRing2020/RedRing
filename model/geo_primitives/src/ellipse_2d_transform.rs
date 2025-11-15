//! Ellipse2D Analysis Matrix統合変換実装
//!
//! Analysis Matrix3x3を直接使用した効率的な楕円変換
//! 中心点変換＋軸変換による楕円パラメータ更新
//! Circle2D/Triangle2D Analysis Transform パターンを基盤とする統一実装

use crate::{Ellipse2D, Point2D, Vector2D};
use analysis::linalg::{matrix::Matrix3x3, vector::Vector2};
use geo_foundation::{AnalysisTransform2D, Angle, Scalar, TransformError};

/// Ellipse2D用Analysis Matrix3x3変換モジュール
pub mod analysis_transform {
    use super::*;

    /// 単一楕円の行列変換（中心点＋軸ベクトル変換）
    pub fn transform_ellipse_2d<T: Scalar>(
        ellipse: &Ellipse2D<T>,
        matrix: &Matrix3x3<T>,
    ) -> Ellipse2D<T> {
        // 中心点の変換
        let center_vec = Vector2::new(ellipse.center().x(), ellipse.center().y());
        let transformed_center_vec = matrix.transform_point_2d(&center_vec);
        let new_center = Point2D::new(transformed_center_vec.x(), transformed_center_vec.y());

        // 長軸・短軸ベクトルの変換
        let rotation = ellipse.rotation();
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();

        // 元の軸ベクトル（単位ベクトル）
        let major_axis_vec = Vector2::new(cos_r, sin_r);
        let minor_axis_vec = Vector2::new(-sin_r, cos_r);

        // Matrix3x3による軸ベクトル変換（平行移動成分を除去）
        let transform_direction = |v: Vector2<T>| -> Vector2<T> {
            let origin = Vector2::new(T::ZERO, T::ZERO);
            let transformed_point = matrix.transform_point_2d(&v);
            let transformed_origin = matrix.transform_point_2d(&origin);
            Vector2::new(
                transformed_point.x() - transformed_origin.x(),
                transformed_point.y() - transformed_origin.y(),
            )
        };

        let transformed_major = transform_direction(major_axis_vec);
        let transformed_minor = transform_direction(minor_axis_vec);

        // 変換後の軸長計算（スケーリング効果を反映）
        let transformed_major_length = (transformed_major.x() * transformed_major.x()
            + transformed_major.y() * transformed_major.y())
        .sqrt();
        let transformed_minor_length = (transformed_minor.x() * transformed_minor.x()
            + transformed_minor.y() * transformed_minor.y())
        .sqrt();

        let new_semi_major = ellipse.semi_major() * transformed_major_length;
        let new_semi_minor = ellipse.semi_minor() * transformed_minor_length;

        // 変換後の回転角計算
        let new_rotation = transformed_major.y().atan2(transformed_major.x());

        // 長軸・短軸の順序確認（変換により逆転する可能性）
        if new_semi_major >= new_semi_minor {
            Ellipse2D::new(new_center, new_semi_major, new_semi_minor, new_rotation).unwrap_or_else(
                || {
                    Ellipse2D::axis_aligned(Point2D::origin(), T::ONE, T::ONE / (T::ONE + T::ONE))
                        .unwrap()
                },
            )
        } else {
            // 長軸と短軸が逆転した場合、90度回転して修正
            let corrected_rotation = new_rotation + T::from_f64(std::f64::consts::PI / 2.0);
            Ellipse2D::new(
                new_center,
                new_semi_minor,
                new_semi_major,
                corrected_rotation,
            )
            .unwrap_or_else(|| {
                Ellipse2D::axis_aligned(Point2D::origin(), T::ONE, T::ONE / (T::ONE + T::ONE))
                    .unwrap()
            })
        }
    }

    /// 複数楕円の一括行列変換
    pub fn transform_ellipses_2d<T: Scalar>(
        ellipses: &[Ellipse2D<T>],
        matrix: &Matrix3x3<T>,
    ) -> Vec<Ellipse2D<T>> {
        ellipses
            .iter()
            .map(|ellipse| transform_ellipse_2d(ellipse, matrix))
            .collect()
    }

    /// 平行移動行列生成
    pub fn translation_matrix_2d<T: Scalar>(translation: &Vector2D<T>) -> Matrix3x3<T> {
        let translation_vec2 = Vector2::new(translation.x(), translation.y());
        Matrix3x3::translation_2d(&translation_vec2)
    }

    /// 回転行列生成（中心点指定版）
    pub fn rotation_matrix_2d<T: Scalar>(center: &Point2D<T>, angle: Angle<T>) -> Matrix3x3<T> {
        let center_vec2 = Vector2::new(center.x(), center.y());
        Matrix3x3::rotation_around_point_2d(&center_vec2, angle.to_radians())
    }

    /// スケール行列生成
    pub fn scale_matrix_2d<T: Scalar>(
        _center: &Point2D<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        // スケール倍率のゼロチェック
        if scale_x.is_zero() || scale_y.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        let scale_vec2 = Vector2::new(scale_x, scale_y);
        Ok(Matrix3x3::scale_2d(&scale_vec2))
    }

    /// 均等スケール行列生成
    pub fn uniform_scale_matrix_2d<T: Scalar>(
        center: &Point2D<T>,
        scale_factor: T,
    ) -> Result<Matrix3x3<T>, TransformError> {
        scale_matrix_2d(center, scale_factor, scale_factor)
    }

    /// 複合変換行列生成（平行移動+回転+スケール）
    pub fn composite_ellipse_transform_2d<T: Scalar>(
        translation: Option<&Vector2D<T>>,
        rotation: Option<(&Point2D<T>, Angle<T>)>,
        scale: Option<(T, T)>,
    ) -> Result<Matrix3x3<T>, TransformError> {
        let mut matrix = Matrix3x3::identity();

        // スケール変換（最初に適用）
        if let Some((sx, sy)) = scale {
            let origin = Point2D::origin();
            let scale_mat = scale_matrix_2d(&origin, sx, sy)?;
            matrix = matrix * scale_mat;
        }

        // 回転変換
        if let Some((rot_center, angle)) = rotation {
            let rotation_mat = rotation_matrix_2d(rot_center, angle);
            matrix = matrix * rotation_mat;
        }

        // 平行移動変換（最後に適用）
        if let Some(translation_vec) = translation {
            let translation_mat = translation_matrix_2d(translation_vec);
            matrix = matrix * translation_mat;
        }

        Ok(matrix)
    }
}

/// Ellipse2DでのAnalysisTransform2D実装（geo_foundation統一トレイト）
impl<T: Scalar> AnalysisTransform2D<T> for Ellipse2D<T> {
    type Matrix3x3 = Matrix3x3<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix_2d(&self, matrix: &Matrix3x3<T>) -> Self {
        analysis_transform::transform_ellipse_2d(self, matrix)
    }

    fn translate_analysis_2d(&self, translation: &Vector2<T>) -> Result<Self, TransformError> {
        // Vector2からVector2Dへの変換
        let vector2d = Vector2D::new(translation.x(), translation.y());
        let matrix = analysis_transform::translation_matrix_2d(&vector2d);
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn rotate_analysis_2d(&self, center: &Self, angle: Angle<T>) -> Result<Self, TransformError> {
        // Ellipse2D を Point2D として中心点を使用
        let center_point = center.center();
        let matrix = analysis_transform::rotation_matrix_2d(&center_point, angle);
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        let center_point = center.center();
        let matrix = analysis_transform::scale_matrix_2d(&center_point, scale_x, scale_y)?;
        Ok(self.transform_point_matrix_2d(&matrix))
    }

    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        let center_point = center.center();
        let matrix = analysis_transform::uniform_scale_matrix_2d(&center_point, scale_factor)?;
        Ok(self.transform_point_matrix_2d(&matrix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn create_test_ellipse() -> Ellipse2D<f64> {
        Ellipse2D::new(
            Point2D::new(0.0, 0.0),
            2.0, // semi_major
            1.0, // semi_minor
            0.0, // rotation
        )
        .unwrap()
    }

    fn create_center_ellipse() -> Ellipse2D<f64> {
        // 原点周辺の小さな楕円
        Ellipse2D::new(
            Point2D::new(0.0, 0.0),
            0.1,  // semi_major
            0.05, // semi_minor
            0.0,  // rotation
        )
        .unwrap()
    }

    #[test]
    fn test_analysis_translation() {
        let ellipse = create_test_ellipse();
        let translation = Vector2::new(3.0, 4.0);

        let result = ellipse.translate_analysis_2d(&translation).unwrap();

        assert!((result.center().x() - 3.0).abs() < 1e-10);
        assert!((result.center().y() - 4.0).abs() < 1e-10);
        // 軸長は変化しない
        assert!((result.semi_major() - 2.0).abs() < 1e-10);
        assert!((result.semi_minor() - 1.0).abs() < 1e-10);
        // 回転は変化しない
        assert!((result.rotation() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_rotation() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();
        let angle = Angle::from_degrees(90.0);

        let result = ellipse.rotate_analysis_2d(&center, angle).unwrap();

        // 回転により軸の向きが変わる
        assert!((result.rotation() - PI / 2.0).abs() < 1e-10);
        // 軸長は保持される
        assert!((result.semi_major() - 2.0).abs() < 1e-10);
        assert!((result.semi_minor() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_scale() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();

        let result = ellipse.scale_analysis_2d(&center, 2.0, 3.0).unwrap();

        // 各軸が対応する方向にスケールされる
        assert!((result.semi_major() - 4.0).abs() < 1e-10); // 2.0 * 2.0
        assert!((result.semi_minor() - 3.0).abs() < 1e-10); // 1.0 * 3.0
    }

    #[test]
    fn test_analysis_uniform_scale() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();

        let result = ellipse.uniform_scale_analysis_2d(&center, 1.5).unwrap();

        // 両軸とも均等にスケールされる
        assert!((result.semi_major() - 3.0).abs() < 1e-10); // 2.0 * 1.5
        assert!((result.semi_minor() - 1.5).abs() < 1e-10); // 1.0 * 1.5
    }

    #[test]
    fn test_analysis_matrix_transform() {
        let ellipse = create_test_ellipse();

        // 平行移動行列（2, 3移動）
        let translation_vec = Vector2::new(2.0, 3.0);
        let matrix = Matrix3x3::translation_2d(&translation_vec);
        let result = ellipse.transform_point_matrix_2d(&matrix);

        assert!((result.center().x() - 2.0).abs() < 1e-10);
        assert!((result.center().y() - 3.0).abs() < 1e-10);
        // 軸長は変化しない
        assert!((result.semi_major() - 2.0).abs() < 1e-10);
        assert!((result.semi_minor() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_analysis_multiple_ellipses() {
        let ellipses = vec![
            create_test_ellipse(),
            Ellipse2D::new(
                Point2D::new(5.0, 5.0),
                3.0,      // semi_major
                2.0,      // semi_minor
                PI / 4.0, // 45度回転
            )
            .unwrap(),
        ];

        let translation_vec = Vector2::new(1.0, 1.0);
        let matrix = Matrix3x3::translation_2d(&translation_vec);
        let results = analysis_transform::transform_ellipses_2d(&ellipses, &matrix);

        assert_eq!(results.len(), 2);

        // 最初の楕円
        assert!((results[0].center().x() - 1.0).abs() < 1e-10);
        assert!((results[0].center().y() - 1.0).abs() < 1e-10);

        // 2番目の楕円
        assert!((results[1].center().x() - 6.0).abs() < 1e-10);
        assert!((results[1].center().y() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_handling_zero_scale() {
        let ellipse = create_test_ellipse();
        let center = create_center_ellipse();

        let result = ellipse.scale_analysis_2d(&center, 0.0, 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }
}
