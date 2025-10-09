//! 4x4行列（3D変換・プロジェクション用）
//!
//! 3D変換、射影変換、カメラ変換のための専用行列
//! OpenGL/DirectX互換の行列演算を提供
use crate::abstract_types::Scalar;
use crate::linalg::vector::{Vector3, Vector4};
use std::ops::{Add, Mul};

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
                    result.data[i][j] = result.data[i][j] + self.data[i][k] * other.data[k][j];
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

/// 型エイリアス
pub type Matrix4x4f = Matrix4x4<f32>;
pub type Matrix4x4d = Matrix4x4<f64>;
