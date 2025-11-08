//! SphericalSurface3D の安全な変換操作
//!
//! 失敗する可能性がある変換操作をResult型で提供し、
//! 鏡像スケール（負のスケール）を防止し、幾何学的制約を適用します。
//!
//! ## 主な特徴
//! - **スケール制限**: 半径がトレランス以下にならないよう制限
//! - **鏡像防止**: 負のスケール倍率を拒否
//! - **数値安定性**: NaN、無限大の検出と拒否
//!
//! **作成日: 2025年11月8日**
//! **最終更新: 2025年11月8日**

use crate::{Point3D, SphericalSurface3D};
use geo_foundation::{extensions::transform_error::TransformError, GeometricTolerance, Scalar};

/// SphericalSurface3D の安全な変換操作
impl<T: Scalar + GeometricTolerance> SphericalSurface3D<T> {
    /// 安全な均一スケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(SphericalSurface3D)` - スケール後の球面
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール倍率が無限大またはNaN
    /// - スケール後の半径がトレランス以下
    pub fn safe_scale_origin(&self, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        // スケール後の半径を計算
        let new_radius = self.radius() * factor;

        // 半径の幾何学的制約チェック
        let min_radius = T::DISTANCE_TOLERANCE;
        if new_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半径({:?})がトレランス({:?})以下になります",
                new_radius, min_radius
            )));
        }

        // 数値安定性チェック
        if !new_radius.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効です".to_string(),
            ));
        }

        // 中心をスケール
        let scaled_center = Point3D::new(
            self.center().x() * factor,
            self.center().y() * factor,
            self.center().z() * factor,
        );

        // 新しい球面を作成
        Self::new(
            scaled_center,
            self.axis().to_vector(),
            self.ref_direction().to_vector(),
            new_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後の球面の作成に失敗しました".to_string(),
        ))
    }

    /// 安全な均一スケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(SphericalSurface3D)` - スケール後の球面
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール中心点です".to_string(),
            ));
        }

        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        // スケール後の半径を計算
        let new_radius = self.radius() * factor;

        // 半径の幾何学的制約チェック
        let min_radius = T::DISTANCE_TOLERANCE;
        if new_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半径({:?})がトレランス({:?})以下になります",
                new_radius, min_radius
            )));
        }

        // 中心をスケール
        let center_to_sphere = self.center().to_vector() - center.to_vector();
        let scaled_offset = center_to_sphere * factor;
        let new_center_vector = center.to_vector() + scaled_offset;
        let scaled_center = Point3D::new(
            new_center_vector.x(),
            new_center_vector.y(),
            new_center_vector.z(),
        );

        // 数値安定性チェック
        if !scaled_center.x().is_finite()
            || !scaled_center.y().is_finite()
            || !scaled_center.z().is_finite()
            || !new_radius.is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効です".to_string(),
            ));
        }

        // 新しい球面を作成
        Self::new(
            scaled_center,
            self.axis().to_vector(),
            self.ref_direction().to_vector(),
            new_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後の球面の作成に失敗しました".to_string(),
        ))
    }

    /// 安全な半径のみスケール（中心固定）
    ///
    /// # 引数
    /// * `factor` - 半径のスケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(SphericalSurface3D)` - 半径スケール後の球面
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    pub fn safe_scale_radius(&self, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "半径スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        // スケール後の半径を計算
        let new_radius = self.radius() * factor;

        // 半径の幾何学的制約チェック
        let min_radius = T::DISTANCE_TOLERANCE;
        if new_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半径({:?})がトレランス({:?})以下になります",
                new_radius, min_radius
            )));
        }

        // 数値安定性チェック
        if !new_radius.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "半径スケール計算結果が無効です".to_string(),
            ));
        }

        // 新しい球面を作成（中心は変更せず）
        Self::new(
            self.center(),
            self.axis().to_vector(),
            self.ref_direction().to_vector(),
            new_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "半径スケール後の球面の作成に失敗しました".to_string(),
        ))
    }

    /// 安全な新しい半径設定
    ///
    /// # 引数
    /// * `new_radius` - 新しい半径（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(SphericalSurface3D)` - 新しい半径の球面
    /// * `Err(TransformError)` - 無効な半径値
    pub fn safe_with_radius(&self, new_radius: T) -> Result<Self, TransformError> {
        // 半径の有効性チェック
        if new_radius <= T::ZERO || !new_radius.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "半径は正の有限値である必要があります".to_string(),
            ));
        }

        // 半径の幾何学的制約チェック
        let min_radius = T::DISTANCE_TOLERANCE;
        if new_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "半径({:?})がトレランス({:?})以下です",
                new_radius, min_radius
            )));
        }

        // 新しい球面を作成
        Self::new(
            self.center(),
            self.axis().to_vector(),
            self.ref_direction().to_vector(),
            new_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい半径での球面の作成に失敗しました".to_string(),
        ))
    }

    /// スケール倍率の事前検証
    ///
    /// # 引数
    /// * `factor` - チェックするスケール倍率
    ///
    /// # 戻り値
    /// スケール倍率が有効かどうか
    pub fn validate_scale_factor(&self, factor: T) -> bool {
        if factor <= T::ZERO || !factor.is_finite() {
            return false;
        }

        let new_radius = self.radius() * factor;
        let min_radius = T::DISTANCE_TOLERANCE;

        new_radius >= min_radius && new_radius.is_finite()
    }

    /// 最小許容スケール倍率を取得
    ///
    /// # 戻り値
    /// この球面に適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_radius = T::DISTANCE_TOLERANCE;
        let current_radius = self.radius();

        if current_radius <= T::ZERO {
            T::ZERO
        } else {
            // 最小半径を維持するための倍率
            // スケール後半径 = 現在半径 × 倍率 >= トレランス
            // 最小倍率 = トレランス / 現在半径
            min_radius / current_radius
        }
    }
}
