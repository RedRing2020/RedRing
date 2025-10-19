//! BBox2D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! 境界ボックスの変換では、変換後の頂点から新しい境界ボックスを再計算

use crate::{BBox2D, Point2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for BBox2D<T> {
    type Transformed = BBox2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい境界ボックス
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        // 境界ボックスの平行移動は min と max を同じベクトルで移動
        let new_min = self.min().translate(translation);
        let new_max = self.max().translate(translation);
        BBox2D::new(new_min, new_max)
    }

    /// 指定中心での回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転後の新しい境界ボックス（4つの角を回転させて再計算）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 境界ボックスの4つの角を取得
        let corners = [
            self.min(),                                   // 左下
            Point2D::new(self.max().x(), self.min().y()), // 右下
            self.max(),                                   // 右上
            Point2D::new(self.min().x(), self.max().y()), // 左上
        ];

        // 各角を回転
        let rotated_corners: Vec<Point2D<T>> = corners
            .iter()
            .map(|corner| BasicTransform::rotate(corner, center, angle))
            .collect();

        // 回転した角から新しい境界ボックスを構築
        BBox2D::from_points(&rotated_corners)
            .expect("4つの角から境界ボックスが作成できないはずがない")
    }

    /// 指定中心でのスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい境界ボックス
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        // 境界ボックスの4つの角をスケール
        let corners = [
            self.min(),
            Point2D::new(self.max().x(), self.min().y()),
            self.max(),
            Point2D::new(self.min().x(), self.max().y()),
        ];

        // 各角をスケール
        let scaled_corners: Vec<Point2D<T>> = corners
            .iter()
            .map(|corner| BasicTransform::scale(corner, center, factor))
            .collect();

        // スケールした角から新しい境界ボックスを構築
        BBox2D::from_points(&scaled_corners)
            .expect("4つの角から境界ボックスが作成できないはずがない")
    }
}

// ============================================================================
// Required implementations for BasicTransform
// ============================================================================

impl<T: Scalar> Default for BBox2D<T> {
    fn default() -> Self {
        // 原点を中心とした単位正方形
        BBox2D::centered_square(T::ONE / (T::ONE + T::ONE))
    }
}

// ============================================================================
// 特別な変換メソッド（境界ボックス固有）
// ============================================================================

impl<T: Scalar> BBox2D<T> {
    /// 境界ボックスを指定の比率でスケール拡張/縮小
    ///
    /// # 引数
    /// * `factor` - 拡張倍率（1.0より大きいと拡張、小さいと縮小）
    ///
    /// # 戻り値
    /// 拡張/縮小された新しい境界ボックス
    pub fn scale_uniform(&self, factor: T) -> Self {
        let center = self.center();
        BasicTransform::scale(self, center, factor)
    }

    /// 境界ボックスを指定のマージンで拡張
    ///
    /// # 引数
    /// * `margin` - 全方向への拡張マージン
    ///
    /// # 戻り値
    /// マージンで拡張された新しい境界ボックス
    pub fn expand_by_margin(&self, margin: T) -> Self {
        BBox2D::new(
            Point2D::new(self.min().x() - margin, self.min().y() - margin),
            Point2D::new(self.max().x() + margin, self.max().y() + margin),
        )
    }

    /// 境界ボックスを非等方でスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    ///
    /// # 戻り値
    /// 非等方スケールされた新しい境界ボックス
    pub fn scale_non_uniform(&self, center: Point2D<T>, scale_x: T, scale_y: T) -> Self {
        let corners = [
            self.min(),
            Point2D::new(self.max().x(), self.min().y()),
            self.max(),
            Point2D::new(self.min().x(), self.max().y()),
        ];

        // 各角を非等方スケール
        let scaled_corners: Vec<Point2D<T>> = corners
            .iter()
            .map(|corner| {
                let dx = corner.x() - center.x();
                let dy = corner.y() - center.y();
                Point2D::new(center.x() + dx * scale_x, center.y() + dy * scale_y)
            })
            .collect();

        BBox2D::from_points(&scaled_corners)
            .expect("4つの角から境界ボックスが作成できないはずがない")
    }
}

// ============================================================================
// Tests Module
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::extensions::TransformHelpers;

    #[test]
    fn test_translate() {
        let bbox = BBox2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 3.0));
        let translation = Vector2D::new(1.0, 2.0);
        let translated = BasicTransform::translate(&bbox, translation);

        assert_eq!(translated.min(), Point2D::new(1.0, 2.0));
        assert_eq!(translated.max(), Point2D::new(3.0, 5.0));
        assert_eq!(translated.width(), bbox.width());
        assert_eq!(translated.height(), bbox.height());
    }

    #[test]
    fn test_scale_origin() {
        let bbox = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 4.0));
        let scaled = bbox.scale_origin(2.0);

        assert_eq!(scaled.min(), Point2D::new(2.0, 2.0));
        assert_eq!(scaled.max(), Point2D::new(6.0, 8.0));
        assert_eq!(scaled.width(), bbox.width() * 2.0);
        assert_eq!(scaled.height(), bbox.height() * 2.0);
    }

    #[test]
    fn test_scale_uniform() {
        let bbox = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 4.0));
        let expanded = bbox.scale_uniform(2.0);

        // 中心点は変わらず、サイズが2倍に
        assert_eq!(expanded.center(), bbox.center());
        assert_eq!(expanded.width(), bbox.width() * 2.0);
        assert_eq!(expanded.height(), bbox.height() * 2.0);
    }

    #[test]
    fn test_expand_by_margin() {
        let bbox = BBox2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 4.0));
        let expanded = bbox.expand_by_margin(0.5);

        assert_eq!(expanded.min(), Point2D::new(0.5, 0.5));
        assert_eq!(expanded.max(), Point2D::new(3.5, 4.5));
        assert_eq!(expanded.width(), bbox.width() + 1.0);
        assert_eq!(expanded.height(), bbox.height() + 1.0);
    }

    #[test]
    fn test_scale_non_uniform() {
        let bbox = BBox2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 4.0));
        let center = Point2D::new(1.0, 2.0);
        let scaled = bbox.scale_non_uniform(center, 2.0, 0.5);

        // X軸方向に2倍、Y軸方向に0.5倍
        assert_eq!(scaled.width(), bbox.width() * 2.0);
        assert_eq!(scaled.height(), bbox.height() * 0.5);
    }

    #[test]
    fn test_translate_xy() {
        let bbox = BBox2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 3.0));
        let translated = bbox.translate_xy(1.0, 2.0);

        assert_eq!(translated.min(), Point2D::new(1.0, 2.0));
        assert_eq!(translated.max(), Point2D::new(3.0, 5.0));
    }
}
