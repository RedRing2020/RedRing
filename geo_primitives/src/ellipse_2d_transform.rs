//! Ellipse2D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! Ellipse2Dの変換では中心点・軸長・回転角の変換を適用

use crate::{Ellipse2D, LineSegment2D, Point2D, Vector2D};
use analysis::linalg::Matrix3x3;
use geo_foundation::{
    extensions::{AdvancedTransform, BasicTransform},
    Angle, Scalar,
};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Ellipse2D<T> {
    type Transformed = Ellipse2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// 楕円の中心点を移動し、軸長・回転角は変更しない
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい楕円
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_center = BasicTransform::translate(&self.center(), translation);
        Ellipse2D::new(
            new_center,
            self.semi_major(),
            self.semi_minor(),
            self.rotation(),
        )
        .expect("元の楕円が有効なら変換後も有効なはず")
    }

    /// 回転
    ///
    /// 指定中心周りでの回転変換
    /// 楕円の中心点と回転角が変更される
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい楕円
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let new_center = BasicTransform::rotate(&self.center(), center, angle);
        let new_rotation = self.rotation() + angle.to_radians();

        Ellipse2D::new(
            new_center,
            self.semi_major(),
            self.semi_minor(),
            new_rotation,
        )
        .expect("回転後も楕円は有効なはず")
    }

    /// スケール変換
    ///
    /// 指定中心周りでの拡大縮小
    /// 軸長がスケールされ、中心位置も変更される
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい楕円
    fn scale(&self, center: Self::Point2D, scale: T) -> Self::Transformed {
        if scale <= T::ZERO {
            panic!("スケール倍率は正の値である必要があります");
        }

        let new_center = BasicTransform::scale(&self.center(), center, scale);
        let new_semi_major = self.semi_major() * scale;
        let new_semi_minor = self.semi_minor() * scale;

        Ellipse2D::new(new_center, new_semi_major, new_semi_minor, self.rotation())
            .expect("スケール後も楕円は有効なはず")
    }
}

// ============================================================================
// AdvancedTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> AdvancedTransform<T> for Ellipse2D<T> {
    type Line2D = LineSegment2D<T>;
    type Matrix3 = Matrix3x3<T>;

    /// 鏡像反転
    fn mirror(&self, axis: Self::Line2D) -> Self::Transformed {
        // LineSegment2D を軸とした鏡像反転
        let axis_start = axis.start_point();
        let axis_end = axis.end_point();

        // 軸ベクトル
        let axis_vec = Vector2D::new(axis_end.x() - axis_start.x(), axis_end.y() - axis_start.y());
        let axis_len_sq = axis_vec.x() * axis_vec.x() + axis_vec.y() * axis_vec.y();

        if axis_len_sq <= T::ZERO {
            // 軸が点の場合は元の楕円を返す
            return *self;
        }

        // 中心点の鏡像反転
        let center = self.center();
        let to_center = Vector2D::new(center.x() - axis_start.x(), center.y() - axis_start.y());

        // 軸上への射影
        let projection =
            (to_center.x() * axis_vec.x() + to_center.y() * axis_vec.y()) / axis_len_sq;
        let projection_point = Point2D::new(
            axis_start.x() + projection * axis_vec.x(),
            axis_start.y() + projection * axis_vec.y(),
        );

        // 鏡像点
        let mirrored_center = Point2D::new(
            T::from_f64(2.0) * projection_point.x() - center.x(),
            T::from_f64(2.0) * projection_point.y() - center.y(),
        );

        // 回転角も反転（軸の角度に応じて）
        let axis_angle = axis_vec.y().atan2(axis_vec.x());
        let new_rotation = T::from_f64(2.0) * axis_angle - self.rotation();

        Ellipse2D::new(
            mirrored_center,
            self.semi_major(),
            self.semi_minor(),
            new_rotation,
        )
        .expect("鏡像反転後も楕円は有効なはず")
    }

    /// 非等方スケール
    fn scale_non_uniform(
        &self,
        center: Self::Point2D,
        scale_x: T,
        scale_y: T,
    ) -> Self::Transformed {
        if scale_x <= T::ZERO || scale_y <= T::ZERO {
            panic!("Scale factors must be positive");
        }

        // 楕円の非等方スケールは複雑な計算が必要
        // 簡略化として、軸ごとのスケールを適用
        let new_center = Point2D::new(
            center.x() + (self.center().x() - center.x()) * scale_x,
            center.y() + (self.center().y() - center.y()) * scale_y,
        );

        // 回転角を考慮したスケール（近似）
        let cos_r = self.rotation().cos();
        let sin_r = self.rotation().sin();

        // 新しい半軸を計算（回転を考慮した近似）
        let scale_major = (scale_x * cos_r * cos_r + scale_y * sin_r * sin_r).sqrt();
        let scale_minor = (scale_x * sin_r * sin_r + scale_y * cos_r * cos_r).sqrt();

        let new_semi_major = self.semi_major() * scale_major;
        let new_semi_minor = self.semi_minor() * scale_minor;

        // 軸長の大小関係を維持
        let (final_major, final_minor) = if new_semi_major >= new_semi_minor {
            (new_semi_major, new_semi_minor)
        } else {
            (new_semi_minor, new_semi_major)
        };

        Ellipse2D::new(new_center, final_major, final_minor, self.rotation())
            .expect("非等方スケール後も楕円は有効なはず")
    }

    /// アフィン変換行列による変換
    fn transform_matrix(&self, matrix: &Self::Matrix3) -> Self::Transformed {
        // 中心点を変換 (Vector2D から analysis::Vector2 に変換)
        let center_vec =
            analysis::linalg::vector::Vector2::new(self.center().x(), self.center().y());
        let transformed_center_vec = matrix.transform_point_2d(&center_vec);
        let new_center = Point2D::new(transformed_center_vec.x(), transformed_center_vec.y());

        // 行列からスケール成分を抽出
        let m11 = matrix.get(0, 0);
        let m12 = matrix.get(0, 1);
        let m21 = matrix.get(1, 0);
        let m22 = matrix.get(1, 1);

        // 特異値分解を簡略化（主要な軸変換を近似）
        let scale_x = (m11 * m11 + m21 * m21).sqrt();
        let scale_y = (m12 * m12 + m22 * m22).sqrt();

        // 回転成分を取得
        let rotation_angle = m21.atan2(m11);
        let new_rotation = self.rotation() + rotation_angle;

        // 軸長をスケール
        let new_semi_major = self.semi_major() * scale_x;
        let new_semi_minor = self.semi_minor() * scale_y;

        // 軸長の大小関係を維持
        let (final_major, final_minor) = if new_semi_major >= new_semi_minor {
            (new_semi_major, new_semi_minor)
        } else {
            (new_semi_minor, new_semi_major)
        };

        Ellipse2D::new(new_center, final_major, final_minor, new_rotation)
            .expect("行列変換後も楕円は有効なはず")
    }

    /// 反転（向きの逆転）
    fn reverse(&self) -> Self::Transformed {
        // 楕円では「向き」の概念が明確でないため、回転角を180度回転として実装
        let new_rotation = self.rotation() + T::PI;

        Ellipse2D::new(
            self.center(),
            self.semi_major(),
            self.semi_minor(),
            new_rotation,
        )
        .expect("反転後も楕円は有効なはず")
    }
}

// ============================================================================
// Ellipse2D-specific Transform Methods
// ============================================================================

impl<T: Scalar> Ellipse2D<T> {
    /// X, Y座標での個別平行移動
    pub fn translate_xy(&self, dx: T, dy: T) -> Self {
        let translation = Vector2D::new(dx, dy);
        <Self as BasicTransform<T>>::translate(self, translation)
    }

    /// ベクトルによる平行移動
    pub fn translate(&self, vector: &Vector2D<T>) -> Self {
        self.translate_xy(vector.x(), vector.y())
    }

    /// 原点中心でのスケール変換
    pub fn scale_origin(&self, factor: T) -> Self {
        <Self as BasicTransform<T>>::scale(self, Point2D::origin(), factor)
    }

    /// 指定点を中心とした回転
    pub fn rotate_around_point(&self, center: &Point2D<T>, angle: Angle<T>) -> Self {
        <Self as BasicTransform<T>>::rotate(self, *center, angle)
    }

    /// 指定点を中心とした拡大縮小
    pub fn scale_around_point(&self, center: &Point2D<T>, factor: T) -> Self {
        <Self as BasicTransform<T>>::scale(self, *center, factor)
    }

    /// 長軸のみのスケール変換
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// 新しい長軸の楕円（失敗時はNone）
    pub fn scale_major_axis(&self, factor: T) -> Option<Self> {
        let new_semi_major = self.semi_major() * factor;
        if new_semi_major > T::ZERO && new_semi_major >= self.semi_minor() {
            Ellipse2D::new(
                self.center(),
                new_semi_major,
                self.semi_minor(),
                self.rotation(),
            )
        } else {
            None
        }
    }

    /// 短軸のみのスケール変換
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// 新しい短軸の楕円（失敗時はNone）
    pub fn scale_minor_axis(&self, factor: T) -> Option<Self> {
        let new_semi_minor = self.semi_minor() * factor;
        if new_semi_minor > T::ZERO && self.semi_major() >= new_semi_minor {
            Ellipse2D::new(
                self.center(),
                self.semi_major(),
                new_semi_minor,
                self.rotation(),
            )
        } else {
            None
        }
    }

    /// 中心点の変更
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// 新しい中心の楕円
    pub fn with_center(&self, new_center: Point2D<T>) -> Self {
        Ellipse2D::new(
            new_center,
            self.semi_major(),
            self.semi_minor(),
            self.rotation(),
        )
        .unwrap()
    }

    /// 回転角の変更
    ///
    /// # 引数
    /// * `new_rotation` - 新しい回転角（ラジアン）
    ///
    /// # 戻り値
    /// 新しい回転角の楕円
    pub fn with_rotation(&self, new_rotation: T) -> Self {
        Ellipse2D::new(
            self.center(),
            self.semi_major(),
            self.semi_minor(),
            new_rotation,
        )
        .unwrap()
    }

    /// 軸長の変更
    ///
    /// # 引数
    /// * `new_semi_major` - 新しい長半軸
    /// * `new_semi_minor` - 新しい短半軸
    ///
    /// # 戻り値
    /// 新しい軸長の楕円（失敗時はNone）
    pub fn with_axes(&self, new_semi_major: T, new_semi_minor: T) -> Option<Self> {
        Ellipse2D::new(
            self.center(),
            new_semi_major,
            new_semi_minor,
            self.rotation(),
        )
    }

    /// 均等拡大縮小（互換性維持）
    pub fn uniform_scale(&self, factor: T) -> Self {
        self.scale_origin(factor)
    }

    /// 非均等拡大縮小（互換性維持）
    pub fn non_uniform_scale(&self, x_factor: T, y_factor: T) -> Self {
        <Self as AdvancedTransform<T>>::scale_non_uniform(
            self,
            Point2D::origin(),
            x_factor,
            y_factor,
        )
    }

    /// 水平反転（Y軸に対する鏡像）（互換性維持）
    pub fn flip_horizontal(&self) -> Self {
        let center = self.center();
        let flipped_center = Point2D::new(-center.x(), center.y());
        let new_rotation = T::PI - self.rotation();

        Ellipse2D::new(
            flipped_center,
            self.semi_major(),
            self.semi_minor(),
            new_rotation,
        )
        .expect("Horizontal flip should preserve ellipse validity")
    }

    /// 垂直反転（X軸に対する鏡像）（互換性維持）
    pub fn flip_vertical(&self) -> Self {
        let center = self.center();
        let flipped_center = Point2D::new(center.x(), -center.y());
        let new_rotation = -self.rotation();

        Ellipse2D::new(
            flipped_center,
            self.semi_major(),
            self.semi_minor(),
            new_rotation,
        )
        .expect("Vertical flip should preserve ellipse validity")
    }
}
