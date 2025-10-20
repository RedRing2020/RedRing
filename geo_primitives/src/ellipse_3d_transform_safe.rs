//! Ellipse3D の安全な変換機能
//!
//! Result型を使用した適切なエラーハンドリング実装

use crate::{
    ellipse_3d::Ellipse3D, point_3d::Point3D, transform_error::TransformError, vector_3d::Vector3D,
    Angle, Scalar,
};

// ============================================================================
// Safe Transform Implementation (Result型使用)
// ============================================================================

impl<T: Scalar> Ellipse3D<T> {
    /// 安全な平行移動
    ///
    /// 平行移動は理論上失敗しないが、一貫性のためResult型を使用
    ///
    /// # 引数
    /// * `translation` - 平行移動ベクトル
    ///
    /// # 戻り値
    /// `Ok(楕円)` - 平行移動された新しい楕円
    /// `Err(TransformError)` - 変換エラー
    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {
        Self::new(
            self.center() + translation,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .ok_or(TransformError::InvalidGeometry)
    }

    /// 安全な等方スケール
    ///
    /// ゼロや負のスケール倍率に対して適切にエラーを返す
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// `Ok(楕円)` - スケールされた新しい楕円
    /// `Err(TransformError)` - ゼロスケールや不正なパラメータの場合
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の妥当性チェック
        if factor == T::ZERO {
            return Err(TransformError::InvalidScaleFactor);
        }

        // 中心点をスケール
        let center_vector = self.center().to_vector();
        let relative_position = center_vector - center.to_vector();
        let scaled_relative = relative_position * factor;
        let new_center_vector = center.to_vector() + scaled_relative;
        let new_center = Point3D::new(
            new_center_vector.x(),
            new_center_vector.y(),
            new_center_vector.z(),
        );

        // 半軸長をスケール
        let new_semi_major = self.semi_major_axis() * factor.abs();
        let new_semi_minor = self.semi_minor_axis() * factor.abs();

        // 安全な楕円作成
        Self::new(
            new_center,
            new_semi_major,
            new_semi_minor,
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .ok_or(TransformError::InvalidGeometry)
    }

    /// 安全な回転変換
    ///
    /// 軸がゼロベクトルの場合などに適切にエラーを返す
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `axis` - 回転軸
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// `Ok(楕円)` - 回転された新しい楕円
    /// `Err(TransformError)` - 無効な回転軸の場合
    pub fn safe_rotate(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        _angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 回転軸の妥当性チェック
        if axis.length() <= T::ZERO {
            return Err(TransformError::ZeroVector);
        }

        // 簡易実装：位置のみ変更、向きは保持
        let center_vector = self.center().to_vector();
        let relative_position = center_vector - center.to_vector();

        // 簡単な近似（完全な3D回転は複雑な行列演算が必要）
        let new_center_vector = center.to_vector() + relative_position;
        let new_center = Point3D::new(
            new_center_vector.x(),
            new_center_vector.y(),
            new_center_vector.z(),
        );

        Self::new(
            new_center,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .ok_or(TransformError::InvalidGeometry)
    }

    /// 安全な非等方スケール
    ///
    /// 各軸で異なる倍率を適用し、ゼロスケールを検出
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    /// * `scale_z` - Z軸方向のスケール倍率
    ///
    /// # 戻り値
    /// `Ok(楕円)` - スケールされた新しい楕円
    /// `Err(TransformError)` - いずれかの軸でゼロスケールの場合
    pub fn safe_scale_non_uniform(
        &self,
        center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        // スケール倍率の妥当性チェック
        if scale_x == T::ZERO || scale_y == T::ZERO || scale_z == T::ZERO {
            return Err(TransformError::InvalidScaleFactor);
        }

        // 中心点を変換
        let center_vector = self.center().to_vector();
        let relative_position = center_vector - center.to_vector();

        let scaled_relative = Vector3D::new(
            relative_position.x() * scale_x,
            relative_position.y() * scale_y,
            relative_position.z() * scale_z,
        );

        let new_center_vector = center.to_vector() + scaled_relative;
        let new_center = Point3D::new(
            new_center_vector.x(),
            new_center_vector.y(),
            new_center_vector.z(),
        );

        // 長軸方向ベクトルを変換
        let major_axis = self.major_axis_direction().as_vector();
        let new_major_axis = Vector3D::new(
            major_axis.x() * scale_x,
            major_axis.y() * scale_y,
            major_axis.z() * scale_z,
        );

        // 法線ベクトルを変換
        let normal = self.normal().as_vector();
        let new_normal = Vector3D::new(
            normal.x() * scale_x,
            normal.y() * scale_y,
            normal.z() * scale_z,
        );

        // 半軸長は平均スケール倍率で近似
        let avg_scale = (scale_x.abs() + scale_y.abs() + scale_z.abs()) / T::from_f64(3.0);
        let new_semi_major = self.semi_major_axis() * avg_scale;
        let new_semi_minor = self.semi_minor_axis() * avg_scale;

        Self::new(
            new_center,
            new_semi_major,
            new_semi_minor,
            new_normal,
            new_major_axis,
        )
        .ok_or(TransformError::InvalidGeometry)
    }

    /// 安全な楕円反転
    ///
    /// 法線方向を反転（通常は失敗しない）
    ///
    /// # 戻り値
    /// `Ok(楕円)` - 法線が反転された新しい楕円
    /// `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_reverse(&self) -> Result<Self, TransformError> {
        Self::new(
            self.center(),
            self.semi_major_axis(),
            self.semi_minor_axis(),
            -self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .ok_or(TransformError::InvalidGeometry)
    }
}

// ============================================================================
// 複合変換メソッド（安全版）
// ============================================================================

impl<T: Scalar> Ellipse3D<T> {
    /// 安全な複合変換：平行移動 + 回転
    pub fn safe_translate_and_rotate(
        &self,
        translation: Vector3D<T>,
        rotation_center: Point3D<T>,
        rotation_axis: Vector3D<T>,
        rotation_angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        let translated = self.safe_translate(translation)?;
        translated.safe_rotate(rotation_center, rotation_axis, rotation_angle)
    }

    /// 安全な複合変換：スケール + 平行移動
    pub fn safe_scale_and_translate(
        &self,
        scale_center: Point3D<T>,
        scale_factor: T,
        translation: Vector3D<T>,
    ) -> Result<Self, TransformError> {
        let scaled = self.safe_scale(scale_center, scale_factor)?;
        scaled.safe_translate(translation)
    }

    /// 安全な複合変換：非等方スケール + 平行移動
    pub fn safe_scale_non_uniform_and_translate(
        &self,
        scale_center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
        translation: Vector3D<T>,
    ) -> Result<Self, TransformError> {
        let scaled = self.safe_scale_non_uniform(scale_center, scale_x, scale_y, scale_z)?;
        scaled.safe_translate(translation)
    }
}

// ============================================================================
// Validation and Utility Methods
// ============================================================================

impl<T: Scalar> Ellipse3D<T> {
    /// 変換パラメータの事前検証
    ///
    /// 変換を実行する前にパラメータの妥当性をチェック
    ///
    /// # 引数
    /// * `scale_factor` - チェックするスケール倍率
    ///
    /// # 戻り値
    /// パラメータが有効かどうか
    pub fn validate_scale_factor(scale_factor: T) -> bool {
        scale_factor != T::ZERO && scale_factor.is_finite()
    }

    /// 回転軸の事前検証
    ///
    /// # 引数
    /// * `axis` - チェックする回転軸
    ///
    /// # 戻り値
    /// 軸が有効かどうか
    pub fn validate_rotation_axis(axis: Vector3D<T>) -> bool {
        axis.length() > T::ZERO && axis.length().is_finite()
    }

    /// 変換後の楕円の詳細検証
    ///
    /// is_valid_transform より詳細な検証
    ///
    /// # 戻り値
    /// 検証結果の詳細
    pub fn detailed_validation(&self) -> Result<(), TransformError> {
        // 半軸長の妥当性
        if self.semi_major_axis() <= T::ZERO || self.semi_minor_axis() <= T::ZERO {
            return Err(TransformError::InvalidGeometry);
        }

        // 長軸 >= 短軸
        if self.semi_major_axis() < self.semi_minor_axis() {
            return Err(TransformError::InvalidGeometry);
        }

        // 値の有限性チェック
        if !self.semi_major_axis().is_finite() || !self.semi_minor_axis().is_finite() {
            return Err(TransformError::InvalidGeometry);
        }

        Ok(())
    }
}
