//! NURBS座標変換操作
//!
//! NURBS曲線・曲面の制御点に対する座標変換（平行移動、回転、スケール）を提供します。
//! 行列変換による効率的な一括変換に対応します。
//!
//! ## 注意
//! このモジュールは**座標変換のみ**を扱います。
//! NURBS編集操作（ノット挿入、次数上昇、分割）は `operations` モジュールを参照してください。

use analysis::linalg::{
    matrix::{Matrix3x3, Matrix4x4},
    vector::{Vector2, Vector3},
};
use geo_foundation::Scalar;

/// 2Dコントロールポイントを行列で一括変換
pub fn transform_control_points_2d<T: Scalar>(
    control_points: &[Vector2<T>],
    matrix: &Matrix3x3<T>,
) -> Vec<Vector2<T>> {
    control_points
        .iter()
        .map(|cp| matrix.transform_point_2d(cp))
        .collect()
}

/// 3Dコントロールポイントを行列で一括変換
pub fn transform_control_points_3d<T: Scalar>(
    control_points: &[Vector3<T>],
    matrix: &Matrix4x4<T>,
) -> Vec<Vector3<T>> {
    matrix.transform_points_3d(control_points)
}

/// 2D回転行列の生成
pub fn rotation_matrix_2d<T: Scalar>(angle: T) -> Matrix3x3<T> {
    Matrix3x3::rotation_2d(angle)
}

/// 3D軸回転行列の生成
pub fn rotation_matrix_3d<T: Scalar>(axis: &Vector3<T>, angle: T) -> Matrix4x4<T> {
    Matrix4x4::rotation_axis(axis, angle)
}

/// 2Dスケール行列の生成
pub fn scale_matrix_2d<T: Scalar>(scale_x: T, scale_y: T) -> Matrix3x3<T> {
    let scale_vector = Vector2::new(scale_x, scale_y);
    Matrix3x3::scale_2d(&scale_vector)
}

/// 3D平行移動行列の生成
pub fn translation_matrix_3d<T: Scalar>(translation: &Vector3<T>) -> Matrix4x4<T> {
    Matrix4x4::translation_3d(translation)
}

/// 複合変換行列の構築（3D）
#[must_use]
pub fn composite_transform_3d<T: Scalar>(
    translation: Option<&Vector3<T>>,
    rotation: Option<(&Vector3<T>, T)>, // (axis, angle)
    scale: Option<T>,
) -> Matrix4x4<T> {
    let mut result = Matrix4x4::identity();

    // スケール適用
    if let Some(scale_factor) = scale {
        let scale_matrix = Matrix4x4::uniform_scale_3d(scale_factor);
        result = result * scale_matrix;
    }

    // 回転適用
    if let Some((axis, angle)) = rotation {
        let rotation_matrix = Matrix4x4::rotation_axis(axis, angle);
        result = result * rotation_matrix;
    }

    // 平行移動適用
    if let Some(trans) = translation {
        let translation_matrix = Matrix4x4::translation_3d(trans);
        result = translation_matrix * result;
    }

    result
}

/// NURBS曲線の制御点を行列変換するヘルパー
pub fn transform_nurbs_curve_2d<T: Scalar>(
    control_points: &[Vector2<T>],
    transformation: &Matrix3x3<T>,
) -> Vec<Vector2<T>> {
    transform_control_points_2d(control_points, transformation)
}

/// NURBS曲面の制御点を行列変換するヘルパー
pub fn transform_nurbs_surface_3d<T: Scalar>(
    control_points: &[Vector3<T>],
    transformation: &Matrix4x4<T>,
) -> Vec<Vector3<T>> {
    transform_control_points_3d(control_points, transformation)
}

/// 効率的な複合変換の例：回転+平行移動+スケール
pub fn apply_transform_sequence_3d<T: Scalar>(
    control_points: &[Vector3<T>],
    translation: &Vector3<T>,
    rotation_axis: &Vector3<T>,
    rotation_angle: T,
    scale_factor: T,
) -> Vec<Vector3<T>> {
    let transform_matrix = composite_transform_3d(
        Some(translation),
        Some((rotation_axis, rotation_angle)),
        Some(scale_factor),
    );
    transform_control_points_3d(control_points, &transform_matrix)
}
