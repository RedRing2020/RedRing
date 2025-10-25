//! Circle3D Transform Operations - Matrix transformations and geometric transforms
//!
//! 3次元円の座標変換操作：平行移動、スケール変換、回転変換

use crate::{Circle3D, Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

impl<T: Scalar> Circle3D<T> {
    /// 平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動後の円
    pub fn translate(&self, translation: &Vector3D<T>) -> Self {
        Circle3D::new(
            Point3D::new(
                self.center().x() + translation.x(),
                self.center().y() + translation.y(),
                self.center().z() + translation.z(),
            ),
            self.normal(),
            self.radius(),
        )
        .unwrap() // 元の円が有効なら変換後も有効
    }

    /// 一様スケール変換
    ///
    /// # 引数
    /// * `scale` - スケール係数（正の値）
    ///
    /// # 戻り値
    /// スケール変換後の円、スケール係数が0以下の場合は `None`
    pub fn scale(&self, scale: T) -> Option<Self> {
        if scale <= T::ZERO {
            return None;
        }

        Circle3D::new(
            Point3D::new(
                self.center().x() * scale,
                self.center().y() * scale,
                self.center().z() * scale,
            ),
            self.normal(), // 方向は変わらない
            self.radius() * scale,
        )
    }

    /// 原点中心での一様スケール変換
    ///
    /// # 引数
    /// * `scale` - スケール係数（正の値）
    ///
    /// # 戻り値
    /// スケール変換後の円、スケール係数が0以下の場合は `None`
    pub fn scale_from_origin(&self, scale: T) -> Option<Self> {
        self.scale(scale)
    }

    /// 指定点中心でのスケール変換
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale` - スケール係数（正の値）
    ///
    /// # 戻り値
    /// スケール変換後の円、スケール係数が0以下の場合は `None`
    pub fn scale_from_point(&self, center: &Point3D<T>, scale: T) -> Option<Self> {
        if scale <= T::ZERO {
            return None;
        }

        // center からの相対位置をスケール
        let relative_pos = Vector3D::from_points(center, &self.center());
        let scaled_relative = Vector3D::new(
            relative_pos.x() * scale,
            relative_pos.y() * scale,
            relative_pos.z() * scale,
        );

        let new_center = Point3D::new(
            center.x() + scaled_relative.x(),
            center.y() + scaled_relative.y(),
            center.z() + scaled_relative.z(),
        );

        Circle3D::new(
            new_center,
            self.normal(), // 方向は変わらない
            self.radius() * scale,
        )
    }

    /// Z軸中心での回転変換
    ///
    /// # 引数
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転変換後の円
    pub fn rotate_z(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 中心点の回転
        let new_center = Point3D::new(
            cos_a * self.center().x() - sin_a * self.center().y(),
            sin_a * self.center().x() + cos_a * self.center().y(),
            self.center().z(),
        );

        // 法線ベクトルの回転
        let normal_vec = self.normal().as_vector();
        let new_normal_vec = Vector3D::new(
            cos_a * normal_vec.x() - sin_a * normal_vec.y(),
            sin_a * normal_vec.x() + cos_a * normal_vec.y(),
            normal_vec.z(),
        );

        Circle3D::new(
            new_center,
            Direction3D::from_vector(new_normal_vec).unwrap(),
            self.radius(),
        )
        .unwrap()
    }

    /// X軸中心での回転変換
    ///
    /// # 引数
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転変換後の円
    pub fn rotate_x(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 中心点の回転
        let new_center = Point3D::new(
            self.center().x(),
            cos_a * self.center().y() - sin_a * self.center().z(),
            sin_a * self.center().y() + cos_a * self.center().z(),
        );

        // 法線ベクトルの回転
        let normal_vec = self.normal().as_vector();
        let new_normal_vec = Vector3D::new(
            normal_vec.x(),
            cos_a * normal_vec.y() - sin_a * normal_vec.z(),
            sin_a * normal_vec.y() + cos_a * normal_vec.z(),
        );

        Circle3D::new(
            new_center,
            Direction3D::from_vector(new_normal_vec).unwrap(),
            self.radius(),
        )
        .unwrap()
    }

    /// Y軸中心での回転変換
    ///
    /// # 引数
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転変換後の円
    pub fn rotate_y(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 中心点の回転
        let new_center = Point3D::new(
            cos_a * self.center().x() + sin_a * self.center().z(),
            self.center().y(),
            -sin_a * self.center().x() + cos_a * self.center().z(),
        );

        // 法線ベクトルの回転
        let normal_vec = self.normal().as_vector();
        let new_normal_vec = Vector3D::new(
            cos_a * normal_vec.x() + sin_a * normal_vec.z(),
            normal_vec.y(),
            -sin_a * normal_vec.x() + cos_a * normal_vec.z(),
        );

        Circle3D::new(
            new_center,
            Direction3D::from_vector(new_normal_vec).unwrap(),
            self.radius(),
        )
        .unwrap()
    }

    /// 任意軸中心での回転変換
    ///
    /// # 引数
    /// * `axis` - 回転軸の方向ベクトル（正規化済み）
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転変換後の円
    pub fn rotate_around_axis(&self, axis: &Direction3D<T>, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = T::ONE - cos_a;

        let ux = axis.x();
        let uy = axis.y();
        let uz = axis.z();

        // ロドリゲスの回転公式による回転行列
        let m11 = cos_a + ux * ux * one_minus_cos;
        let m12 = ux * uy * one_minus_cos - uz * sin_a;
        let m13 = ux * uz * one_minus_cos + uy * sin_a;

        let m21 = uy * ux * one_minus_cos + uz * sin_a;
        let m22 = cos_a + uy * uy * one_minus_cos;
        let m23 = uy * uz * one_minus_cos - ux * sin_a;

        let m31 = uz * ux * one_minus_cos - uy * sin_a;
        let m32 = uz * uy * one_minus_cos + ux * sin_a;
        let m33 = cos_a + uz * uz * one_minus_cos;

        // 中心点の回転
        let new_center = Point3D::new(
            m11 * self.center().x() + m12 * self.center().y() + m13 * self.center().z(),
            m21 * self.center().x() + m22 * self.center().y() + m23 * self.center().z(),
            m31 * self.center().x() + m32 * self.center().y() + m33 * self.center().z(),
        );

        // 法線ベクトルの回転
        let normal_vec = self.normal().as_vector();
        let new_normal_vec = Vector3D::new(
            m11 * normal_vec.x() + m12 * normal_vec.y() + m13 * normal_vec.z(),
            m21 * normal_vec.x() + m22 * normal_vec.y() + m23 * normal_vec.z(),
            m31 * normal_vec.x() + m32 * normal_vec.y() + m33 * normal_vec.z(),
        );

        Circle3D::new(
            new_center,
            Direction3D::from_vector(new_normal_vec).unwrap(),
            self.radius(),
        )
        .unwrap()
    }

    /// 3x3行列による変換（一様変換のみサポート）
    ///
    /// # 引数
    /// * `matrix` - 3x3変換行列 [[m11, m12, m13], [m21, m22, m23], [m31, m32, m33]]
    ///
    /// # 戻り値
    /// 変換後の円、非一様変換や行列式が0の場合は `None`
    pub fn transform_matrix(&self, matrix: &[[T; 3]; 3]) -> Option<Self> {
        let [[m11, m12, m13], [m21, m22, m23], [m31, m32, m33]] = *matrix;

        // 行列式の計算
        let det = m11 * (m22 * m33 - m23 * m32) - m12 * (m21 * m33 - m23 * m31)
            + m13 * (m21 * m32 - m22 * m31);

        if det.abs() < T::EPSILON {
            return None; // 特異行列
        }

        // 中心点の変換
        let new_center = Point3D::new(
            m11 * self.center().x() + m12 * self.center().y() + m13 * self.center().z(),
            m21 * self.center().x() + m22 * self.center().y() + m23 * self.center().z(),
            m31 * self.center().x() + m32 * self.center().y() + m33 * self.center().z(),
        );

        // 法線ベクトルの変換（逆転置行列を使用）
        let inv_det = T::ONE / det;
        let inv_t11 = (m22 * m33 - m23 * m32) * inv_det;
        let inv_t12 = (m13 * m32 - m12 * m33) * inv_det;
        let inv_t13 = (m12 * m23 - m13 * m22) * inv_det;
        let inv_t21 = (m23 * m31 - m21 * m33) * inv_det;
        let inv_t22 = (m11 * m33 - m13 * m31) * inv_det;
        let inv_t23 = (m13 * m21 - m11 * m23) * inv_det;
        let inv_t31 = (m21 * m32 - m22 * m31) * inv_det;
        let inv_t32 = (m12 * m31 - m11 * m32) * inv_det;
        let inv_t33 = (m11 * m22 - m12 * m21) * inv_det;

        let normal_vec = self.normal().as_vector();
        let new_normal_vec = Vector3D::new(
            inv_t11 * normal_vec.x() + inv_t21 * normal_vec.y() + inv_t31 * normal_vec.z(),
            inv_t12 * normal_vec.x() + inv_t22 * normal_vec.y() + inv_t32 * normal_vec.z(),
            inv_t13 * normal_vec.x() + inv_t23 * normal_vec.y() + inv_t33 * normal_vec.z(),
        );

        // 一様スケール係数の計算（行列式の立方根）
        let scale_factor = det.abs().powf(T::ONE / (T::ONE + T::ONE + T::ONE)); // det^(1/3)

        Circle3D::new(
            new_center,
            Direction3D::from_vector(new_normal_vec)?,
            self.radius() * scale_factor,
        )
    }
}
