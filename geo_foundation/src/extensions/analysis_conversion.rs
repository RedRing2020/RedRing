//! analysisクレートとの型変換
//!
//! geo_foundationの抽象型とanalysisの具体型間の変換を提供

use crate::Scalar;
use analysis::linalg::{Matrix3x3, Vector2, Vector3};

/// analysisクレートの Vector2 との相互変換
pub trait ToAnalysisVector2<T: Scalar> {
    /// analysis::Vector2 に変換
    fn to_analysis_vector2(&self) -> Vector2<T>;
}

/// analysisクレートの Vector2 から変換
pub trait FromAnalysisVector2<T: Scalar> {
    /// analysis::Vector2 から変換
    fn from_analysis_vector2(vector: &Vector2<T>) -> Self;
}

/// analysisクレートの Vector3 との相互変換
pub trait ToAnalysisVector3<T: Scalar> {
    /// analysis::Vector3 に変換
    fn to_analysis_vector3(&self) -> Vector3<T>;
}

/// analysisクレートの Vector3 から変換
pub trait FromAnalysisVector3<T: Scalar> {
    /// analysis::Vector3 から変換
    fn from_analysis_vector3(vector: &Vector3<T>) -> Self;
}

/// 2D 座標変換用のヘルパー関数
pub mod transform_2d {
    use super::*;

    /// 2D 平行移動行列を生成
    pub fn translation_matrix<T: Scalar>(dx: T, dy: T) -> Matrix3x3<T> {
        Matrix3x3::new(
            T::ONE,
            T::ZERO,
            dx,
            T::ZERO,
            T::ONE,
            dy,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 2D 回転行列を生成（原点中心）
    pub fn rotation_matrix<T: Scalar>(angle_rad: T) -> Matrix3x3<T> {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        Matrix3x3::new(
            cos_a,
            -sin_a,
            T::ZERO,
            sin_a,
            cos_a,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 2D スケール行列を生成（原点中心）
    pub fn scale_matrix<T: Scalar>(sx: T, sy: T) -> Matrix3x3<T> {
        Matrix3x3::new(
            sx,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            sy,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 2D 等方スケール行列を生成（原点中心）
    pub fn uniform_scale_matrix<T: Scalar>(scale: T) -> Matrix3x3<T> {
        scale_matrix(scale, scale)
    }

    /// 指定点中心の回転行列を生成
    pub fn rotation_around_point<T: Scalar>(
        center_x: T,
        center_y: T,
        angle_rad: T,
    ) -> Matrix3x3<T> {
        let translate_to_origin = translation_matrix(-center_x, -center_y);
        let rotate = rotation_matrix(angle_rad);
        let translate_back = translation_matrix(center_x, center_y);

        // 行列の合成: T_back * R * T_to_origin
        translate_back.mul_matrix(&rotate.mul_matrix(&translate_to_origin))
    }

    /// 指定点中心のスケール行列を生成
    pub fn scale_around_point<T: Scalar>(center_x: T, center_y: T, sx: T, sy: T) -> Matrix3x3<T> {
        let translate_to_origin = translation_matrix(-center_x, -center_y);
        let scale = scale_matrix(sx, sy);
        let translate_back = translation_matrix(center_x, center_y);

        // 行列の合成: T_back * S * T_to_origin
        translate_back.mul_matrix(&scale.mul_matrix(&translate_to_origin))
    }
}

/// 3D 座標変換用のヘルパー関数
pub mod transform_3d {
    use super::*;
    use analysis::linalg::Matrix4x4;

    /// 3D 平行移動行列を生成
    pub fn translation_matrix<T: Scalar>(dx: T, dy: T, dz: T) -> Matrix4x4<T> {
        Matrix4x4::new(
            T::ONE,
            T::ZERO,
            T::ZERO,
            dx,
            T::ZERO,
            T::ONE,
            T::ZERO,
            dy,
            T::ZERO,
            T::ZERO,
            T::ONE,
            dz,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 3D X軸回転行列を生成
    pub fn rotation_x_matrix<T: Scalar>(angle_rad: T) -> Matrix4x4<T> {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        Matrix4x4::new(
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            cos_a,
            -sin_a,
            T::ZERO,
            T::ZERO,
            sin_a,
            cos_a,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 3D Y軸回転行列を生成
    pub fn rotation_y_matrix<T: Scalar>(angle_rad: T) -> Matrix4x4<T> {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        Matrix4x4::new(
            cos_a,
            T::ZERO,
            sin_a,
            T::ZERO,
            T::ZERO,
            T::ONE,
            T::ZERO,
            T::ZERO,
            -sin_a,
            T::ZERO,
            cos_a,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 3D Z軸回転行列を生成
    pub fn rotation_z_matrix<T: Scalar>(angle_rad: T) -> Matrix4x4<T> {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        Matrix4x4::new(
            cos_a,
            -sin_a,
            T::ZERO,
            T::ZERO,
            sin_a,
            cos_a,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 3D 等方スケール行列を生成
    pub fn uniform_scale_matrix<T: Scalar>(scale: T) -> Matrix4x4<T> {
        Matrix4x4::new(
            scale,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            scale,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            scale,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }
}
