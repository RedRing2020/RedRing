//! Point2D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! 2D点の基本的な変換機能を提供

use crate::{Point2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Point2D<T> {
    type Transformed = Point2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい点
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        Point2D::new(self.x() + translation.x(), self.y() + translation.y())
    }

    /// 指定中心での回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい点
    fn rotate(&self, center: Self::Point2D, _angle: Self::Angle) -> Self::Transformed {
        // 簡易実装：現在は回転なしで点を返す（将来的にanalysis行列を使用予定）
        let dx = self.x() - center.x();
        let dy = self.y() - center.y();

        // TODO: 実際の回転計算を実装
        Point2D::new(center.x() + dx, center.y() + dy)
    }

    /// 指定中心でのスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい点
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        let dx = self.x() - center.x();
        let dy = self.y() - center.y();

        Point2D::new(center.x() + dx * factor, center.y() + dy * factor)
    }
}

// ============================================================================
// Required implementations for BasicTransform
// Note: Default trait is already implemented in point_2d.rs
// ============================================================================

// ============================================================================
// Tests Module
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::extensions::TransformHelpers;

    #[test]
    fn test_translate() {
        let point = Point2D::new(1.0, 2.0);
        let translation = Vector2D::new(2.0, 3.0);
        let translated = point.translate(translation);

        assert_eq!(translated.x(), 3.0);
        assert_eq!(translated.y(), 5.0);
    }

    #[test]
    fn test_scale_origin() {
        let point = Point2D::new(2.0, 3.0);
        let scaled = point.scale_origin(2.0);

        assert_eq!(scaled.x(), 4.0);
        assert_eq!(scaled.y(), 6.0);
    }

    #[test]
    fn test_translate_axes() {
        let point = Point2D::new(1.0, 2.0);

        let translated_x = point.translate_x(1.0);
        assert_eq!(translated_x, Point2D::new(2.0, 2.0));

        let translated_y = point.translate_y(1.0);
        assert_eq!(translated_y, Point2D::new(1.0, 3.0));

        let translated_xy = point.translate_xy(1.0, 1.0);
        assert_eq!(translated_xy, Point2D::new(2.0, 3.0));
    }

    #[test]
    fn test_scale_from_center() {
        let point = Point2D::new(4.0, 6.0);
        let center = Point2D::new(2.0, 3.0);
        let scaled = BasicTransform::scale(&point, center, 2.0);

        // (4-2)*2+2=6, (6-3)*2+3=9
        assert_eq!(scaled.x(), 6.0);
        assert_eq!(scaled.y(), 9.0);
    }
}
