//! Plane3DCoordinateSystem Transform Operations
//!
//! STEP準拠平面座標系の変換操作実装
//!
//! **作成日: 2025年10月29日**
//! **最終更新: 2025年10月29日**
//!
//! ## 実装内容
//! - 平行移動：origin の移動
//! - 回転：座標系全体の回転（normal, u_axis, v_axis）
//! - スケール：座標系のスケーリング
//! - BasicTransform トレイト実装（Foundation パターン準拠）
//!
//! ## 座標系変換の特性
//! - 直交性保持：変換後も u_axis ⊥ v_axis ⊥ normal を維持
//! - 右手系保持：v_axis = normal × u_axis 関係を維持
//! - 正規化保持：全ての軸ベクトルが単位ベクトルのまま

use crate::{Direction3D, Plane3DCoordinateSystem, Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// Basic Transform Operations
// ============================================================================

impl<T: Scalar> Plane3DCoordinateSystem<T> {
    /// 平行移動
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// 平行移動後の座標系
    ///
    /// # Note
    /// 座標軸（normal, u_axis, v_axis）は変更されず、origin のみ移動
    pub fn translate(&self, translation: &Vector3D<T>) -> Self {
        let new_origin = Point3D::new(
            self.origin().x() + translation.x(),
            self.origin().y() + translation.y(),
            self.origin().z() + translation.z(),
        );

        // 新しい座標系を再構築
        Self::from_origin_and_axes(
            new_origin,
            self.normal().as_vector(),
            self.u_axis().as_vector(),
        )
        .unwrap() // 元の座標系が有効なら、平行移動後も有効
    }

    /// 一様スケール変換
    ///
    /// # Arguments
    /// * `scale_factor` - スケール係数（正の値）
    /// * `scale_center` - スケールの中心点
    ///
    /// # Returns
    /// スケール変換後の座標系、無効なスケール係数の場合は None
    ///
    /// # Note
    /// - origin は scale_center を基準にスケーリング
    /// - 座標軸の方向は変更されない（単位ベクトルのまま）
    pub fn scale_uniform(&self, scale_factor: T, scale_center: Point3D<T>) -> Option<Self> {
        if scale_factor <= T::ZERO {
            return None;
        }

        // origin をスケール中心を基準にスケーリング
        let relative_origin = Vector3D::new(
            self.origin().x() - scale_center.x(),
            self.origin().y() - scale_center.y(),
            self.origin().z() - scale_center.z(),
        );

        let scaled_relative = relative_origin * scale_factor;
        let new_origin = Point3D::new(
            scale_center.x() + scaled_relative.x(),
            scale_center.y() + scaled_relative.y(),
            scale_center.z() + scaled_relative.z(),
        );

        // 座標系を再構築（座標軸は変更されない）
        Self::from_origin_and_axes(
            new_origin,
            self.normal().as_vector(),
            self.u_axis().as_vector(),
        )
    }

    /// 非一様スケール変換
    ///
    /// # Arguments
    /// * `scale_x` - X軸方向のスケール係数
    /// * `scale_y` - Y軸方向のスケール係数
    /// * `scale_z` - Z軸方向のスケール係数
    /// * `scale_center` - スケールの中心点
    ///
    /// # Returns
    /// スケール変換後の座標系、無効なスケール係数の場合は None
    ///
    /// # Note
    /// 非一様スケールでは座標軸の直交性が失われる可能性があるため、
    /// 変換後にグラム・シュミット正規直交化を適用
    pub fn scale_non_uniform(
        &self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
        scale_center: Point3D<T>,
    ) -> Option<Self> {
        if scale_x <= T::ZERO || scale_y <= T::ZERO || scale_z <= T::ZERO {
            return None;
        }

        // origin の非一様スケーリング
        let relative_origin = Vector3D::new(
            self.origin().x() - scale_center.x(),
            self.origin().y() - scale_center.y(),
            self.origin().z() - scale_center.z(),
        );

        let scaled_relative = Vector3D::new(
            relative_origin.x() * scale_x,
            relative_origin.y() * scale_y,
            relative_origin.z() * scale_z,
        );

        let new_origin = Point3D::new(
            scale_center.x() + scaled_relative.x(),
            scale_center.y() + scaled_relative.y(),
            scale_center.z() + scaled_relative.z(),
        );

        // 座標軸の非一様スケーリング
        let scaled_normal = Vector3D::new(
            self.normal().x() * scale_x,
            self.normal().y() * scale_y,
            self.normal().z() * scale_z,
        );

        let scaled_u_axis = Vector3D::new(
            self.u_axis().x() * scale_x,
            self.u_axis().y() * scale_y,
            self.u_axis().z() * scale_z,
        );

        // グラム・シュミット正規直交化で座標系を再構築
        Self::from_origin_and_axes(new_origin, scaled_normal, scaled_u_axis)
    }

    /// 点を中心とした3D回転
    ///
    /// # Arguments
    /// * `rotation_center` - 回転中心点
    /// * `rotation_axis` - 回転軸（正規化されたベクトル）
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 回転後の座標系、無効な回転軸の場合は None
    pub fn rotate_around_axis(
        &self,
        rotation_center: Point3D<T>,
        rotation_axis: Vector3D<T>,
        angle: T,
    ) -> Option<Self> {
        let axis = rotation_axis.normalize();
        if axis == Vector3D::new(T::ZERO, T::ZERO, T::ZERO) {
            return None;
        }

        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = T::ONE - cos_a;

        // ロドリゲスの回転公式用の成分
        let (ux, uy, uz) = (axis.x(), axis.y(), axis.z());

        // 回転行列の構築
        let rotation_matrix = [
            [
                cos_a + ux * ux * one_minus_cos,
                ux * uy * one_minus_cos - uz * sin_a,
                ux * uz * one_minus_cos + uy * sin_a,
            ],
            [
                uy * ux * one_minus_cos + uz * sin_a,
                cos_a + uy * uy * one_minus_cos,
                uy * uz * one_minus_cos - ux * sin_a,
            ],
            [
                uz * ux * one_minus_cos - uy * sin_a,
                uz * uy * one_minus_cos + ux * sin_a,
                cos_a + uz * uz * one_minus_cos,
            ],
        ];

        // origin を回転中心を基準に回転
        let relative_origin = Vector3D::new(
            self.origin().x() - rotation_center.x(),
            self.origin().y() - rotation_center.y(),
            self.origin().z() - rotation_center.z(),
        );

        let rotated_relative = Self::apply_rotation_matrix(&rotation_matrix, &relative_origin);
        let new_origin = Point3D::new(
            rotation_center.x() + rotated_relative.x(),
            rotation_center.y() + rotated_relative.y(),
            rotation_center.z() + rotated_relative.z(),
        );

        // 座標軸を回転
        let new_normal_vec =
            Self::apply_rotation_matrix(&rotation_matrix, &self.normal().as_vector());
        let new_u_axis_vec =
            Self::apply_rotation_matrix(&rotation_matrix, &self.u_axis().as_vector());

        let new_normal = Direction3D::from_vector(new_normal_vec)?;
        let new_u_axis = Direction3D::from_vector(new_u_axis_vec)?;

        // 座標系を再構築（v_axisは自動計算される）
        Self::from_origin_and_axes(new_origin, new_normal.as_vector(), new_u_axis.as_vector())
    }

    /// X軸周りの回転（簡易メソッド）
    pub fn rotate_x(&self, center: Point3D<T>, angle: T) -> Option<Self> {
        self.rotate_around_axis(center, Vector3D::new(T::ONE, T::ZERO, T::ZERO), angle)
    }

    /// Y軸周りの回転（簡易メソッド）
    pub fn rotate_y(&self, center: Point3D<T>, angle: T) -> Option<Self> {
        self.rotate_around_axis(center, Vector3D::new(T::ZERO, T::ONE, T::ZERO), angle)
    }

    /// Z軸周りの回転（簡易メソッド）
    pub fn rotate_z(&self, center: Point3D<T>, angle: T) -> Option<Self> {
        self.rotate_around_axis(center, Vector3D::new(T::ZERO, T::ZERO, T::ONE), angle)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// 回転行列をベクトルに適用
    fn apply_rotation_matrix(matrix: &[[T; 3]; 3], vector: &Vector3D<T>) -> Vector3D<T> {
        Vector3D::new(
            matrix[0][0] * vector.x() + matrix[0][1] * vector.y() + matrix[0][2] * vector.z(),
            matrix[1][0] * vector.x() + matrix[1][1] * vector.y() + matrix[1][2] * vector.z(),
            matrix[2][0] * vector.x() + matrix[2][1] * vector.y() + matrix[2][2] * vector.z(),
        )
    }

    /// 座標系のミラー変換（指定平面に対する鏡像）
    ///
    /// # Arguments
    /// * `mirror_origin` - 鏡像平面の原点
    /// * `mirror_normal` - 鏡像平面の法線
    ///
    /// # Returns
    /// ミラー変換後の座標系、無効な法線の場合は None
    pub fn mirror(&self, mirror_origin: Point3D<T>, mirror_normal: Vector3D<T>) -> Option<Self> {
        let normal = mirror_normal.normalize();
        if normal == Vector3D::new(T::ZERO, T::ZERO, T::ZERO) {
            return None;
        }

        // 点のミラー変換
        let mirror_point = |point: Point3D<T>| -> Point3D<T> {
            let to_point = Vector3D::new(
                point.x() - mirror_origin.x(),
                point.y() - mirror_origin.y(),
                point.z() - mirror_origin.z(),
            );
            let distance = to_point.dot(&normal);
            let reflected = to_point - normal * (T::from_f64(2.0) * distance);

            Point3D::new(
                mirror_origin.x() + reflected.x(),
                mirror_origin.y() + reflected.y(),
                mirror_origin.z() + reflected.z(),
            )
        };

        // ベクトルのミラー変換
        let mirror_vector = |vector: Vector3D<T>| -> Vector3D<T> {
            let distance = vector.dot(&normal);
            vector - normal * (T::from_f64(2.0) * distance)
        };

        let new_origin = mirror_point(self.origin());
        let new_normal_vec = mirror_vector(self.normal().as_vector());
        let new_u_axis_vec = mirror_vector(self.u_axis().as_vector());

        // ミラー変換後の座標系を再構築
        Self::from_origin_and_axes(new_origin, new_normal_vec, new_u_axis_vec)
    }
}

// ============================================================================
// BasicTransform Trait Implementation (Foundation パターン準拠)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Plane3DCoordinateSystem<T> {
    type Transformed = Plane3DCoordinateSystem<T>;
    type Vector2D = Vector3D<T>; // 3D なので Vector3D を使用
    type Point2D = Point3D<T>; // 3D なので Point3D を使用
    type Angle = Angle<T>;

    /// 平行移動（Foundation パターン統一インターフェース）
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        self.translate(&translation)
    }

    /// 回転（Z軸周りの回転として実装）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        self.rotate_z(center, angle.to_radians())
            .unwrap_or_else(|| {
                // エラー時は元の座標系を返す
                Self::from_origin_and_axes(
                    self.origin(),
                    self.normal().as_vector(),
                    self.u_axis().as_vector(),
                )
                .unwrap()
            })
    }

    /// スケール変換（一様スケール）
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        self.scale_uniform(factor, center).unwrap_or_else(|| {
            // エラー時は元の座標系を返す
            Self::from_origin_and_axes(
                self.origin(),
                self.normal().as_vector(),
                self.u_axis().as_vector(),
            )
            .unwrap()
        })
    }
}
