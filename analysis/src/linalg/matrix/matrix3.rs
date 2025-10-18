//! 3x3行列（高速演算用）
//!
//! 3D変換、回転、投影に特化した固定サイズ行列
//! CAD計算とグラフィックス処理の両方に対応
use crate::abstract_types::Scalar;
use crate::linalg::vector::{Vector2, Vector3};
use std::ops::{Add, Mul};

/// 2Dアフィン変換の分解結果
/// (translation, rotation_angle, scale, shear)
type AffineComponents2D<T> = (Vector2<T>, T, Vector2<T>, Vector2<T>);

/// 3x3行列
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3<T: Scalar> {
    pub data: [[T; 3]; 3],
}

impl<T: Scalar> Matrix3x3<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(a11: T, a12: T, a13: T, a21: T, a22: T, a23: T, a31: T, a32: T, a33: T) -> Self {
        Self {
            data: [[a11, a12, a13], [a21, a22, a23], [a31, a32, a33]],
        }
    }

    pub fn zeros() -> Self {
        Self::new(
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
        )
    }

    pub fn identity() -> Self {
        Self::new(
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    pub fn determinant(&self) -> T {
        let a11 = self.data[0][0];
        let a12 = self.data[0][1];
        let a13 = self.data[0][2];
        let a21 = self.data[1][0];
        let a22 = self.data[1][1];
        let a23 = self.data[1][2];
        let a31 = self.data[2][0];
        let a32 = self.data[2][1];
        let a33 = self.data[2][2];

        a11 * (a22 * a33 - a23 * a32) - a12 * (a21 * a33 - a23 * a31)
            + a13 * (a21 * a32 - a22 * a31)
    }

    pub fn trace(&self) -> T {
        self.data[0][0] + self.data[1][1] + self.data[2][2]
    }

    pub fn transpose(&self) -> Self {
        Self::new(
            self.data[0][0],
            self.data[1][0],
            self.data[2][0],
            self.data[0][1],
            self.data[1][1],
            self.data[2][1],
            self.data[0][2],
            self.data[1][2],
            self.data[2][2],
        )
    }

    pub fn inverse(&self) -> Result<Self, String> {
        let det = self.determinant();
        if det.is_zero() {
            return Err("Matrix is singular".to_string());
        }

        // Adjugate matrix / determinant
        let inv_det = T::ONE / det;

        Ok(Self::new(
            (self.data[1][1] * self.data[2][2] - self.data[1][2] * self.data[2][1]) * inv_det,
            (self.data[0][2] * self.data[2][1] - self.data[0][1] * self.data[2][2]) * inv_det,
            (self.data[0][1] * self.data[1][2] - self.data[0][2] * self.data[1][1]) * inv_det,
            (self.data[1][2] * self.data[2][0] - self.data[1][0] * self.data[2][2]) * inv_det,
            (self.data[0][0] * self.data[2][2] - self.data[0][2] * self.data[2][0]) * inv_det,
            (self.data[0][2] * self.data[1][0] - self.data[0][0] * self.data[1][2]) * inv_det,
            (self.data[1][0] * self.data[2][1] - self.data[1][1] * self.data[2][0]) * inv_det,
            (self.data[0][1] * self.data[2][0] - self.data[0][0] * self.data[2][1]) * inv_det,
            (self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]) * inv_det,
        ))
    }

    pub fn mul_vector(&self, vec: &Vector3<T>) -> Vector3<T> {
        Vector3::new(
            self.data[0][0] * vec.x() + self.data[0][1] * vec.y() + self.data[0][2] * vec.z(),
            self.data[1][0] * vec.x() + self.data[1][1] * vec.y() + self.data[1][2] * vec.z(),
            self.data[2][0] * vec.x() + self.data[2][1] * vec.y() + self.data[2][2] * vec.z(),
        )
    }

    pub fn mul_matrix(&self, other: &Self) -> Self {
        let mut result = Self::zeros();
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result.data[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        result
    }

    /// 行列の要素にアクセス
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[row][col]
    }

    /// 行列の要素を設定
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.data[row][col] = value;
    }

    /// フロベニウスノルム
    pub fn frobenius_norm(&self) -> T {
        let mut sum = T::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                sum += self.data[i][j] * self.data[i][j];
            }
        }
        sum.sqrt()
    }

    /// X軸周りの回転行列（ラジアン）
    pub fn rotation_x(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            cos_a,
            -sin_a,
            T::ZERO,
            sin_a,
            cos_a,
        )
    }

    /// Y軸周りの回転行列（ラジアン）
    pub fn rotation_y(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            cos_a,
            T::ZERO,
            sin_a,
            T::ZERO,
            T::ONE,
            T::ZERO,
            -sin_a,
            T::ZERO,
            cos_a,
        )
    }

    /// Z軸周りの回転行列（ラジアン）
    pub fn rotation_z(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
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

    /// スケール行列を作成
    pub fn scale(sx: T, sy: T, sz: T) -> Self {
        Self::new(
            sx,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            sy,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            sz,
        )
    }

    /// 平行移動行列を作成（同次座標用）
    pub fn translation(tx: T, ty: T) -> Self {
        Self::new(
            T::ONE,
            T::ZERO,
            tx,
            T::ZERO,
            T::ONE,
            ty,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    // === 2D変換メソッド（Vector2対応） ===

    /// 2Dベクトルを同次座標として変換（点として扱う）
    /// Vector2を(x, y, 1)として扱い、3x3行列で変換
    pub fn transform_point_2d(&self, point: &Vector2<T>) -> Vector2<T> {
        let x = self.data[0][0] * point.x() + self.data[0][1] * point.y() + self.data[0][2];
        let y = self.data[1][0] * point.x() + self.data[1][1] * point.y() + self.data[1][2];
        Vector2::new(x, y)
    }

    /// 2Dベクトルを方向ベクトルとして変換（移動なし）
    /// Vector2を(x, y, 0)として扱い、回転・スケールのみ適用
    pub fn transform_vector_2d(&self, vector: &Vector2<T>) -> Vector2<T> {
        let x = self.data[0][0] * vector.x() + self.data[0][1] * vector.y();
        let y = self.data[1][0] * vector.x() + self.data[1][1] * vector.y();
        Vector2::new(x, y)
    }

    /// 2D点の配列を一括変換
    pub fn transform_points_2d(&self, points: &[Vector2<T>]) -> Vec<Vector2<T>> {
        points.iter().map(|p| self.transform_point_2d(p)).collect()
    }

    /// 2D方向ベクトルの配列を一括変換
    pub fn transform_vectors_2d(&self, vectors: &[Vector2<T>]) -> Vec<Vector2<T>> {
        vectors
            .iter()
            .map(|v| self.transform_vector_2d(v))
            .collect()
    }

    // === 2D変換行列の構築メソッド ===

    /// Vector2による平行移動行列を作成
    pub fn translation_2d(translation: &Vector2<T>) -> Self {
        Self::translation(translation.x(), translation.y())
    }

    /// 原点中心の2D回転行列を作成
    pub fn rotation_2d(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
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

    /// Vector2による2Dスケール行列を作成
    pub fn scale_2d(scale: &Vector2<T>) -> Self {
        Self::new(
            scale.x(),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            scale.y(),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 均等2Dスケール行列を作成
    pub fn uniform_scale_2d(scale: T) -> Self {
        Self::scale_2d(&Vector2::new(scale, scale))
    }

    /// 指定点中心の2D回転行列を作成
    pub fn rotation_around_point_2d(center: &Vector2<T>, angle: T) -> Self {
        let translate_to_origin = Self::translation_2d(&(-*center));
        let rotation = Self::rotation_2d(angle);
        let translate_back = Self::translation_2d(center);
        translate_back * rotation * translate_to_origin
    }

    /// 複合2D変換行列を作成（TRS: Translation, Rotation, Scale順）
    pub fn trs_2d(translation: &Vector2<T>, rotation: T, scale: &Vector2<T>) -> Self {
        let t = Self::translation_2d(translation);
        let r = Self::rotation_2d(rotation);
        let s = Self::scale_2d(scale);
        t * r * s
    }

    // === 2D変換の抽出・分解メソッド ===

    /// 2D変換行列から平行移動成分を抽出
    pub fn extract_translation_2d(&self) -> Vector2<T> {
        Vector2::new(self.data[0][2], self.data[1][2])
    }

    /// 2D変換行列から回転角を抽出（ラジアン）
    /// スケールが適用されている場合は正確でない可能性があります
    pub fn extract_rotation_2d(&self) -> T {
        self.data[1][0].atan2(self.data[0][0])
    }

    /// 2D変換行列からスケールを抽出
    pub fn extract_scale_2d(&self) -> Vector2<T> {
        let scale_x =
            (self.data[0][0] * self.data[0][0] + self.data[1][0] * self.data[1][0]).sqrt();
        let scale_y =
            (self.data[0][1] * self.data[0][1] + self.data[1][1] * self.data[1][1]).sqrt();
        Vector2::new(scale_x, scale_y)
    }

    /// 2D変換行列を分解（Translation, Rotation, Scale）
    pub fn decompose_2d(&self) -> (Vector2<T>, T, Vector2<T>) {
        let translation = self.extract_translation_2d();
        let scale = self.extract_scale_2d();

        // スケールを除去して回転を抽出
        let rotation_matrix = Self::new(
            self.data[0][0] / scale.x(),
            self.data[0][1] / scale.y(),
            T::ZERO,
            self.data[1][0] / scale.x(),
            self.data[1][1] / scale.y(),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        );
        let rotation = rotation_matrix.extract_rotation_2d();

        (translation, rotation, scale)
    }

    /// 2D変換が剛体変換（回転+移動のみ）かどうかを判定
    pub fn is_rigid_2d(&self) -> bool {
        let scale = self.extract_scale_2d();
        (scale.x() - T::ONE).abs() < T::EPSILON && (scale.y() - T::ONE).abs() < T::EPSILON
    }

    /// 2D変換が均等スケールかどうかを判定
    pub fn is_uniform_scale_2d(&self) -> bool {
        let scale = self.extract_scale_2d();
        (scale.x() - scale.y()).abs() < T::EPSILON
    }

    // === 2D幾何学的操作 ===

    /// 2D反射行列を作成（X軸）
    pub fn reflection_x_2d() -> Self {
        Self::new(
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            -T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 2D反射行列を作成（Y軸）
    pub fn reflection_y_2d() -> Self {
        Self::new(
            -T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 任意の直線に対する2D反射行列を作成
    /// line_point: 直線上の点, line_direction: 直線の方向ベクトル
    pub fn reflection_line_2d(
        line_point: &Vector2<T>,
        line_direction: &Vector2<T>,
    ) -> Result<Self, String> {
        let normalized_dir = line_direction.normalize()?;
        let normal = normalized_dir.perpendicular();

        // 反射行程: I - 2 * n * n^T (nは正規化された法線)
        let nx = normal.x();
        let ny = normal.y();

        let reflection = Self::new(
            T::ONE - T::from_f64(2.0) * nx * nx,
            -T::from_f64(2.0) * nx * ny,
            T::ZERO,
            -T::from_f64(2.0) * nx * ny,
            T::ONE - T::from_f64(2.0) * ny * ny,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        );

        // 直線が原点を通らない場合の調整
        let translated_point = reflection.transform_point_2d(line_point);
        let correction = *line_point - translated_point;
        let final_translation = Self::translation_2d(&correction);

        Ok(final_translation * reflection)
    }

    /// 2Dせん断変換行列を作成
    pub fn shear_2d(shear_x: T, shear_y: T) -> Self {
        Self::new(
            T::ONE,
            shear_x,
            T::ZERO,
            shear_y,
            T::ONE,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    // === 3D空間での2D変換サポート ===

    /// 3DベクトルのXY成分に2D変換を適用（Z成分は保持）
    pub fn transform_vector3_as_2d(&self, vector: &Vector3<T>) -> Vector3<T> {
        let transformed_2d = self.transform_point_2d(&Vector2::new(vector.x(), vector.y()));
        Vector3::new(transformed_2d.x(), transformed_2d.y(), vector.z())
    }

    /// 3Dベクトルの配列のXY成分に2D変換を適用
    pub fn transform_vectors3_as_2d(&self, vectors: &[Vector3<T>]) -> Vec<Vector3<T>> {
        vectors
            .iter()
            .map(|v| self.transform_vector3_as_2d(v))
            .collect()
    }

    // === 便利メソッド・ユーティリティ ===

    /// 2D変換行列の逆変換を計算（より効率的な実装）
    pub fn inverse_2d(&self) -> Result<Self, String> {
        // 2D同次変換行列の逆変換は構造を利用して効率化可能
        let (translation, rotation, scale) = self.decompose_2d();

        // 各成分の逆変換
        let inv_scale = Vector2::new(T::ONE / scale.x(), T::ONE / scale.y());
        let inv_rotation = -rotation;
        let inv_translation = -translation;

        // 逆順で合成 (TRS)^-1 = S^-1 * R^-1 * T^-1
        let s_inv = Self::scale_2d(&inv_scale);
        let r_inv = Self::rotation_2d(inv_rotation);
        let t_inv = Self::translation_2d(&inv_translation);

        Ok(s_inv * r_inv * t_inv)
    }

    /// 2D変換行列の行列式（面積スケールファクター）
    pub fn determinant_2d(&self) -> T {
        self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]
    }

    /// 2D変換行列が向きを保持するかどうか（行列式が正）
    pub fn preserves_orientation_2d(&self) -> bool {
        self.determinant_2d() > T::ZERO
    }

    // === アフィン変換・同次座標系の判定 ===

    /// 同次座標系でのアフィン変換行列かどうかを判定
    /// 3x3行列の最下行が [0, 0, 1] であることを確認
    pub fn is_affine_transform(&self) -> bool {
        self.data[2][0].is_zero()
            && self.data[2][1].is_zero()
            && (self.data[2][2] - T::ONE).abs() < T::EPSILON
    }

    /// 同次座標系での射影変換（perspective）要素があるかを判定
    /// 最下行に0以外の値がある場合は射影変換
    pub fn has_perspective(&self) -> bool {
        !self.data[2][0].is_zero() || !self.data[2][1].is_zero()
    }

    /// 純粋な2Dアフィン変換かどうかを判定（射影要素なし、スケールファクター=1）
    pub fn is_pure_affine_2d(&self) -> bool {
        self.is_affine_transform() && self.data[2][2] == T::ONE
    }

    /// 同次座標系での正規化（最下行右下を1にスケール）
    pub fn normalize_homogeneous(&self) -> Result<Self, String> {
        let w = self.data[2][2];
        if w.is_zero() {
            return Err("Cannot normalize: bottom-right element is zero".to_string());
        }

        if (w - T::ONE).abs() < T::EPSILON {
            // 既に正規化済み
            return Ok(*self);
        }

        Ok(Self::new(
            self.data[0][0] / w,
            self.data[0][1] / w,
            self.data[0][2] / w,
            self.data[1][0] / w,
            self.data[1][1] / w,
            self.data[1][2] / w,
            self.data[2][0] / w,
            self.data[2][1] / w,
            T::ONE,
        ))
    }

    /// アフィン変換行列として有効かどうかを検証
    pub fn is_valid_affine(&self) -> bool {
        // 最下行が [0, 0, 1] であること
        if !self.is_affine_transform() {
            return false;
        }

        // 行列式が0でないこと（可逆性）
        let det = self.determinant();
        if det.is_zero() {
            return false;
        }

        // 数値的に安定していること（極端に大きな値がない）
        let max_element = self
            .data
            .iter()
            .flat_map(|row| row.iter())
            .fold(T::ZERO, |acc, &x| acc.max(x.abs()));

        // 実用的な範囲内の値であること
        max_element < T::from_f64(1e12)
    }

    /// 2Dアフィン変換の線形部分を抽出（3x3行列の左上2x2部分）
    pub fn linear_part_2d(&self) -> [[T; 2]; 2] {
        [
            [self.data[0][0], self.data[0][1]],
            [self.data[1][0], self.data[1][1]],
        ]
    }

    /// アフィン変換行列から2D変換成分を抽出
    pub fn extract_affine_components_2d(&self) -> Result<AffineComponents2D<T>, String> {
        if !self.is_affine_transform() {
            return Err("Matrix is not a valid affine transformation".to_string());
        }

        let translation = self.extract_translation_2d();
        let linear = self.linear_part_2d();

        // 線形部分からスケール、回転、せん断を分解
        let scale_x = (linear[0][0] * linear[0][0] + linear[1][0] * linear[1][0]).sqrt();
        let scale_y_with_shear = (linear[0][1] * linear[0][1] + linear[1][1] * linear[1][1]).sqrt();

        // 回転角の計算
        let rotation = linear[1][0].atan2(linear[0][0]);

        // せん断の計算
        let shear_x =
            (linear[0][0] * linear[0][1] + linear[1][0] * linear[1][1]) / (scale_x * scale_x);

        // Y方向のスケール（せん断の影響を除去）
        let scale_y = scale_y_with_shear; // 簡略化

        let scale = Vector2::new(scale_x, scale_y);
        let shear = Vector2::new(shear_x, T::ZERO); // Y方向のせん断は通常0

        Ok((translation, rotation, scale, shear))
    }

    // === アフィン変換専用の構築メソッド ===

    /// 一般的なアフィン変換行列を作成
    /// linear_transform: 2x2線形変換行列, translation: 平行移動ベクトル
    pub fn affine_2d(linear_transform: [[T; 2]; 2], translation: Vector2<T>) -> Self {
        Self::new(
            linear_transform[0][0],
            linear_transform[0][1],
            translation.x(),
            linear_transform[1][0],
            linear_transform[1][1],
            translation.y(),
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 完全な2Dアフィン変換行列を作成（Translation + Rotation + Scale + Shear）
    pub fn full_affine_2d(
        translation: Vector2<T>,
        rotation: T,
        scale: Vector2<T>,
        shear: Vector2<T>,
    ) -> Self {
        // 各変換行列を構築
        let t = Self::translation_2d(&translation);
        let r = Self::rotation_2d(rotation);
        let s = Self::scale_2d(&scale);
        let sh = Self::shear_2d(shear.x(), shear.y());

        // 合成: T * R * S * Sh
        t * r * s * sh
    }

    /// 同次座標による点の変換（w成分も計算）
    pub fn transform_homogeneous_2d(&self, point: Vector2<T>) -> (Vector2<T>, T) {
        let x = self.data[0][0] * point.x() + self.data[0][1] * point.y() + self.data[0][2];
        let y = self.data[1][0] * point.x() + self.data[1][1] * point.y() + self.data[1][2];
        let w = self.data[2][0] * point.x() + self.data[2][1] * point.y() + self.data[2][2];

        (Vector2::new(x, y), w)
    }

    /// 射影変換による点の変換（w成分で正規化）
    pub fn transform_projective_2d(&self, point: Vector2<T>) -> Result<Vector2<T>, String> {
        let (transformed, w) = self.transform_homogeneous_2d(point);

        if w.is_zero() {
            return Err("Point at infinity".to_string());
        }

        Ok(Vector2::new(transformed.x() / w, transformed.y() / w))
    }

    /// 2D点が変換後にどの象限に移動するかを判定
    pub fn transform_quadrant(&self, point: &Vector2<T>) -> (i32, i32) {
        let transformed = self.transform_point_2d(point);
        let quad_x = if transformed.x() >= T::ZERO { 1 } else { -1 };
        let quad_y = if transformed.y() >= T::ZERO { 1 } else { -1 };
        (quad_x, quad_y)
    }

    /// 2D境界ボックスを変換
    pub fn transform_bounding_box_2d(
        &self,
        min: &Vector2<T>,
        max: &Vector2<T>,
    ) -> (Vector2<T>, Vector2<T>) {
        let corners = [
            *min,
            Vector2::new(max.x(), min.y()),
            *max,
            Vector2::new(min.x(), max.y()),
        ];

        let transformed_corners: Vec<Vector2<T>> = corners
            .iter()
            .map(|corner| self.transform_point_2d(corner))
            .collect();

        let mut new_min = transformed_corners[0];
        let mut new_max = transformed_corners[0];

        for corner in &transformed_corners[1..] {
            new_min = new_min.min(corner);
            new_max = new_max.max(corner);
        }

        (new_min, new_max)
    }

    /// 複合2D変換行列を作成（SRT: Scale, Rotation, Translation順）
    pub fn srt_2d(scale: &Vector2<T>, rotation: T, translation: &Vector2<T>) -> Self {
        let s = Self::scale_2d(scale);
        let r = Self::rotation_2d(rotation);
        let t = Self::translation_2d(translation);
        t * r * s
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Matrix3x3<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let mut result = Self::zeros();
        for i in 0..3 {
            for j in 0..3 {
                result.data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        result
    }
}

impl<T: Scalar> Mul<T> for Matrix3x3<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        let mut result = Self::zeros();
        for i in 0..3 {
            for j in 0..3 {
                result.data[i][j] = self.data[i][j] * scalar;
            }
        }
        result
    }
}

impl<T: Scalar> Mul for Matrix3x3<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        self.mul_matrix(&other)
    }
}

// Vector2 との演算子オーバーロード
impl<T: Scalar> Mul<Vector2<T>> for Matrix3x3<T> {
    type Output = Vector2<T>;
    fn mul(self, vector: Vector2<T>) -> Self::Output {
        self.transform_point_2d(&vector)
    }
}

// Vector3 との既存の演算子オーバーロードを明示的に追加
impl<T: Scalar> Mul<Vector3<T>> for Matrix3x3<T> {
    type Output = Vector3<T>;
    fn mul(self, vector: Vector3<T>) -> Self::Output {
        self.mul_vector(&vector)
    }
}

/// 型エイリアス
pub type Matrix3x3f = Matrix3x3<f32>;
pub type Matrix3x3d = Matrix3x3<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::vector::Vector2;
    use std::f64::consts::PI;

    #[test]
    fn test_2d_translation() {
        let translation = Vector2::new(5.0, 3.0);
        let matrix = Matrix3x3::translation_2d(&translation);
        let point = Vector2::new(1.0, 2.0);
        let result = matrix.transform_point_2d(&point);

        assert!((result.x() - 6.0).abs() < f64::EPSILON);
        assert!((result.y() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_2d_rotation() {
        let angle = PI / 2.0; // 90度回転
        let matrix = Matrix3x3::rotation_2d(angle);
        let point = Vector2::new(1.0, 0.0);
        let result = matrix.transform_point_2d(&point);

        // (1,0) を90度回転すると (0,1) になる
        assert!(result.x().abs() < f64::EPSILON);
        assert!((result.y() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_2d_scale() {
        let scale = Vector2::new(2.0, 3.0);
        let matrix = Matrix3x3::scale_2d(&scale);
        let point = Vector2::new(1.0, 1.0);
        let result = matrix.transform_point_2d(&point);

        assert!((result.x() - 2.0).abs() < f64::EPSILON);
        assert!((result.y() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_2d_trs_composition() {
        let translation = Vector2::new(10.0, 5.0);
        let rotation = PI / 4.0; // 45度回転
        let scale = Vector2::new(2.0, 2.0);

        let matrix = Matrix3x3::trs_2d(&translation, rotation, &scale);
        let point = Vector2::new(1.0, 0.0);
        let result = matrix.transform_point_2d(&point);

        // 複合変換の結果を確認
        // スケール -> 回転 -> 移動の順序
        let expected_x = 10.0 + 2.0 * (PI / 4.0).cos();
        let expected_y = 5.0 + 2.0 * (PI / 4.0).sin();

        assert!((result.x() - expected_x).abs() < 1e-10);
        assert!((result.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_vector_multiplication_operator() {
        let translation = Vector2::new(2.0, 3.0);
        let matrix = Matrix3x3::translation_2d(&translation);
        let point = Vector2::new(1.0, 1.0);

        // 演算子オーバーロードのテスト
        let result = matrix * point;

        assert!((result.x() - 3.0).abs() < f64::EPSILON);
        assert!((result.y() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rigid_body_detection() {
        let translation = Vector2::new(5.0, 3.0);
        let rotation = PI / 4.0;
        let rigid_matrix =
            Matrix3x3::translation_2d(&translation) * Matrix3x3::rotation_2d(rotation);

        assert!(rigid_matrix.is_rigid_2d());

        let scale = Vector2::new(2.0, 2.0);
        let scaled_matrix = Matrix3x3::scale_2d(&scale);
        assert!(!scaled_matrix.is_rigid_2d());
    }

    #[test]
    fn test_affine_transform_detection() {
        // 標準的なアフィン変換行列
        let affine_matrix =
            Matrix3x3::trs_2d(&Vector2::new(5.0, 3.0), PI / 4.0, &Vector2::new(2.0, 1.5));

        assert!(affine_matrix.is_affine_transform());
        assert!(affine_matrix.is_pure_affine_2d());
        assert!(affine_matrix.is_valid_affine());
        assert!(!affine_matrix.has_perspective());

        // 射影変換要素を持つ行列
        let perspective_matrix = Matrix3x3::new(
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.05, 1.0, // 射影変換要素
        );

        assert!(!perspective_matrix.is_affine_transform());
        assert!(perspective_matrix.has_perspective());
    }

    #[test]
    fn test_homogeneous_normalization() {
        let matrix = Matrix3x3::new(2.0, 0.0, 4.0, 0.0, 2.0, 6.0, 0.0, 0.0, 2.0);

        let normalized = matrix.normalize_homogeneous().unwrap();
        let expected = Matrix3x3::new(1.0, 0.0, 2.0, 0.0, 1.0, 3.0, 0.0, 0.0, 1.0);

        for i in 0..3 {
            for j in 0..3 {
                assert!((normalized.get(i, j) - expected.get(i, j)).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_projective_transformation() {
        let matrix = Matrix3x3::new(
            1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.1, 0.0, 1.0, // 射影変換
        );

        let point = Vector2::new(1.0, 1.0);
        let result = matrix.transform_projective_2d(point).unwrap();

        // 射影変換後の正規化された座標
        let expected_x = 2.0 / 1.1; // (1*1 + 0*1 + 1) / (0.1*1 + 0*1 + 1)
        let expected_y = 2.0 / 1.1; // (0*1 + 1*1 + 1) / (0.1*1 + 0*1 + 1)

        assert!((result.x() - expected_x).abs() < 1e-10);
        assert!((result.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_affine_factory_methods() {
        // 線形変換行列とベクトル
        let linear = [[2.0, 0.5], [0.0, 1.5]];
        let translation = Vector2::new(3.0, 4.0);

        let affine_matrix = Matrix3x3::affine_2d(linear, translation);

        assert!(affine_matrix.is_affine_transform());
        assert_eq!(affine_matrix.get(0, 0), 2.0);
        assert_eq!(affine_matrix.get(0, 1), 0.5);
        assert_eq!(affine_matrix.get(0, 2), 3.0);
        assert_eq!(affine_matrix.get(1, 0), 0.0);
        assert_eq!(affine_matrix.get(1, 1), 1.5);
        assert_eq!(affine_matrix.get(1, 2), 4.0);
        assert_eq!(affine_matrix.get(2, 0), 0.0);
        assert_eq!(affine_matrix.get(2, 1), 0.0);
        assert_eq!(affine_matrix.get(2, 2), 1.0);
    }
}
