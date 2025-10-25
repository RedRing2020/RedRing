//! Arc3D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! Arc3Dの変換では中心点・法線・開始方向の変換と半径のスケールを適用

use crate::{Angle, Arc3D, Direction3D, Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Arc3D<T> {
    type Transformed = Arc3D<T>;
    type Vector2D = Vector3D<T>; // 3Dなので Vector3D を使用
    type Point2D = Point3D<T>; // 3Dなので Point3D を使用
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// 円弧の中心点を移動し、法線・開始方向・角度・半径は変更しない
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい円弧
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_center = BasicTransform::translate(&self.center(), translation);
        Arc3D::new(
            new_center,
            self.radius(),
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .expect("元の円弧が有効なら変換後も有効なはず")
    }

    /// 回転
    ///
    /// 指定中心周りでの回転変換（3D回転）
    /// 法線ベクトルと開始方向も同様に回転される
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（現在は簡易実装でZ軸回転のみ）
    ///
    /// # 戻り値
    /// 回転された新しい円弧
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 簡易実装：Z軸周りの回転のみ
        let new_center = BasicTransform::rotate(&self.center(), center, angle);

        // 法線と開始方向も回転（Z軸回転では法線は変わらない場合が多いが一般化）
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 法線ベクトルの回転（Z軸回転行列適用）
        let normal_vec = Vector3D::new(
            self.normal().x() * cos_a - self.normal().y() * sin_a,
            self.normal().x() * sin_a + self.normal().y() * cos_a,
            self.normal().z(),
        );

        // 開始方向ベクトルの回転
        let start_dir_vec = Vector3D::new(
            self.start_direction().x() * cos_a - self.start_direction().y() * sin_a,
            self.start_direction().x() * sin_a + self.start_direction().y() * cos_a,
            self.start_direction().z(),
        );

        let new_normal = Direction3D::from_vector(normal_vec).unwrap_or(self.normal());
        let new_start_dir =
            Direction3D::from_vector(start_dir_vec).unwrap_or(self.start_direction());

        Arc3D::new(
            new_center,
            self.radius(),
            new_normal,
            new_start_dir,
            self.start_angle(),
            self.end_angle(),
        )
        .expect("元の円弧が有効なら変換後も有効なはず")
    }

    /// スケール変換
    ///
    /// 指定中心周りでの拡大縮小
    /// 半径がスケールされ、中心位置も変更される
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい円弧（スケールが0以下の場合はNone）
    fn scale(&self, center: Self::Point2D, scale: T) -> Self::Transformed {
        if scale <= T::ZERO {
            panic!("スケール倍率は正の値である必要があります");
        }

        let new_center = BasicTransform::scale(&self.center(), center, scale);
        let new_radius = self.radius() * scale;

        Arc3D::new(
            new_center,
            new_radius,
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .expect("元の円弧が有効なら変換後も有効なはず")
    }
}

// ============================================================================
// Arc3D-specific Transform Methods
// ============================================================================

impl<T: Scalar> Arc3D<T> {
    /// 半径のみのスケール変換
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// 新しい半径の円弧（失敗時はNone）
    pub fn scale_radius(&self, factor: T) -> Option<Self> {
        let new_radius = self.radius() * factor;
        if new_radius > T::ZERO {
            Arc3D::new(
                self.center(),
                new_radius,
                self.normal(),
                self.start_direction(),
                self.start_angle(),
                self.end_angle(),
            )
        } else {
            None
        }
    }

    /// 均等スケール（中心固定）
    ///
    /// # 引数
    /// * `scale` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい円弧
    pub fn scale_uniform(&self, scale: T) -> Option<Self> {
        if scale <= T::ZERO {
            return None;
        }

        let new_radius = self.radius() * scale;
        Arc3D::new(
            self.center(),
            new_radius,
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
    }

    /// 中心点の変更
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// 新しい中心の円弧
    pub fn with_center(&self, new_center: Point3D<T>) -> Self {
        Arc3D::new(
            new_center,
            self.radius(),
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .unwrap()
    }

    /// 半径の変更
    ///
    /// # 引数
    /// * `new_radius` - 新しい半径
    ///
    /// # 戻り値
    /// 新しい半径の円弧（失敗時はNone）
    pub fn with_radius(&self, new_radius: T) -> Option<Self> {
        if new_radius > T::ZERO {
            Arc3D::new(
                self.center(),
                new_radius,
                self.normal(),
                self.start_direction(),
                self.start_angle(),
                self.end_angle(),
            )
        } else {
            None
        }
    }

    /// 法線方向の変更
    ///
    /// # 引数
    /// * `new_normal` - 新しい法線方向
    ///
    /// # 戻り値
    /// 新しい法線の円弧（直交性チェック）
    pub fn with_normal(&self, new_normal: Direction3D<T>) -> Option<Self> {
        Arc3D::new(
            self.center(),
            self.radius(),
            new_normal,
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
    }

    /// 開始方向の変更
    ///
    /// # 引数
    /// * `new_start_dir` - 新しい開始方向
    ///
    /// # 戻り値
    /// 新しい開始方向の円弧（直交性チェック）
    pub fn with_start_direction(&self, new_start_dir: Direction3D<T>) -> Option<Self> {
        Arc3D::new(
            self.center(),
            self.radius(),
            self.normal(),
            new_start_dir,
            self.start_angle(),
            self.end_angle(),
        )
    }

    /// 角度範囲の変更
    ///
    /// # 引数
    /// * `new_start` - 新しい開始角度
    /// * `new_end` - 新しい終了角度
    ///
    /// # 戻り値
    /// 新しい角度範囲の円弧
    pub fn with_angle_range(&self, new_start: Angle<T>, new_end: Angle<T>) -> Option<Self> {
        Arc3D::new(
            self.center(),
            self.radius(),
            self.normal(),
            self.start_direction(),
            new_start,
            new_end,
        )
    }

    /// 円弧を指定点まで延長
    ///
    /// # 引数
    /// * `point` - 延長目標点
    ///
    /// # 戻り値
    /// 延長された円弧（同じ平面上の点の場合）
    pub fn extend_to_point(&self, point: Point3D<T>) -> Option<Self> {
        // 簡易実装：点が円弧平面上にあるかは厳密にはチェックしない
        let radius_to_point = self.center().distance_to(&point);

        // 新しい半径で円弧を作成
        Arc3D::new(
            self.center(),
            radius_to_point,
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
    }
}
