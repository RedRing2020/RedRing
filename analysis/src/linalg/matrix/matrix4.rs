//! 4x4行列（3D変換・プロジェクション用）
//!
//! 3D変換、射影変換、カメラ変換のための専用行列
//! OpenGL/DirectX互換の行列演算を提供
use crate::abstract_types::Scalar;
use crate::linalg::quaternion::Quaternion;
use crate::linalg::vector::{Vector3, Vector4};
use std::ops::{Add, Mul};

/// TQS分解の結果を表す型エイリアス（Translation, Quaternion, Scale）
pub type TqsDecomposition<T> = (Vector3<T>, Quaternion<T>, Vector3<T>);

/// 4x4行列
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4<T: Scalar> {
    pub data: [[T; 4]; 4],
}

impl<T: Scalar> Matrix4x4<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        a11: T,
        a12: T,
        a13: T,
        a14: T,
        a21: T,
        a22: T,
        a23: T,
        a24: T,
        a31: T,
        a32: T,
        a33: T,
        a34: T,
        a41: T,
        a42: T,
        a43: T,
        a44: T,
    ) -> Self {
        Self {
            data: [
                [a11, a12, a13, a14],
                [a21, a22, a23, a24],
                [a31, a32, a33, a34],
                [a41, a42, a43, a44],
            ],
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
            T::ZERO,
            T::ONE,
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

    pub fn transpose(&self) -> Self {
        let mut result = Self::zeros();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = self.data[j][i];
            }
        }
        result
    }

    pub fn trace(&self) -> T {
        self.data[0][0] + self.data[1][1] + self.data[2][2] + self.data[3][3]
    }

    pub fn determinant(&self) -> T {
        // 4x4行列の行列式計算（余因子展開）
        let m = &self.data;
        m[0][0]
            * (m[1][1] * (m[2][2] * m[3][3] - m[2][3] * m[3][2])
                - m[1][2] * (m[2][1] * m[3][3] - m[2][3] * m[3][1])
                + m[1][3] * (m[2][1] * m[3][2] - m[2][2] * m[3][1]))
            - m[0][1]
                * (m[1][0] * (m[2][2] * m[3][3] - m[2][3] * m[3][2])
                    - m[1][2] * (m[2][0] * m[3][3] - m[2][3] * m[3][0])
                    + m[1][3] * (m[2][0] * m[3][2] - m[2][2] * m[3][0]))
            + m[0][2]
                * (m[1][0] * (m[2][1] * m[3][3] - m[2][3] * m[3][1])
                    - m[1][1] * (m[2][0] * m[3][3] - m[2][3] * m[3][0])
                    + m[1][3] * (m[2][0] * m[3][1] - m[2][1] * m[3][0]))
            - m[0][3]
                * (m[1][0] * (m[2][1] * m[3][2] - m[2][2] * m[3][1])
                    - m[1][1] * (m[2][0] * m[3][2] - m[2][2] * m[3][0])
                    + m[1][2] * (m[2][0] * m[3][1] - m[2][1] * m[3][0]))
    }

    pub fn mul_vector(&self, vec: &Vector4<T>) -> Vector4<T> {
        Vector4::new(
            self.data[0][0] * vec.x()
                + self.data[0][1] * vec.y()
                + self.data[0][2] * vec.z()
                + self.data[0][3] * vec.w(),
            self.data[1][0] * vec.x()
                + self.data[1][1] * vec.y()
                + self.data[1][2] * vec.z()
                + self.data[1][3] * vec.w(),
            self.data[2][0] * vec.x()
                + self.data[2][1] * vec.y()
                + self.data[2][2] * vec.z()
                + self.data[2][3] * vec.w(),
            self.data[3][0] * vec.x()
                + self.data[3][1] * vec.y()
                + self.data[3][2] * vec.z()
                + self.data[3][3] * vec.w(),
        )
    }

    pub fn mul_matrix(&self, other: &Self) -> Self {
        let mut result = Self::zeros();
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.data[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        result
    }

    /// 要素アクセス
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[row][col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.data[row][col] = value;
    }

    /// 平行移動行列を作成
    pub fn translation(tx: T, ty: T, tz: T) -> Self {
        Self::new(
            T::ONE,
            T::ZERO,
            T::ZERO,
            tx,
            T::ZERO,
            T::ONE,
            T::ZERO,
            ty,
            T::ZERO,
            T::ZERO,
            T::ONE,
            tz,
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
            T::ZERO,
            sy,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            sz,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// X軸周りの回転行列
    pub fn rotation_x(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
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

    /// Y軸周りの回転行列
    pub fn rotation_y(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
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

    /// Z軸周りの回転行列
    pub fn rotation_z(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
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

    /// 任意軸周りの回転行列（Rodriguesの公式）
    pub fn rotation_axis(axis: &Vector3<T>, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = T::ONE - cos_a;

        let x = axis.x();
        let y = axis.y();
        let z = axis.z();

        Self::new(
            cos_a + x * x * one_minus_cos,
            x * y * one_minus_cos - z * sin_a,
            x * z * one_minus_cos + y * sin_a,
            T::ZERO,
            y * x * one_minus_cos + z * sin_a,
            cos_a + y * y * one_minus_cos,
            y * z * one_minus_cos - x * sin_a,
            T::ZERO,
            z * x * one_minus_cos - y * sin_a,
            z * y * one_minus_cos + x * sin_a,
            cos_a + z * z * one_minus_cos,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// Look-At行列（右手座標系）
    pub fn look_at(eye: &Vector3<T>, target: &Vector3<T>, up: &Vector3<T>) -> Result<Self, String> {
        let forward = (*target - *eye).normalize()?;
        let right = forward.cross(up).normalize()?;
        let up_corrected = right.cross(&forward);

        Ok(Self::new(
            right.x(),
            right.y(),
            right.z(),
            -right.dot(eye),
            up_corrected.x(),
            up_corrected.y(),
            up_corrected.z(),
            -up_corrected.dot(eye),
            -forward.x(),
            -forward.y(),
            -forward.z(),
            forward.dot(eye),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        ))
    }

    /// 透視投影行列（右手座標系、Z範囲 [-1, 1] OpenGL）
    pub fn perspective(fovy: T, aspect: T, near: T, far: T) -> Self {
        let f = T::ONE / (fovy / (T::ONE + T::ONE)).tan();
        let range_inv = T::ONE / (near - far);

        Self::new(
            f / aspect,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            f,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            (near + far) * range_inv,
            (T::ONE + T::ONE) * near * far * range_inv,
            T::ZERO,
            T::ZERO,
            -T::ONE,
            T::ZERO,
        )
    }

    /// 透視投影行列（右手座標系、Z範囲 [0, 1] DirectX）
    pub fn perspective_rh_01(fovy: T, aspect: T, near: T, far: T) -> Self {
        let f = T::ONE / (fovy / (T::ONE + T::ONE)).tan();
        let range_inv = T::ONE / (near - far);

        Self::new(
            f / aspect,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            f,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            far * range_inv,
            near * far * range_inv,
            T::ZERO,
            T::ZERO,
            -T::ONE,
            T::ZERO,
        )
    }

    /// 正射影行列（OpenGL）
    pub fn orthographic(left: T, right: T, bottom: T, top: T, near: T, far: T) -> Self {
        let rl_inv = T::ONE / (right - left);
        let tb_inv = T::ONE / (top - bottom);
        let fn_inv = T::ONE / (far - near);
        let two = T::ONE + T::ONE;

        Self::new(
            two * rl_inv,
            T::ZERO,
            T::ZERO,
            -(right + left) * rl_inv,
            T::ZERO,
            two * tb_inv,
            T::ZERO,
            -(top + bottom) * tb_inv,
            T::ZERO,
            T::ZERO,
            -two * fn_inv,
            -(far + near) * fn_inv,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// ビューポート変換行列
    pub fn viewport(x: T, y: T, width: T, height: T) -> Self {
        let half_width = width / (T::ONE + T::ONE);
        let half_height = height / (T::ONE + T::ONE);

        Self::new(
            half_width,
            T::ZERO,
            T::ZERO,
            x + half_width,
            T::ZERO,
            -half_height,
            T::ZERO,
            y + half_height,
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

    /// モデル・ビュー・プロジェクション行列の合成
    pub fn mvp(model: &Self, view: &Self, projection: &Self) -> Self {
        projection.mul_matrix(&view.mul_matrix(model))
    }

    /// 法線変換行列（3x3部分のみを逆転置）
    pub fn normal_matrix(&self) -> Result<[[T; 3]; 3], String> {
        // 上位3x3部分を抽出し、逆転置を計算
        let m3x3 = [
            [self.data[0][0], self.data[0][1], self.data[0][2]],
            [self.data[1][0], self.data[1][1], self.data[1][2]],
            [self.data[2][0], self.data[2][1], self.data[2][2]],
        ];

        // 3x3逆行列の計算（行列式による）
        let det = m3x3[0][0] * (m3x3[1][1] * m3x3[2][2] - m3x3[1][2] * m3x3[2][1])
            - m3x3[0][1] * (m3x3[1][0] * m3x3[2][2] - m3x3[1][2] * m3x3[2][0])
            + m3x3[0][2] * (m3x3[1][0] * m3x3[2][1] - m3x3[1][1] * m3x3[2][0]);

        if det.is_zero() {
            return Err("Upper 3x3 matrix is singular".to_string());
        }

        let inv_det = T::ONE / det;

        // 転置逆行列を計算
        Ok([
            [
                (m3x3[1][1] * m3x3[2][2] - m3x3[1][2] * m3x3[2][1]) * inv_det,
                (m3x3[1][2] * m3x3[2][0] - m3x3[1][0] * m3x3[2][2]) * inv_det,
                (m3x3[1][0] * m3x3[2][1] - m3x3[1][1] * m3x3[2][0]) * inv_det,
            ],
            [
                (m3x3[0][2] * m3x3[2][1] - m3x3[0][1] * m3x3[2][2]) * inv_det,
                (m3x3[0][0] * m3x3[2][2] - m3x3[0][2] * m3x3[2][0]) * inv_det,
                (m3x3[0][1] * m3x3[2][0] - m3x3[0][0] * m3x3[2][1]) * inv_det,
            ],
            [
                (m3x3[0][1] * m3x3[1][2] - m3x3[0][2] * m3x3[1][1]) * inv_det,
                (m3x3[0][2] * m3x3[1][0] - m3x3[0][0] * m3x3[1][2]) * inv_det,
                (m3x3[0][0] * m3x3[1][1] - m3x3[0][1] * m3x3[1][0]) * inv_det,
            ],
        ])
    }

    // === 3D アフィン変換メソッド ===

    /// 3Dベクトルを同次座標として変換（点として扱う）
    /// Vector3を(x, y, z, 1)として扱い、4x4行列で変換
    pub fn transform_point_3d(&self, point: &Vector3<T>) -> Vector3<T> {
        let x = self.data[0][0] * point.x()
            + self.data[0][1] * point.y()
            + self.data[0][2] * point.z()
            + self.data[0][3];
        let y = self.data[1][0] * point.x()
            + self.data[1][1] * point.y()
            + self.data[1][2] * point.z()
            + self.data[1][3];
        let z = self.data[2][0] * point.x()
            + self.data[2][1] * point.y()
            + self.data[2][2] * point.z()
            + self.data[2][3];
        Vector3::new(x, y, z)
    }

    /// 3Dベクトルを方向ベクトルとして変換（移動なし）
    /// Vector3を(x, y, z, 0)として扱い、回転・スケールのみ適用
    pub fn transform_vector_3d(&self, vector: &Vector3<T>) -> Vector3<T> {
        let x = self.data[0][0] * vector.x()
            + self.data[0][1] * vector.y()
            + self.data[0][2] * vector.z();
        let y = self.data[1][0] * vector.x()
            + self.data[1][1] * vector.y()
            + self.data[1][2] * vector.z();
        let z = self.data[2][0] * vector.x()
            + self.data[2][1] * vector.y()
            + self.data[2][2] * vector.z();
        Vector3::new(x, y, z)
    }

    /// 法線ベクトルを変換（逆転置行列を使用）
    pub fn transform_normal_3d(&self, normal: &Vector3<T>) -> Result<Vector3<T>, String> {
        let normal_matrix = self.normal_matrix()?;
        let x = normal_matrix[0][0] * normal.x()
            + normal_matrix[0][1] * normal.y()
            + normal_matrix[0][2] * normal.z();
        let y = normal_matrix[1][0] * normal.x()
            + normal_matrix[1][1] * normal.y()
            + normal_matrix[1][2] * normal.z();
        let z = normal_matrix[2][0] * normal.x()
            + normal_matrix[2][1] * normal.y()
            + normal_matrix[2][2] * normal.z();
        Ok(Vector3::new(x, y, z))
    }

    /// 3D点の配列を一括変換
    pub fn transform_points_3d(&self, points: &[Vector3<T>]) -> Vec<Vector3<T>> {
        points.iter().map(|p| self.transform_point_3d(p)).collect()
    }

    /// 3D方向ベクトルの配列を一括変換
    pub fn transform_vectors_3d(&self, vectors: &[Vector3<T>]) -> Vec<Vector3<T>> {
        vectors
            .iter()
            .map(|v| self.transform_vector_3d(v))
            .collect()
    }

    // === 3D変換行列の構築メソッド ===

    /// Vector3による平行移動行列を作成
    pub fn translation_3d(translation: &Vector3<T>) -> Self {
        Self::translation(translation.x(), translation.y(), translation.z())
    }

    /// Vector3による3Dスケール行列を作成
    pub fn scale_3d(scale: &Vector3<T>) -> Self {
        Self::scale(scale.x(), scale.y(), scale.z())
    }

    /// 均等3Dスケール行列を作成
    pub fn uniform_scale_3d(scale: T) -> Self {
        Self::scale_3d(&Vector3::new(scale, scale, scale))
    }

    /// 複合3D変換行列を作成（TRS: Translation, Rotation, Scale順）
    pub fn trs_3d(translation: &Vector3<T>, rotation: &Self, scale: &Vector3<T>) -> Self {
        let t = Self::translation_3d(translation);
        let s = Self::scale_3d(scale);
        t * *rotation * s
    }

    /// 複合3D変換行列を作成（SRT: Scale, Rotation, Translation順）
    pub fn srt_3d(scale: &Vector3<T>, rotation: &Self, translation: &Vector3<T>) -> Self {
        let s = Self::scale_3d(scale);
        let t = Self::translation_3d(translation);
        t * *rotation * s
    }

    /// オイラー角から回転行列を作成（ZYX順、内因性回転）
    pub fn rotation_euler_zyx(x: T, y: T, z: T) -> Self {
        let rx = Self::rotation_x(x);
        let ry = Self::rotation_y(y);
        let rz = Self::rotation_z(z);
        rz * ry * rx
    }

    /// オイラー角から回転行列を作成（XYZ順、外因性回転）
    pub fn rotation_euler_xyz(x: T, y: T, z: T) -> Self {
        let rx = Self::rotation_x(x);
        let ry = Self::rotation_y(y);
        let rz = Self::rotation_z(z);
        rx * ry * rz
    }

    /// クォータニオンから回転行列を作成（ジンバルロック回避）
    ///
    /// このメソッドはオイラー角による回転よりも推奨されます：
    /// - ジンバルロックを回避
    /// - 滑らかな補間が可能
    /// - 数値安定性が高い
    pub fn from_quaternion(q: &Quaternion<T>) -> Result<Self, String> {
        q.to_rotation_matrix()
    }

    /// 複合3D変換行列をクォータニオンで作成（TQS: Translation, Quaternion, Scale順）
    /// ジンバルロック回避版のTRS
    pub fn tqs_3d(
        translation: &Vector3<T>,
        rotation: &Quaternion<T>,
        scale: &Vector3<T>,
    ) -> Result<Self, String> {
        let t = Self::translation(translation.x(), translation.y(), translation.z());
        let r = Self::from_quaternion(rotation)?;
        let s = Self::scale(scale.x(), scale.y(), scale.z());
        Ok(t * r * s)
    }

    /// 複合3D変換行列をクォータニオンで作成（SQT: Scale, Quaternion, Translation順）
    /// ジンバルロック回避版のSRT
    pub fn sqt_3d(
        scale: &Vector3<T>,
        rotation: &Quaternion<T>,
        translation: &Vector3<T>,
    ) -> Result<Self, String> {
        let s = Self::scale(scale.x(), scale.y(), scale.z());
        let r = Self::from_quaternion(rotation)?;
        let t = Self::translation(translation.x(), translation.y(), translation.z());
        Ok(t * r * s)
    }

    // === 3Dアフィン変換の分解・抽出メソッド ===

    /// 3D変換行列から平行移動成分を抽出
    pub fn extract_translation_3d(&self) -> Vector3<T> {
        Vector3::new(self.data[0][3], self.data[1][3], self.data[2][3])
    }

    /// 3D変換行列からスケールを抽出
    pub fn extract_scale_3d(&self) -> Vector3<T> {
        let scale_x = (self.data[0][0] * self.data[0][0]
            + self.data[1][0] * self.data[1][0]
            + self.data[2][0] * self.data[2][0])
            .sqrt();
        let scale_y = (self.data[0][1] * self.data[0][1]
            + self.data[1][1] * self.data[1][1]
            + self.data[2][1] * self.data[2][1])
            .sqrt();
        let scale_z = (self.data[0][2] * self.data[0][2]
            + self.data[1][2] * self.data[1][2]
            + self.data[2][2] * self.data[2][2])
            .sqrt();
        Vector3::new(scale_x, scale_y, scale_z)
    }

    /// 3D変換行列から回転行列を抽出（スケールを除去）
    pub fn extract_rotation_3d(&self) -> Self {
        let scale = self.extract_scale_3d();
        Self::new(
            self.data[0][0] / scale.x(),
            self.data[0][1] / scale.y(),
            self.data[0][2] / scale.z(),
            T::ZERO,
            self.data[1][0] / scale.x(),
            self.data[1][1] / scale.y(),
            self.data[1][2] / scale.z(),
            T::ZERO,
            self.data[2][0] / scale.x(),
            self.data[2][1] / scale.y(),
            self.data[2][2] / scale.z(),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 3D変換行列を分解（Translation, Rotation, Scale）
    pub fn decompose_3d(&self) -> (Vector3<T>, Self, Vector3<T>) {
        let translation = self.extract_translation_3d();
        let scale = self.extract_scale_3d();
        let rotation = self.extract_rotation_3d();
        (translation, rotation, scale)
    }

    /// 3D変換行列からクォータニオンを抽出（ジンバルロック回避）
    ///
    /// Shepperd's method を使用して数値安定性を保証
    /// オイラー角変換よりも推奨される方法
    pub fn extract_quaternion_3d(&self) -> Result<Quaternion<T>, String> {
        let rotation_matrix = self.extract_rotation_3d();

        // 3x3回転行列の要素を取得
        let m11 = rotation_matrix.data[0][0];
        let m12 = rotation_matrix.data[0][1];
        let m13 = rotation_matrix.data[0][2];
        let m21 = rotation_matrix.data[1][0];
        let m22 = rotation_matrix.data[1][1];
        let m23 = rotation_matrix.data[1][2];
        let m31 = rotation_matrix.data[2][0];
        let m32 = rotation_matrix.data[2][1];
        let m33 = rotation_matrix.data[2][2];

        // Shepperd's method for numerical stability
        let trace = m11 + m22 + m33;

        if trace > T::ZERO {
            let s = (trace + T::ONE).sqrt() * T::from_f64(2.0); // s = 4 * qw
            let w = T::from_f64(0.25) * s;
            let x = (m32 - m23) / s;
            let y = (m13 - m31) / s;
            let z = (m21 - m12) / s;
            Ok(Quaternion::new(x, y, z, w))
        } else if m11 > m22 && m11 > m33 {
            let s = (T::ONE + m11 - m22 - m33).sqrt() * T::from_f64(2.0); // s = 4 * qx
            let w = (m32 - m23) / s;
            let x = T::from_f64(0.25) * s;
            let y = (m12 + m21) / s;
            let z = (m13 + m31) / s;
            Ok(Quaternion::new(x, y, z, w))
        } else if m22 > m33 {
            let s = (T::ONE + m22 - m11 - m33).sqrt() * T::from_f64(2.0); // s = 4 * qy
            let w = (m13 - m31) / s;
            let x = (m12 + m21) / s;
            let y = T::from_f64(0.25) * s;
            let z = (m23 + m32) / s;
            Ok(Quaternion::new(x, y, z, w))
        } else {
            let s = (T::ONE + m33 - m11 - m22).sqrt() * T::from_f64(2.0); // s = 4 * qz
            let w = (m21 - m12) / s;
            let x = (m13 + m31) / s;
            let y = (m23 + m32) / s;
            let z = T::from_f64(0.25) * s;
            Ok(Quaternion::new(x, y, z, w))
        }
    }

    /// 3D変換行列をクォータニオンで分解（Translation, Quaternion, Scale）
    /// ジンバルロック回避版の分解
    pub fn decompose_tqs_3d(&self) -> Result<TqsDecomposition<T>, String> {
        let translation = self.extract_translation_3d();
        let scale = self.extract_scale_3d();
        let rotation = self.extract_quaternion_3d()?;
        Ok((translation, rotation, scale))
    }

    // === 3Dアフィン変換・同次座標系の判定 ===

    /// 同次座標系での3Dアフィン変換行列かどうかを判定
    /// 4x4行列の最下行が [0, 0, 0, 1] であることを確認
    pub fn is_affine_transform_3d(&self) -> bool {
        self.data[3][0].is_zero()
            && self.data[3][1].is_zero()
            && self.data[3][2].is_zero()
            && (self.data[3][3] - T::ONE).abs() < T::EPSILON
    }

    /// 同次座標系での射影変換（perspective）要素があるかを判定
    pub fn has_perspective_3d(&self) -> bool {
        !self.data[3][0].is_zero() || !self.data[3][1].is_zero() || !self.data[3][2].is_zero()
    }

    /// 純粋な3Dアフィン変換かどうかを判定（射影要素なし、スケールファクター=1）
    pub fn is_pure_affine_3d(&self) -> bool {
        self.is_affine_transform_3d() && self.data[3][3] == T::ONE
    }

    /// 3D変換が剛体変換（回転+移動のみ）かどうかを判定
    pub fn is_rigid_3d(&self) -> bool {
        let scale = self.extract_scale_3d();
        (scale.x() - T::ONE).abs() < T::EPSILON
            && (scale.y() - T::ONE).abs() < T::EPSILON
            && (scale.z() - T::ONE).abs() < T::EPSILON
    }

    /// オイラー角の抽出（XYZ順序）
    pub fn extract_euler_angles_3d(&self) -> (T, T, T) {
        let m = &self.data;

        // XYZ オイラー角の抽出
        let sy = (m[0][2] * m[0][2] + m[1][2] * m[1][2]).sqrt();

        let singular = sy < T::EPSILON;

        let (x, y, z) = if !singular {
            (
                m[1][2].atan2(m[2][2]),
                (-m[0][2]).atan2(sy),
                m[0][1].atan2(m[0][0]),
            )
        } else {
            (m[2][1].atan2(m[1][1]), (-m[0][2]).atan2(sy), T::ZERO)
        };

        (x, y, z)
    }

    /// オイラー角からの回転行列作成（XYZ順序）
    pub fn euler_angles_3d(x: T, y: T, z: T) -> Self {
        let rx = Self::rotation_x_3d(x);
        let ry = Self::rotation_y_3d(y);
        let rz = Self::rotation_z_3d(z);

        rx * ry * rz
    }

    /// 3D変換が均等スケールかどうかを判定
    pub fn is_uniform_scale_3d(&self) -> bool {
        let scale = self.extract_scale_3d();
        (scale.x() - scale.y()).abs() < T::EPSILON && (scale.y() - scale.z()).abs() < T::EPSILON
    }

    /// アフィン変換行列として有効かどうかを検証
    pub fn is_valid_affine_3d(&self) -> bool {
        // 最下行が [0, 0, 0, 1] であること
        if !self.is_affine_transform_3d() {
            return false;
        }

        // 行列式が0でないこと（可逆性）
        let det = self.determinant();
        if det.is_zero() {
            return false;
        }

        // 数値的に安定していること
        let max_element = self
            .data
            .iter()
            .flat_map(|row| row.iter())
            .fold(T::ZERO, |acc, &x| acc.max(x.abs()));

        max_element < T::from_f64(1e12)
    }

    /// 3Dアフィン変換の線形部分を抽出（4x4行列の左上3x3部分）
    pub fn linear_part_3d(&self) -> [[T; 3]; 3] {
        [
            [self.data[0][0], self.data[0][1], self.data[0][2]],
            [self.data[1][0], self.data[1][1], self.data[1][2]],
            [self.data[2][0], self.data[2][1], self.data[2][2]],
        ]
    }

    // === 同次座標系での変換 ===

    /// 同次座標系での正規化（最下行右下を1にスケール）
    pub fn normalize_homogeneous_3d(&self) -> Result<Self, String> {
        let w = self.data[3][3];
        if w.is_zero() {
            return Err("Cannot normalize: bottom-right element is zero".to_string());
        }

        if (w - T::ONE).abs() < T::EPSILON {
            return Ok(*self);
        }

        let mut result = Self::zeros();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = self.data[i][j] / w;
            }
        }
        Ok(result)
    }

    /// 同次座標による点の変換（w成分も計算）
    pub fn transform_homogeneous_3d(&self, point: Vector3<T>) -> (Vector3<T>, T) {
        let x = self.data[0][0] * point.x()
            + self.data[0][1] * point.y()
            + self.data[0][2] * point.z()
            + self.data[0][3];
        let y = self.data[1][0] * point.x()
            + self.data[1][1] * point.y()
            + self.data[1][2] * point.z()
            + self.data[1][3];
        let z = self.data[2][0] * point.x()
            + self.data[2][1] * point.y()
            + self.data[2][2] * point.z()
            + self.data[2][3];
        let w = self.data[3][0] * point.x()
            + self.data[3][1] * point.y()
            + self.data[3][2] * point.z()
            + self.data[3][3];

        (Vector3::new(x, y, z), w)
    }

    /// 射影変換による点の変換（w成分で正規化）
    pub fn transform_projective_3d(&self, point: Vector3<T>) -> Result<Vector3<T>, String> {
        let (transformed, w) = self.transform_homogeneous_3d(point);

        if w.is_zero() {
            return Err("Point at infinity".to_string());
        }

        Ok(Vector3::new(
            transformed.x() / w,
            transformed.y() / w,
            transformed.z() / w,
        ))
    }

    // === 3Dアフィン変換専用の構築メソッド ===

    /// 一般的な3Dアフィン変換行列を作成
    /// linear_transform: 3x3線形変換行列, translation: 平行移動ベクトル
    pub fn affine_3d(linear_transform: [[T; 3]; 3], translation: Vector3<T>) -> Self {
        Self::new(
            linear_transform[0][0],
            linear_transform[0][1],
            linear_transform[0][2],
            translation.x(),
            linear_transform[1][0],
            linear_transform[1][1],
            linear_transform[1][2],
            translation.y(),
            linear_transform[2][0],
            linear_transform[2][1],
            linear_transform[2][2],
            translation.z(),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 3D境界ボックスを変換
    pub fn transform_bounding_box_3d(
        &self,
        min: &Vector3<T>,
        max: &Vector3<T>,
    ) -> (Vector3<T>, Vector3<T>) {
        let corners = [
            *min,
            Vector3::new(max.x(), min.y(), min.z()),
            Vector3::new(min.x(), max.y(), min.z()),
            Vector3::new(max.x(), max.y(), min.z()),
            Vector3::new(min.x(), min.y(), max.z()),
            Vector3::new(max.x(), min.y(), max.z()),
            Vector3::new(min.x(), max.y(), max.z()),
            *max,
        ];

        let transformed_corners: Vec<Vector3<T>> = corners
            .iter()
            .map(|corner| self.transform_point_3d(corner))
            .collect();

        let mut new_min = transformed_corners[0];
        let mut new_max = transformed_corners[0];

        for corner in &transformed_corners[1..] {
            new_min = new_min.min(corner);
            new_max = new_max.max(corner);
        }

        (new_min, new_max)
    }

    /// 3D変換行列の逆変換を計算（効率的な実装）
    pub fn inverse_3d(&self) -> Result<Self, String> {
        if !self.is_affine_transform_3d() {
            return Err("Matrix is not a valid 3D affine transformation".to_string());
        }

        let (translation, rotation, scale) = self.decompose_3d();

        // 各成分の逆変換
        let inv_scale = Vector3::new(T::ONE / scale.x(), T::ONE / scale.y(), T::ONE / scale.z());
        let inv_rotation = rotation.transpose(); // 回転行列の逆は転置
        let inv_translation = -translation;

        // 逆順で合成 (TRS)^-1 = S^-1 * R^-1 * T^-1
        let s_inv = Self::scale_3d(&inv_scale);
        let t_inv = Self::translation_3d(&inv_translation);

        Ok(s_inv * inv_rotation * t_inv)
    }

    // === 特殊な3D変換 ===

    /// X軸周りの回転行列
    pub fn rotation_x_3d(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
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

    /// Y軸周りの回転行列
    pub fn rotation_y_3d(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
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

    /// Z軸周りの回転行列
    pub fn rotation_z_3d(angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
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

    /// XYZ軸の反射行列を作成
    pub fn reflection_3d(reflect_x: bool, reflect_y: bool, reflect_z: bool) -> Self {
        Self::new(
            if reflect_x { -T::ONE } else { T::ONE },
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            if reflect_y { -T::ONE } else { T::ONE },
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            if reflect_z { -T::ONE } else { T::ONE },
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 平面による反射行列（法線ベクトルnと平面上の点pを使用）
    pub fn reflection_plane_3d(normal: Vector3<T>, point: Vector3<T>) -> Self {
        let n = normal.normalize().unwrap();
        let d = n.dot(&point);

        let nx = n.x();
        let ny = n.y();
        let nz = n.z();

        Self::new(
            T::ONE - nx * nx * (T::ONE + T::ONE),
            -nx * ny * (T::ONE + T::ONE),
            -nx * nz * (T::ONE + T::ONE),
            nx * d * (T::ONE + T::ONE),
            -ny * nx * (T::ONE + T::ONE),
            T::ONE - ny * ny * (T::ONE + T::ONE),
            -ny * nz * (T::ONE + T::ONE),
            ny * d * (T::ONE + T::ONE),
            -nz * nx * (T::ONE + T::ONE),
            -nz * ny * (T::ONE + T::ONE),
            T::ONE - nz * nz * (T::ONE + T::ONE),
            nz * d * (T::ONE + T::ONE),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 軸周りの任意角度回転（ロドリゲスの回転公式）
    pub fn rotation_axis_3d(axis: Vector3<T>, angle: T) -> Self {
        let axis = axis.normalize().unwrap();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = T::ONE - cos_a;

        let x = axis.x();
        let y = axis.y();
        let z = axis.z();

        Self::new(
            cos_a + x * x * one_minus_cos,
            x * y * one_minus_cos - z * sin_a,
            x * z * one_minus_cos + y * sin_a,
            T::ZERO,
            y * x * one_minus_cos + z * sin_a,
            cos_a + y * y * one_minus_cos,
            y * z * one_minus_cos - x * sin_a,
            T::ZERO,
            z * x * one_minus_cos - y * sin_a,
            z * y * one_minus_cos + x * sin_a,
            cos_a + z * z * one_minus_cos,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// Look-at変換行列（カメラ座標系）
    pub fn look_at_3d(eye: Vector3<T>, target: Vector3<T>, up: Vector3<T>) -> Self {
        let f = (target - eye).normalize().unwrap(); // Forward
        let s = f.cross(&up).normalize().unwrap(); // Side (right)
        let u = s.cross(&f); // Up

        Self::new(
            s.x(),
            s.y(),
            s.z(),
            -s.dot(&eye),
            u.x(),
            u.y(),
            u.z(),
            -u.dot(&eye),
            -f.x(),
            -f.y(),
            -f.z(),
            f.dot(&eye),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 平行投影行列
    pub fn orthographic_3d(left: T, right: T, bottom: T, top: T, near: T, far: T) -> Self {
        let width = right - left;
        let height = top - bottom;
        let depth = far - near;
        let two = T::ONE + T::ONE;

        Self::new(
            two / width,
            T::ZERO,
            T::ZERO,
            -(right + left) / width,
            T::ZERO,
            two / height,
            T::ZERO,
            -(top + bottom) / height,
            T::ZERO,
            T::ZERO,
            -two / depth,
            -(far + near) / depth,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
        )
    }

    /// 透視投影行列
    pub fn perspective_3d(fovy: T, aspect: T, near: T, far: T) -> Self {
        let tan_half_fovy = (fovy / (T::ONE + T::ONE)).tan();
        let depth = far - near;
        let two = T::ONE + T::ONE;

        Self::new(
            T::ONE / (aspect * tan_half_fovy),
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE / tan_half_fovy,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            -(far + near) / depth,
            -two * far * near / depth,
            T::ZERO,
            T::ZERO,
            -T::ONE,
            T::ZERO,
        )
    }

    // === デバッグ・ユーティリティ ===

    /// Matrix4x4の詳細情報を文字列として取得
    pub fn debug_string(&self) -> String {
        format!(
            "Matrix4x4:\n[{:8.3} {:8.3} {:8.3} {:8.3}]\n[{:8.3} {:8.3} {:8.3} {:8.3}]\n[{:8.3} {:8.3} {:8.3} {:8.3}]\n[{:8.3} {:8.3} {:8.3} {:8.3}]",
            self.data[0][0].to_f64(), self.data[0][1].to_f64(), self.data[0][2].to_f64(), self.data[0][3].to_f64(),
            self.data[1][0].to_f64(), self.data[1][1].to_f64(), self.data[1][2].to_f64(), self.data[1][3].to_f64(),
            self.data[2][0].to_f64(), self.data[2][1].to_f64(), self.data[2][2].to_f64(), self.data[2][3].to_f64(),
            self.data[3][0].to_f64(), self.data[3][1].to_f64(), self.data[3][2].to_f64(), self.data[3][3].to_f64(),
        )
    }

    /// 3Dアフィン変換の詳細分解情報を取得
    pub fn transformation_info_3d(&self) -> String {
        if !self.is_affine_transform_3d() {
            return "Not a valid 3D affine transformation".to_string();
        }

        let (translation, _rotation, scale) = self.decompose_3d();
        let (euler_x, euler_y, euler_z) = self.extract_euler_angles_3d();

        format!(
            "3D Affine Transformation:\nTranslation: ({:.3}, {:.3}, {:.3})\nScale: ({:.3}, {:.3}, {:.3})\nEuler Angles: ({:.3}°, {:.3}°, {:.3}°)\nIs Rigid: {}\nDeterminant: {:.6}",
            translation.x().to_f64(), translation.y().to_f64(), translation.z().to_f64(),
            scale.x().to_f64(), scale.y().to_f64(), scale.z().to_f64(),
            euler_x.to_f64().to_degrees(), euler_y.to_f64().to_degrees(), euler_z.to_f64().to_degrees(),
            self.is_rigid_3d(),
            self.determinant().to_f64()
        )
    }

    /// 3D変換行列が近似的に等しいかチェック
    pub fn is_approximately_equal_3d(&self, other: &Self, tolerance: T) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if (self.data[i][j] - other.data[i][j]).abs() > tolerance {
                    return false;
                }
            }
        }
        true
    }

    /// 3D変換行列の線形部分（3x3）を取得
    pub fn extract_linear_3d(&self) -> [[T; 3]; 3] {
        [
            [self.data[0][0], self.data[0][1], self.data[0][2]],
            [self.data[1][0], self.data[1][1], self.data[1][2]],
            [self.data[2][0], self.data[2][1], self.data[2][2]],
        ]
    }

    /// Vector3によるコンストラクタ（行ベクトル版）
    pub fn from_row_vectors_3d(
        row0: Vector4<T>,
        row1: Vector4<T>,
        row2: Vector4<T>,
        row3: Vector4<T>,
    ) -> Self {
        Self::new(
            row0.x(),
            row0.y(),
            row0.z(),
            row0.w(),
            row1.x(),
            row1.y(),
            row1.z(),
            row1.w(),
            row2.x(),
            row2.y(),
            row2.z(),
            row2.w(),
            row3.x(),
            row3.y(),
            row3.z(),
            row3.w(),
        )
    }

    /// Vector3によるコンストラクタ（列ベクトル版）
    pub fn from_column_vectors_3d(
        col0: Vector4<T>,
        col1: Vector4<T>,
        col2: Vector4<T>,
        col3: Vector4<T>,
    ) -> Self {
        Self::new(
            col0.x(),
            col1.x(),
            col2.x(),
            col3.x(),
            col0.y(),
            col1.y(),
            col2.y(),
            col3.y(),
            col0.z(),
            col1.z(),
            col2.z(),
            col3.z(),
            col0.w(),
            col1.w(),
            col2.w(),
            col3.w(),
        )
    }

    /// 指定した行ベクトルを取得
    pub fn row_vector_3d(&self, row: usize) -> Result<Vector4<T>, String> {
        if row >= 4 {
            return Err("Row index out of bounds".to_string());
        }
        Ok(Vector4::new(
            self.data[row][0],
            self.data[row][1],
            self.data[row][2],
            self.data[row][3],
        ))
    }

    /// 指定した列ベクトルを取得
    pub fn column_vector_3d(&self, col: usize) -> Result<Vector4<T>, String> {
        if col >= 4 {
            return Err("Column index out of bounds".to_string());
        }
        Ok(Vector4::new(
            self.data[0][col],
            self.data[1][col],
            self.data[2][col],
            self.data[3][col],
        ))
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Matrix4x4<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let mut result = Self::zeros();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        result
    }
}

impl<T: Scalar> Mul<T> for Matrix4x4<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        let mut result = Self::zeros();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = self.data[i][j] * scalar;
            }
        }
        result
    }
}

impl<T: Scalar> Mul for Matrix4x4<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        self.mul_matrix(&other)
    }
}

// Vector3 との演算子オーバーロード（点として扱う）
impl<T: Scalar> Mul<Vector3<T>> for Matrix4x4<T> {
    type Output = Vector3<T>;
    fn mul(self, vector: Vector3<T>) -> Self::Output {
        self.transform_point_3d(&vector)
    }
}

// Vector4 との演算子オーバーロード
impl<T: Scalar> Mul<Vector4<T>> for Matrix4x4<T> {
    type Output = Vector4<T>;
    fn mul(self, vector: Vector4<T>) -> Self::Output {
        self.mul_vector(&vector)
    }
}

/// 型エイリアス
pub type Matrix4x4f = Matrix4x4<f32>;
pub type Matrix4x4d = Matrix4x4<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::quaternion::Quaternion;
    use std::f64::consts::PI;

    #[test]
    fn test_matrix_from_quaternion() {
        // Z軸周りの90度回転クォータニオン
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = PI / 2.0;
        let q = Quaternion::from_axis_angle(&axis, angle);

        // クォータニオンから回転行列を作成
        let matrix = Matrix4x4::from_quaternion(&q).unwrap();

        // X軸ベクトルを回転してY軸になるか確認
        let x_vec = Vector4::new(1.0, 0.0, 0.0, 1.0);
        let rotated = matrix * x_vec;

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
        assert!((rotated.w() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_tqs_transform() {
        // Translation, Quaternion, Scale による変換テスト
        let translation = Vector3::new(1.0, 2.0, 3.0);
        let rotation = Quaternion::from_axis_angle(&Vector3::new(0.0, 0.0, 1.0), PI / 4.0);
        let scale = Vector3::new(2.0, 2.0, 2.0);

        let transform = Matrix4x4::tqs_3d(&translation, &rotation, &scale).unwrap();

        // 原点のベクトルを変換
        let origin = Vector4::new(0.0, 0.0, 0.0, 1.0);
        let transformed = transform * origin;

        // 平行移動成分のみが適用されるはず
        assert!((transformed.x() - translation.x()).abs() < 1e-10);
        assert!((transformed.y() - translation.y()).abs() < 1e-10);
        assert!((transformed.z() - translation.z()).abs() < 1e-10);
    }

    #[test]
    fn test_quaternion_extraction() {
        // 既知のクォータニオンから行列を作成し、再抽出してテスト
        let original_q = Quaternion::from_euler_angles(PI / 6.0, PI / 4.0, PI / 3.0);
        let matrix = Matrix4x4::from_quaternion(&original_q).unwrap();
        let extracted_q = matrix.extract_quaternion_3d().unwrap();

        // 回転結果が同じかで検証（クォータニオンの符号不定性を考慮）
        let test_vector = Vector3::new(1.0, 2.0, 3.0);
        let result1 = original_q.rotate_vector(&test_vector);
        let result2 = extracted_q.rotate_vector(&test_vector);

        assert!((result1.x() - result2.x()).abs() < 1e-10);
        assert!((result1.y() - result2.y()).abs() < 1e-10);
        assert!((result1.z() - result2.z()).abs() < 1e-10);
    }

    #[test]
    fn test_tqs_decomposition() {
        // TQS変換の分解テスト
        let original_translation = Vector3::new(1.0, 2.0, 3.0);
        let original_rotation = Quaternion::from_axis_angle(
            &Vector3::new(1.0, 1.0, 1.0).normalize().unwrap(),
            PI / 3.0,
        );
        let original_scale = Vector3::new(2.0, 3.0, 4.0);

        let transform =
            Matrix4x4::tqs_3d(&original_translation, &original_rotation, &original_scale).unwrap();
        let (extracted_translation, extracted_rotation, extracted_scale) =
            transform.decompose_tqs_3d().unwrap();

        // 平行移動の検証
        assert!((original_translation.x() - extracted_translation.x()).abs() < 1e-10);
        assert!((original_translation.y() - extracted_translation.y()).abs() < 1e-10);
        assert!((original_translation.z() - extracted_translation.z()).abs() < 1e-10);

        // スケールの検証
        assert!((original_scale.x() - extracted_scale.x()).abs() < 1e-10);
        assert!((original_scale.y() - extracted_scale.y()).abs() < 1e-10);
        assert!((original_scale.z() - extracted_scale.z()).abs() < 1e-10);

        // 回転の検証（結果が同じかで判定）
        let test_vector = Vector3::new(1.0, 2.0, 3.0);
        let result1 = original_rotation.rotate_vector(&test_vector);
        let result2 = extracted_rotation.rotate_vector(&test_vector);

        assert!((result1.x() - result2.x()).abs() < 1e-9);
        assert!((result1.y() - result2.y()).abs() < 1e-9);
        assert!((result1.z() - result2.z()).abs() < 1e-9);
    }

    #[test]
    fn test_gimbal_lock_avoidance() {
        // ジンバルロック回避の主な利点をテスト

        // 1. 軸角回転によるクォータニオン作成と行列変換の一貫性
        let axis = Vector3::new(1.0, 1.0, 1.0).normalize().unwrap();
        let angle = PI / 3.0; // 60度
        let quaternion = Quaternion::from_axis_angle(&axis, angle);
        let matrix = Matrix4x4::from_quaternion(&quaternion).unwrap();

        // クォータニオンによる直接的なベクトル回転
        let test_vector_3d = Vector3::new(1.0, 0.0, 0.0);
        let quaternion_rotated = quaternion.rotate_vector(&test_vector_3d);

        // 行列による回転（同次座標）
        let test_vector_4d = Vector4::new(1.0, 0.0, 0.0, 1.0);
        let matrix_rotated = matrix * test_vector_4d;

        // 結果が一致することを確認
        assert!((quaternion_rotated.x() - matrix_rotated.x()).abs() < 1e-10);
        assert!((quaternion_rotated.y() - matrix_rotated.y()).abs() < 1e-10);
        assert!((quaternion_rotated.z() - matrix_rotated.z()).abs() < 1e-10);

        // 2. SLERP補間の滑らかさをテスト
        let q1 = Quaternion::<f64>::identity();
        let q2 = Quaternion::from_axis_angle(&Vector3::new(0.0, 0.0, 1.0), PI / 2.0);

        // 複数のt値での補間をテスト
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let interpolated = q1.slerp(&q2, t).unwrap();
            let interpolated_matrix = Matrix4x4::from_quaternion(&interpolated).unwrap();

            // 補間された行列が有効であることを確認
            let test_result = interpolated_matrix * Vector4::new(1.0, 0.0, 0.0, 1.0);
            assert!(test_result.x().is_finite());
            assert!(test_result.y().is_finite());
            assert!(test_result.z().is_finite());
            assert!((test_result.w() - 1.0).abs() < 1e-10);
        }

        // 3. 回転の合成がクォータニオンで正しく動作することをテスト
        let q_x = Quaternion::from_axis_angle(&Vector3::new(1.0, 0.0, 0.0), PI / 4.0);
        let q_y = Quaternion::from_axis_angle(&Vector3::new(0.0, 1.0, 0.0), PI / 4.0);
        let q_combined = q_y * q_x; // クォータニオンの合成

        let matrix_combined = Matrix4x4::from_quaternion(&q_combined).unwrap();
        let test_vec = Vector4::new(1.0, 0.0, 0.0, 1.0);
        let result = matrix_combined * test_vec;

        // 結果が有効であることを確認
        assert!(result.x().is_finite());
        assert!(result.y().is_finite());
        assert!(result.z().is_finite());
        assert!((result.w() - 1.0).abs() < 1e-10);
    }
}
