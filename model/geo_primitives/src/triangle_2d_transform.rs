//! Triangle2D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作

use crate::{Point2D, Triangle2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Triangle2D<T> {
    type Transformed = Triangle2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい三角形
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        // 3頂点をそれぞれ平行移動
        let transformed_a = BasicTransform::translate(&self.vertex_a(), translation);
        let transformed_b = BasicTransform::translate(&self.vertex_b(), translation);
        let transformed_c = BasicTransform::translate(&self.vertex_c(), translation);

        // 変換済み頂点から三角形再構築
        Triangle2D::new(transformed_a, transformed_b, transformed_c)
            .expect("3つの変換済み頂点から三角形が作成できないはずがない")
    }

    /// 指定中心での回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転後の新しい三角形（3つの頂点を一括回転して再構築）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 3頂点をそれぞれ回転
        let transformed_a = BasicTransform::rotate(&self.vertex_a(), center, angle);
        let transformed_b = BasicTransform::rotate(&self.vertex_b(), center, angle);
        let transformed_c = BasicTransform::rotate(&self.vertex_c(), center, angle);

        // 回転済み頂点から新しい三角形を構築
        Triangle2D::new(transformed_a, transformed_b, transformed_c)
            .expect("3つの変換済み頂点から三角形が作成できないはずがない")
    }

    /// 指定中心でのスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい三角形
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        // 3頂点をそれぞれスケール
        let transformed_a = BasicTransform::scale(&self.vertex_a(), center, factor);
        let transformed_b = BasicTransform::scale(&self.vertex_b(), center, factor);
        let transformed_c = BasicTransform::scale(&self.vertex_c(), center, factor);

        // スケール済み頂点から新しい三角形を構築
        Triangle2D::new(transformed_a, transformed_b, transformed_c)
            .expect("3つの変換済み頂点から三角形が作成できないはずがない")
    }
}

// Default実装は triangle_2d.rs で提供済み

// ============================================================================
// Triangle2D 固有の Transform メソッド
// ============================================================================

impl<T: Scalar> Triangle2D<T> {
    /// 重心を中心とした均等スケール
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい三角形
    pub fn scale_from_centroid(&self, factor: T) -> Self {
        let centroid = self.centroid();
        BasicTransform::scale(self, centroid, factor)
    }

    /// 各辺を個別にスケール（非等方スケール）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor_a` - 頂点Aの距離スケール
    /// * `factor_b` - 頂点Bの距離スケール  
    /// * `factor_c` - 頂点Cの距離スケール
    ///
    /// # 戻り値
    /// 非等方スケールされた新しい三角形
    pub fn scale_vertices_individual(
        &self,
        center: Point2D<T>,
        factor_a: T,
        factor_b: T,
        factor_c: T,
    ) -> Option<Self> {
        let vec_a = self.vertex_a() - center;
        let vec_b = self.vertex_b() - center;
        let vec_c = self.vertex_c() - center;

        let new_a = center + vec_a * factor_a;
        let new_b = center + vec_b * factor_b;
        let new_c = center + vec_c * factor_c;

        Triangle2D::new(new_a, new_b, new_c)
    }

    /// 三角形を指定方向にシアー変換
    ///
    /// # 引数
    /// * `shear_factor` - シアー係数
    /// * `direction` - シアー方向（正規化されたベクトル）
    ///
    /// # 戻り値
    /// シアー変換された新しい三角形
    pub fn shear(&self, shear_factor: T, direction: Vector2D<T>) -> Option<Self> {
        // Matrix3x3によるシアー変換（将来実装）
        // 現在は基本的な実装
        let dir_norm = direction.normalize();
        if dir_norm.length().is_zero() {
            return None;
        }

        let transform_point = |p: Point2D<T>| -> Point2D<T> {
            let vec = p.to_vector();
            let dot_product = vec.dot(&dir_norm);
            let shear_offset = dir_norm * (dot_product * shear_factor);
            let result_vec = vec + shear_offset;
            Point2D::new(result_vec.x(), result_vec.y())
        };

        Triangle2D::new(
            transform_point(self.vertex_a()),
            transform_point(self.vertex_b()),
            transform_point(self.vertex_c()),
        )
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
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.5, 1.0),
        )
        .unwrap();
        let translation = Vector2D::new(2.0, 3.0);
        let translated = triangle.translate(translation);

        assert_eq!(translated.vertex_a(), Point2D::new(2.0, 3.0));
        assert_eq!(translated.vertex_b(), Point2D::new(3.0, 3.0));
        assert_eq!(translated.vertex_c(), Point2D::new(2.5, 4.0));
    }

    #[test]
    fn test_rotate_origin() {
        use std::f64::consts::PI;
        let triangle = Triangle2D::new(
            Point2D::new(1.0, 0.0),
            Point2D::new(0.0, 1.0),
            Point2D::new(-1.0, 0.0),
        )
        .unwrap();
        let center = Point2D::origin();
        let angle = Angle::from_radians(PI / 2.0); // 90度回転

        let rotated = triangle.rotate(center, angle);

        // 90度回転後の頂点位置を確認（許容誤差内）
        assert!((rotated.vertex_a().x() - 0.0).abs() < 1e-10);
        assert!((rotated.vertex_a().y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_uniform() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 2.0),
        )
        .unwrap();
        let center = Point2D::origin();
        let factor = 2.0;

        let scaled = triangle.scale(center, factor);

        assert_eq!(scaled.vertex_a(), Point2D::new(0.0, 0.0));
        assert_eq!(scaled.vertex_b(), Point2D::new(4.0, 0.0));
        assert_eq!(scaled.vertex_c(), Point2D::new(2.0, 4.0));
    }

    #[test]
    fn test_scale_from_centroid() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(3.0, 0.0),
            Point2D::new(1.5, 3.0),
        )
        .unwrap();
        let scaled = triangle.scale_from_centroid(2.0);

        // 重心は変わらず、各頂点は重心からの距離が2倍になる
        assert_eq!(scaled.centroid(), triangle.centroid());

        // 面積は4倍になる
        let original_area = triangle.area();
        let scaled_area = scaled.area();
        assert!((scaled_area - original_area * 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_bulk_transform_vertices_consistency() {
        // 3頂点一括変換の整合性テスト
        let triangle = Triangle2D::new(
            Point2D::new(1.0, 2.0),
            Point2D::new(3.0, 1.0),
            Point2D::new(2.0, 4.0),
        )
        .unwrap();
        let translation = Vector2D::new(5.0, 10.0);

        // 一括変換結果
        let bulk_transformed = triangle.translate(translation);
        let bulk_vertices = [
            bulk_transformed.vertex_a(),
            bulk_transformed.vertex_b(),
            bulk_transformed.vertex_c(),
        ];

        // 個別変換結果（比較用）
        let original_vertices = [
            triangle.vertex_a(),
            triangle.vertex_b(),
            triangle.vertex_c(),
        ];
        let individual_vertices: Vec<Point2D<f64>> = original_vertices
            .iter()
            .map(|v| v.translate(translation))
            .collect();

        // 3頂点すべてが個別変換と一致することを確認
        for (bulk_vertex, &individual_vertex) in
            bulk_vertices.iter().zip(individual_vertices.iter())
        {
            assert!((bulk_vertex.x() - individual_vertex.x()).abs() < 1e-10);
            assert!((bulk_vertex.y() - individual_vertex.y()).abs() < 1e-10);
        }
    }
}
