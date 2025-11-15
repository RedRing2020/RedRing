//! Triangle3D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作

use crate::{Point3D, Triangle3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Triangle3D<T> {
    type Transformed = Triangle3D<T>;
    type Vector2D = Vector3D<T>; // 3D なので Vector3D を使用
    type Point2D = Point3D<T>; // 3D なので Point3D を使用
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
        Triangle3D::new(transformed_a, transformed_b, transformed_c)
            .expect("3つの変換済み頂点から三角形が作成できないはずがない")
    }

    /// 指定中心でのZ軸回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - Z軸回転角度
    ///
    /// # 戻り値
    /// 回転後の新しい三角形（3つの頂点を一括回転して再構築）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 3頂点をそれぞれZ軸回転（BasicTransformは Z軸回転のみサポート）
        let transformed_a = BasicTransform::rotate(&self.vertex_a(), center, angle);
        let transformed_b = BasicTransform::rotate(&self.vertex_b(), center, angle);
        let transformed_c = BasicTransform::rotate(&self.vertex_c(), center, angle);

        // 回転済み頂点から新しい三角形を構築
        Triangle3D::new(transformed_a, transformed_b, transformed_c)
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
        Triangle3D::new(transformed_a, transformed_b, transformed_c)
            .expect("3つの変換済み頂点から三角形が作成できないはずがない")
    }
}

// ============================================================================
// Required implementations for BasicTransform
// ============================================================================

// Default実装は triangle_3d.rs で提供済み

// ============================================================================
// Triangle3D 固有の Transform メソッド
// ============================================================================

impl<T: Scalar> Triangle3D<T> {
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
        center: Point3D<T>,
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

        Triangle3D::new(new_a, new_b, new_c)
    }

    /// X軸周りの回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// X軸回転された新しい三角形
    pub fn rotate_x(&self, center: Point3D<T>, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let transform_point = |p: Point3D<T>| -> Point3D<T> {
            let translated = p - center;
            let rotated = Point3D::new(
                translated.x(),
                translated.y() * cos_a - translated.z() * sin_a,
                translated.y() * sin_a + translated.z() * cos_a,
            );
            center + rotated.to_vector()
        };

        Triangle3D::new(
            transform_point(self.vertex_a()),
            transform_point(self.vertex_b()),
            transform_point(self.vertex_c()),
        )
        .expect("X軸回転後の頂点は常に有効")
    }

    /// Y軸周りの回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// Y軸回転された新しい三角形
    pub fn rotate_y(&self, center: Point3D<T>, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let transform_point = |p: Point3D<T>| -> Point3D<T> {
            let translated = p - center;
            let rotated = Point3D::new(
                translated.x() * cos_a + translated.z() * sin_a,
                translated.y(),
                -translated.x() * sin_a + translated.z() * cos_a,
            );
            center + rotated.to_vector()
        };

        Triangle3D::new(
            transform_point(self.vertex_a()),
            transform_point(self.vertex_b()),
            transform_point(self.vertex_c()),
        )
        .expect("Y軸回転後の頂点は常に有効")
    }

    /// 任意軸周りの回転（Rodriguesの回転公式）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `axis` - 回転軸（正規化されたベクトル）
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 任意軸回転された新しい三角形
    pub fn rotate_axis(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        angle: Angle<T>,
    ) -> Option<Self> {
        let axis_norm = axis.normalize();
        if axis_norm.length().is_zero() {
            return None;
        }
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = T::ONE - cos_a;

        let transform_point = |p: Point3D<T>| -> Point3D<T> {
            let v = p - center;

            // Rodriguesの公式: v_rot = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
            let k_dot_v = axis_norm.dot(&v);
            let k_cross_v = axis_norm.cross(&v);

            let rotated = v * cos_a + k_cross_v * sin_a + axis_norm * (k_dot_v * one_minus_cos);
            center + rotated
        };

        Triangle3D::new(
            transform_point(self.vertex_a()),
            transform_point(self.vertex_b()),
            transform_point(self.vertex_c()),
        )
    }

    /// 三角形の法線ベクトル（Transform版）
    ///
    /// # 戻り値
    /// 正規化された法線ベクトル
    pub fn normal_transform(&self) -> Option<Vector3D<T>> {
        let edge1 = self.vertex_b() - self.vertex_a();
        let edge2 = self.vertex_c() - self.vertex_a();
        Some(edge1.cross(&edge2).normalize())
    }

    /// 三角形を法線方向にオフセット
    ///
    /// # 引数
    /// * `distance` - オフセット距離
    ///
    /// # 戻り値
    /// 法線方向にオフセットされた新しい三角形
    pub fn offset_along_normal(&self, distance: T) -> Option<Self> {
        let normal = self.normal_transform()?;
        let offset = normal * distance;

        Some(self.translate(offset))
    }
}

// ============================================================================
// Tests Module
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_translate() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.5, 1.0, 0.0),
        )
        .unwrap();
        let translation = Vector3D::new(2.0, 3.0, 4.0);
        let translated = triangle.translate(translation);

        assert_eq!(translated.vertex_a(), Point3D::new(2.0, 3.0, 4.0));
        assert_eq!(translated.vertex_b(), Point3D::new(3.0, 3.0, 4.0));
        assert_eq!(translated.vertex_c(), Point3D::new(2.5, 4.0, 4.0));
    }

    #[test]
    fn test_rotate_z() {
        let triangle = Triangle3D::new(
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(-1.0, 0.0, 0.0),
        )
        .unwrap();
        let center = Point3D::origin();
        let angle = Angle::from_radians(PI / 2.0); // 90度回転

        let rotated = triangle.rotate(center, angle);

        // 90度Z軸回転後の頂点位置を確認（許容誤差内）
        assert!((rotated.vertex_a().x() - 0.0).abs() < 1e-10);
        assert!((rotated.vertex_a().y() - 1.0).abs() < 1e-10);
        assert!((rotated.vertex_a().z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_uniform() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
            Point3D::new(1.0, 2.0, 0.0),
        )
        .unwrap();
        let center = Point3D::origin();
        let factor = 2.0;

        let scaled = triangle.scale(center, factor);

        assert_eq!(scaled.vertex_a(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(scaled.vertex_b(), Point3D::new(4.0, 0.0, 0.0));
        assert_eq!(scaled.vertex_c(), Point3D::new(2.0, 4.0, 0.0));
    }

    #[test]
    fn test_scale_from_centroid() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(3.0, 0.0, 0.0),
            Point3D::new(1.5, 3.0, 0.0),
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
    fn test_rotate_x() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
            Point3D::new(0.0, -1.0, 0.0),
        )
        .unwrap();
        let center = Point3D::origin();
        let angle = Angle::from_radians(PI / 2.0);

        let rotated = triangle.rotate_x(center, angle);

        // X軸90度回転: (0,1,0) -> (0,0,1), (0,0,1) -> (0,-1,0)
        assert!((rotated.vertex_a().x() - 0.0).abs() < 1e-10);
        assert!((rotated.vertex_a().y() - 0.0).abs() < 1e-10);
        assert!((rotated.vertex_a().z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normal_calculation() {
        // XY平面上の三角形
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let normal = triangle.normal().unwrap();

        // 法線は+Z方向を向くはず
        assert!((normal.x() - 0.0).abs() < 1e-10);
        assert!((normal.y() - 0.0).abs() < 1e-10);
        assert!((normal.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_bulk_transform_vertices_consistency() {
        // 3頂点一括変換の整合性テスト
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let translation = Vector3D::new(10.0, 20.0, 30.0);

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
        let individual_vertices: Vec<Point3D<f64>> = original_vertices
            .iter()
            .map(|v| v.translate(translation))
            .collect();

        // 3頂点すべてが個別変換と一致することを確認
        for (bulk_vertex, &individual_vertex) in
            bulk_vertices.iter().zip(individual_vertices.iter())
        {
            assert!((bulk_vertex.x() - individual_vertex.x()).abs() < 1e-10);
            assert!((bulk_vertex.y() - individual_vertex.y()).abs() < 1e-10);
            assert!((bulk_vertex.z() - individual_vertex.z()).abs() < 1e-10);
        }
    }
}
