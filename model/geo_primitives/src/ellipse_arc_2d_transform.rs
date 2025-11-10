//! EllipseArc2D Transform実装
//!
//! BasicTransform と AdvancedTransform の完全実装
//! 楕円弧の変換操作に特化した実装

use crate::{EllipseArc2D, LineSegment2D, Point2D, Vector2D};
use analysis::linalg::Matrix3x3;
use geo_foundation::extensions::{AdvancedTransform, BasicTransform};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// BasicTransform Implementation
// ============================================================================

impl<T: Scalar> BasicTransform<T> for EllipseArc2D<T> {
    type Transformed = EllipseArc2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// 楕円弧の基底楕円を平行移動し、角度範囲は保持
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい楕円弧
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        Self::Transformed::new(
            self.ellipse().translate(&translation),
            self.start_angle(),
            self.end_angle(),
        )
    }

    /// 回転
    ///
    /// 楕円弧を指定された点を中心に回転
    /// 基底楕円と角度範囲の両方を回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい楕円弧
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        Self::Transformed::new(
            self.ellipse().rotate(center, angle),
            self.start_angle() + angle,
            self.end_angle() + angle,
        )
    }

    /// スケール
    ///
    /// 楕円弧を指定された点を中心にスケール
    /// 基底楕円をスケールし、角度範囲は保持
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい楕円弧
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        Self::Transformed::new(
            self.ellipse().scale(center, factor),
            self.start_angle(),
            self.end_angle(),
        )
    }
}

// ============================================================================
// AdvancedTransform Implementation
// ============================================================================

impl<T: Scalar> AdvancedTransform<T> for EllipseArc2D<T> {
    type Line2D = LineSegment2D<T>;
    type Matrix3 = Matrix3x3<T>;

    /// 鏡像反転
    ///
    /// 楕円弧を指定された軸に対して鏡像反転
    /// 基底楕円を反転し、角度範囲も適切に反転
    ///
    /// # 引数
    /// * `axis` - 反転軸となる直線
    ///
    /// # 戻り値
    /// 鏡像反転された新しい楕円弧
    fn mirror(&self, axis: Self::Line2D) -> Self::Transformed {
        // 基底楕円を鏡像反転
        let mirrored_ellipse = self.ellipse().mirror(axis);

        // 角度範囲の反転処理
        // 鏡像反転では角度の順序が逆転する
        let mirrored_start = self.mirror_angle(self.end_angle(), axis);
        let mirrored_end = self.mirror_angle(self.start_angle(), axis);

        Self::Transformed::new(mirrored_ellipse, mirrored_start, mirrored_end)
    }

    /// 非等方スケール
    ///
    /// 楕円弧をX軸・Y軸で異なる倍率でスケール
    /// 基底楕円を非等方スケールし、角度範囲は再計算
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    ///
    /// # 戻り値
    /// 非等方スケールされた新しい楕円弧
    fn non_uniform_scale(
        &self,
        center: Self::Point2D,
        scale_x: T,
        scale_y: T,
    ) -> Self::Transformed {
        // 基底楕円を非等方スケール
        let scaled_ellipse = self.ellipse().non_uniform_scale(scale_x, scale_y);

        // 非等方スケールでは角度が変化するため再計算が必要
        let start_point = self.start_point();
        let end_point = self.end_point();

        // スケール後の点での角度を再計算
        let scaled_start = Point2D::new(
            (start_point.x() - center.x()) * scale_x + center.x(),
            (start_point.y() - center.y()) * scale_y + center.y(),
        );
        let scaled_end = Point2D::new(
            (end_point.x() - center.x()) * scale_x + center.x(),
            (end_point.y() - center.y()) * scale_y + center.y(),
        );

        let new_start_angle = self.calculate_angle_for_point(&scaled_ellipse, &scaled_start);
        let new_end_angle = self.calculate_angle_for_point(&scaled_ellipse, &scaled_end);

        Self::Transformed::new(scaled_ellipse, new_start_angle, new_end_angle)
    }

    /// 行列変換
    ///
    /// 楕円弧を3x3変換行列で変換
    /// アフィン変換により基底楕円と角度範囲を変換
    ///
    /// # 引数
    /// * `matrix` - 3x3変換行列
    ///
    /// # 戻り値
    /// 行列変換された新しい楕円弧
    fn transform_matrix(&self, matrix: &Self::Matrix3) -> Self::Transformed {
        // 基底楕円を行列変換
        let transformed_ellipse = self.ellipse().transform_matrix(matrix);

        // 開始点と終了点を変換
        let start_point = self.start_point();
        let end_point = self.end_point();

        // 同次座標での変換
        let start_homogeneous =
            analysis::linalg::Vector3::new(start_point.x(), start_point.y(), T::ONE);
        let end_homogeneous = analysis::linalg::Vector3::new(end_point.x(), end_point.y(), T::ONE);

        let transformed_start_homogeneous = *matrix * start_homogeneous;
        let transformed_end_homogeneous = *matrix * end_homogeneous;

        // 同次座標から2D座標に変換
        let transformed_start = Point2D::new(
            transformed_start_homogeneous.x() / transformed_start_homogeneous.z(),
            transformed_start_homogeneous.y() / transformed_start_homogeneous.z(),
        );
        let transformed_end = Point2D::new(
            transformed_end_homogeneous.x() / transformed_end_homogeneous.z(),
            transformed_end_homogeneous.y() / transformed_end_homogeneous.z(),
        );

        // 変換後の楕円での角度を再計算
        let new_start_angle =
            self.calculate_angle_for_point(&transformed_ellipse, &transformed_start);
        let new_end_angle = self.calculate_angle_for_point(&transformed_ellipse, &transformed_end);

        Self::Transformed::new(transformed_ellipse, new_start_angle, new_end_angle)
    }

    /// 反転
    ///
    /// 楕円弧の方向を反転（開始角度と終了角度を交換）
    ///
    /// # 戻り値
    /// 方向反転された新しい楕円弧
    fn reverse(&self) -> Self::Transformed {
        Self::Transformed::new(*self.ellipse(), self.end_angle(), self.start_angle())
    }
}

// ============================================================================
// Helper Methods
// ============================================================================

impl<T: Scalar> EllipseArc2D<T> {
    /// 角度を鏡像反転用にヘルパー計算
    fn mirror_angle(&self, angle: Angle<T>, axis: LineSegment2D<T>) -> Angle<T> {
        // 軸の方向角度を取得
        let axis_direction = axis.direction();
        let axis_angle = axis_direction.y().atan2(axis_direction.x());

        // 角度の鏡像反転公式: 2 * axis_angle - original_angle
        let mirrored_radians = axis_angle * (T::ONE + T::ONE) - angle.to_radians();
        Angle::from_radians(mirrored_radians)
    }

    /// 楕円上の点に対応する角度を計算
    fn calculate_angle_for_point(
        &self,
        ellipse: &crate::Ellipse2D<T>,
        point: &Point2D<T>,
    ) -> Angle<T> {
        let center = ellipse.center();
        let to_point = Vector2D::new(point.x() - center.x(), point.y() - center.y());

        // 楕円の回転を考慮した角度計算
        let rotation = ellipse.rotation();
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        // 回転を逆変換して楕円の標準形での角度を求める
        let x_local = to_point.x() * cos_rot + to_point.y() * sin_rot;
        let y_local = -to_point.x() * sin_rot + to_point.y() * cos_rot;

        // 楕円の標準形での角度
        let angle_rad = y_local.atan2(x_local);
        Angle::from_radians(angle_rad)
    }
}

// ============================================================================
// Composite Transform Operations
// ============================================================================

impl<T: Scalar> EllipseArc2D<T> {
    /// 複合変換：平行移動 + 回転
    pub fn translate_and_rotate(
        &self,
        translation: Vector2D<T>,
        center: Point2D<T>,
        angle: Angle<T>,
    ) -> Self {
        let translated = BasicTransform::translate(self, translation);
        BasicTransform::rotate(&translated, center, angle)
    }

    /// 複合変換：スケール + 回転
    pub fn scale_and_rotate(
        &self,
        scale_center: Point2D<T>,
        scale_factor: T,
        rotation_center: Point2D<T>,
        rotation_angle: Angle<T>,
    ) -> Self {
        let scaled = BasicTransform::scale(self, scale_center, scale_factor);
        BasicTransform::rotate(&scaled, rotation_center, rotation_angle)
    }

    /// 複合変換：非等方スケール + 平行移動
    pub fn scale_non_uniform_and_translate(
        &self,
        scale_center: Point2D<T>,
        scale_x: T,
        scale_y: T,
        translation: Vector2D<T>,
    ) -> Self {
        let scaled =
            <Self as AdvancedTransform<T>>::non_uniform_scale(self, scale_center, scale_x, scale_y);
        BasicTransform::translate(&scaled, translation)
    }
}

// ============================================================================
// Validation and Utility Methods
// ============================================================================

impl<T: Scalar> EllipseArc2D<T> {
    /// 変換後の楕円弧の妥当性チェック
    pub fn is_valid_after_transform(&self) -> bool {
        // 基底楕円の妥当性
        if self.ellipse().semi_major() <= T::ZERO || self.ellipse().semi_minor() <= T::ZERO {
            return false;
        }

        // 角度範囲の妥当性
        let angle_diff = (self.end_angle() - self.start_angle()).to_radians().abs();
        angle_diff <= T::TAU && angle_diff > T::ZERO
    }

    /// 変換の等価性チェック（テスト用）
    pub fn transform_equivalent(&self, other: &Self, tolerance: T) -> bool {
        // 基底楕円の等価性（簡易チェック）
        let center_diff = self
            .ellipse()
            .center()
            .distance_to(&other.ellipse().center());
        let major_diff = (self.ellipse().semi_major() - other.ellipse().semi_major()).abs();
        let minor_diff = (self.ellipse().semi_minor() - other.ellipse().semi_minor()).abs();

        if center_diff > tolerance || major_diff > tolerance || minor_diff > tolerance {
            return false;
        }

        // 角度の等価性（2π周期性を考慮）
        let start_diff = (self.start_angle() - other.start_angle()).to_radians();
        let end_diff = (self.end_angle() - other.end_angle()).to_radians();

        let start_normalized = start_diff - (start_diff / T::TAU).round() * T::TAU;
        let end_normalized = end_diff - (end_diff / T::TAU).round() * T::TAU;

        start_normalized.abs() < tolerance && end_normalized.abs() < tolerance
    }

    /// 変換による点の対応チェック
    pub fn point_correspondence_after_transform<F>(
        &self,
        other: &Self,
        transform: F,
        tolerance: T,
    ) -> bool
    where
        F: Fn(Point2D<T>) -> Point2D<T>,
    {
        // サンプル点での対応チェック
        let sample_params = [
            T::ZERO,
            T::ONE / (T::ONE + T::ONE + T::ONE + T::ONE),
            T::ONE / (T::ONE + T::ONE),
            T::ONE,
        ];

        for &param in &sample_params {
            let original_point = self.point_at_parameter(param);
            let transformed_point = transform(original_point);
            let expected_point = other.point_at_parameter(param);

            if transformed_point.distance_to(&expected_point) > tolerance {
                return false;
            }
        }

        true
    }
}

// ============================================================================
// Performance Optimized Methods
// ============================================================================

impl<T: Scalar> EllipseArc2D<T> {
    /// 高速平行移動（インプレース風）
    pub fn fast_translate(&self, translation: Vector2D<T>) -> Self {
        // 楕円弧では角度変更がないため、楕円のみ変換
        Self::new(
            self.ellipse().translate(&translation),
            self.start_angle(),
            self.end_angle(),
        )
    }

    /// 高速回転（角度のみ変更）
    pub fn fast_rotate_around_center(&self, angle: Angle<T>) -> Self {
        // 中心周りの回転では楕円形状は変わらず、角度のみ変更
        if self.ellipse().center() == Point2D::origin() {
            Self::new(
                *self.ellipse(),
                self.start_angle() + angle,
                self.end_angle() + angle,
            )
        } else {
            BasicTransform::rotate(self, self.ellipse().center(), angle)
        }
    }

    /// 高速等方スケール
    pub fn fast_uniform_scale(&self, center: Point2D<T>, factor: T) -> Self {
        // 等方スケールでは角度は変わらない
        Self::new(
            BasicTransform::scale(self.ellipse(), center, factor),
            self.start_angle(),
            self.end_angle(),
        )
    }
}
