//! BBox3D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! 3D境界ボックスの変換では、変換後の8つの頂点から新しい境界ボックスを再計算

use crate::{BBox3D, Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for BBox3D<T> {
    type Transformed = BBox3D<T>;
    type Vector2D = Vector3D<T>; // 3D用にVector3Dを使用
    type Point2D = Point3D<T>; // 3D用にPoint3Dを使用
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
        BBox3D::new(new_min, new_max)
    }

    /// 指定中心での回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転後の新しい境界ボックス（8つの頂点を回転させて再計算）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 境界ボックスの8つの頂点を取得
        let vertices = self.get_all_vertices();

        // 各頂点を回転
        let rotated_vertices: Vec<Point3D<T>> = vertices
            .iter()
            .map(|vertex| BasicTransform::rotate(vertex, center, angle))
            .collect();

        // 回転した頂点から新しい境界ボックスを構築
        Self::from_transform_points(&rotated_vertices)
            .expect("8つの頂点から境界ボックスが作成できないはずがない")
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
        // 境界ボックスの8つの頂点をスケール
        let vertices = self.get_all_vertices();

        // 各頂点をスケール
        let scaled_vertices: Vec<Point3D<T>> = vertices
            .iter()
            .map(|vertex| BasicTransform::scale(vertex, center, factor))
            .collect();

        // スケールした頂点から新しい境界ボックスを構築
        Self::from_transform_points(&scaled_vertices)
            .expect("8つの頂点から境界ボックスが作成できないはずがない")
    }
}

// ============================================================================
// Required implementations for BasicTransform
// ============================================================================
// 追加のヘルパーメソッド
// ============================================================================

impl<T: Scalar> BBox3D<T> {
    /// 複数の点から境界ボックスを作成
    fn from_transform_points(points: &[Point3D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut max_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_y = points[0].y();
        let mut min_z = points[0].z();
        let mut max_z = points[0].z();

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
            min_z = min_z.min(point.z());
            max_z = max_z.max(point.z());
        }

        Some(Self::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        ))
    }

    /// 境界ボックスの8つの頂点をすべて取得
    pub fn get_all_vertices(&self) -> [Point3D<T>; 8] {
        let min = self.min();
        let max = self.max();

        [
            Point3D::from_vector(Vector3D::new(min.x(), min.y(), min.z())), // 前面左下
            Point3D::from_vector(Vector3D::new(max.x(), min.y(), min.z())), // 前面右下
            Point3D::from_vector(Vector3D::new(max.x(), max.y(), min.z())), // 前面右上
            Point3D::from_vector(Vector3D::new(min.x(), max.y(), min.z())), // 前面左上
            Point3D::from_vector(Vector3D::new(min.x(), min.y(), max.z())), // 背面左下
            Point3D::from_vector(Vector3D::new(max.x(), min.y(), max.z())), // 背面右下
            Point3D::from_vector(Vector3D::new(max.x(), max.y(), max.z())), // 背面右上
            Point3D::from_vector(Vector3D::new(min.x(), max.y(), max.z())), // 背面左上
        ]
    }

    /// 境界ボックスの体積を計算
    pub fn volume(&self) -> T {
        self.width() * self.height() * self.depth()
    }

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
        BBox3D::new(
            Point3D::new(
                self.min().x() - margin,
                self.min().y() - margin,
                self.min().z() - margin,
            ),
            Point3D::new(
                self.max().x() + margin,
                self.max().y() + margin,
                self.max().z() + margin,
            ),
        )
    }

    /// 境界ボックスを非等方でスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    /// * `scale_z` - Z軸方向のスケール倍率
    ///
    /// # 戻り値
    /// 非等方スケールされた新しい境界ボックス
    pub fn scale_non_uniform(
        &self,
        center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Self {
        let vertices = self.get_all_vertices();

        // 各頂点を非等方スケール
        let scaled_vertices: Vec<Point3D<T>> = vertices
            .iter()
            .map(|vertex| {
                let dx = vertex.x() - center.x();
                let dy = vertex.y() - center.y();
                let dz = vertex.z() - center.z();
                Point3D::new(
                    center.x() + dx * scale_x,
                    center.y() + dy * scale_y,
                    center.z() + dz * scale_z,
                )
            })
            .collect();

        Self::from_transform_points(&scaled_vertices)
            .expect("8つの頂点から境界ボックスが作成できないはずがない")
    }
}

// ============================================================================
// Tests Module
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate() {
        let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 3.0, 4.0));
        let translation = Vector3D::new(1.0, 2.0, 3.0);
        let translated = bbox.translate(translation);

        assert_eq!(translated.min(), Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(translated.max(), Point3D::new(3.0, 5.0, 7.0));
        assert_eq!(translated.width(), bbox.width());
        assert_eq!(translated.height(), bbox.height());
        assert_eq!(translated.depth(), bbox.depth());
    }

    #[test]
    fn test_scale_from_origin() {
        let bbox = BBox3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(3.0, 4.0, 5.0));
        let scaled = BasicTransform::scale(&bbox, Point3D::origin(), 2.0);

        assert_eq!(scaled.min(), Point3D::new(2.0, 2.0, 2.0));
        assert_eq!(scaled.max(), Point3D::new(6.0, 8.0, 10.0));
        assert_eq!(scaled.width(), bbox.width() * 2.0);
        assert_eq!(scaled.height(), bbox.height() * 2.0);
        assert_eq!(scaled.depth(), bbox.depth() * 2.0);
    }

    #[test]
    fn test_scale_uniform() {
        let bbox = BBox3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(3.0, 4.0, 5.0));
        let expanded = bbox.scale_uniform(2.0);

        // 中心点は変わらず、サイズが2倍に
        assert_eq!(expanded.center(), bbox.center());
        assert_eq!(expanded.width(), bbox.width() * 2.0);
        assert_eq!(expanded.height(), bbox.height() * 2.0);
        assert_eq!(expanded.depth(), bbox.depth() * 2.0);
    }

    #[test]
    fn test_expand_by_margin() {
        let bbox = BBox3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(3.0, 4.0, 5.0));
        let expanded = bbox.expand_by_margin(0.5);

        assert_eq!(expanded.min(), Point3D::new(0.5, 0.5, 0.5));
        assert_eq!(expanded.max(), Point3D::new(3.5, 4.5, 5.5));
        assert_eq!(expanded.width(), bbox.width() + 1.0);
        assert_eq!(expanded.height(), bbox.height() + 1.0);
        assert_eq!(expanded.depth(), bbox.depth() + 1.0);
    }

    #[test]
    fn test_scale_non_uniform() {
        let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 4.0, 6.0));
        let center = Point3D::new(1.0, 2.0, 3.0);
        let scaled = bbox.scale_non_uniform(center, 2.0, 0.5, 1.5);

        // X軸方向に2倍、Y軸方向に0.5倍、Z軸方向に1.5倍
        assert_eq!(scaled.width(), bbox.width() * 2.0);
        assert_eq!(scaled.height(), bbox.height() * 0.5);
        assert_eq!(scaled.depth(), bbox.depth() * 1.5);
    }

    #[test]
    fn test_get_all_vertices() {
        let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 2.0, 3.0));
        let vertices = bbox.get_all_vertices();

        assert_eq!(vertices.len(), 8);
        assert!(vertices.contains(&Point3D::new(0.0, 0.0, 0.0))); // 前面左下
        assert!(vertices.contains(&Point3D::new(1.0, 2.0, 3.0))); // 背面右上
    }

    #[test]
    fn test_volume() {
        let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 3.0, 4.0));

        assert_eq!(bbox.volume(), 24.0); // 2 * 3 * 4 = 24
    }
}
