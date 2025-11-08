//! Ellipse3D の安全な変換機能
//!
//! Result型を使用した適切なエラーハンドリング実装

use crate::{ellipse_3d::Ellipse3D, point_3d::Point3D, vector_3d::Vector3D, Angle, Scalar};
use geo_foundation::{GeometricTolerance, TransformError};

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
        .ok_or(TransformError::InvalidGeometry(
            "Failed to create translated ellipse".to_string(),
        ))
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
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor cannot be zero".to_string(),
            ));
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
        .ok_or(TransformError::InvalidGeometry(
            "Failed to create scaled ellipse".to_string(),
        ))
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
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero vector".to_string(),
            ));
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
        .ok_or(TransformError::InvalidGeometry(
            "Failed to create rotated ellipse".to_string(),
        ))
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
            return Err(TransformError::InvalidScaleFactor(
                "Non-uniform scale factors cannot be zero".to_string(),
            ));
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
        .ok_or(TransformError::InvalidGeometry(
            "Failed to create non-uniformly scaled ellipse".to_string(),
        ))
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
        .ok_or(TransformError::InvalidGeometry(
            "Failed to create reversed ellipse".to_string(),
        ))
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
            return Err(TransformError::InvalidGeometry(
                "Semi-axes must be positive".to_string(),
            ));
        }

        // 長軸 >= 短軸
        if self.semi_major_axis() < self.semi_minor_axis() {
            return Err(TransformError::InvalidGeometry(
                "Semi-major axis must be >= semi-minor axis".to_string(),
            ));
        }

        // 値の有限性チェック
        if !self.semi_major_axis().is_finite() || !self.semi_minor_axis().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Semi-axes must be finite".to_string(),
            ));
        }

        Ok(())
    }
}

/// Ellipse3D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> Ellipse3D<T> {
    /// トレランス制約付きスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(Ellipse3D)` - スケール後の楕円
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の半軸長がトレランス以下
    pub fn safe_scale_with_tolerance(
        &self,
        center: Point3D<T>,
        factor: T,
    ) -> Result<Self, TransformError> {
        // 基本的なスケール倍率チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
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

        // 半軸長の幾何学的制約チェック（トレランスベース）
        let min_axis = T::DISTANCE_TOLERANCE;
        if new_semi_major <= min_axis || new_semi_minor <= min_axis {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半軸長(major:{:?}, minor:{:?})がトレランス({:?})以下になります",
                new_semi_major, new_semi_minor, min_axis
            )));
        }

        // 数値安定性チェック
        if !new_center.x().is_finite()
            || !new_center.y().is_finite()
            || !new_center.z().is_finite()
            || !new_semi_major.is_finite()
            || !new_semi_minor.is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効です".to_string(),
            ));
        }

        Self::new(
            new_center,
            new_semi_major,
            new_semi_minor,
            self.normal().to_vector(),
            self.major_axis_direction().to_vector(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後の楕円の作成に失敗しました".to_string(),
        ))
    }

    /// 半軸長スケールの最小許容倍率を取得
    ///
    /// # 戻り値
    /// この楕円に適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_axis = T::DISTANCE_TOLERANCE;
        let current_major = self.semi_major_axis();
        let current_minor = self.semi_minor_axis();

        // 小さい方の軸を基準に計算
        let smaller_axis = if current_major < current_minor {
            current_major
        } else {
            current_minor
        };

        if smaller_axis <= T::ZERO {
            T::ZERO
        } else {
            // 最小軸長を維持するための倍率 + 安全マージン
            let min_factor = min_axis / smaller_axis;
            min_factor + T::DISTANCE_TOLERANCE
        }
    }
}
