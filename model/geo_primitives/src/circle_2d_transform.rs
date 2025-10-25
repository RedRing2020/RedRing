//! Circle2D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! Circle2Dの変換では中心点の変換と半径のスケールを適用

use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Circle2D<T> {
    type Transformed = Circle2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// 円の中心点を移動し、半径は変更しない
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい円
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_center = BasicTransform::translate(&self.center(), translation);
        Circle2D::new(new_center, self.radius()).expect("元の円が有効なら変換後も有効なはず")
    }

    /// 指定中心での回転
    ///
    /// 回転中心周りに円の中心点を回転し、半径は変更しない
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい円
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let new_center = BasicTransform::rotate(&self.center(), center, angle);
        Circle2D::new(new_center, self.radius()).expect("元の円が有効なら変換後も有効なはず")
    }

    /// 指定中心でのスケール
    ///
    /// 中心点からの距離と半径の両方をスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい円
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        let new_center = BasicTransform::scale(&self.center(), center, factor);
        let new_radius = self.radius() * factor;
        Circle2D::new(new_center, new_radius).expect("正のスケール係数なら半径も正のはず")
    }
}

// ============================================================================
// Circle2D 固有の Transform メソッド
// ============================================================================

impl<T: Scalar> Circle2D<T> {
    /// 円の半径のみをスケール（中心は固定）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// 半径のみスケールされた新しい円
    pub fn scale_radius(&self, factor: T) -> Option<Self> {
        let new_radius = self.radius() * factor;
        if new_radius > T::ZERO {
            Circle2D::new(self.center(), new_radius)
        } else {
            None
        }
    }

    /// 円を一様スケール（中心を基準）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい円
    pub fn scale_uniform(&self, factor: T) -> Option<Self> {
        self.scale_radius(factor)
    }

    /// 円の中心のみを移動
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// 新しい中心の円
    pub fn with_center(&self, new_center: Point2D<T>) -> Self {
        Circle2D::new(new_center, self.radius()).unwrap()
    }

    /// 円の半径のみを変更
    ///
    /// # 引数
    /// * `new_radius` - 新しい半径
    ///
    /// # 戻り値
    /// 新しい半径の円（失敗時はNone）
    pub fn with_radius(&self, new_radius: T) -> Option<Self> {
        if new_radius > T::ZERO {
            Circle2D::new(self.center(), new_radius)
        } else {
            None
        }
    }

    /// 円を指定の点を通るように拡大/縮小
    ///
    /// 中心から指定点までの距離を新しい半径とする
    ///
    /// # 引数
    /// * `point` - 新しい境界となる点
    ///
    /// # 戻り値
    /// 指定点を通る新しい円
    pub fn expand_to_point(&self, point: Point2D<T>) -> Self {
        let new_radius = self.center().distance_to(&point);
        Circle2D::new(self.center(), new_radius).unwrap()
    }

    /// 円を非等方変換（楕円化は未サポート、X/Yスケールが同じ場合のみ）
    ///
    /// # 引数
    /// * `center` - 変換中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    ///
    /// # 戻り値
    /// 等方スケールの場合は円、非等方の場合はNone（楕円は未実装）
    pub fn scale_non_uniform(&self, center: Point2D<T>, scale_x: T, scale_y: T) -> Option<Self> {
        // 非等方スケールは円として表現できないため、等方の場合のみサポート
        if (scale_x - scale_y).abs() < T::EPSILON {
            // 等方スケールとして処理
            Some(BasicTransform::scale(self, center, scale_x))
        } else {
            // 非等方スケールは楕円になるため、Circle2Dでは表現不可
            None
        }
    }
}

// ============================================================================
// Required implementations for BasicTransform
// ============================================================================

impl<T: Scalar> Default for Circle2D<T> {
    fn default() -> Self {
        Circle2D::new(Point2D::origin(), T::ONE).expect("半径1の円は常に有効")
    }
}
