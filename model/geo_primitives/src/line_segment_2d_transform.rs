//! LineSegment2D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! Analysis Matrix3x3による2端点一括変換最適化

use crate::{LineSegment2D, Point2D, Vector2D};
use analysis::linalg::vector::Vector2;
use geo_foundation::{extensions::BasicTransform, AnalysisTransform2D, Angle, Scalar};

// ============================================================================
// Analysis Matrix一括変換ヘルパー関数
// ============================================================================

impl<T: Scalar> LineSegment2D<T> {
    /// 2端点を一括でAnalysis Matrix3x3変換（translate用）
    fn transform_endpoints_translate(&self, translation: &Vector2<T>) -> [Point2D<T>; 2] {
        let endpoints = [self.start_point(), self.end_point()];
        let mut result = [Point2D::origin(); 2];

        for (i, endpoint) in endpoints.iter().enumerate() {
            result[i] = endpoint
                .translate_analysis_2d(translation)
                .unwrap_or(*endpoint);
        }

        result
    }

    /// 2端点を一括でAnalysis Matrix3x3変換（rotate用）
    fn transform_endpoints_rotate(&self, center: Point2D<T>, angle: Angle<T>) -> [Point2D<T>; 2] {
        let endpoints = [self.start_point(), self.end_point()];
        let mut result = [Point2D::origin(); 2];

        for (i, endpoint) in endpoints.iter().enumerate() {
            result[i] = endpoint
                .rotate_analysis_2d(&center, angle)
                .unwrap_or(*endpoint);
        }

        result
    }

    /// 2端点を一括でAnalysis Matrix3x3変換（scale用）
    fn transform_endpoints_scale(&self, center: Point2D<T>, factor: T) -> [Point2D<T>; 2] {
        let endpoints = [self.start_point(), self.end_point()];
        let mut result = [Point2D::origin(); 2];

        for (i, endpoint) in endpoints.iter().enumerate() {
            result[i] = endpoint
                .uniform_scale_analysis_2d(&center, factor)
                .unwrap_or(*endpoint);
        }

        result
    }
}

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for LineSegment2D<T> {
    type Transformed = LineSegment2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// Analysis Matrix3x3による2端点一括変換最適化
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい線分
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        // Analysis Transform統合: Vector2D→Vector2変換で一括Matrix3x3効率活用
        let translation_vector2: Vector2<T> = Vector2::new(translation.x(), translation.y());

        // 2端点を一括でAnalysis Matrix変換
        let transformed_endpoints = self.transform_endpoints_translate(&translation_vector2);

        // 変換済み端点から線分再構築
        Self::new(transformed_endpoints[0], transformed_endpoints[1])
            .expect("2つの変換済み端点から線分が作成できないはずがない")
    }

    /// 指定中心での回転
    ///
    /// Analysis Matrix3x3による2端点一括回転変換最適化
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転後の新しい線分（2つの端点を一括回転して再構築）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 2端点を一括でAnalysis Matrix3x3回転変換
        let transformed_endpoints = self.transform_endpoints_rotate(center, angle);

        // 回転済み端点から新しい線分を構築
        Self::new(transformed_endpoints[0], transformed_endpoints[1])
            .expect("2つの変換済み端点から線分が作成できないはずがない")
    }

    /// 指定中心でのスケール
    ///
    /// Analysis Matrix3x3による2端点一括スケール変換最適化
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい線分
    fn scale(&self, center: Self::Point2D, scale_factor: T) -> Self::Transformed {
        // 2端点を一括でAnalysis Matrix3x3スケール変換
        let transformed_endpoints = self.transform_endpoints_scale(center, scale_factor);

        // スケール済み端点から新しい線分を構築
        Self::new(transformed_endpoints[0], transformed_endpoints[1])
            .expect("2つの変換済み端点から線分が作成できないはずがない")
    }
}

// ============================================================================
// LineSegment2D 固有の Transform メソッド
// ============================================================================

impl<T: Scalar> LineSegment2D<T> {
    /// 中点を中心とした均等スケール
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい線分
    pub fn scale_from_midpoint(&self, factor: T) -> Self {
        let midpoint = self.midpoint();
        BasicTransform::scale(self, midpoint, factor)
    }

    /// 線分を指定方向に延長
    ///
    /// # 引数
    /// * `start_extension` - 始点側の延長距離
    /// * `end_extension` - 終点側の延長距離
    ///
    /// # 戻り値
    /// 延長された新しい線分
    pub fn extend(&self, start_extension: T, end_extension: T) -> Option<Self> {
        let direction = self.direction();
        let length = self.length();
        if length.is_zero() {
            return None;
        }
        let unit_direction = direction / length;

        let new_start = self.start_point() - unit_direction * start_extension;
        let new_end = self.end_point() + unit_direction * end_extension;

        Self::new(new_start, new_end)
    }

    /// 線分をパラメータで内分点にカット
    ///
    /// # 引数
    /// * `t_start` - 新しい始点のパラメータ（0.0-1.0）
    /// * `t_end` - 新しい終点のパラメータ（0.0-1.0）
    ///
    /// # 戻り値
    /// カットされた新しい線分
    pub fn trim(&self, t_start: T, t_end: T) -> Option<Self> {
        if t_start >= t_end || t_start < T::ZERO || t_end > T::ONE {
            return None;
        }

        let vector = self.vector(); // 始点から終点へのベクトル
        let new_start = self.start_point() + vector * t_start;
        let new_end = self.start_point() + vector * t_end;

        Self::new(new_start, new_end)
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
        let line = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)).unwrap();
        let translation = Vector2D::new(2.0, 3.0);
        let translated = line.translate(translation);

        assert_eq!(translated.start_point(), Point2D::new(2.0, 3.0));
        assert_eq!(translated.end_point(), Point2D::new(3.0, 4.0));
    }

    #[test]
    fn test_rotate() {
        use std::f64::consts::PI;
        let line = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)).unwrap();
        let center = Point2D::origin();
        let angle = Angle::from_radians(PI / 2.0); // 90度回転

        let rotated = line.rotate(center, angle);

        // 90度回転後の端点位置を確認（許容誤差内）
        assert!((rotated.start_point().x() - 0.0).abs() < 1e-10);
        assert!((rotated.start_point().y() - 0.0).abs() < 1e-10);
        assert!((rotated.end_point().x() - 0.0).abs() < 1e-10);
        assert!((rotated.end_point().y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let line = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0)).unwrap();
        let center = Point2D::origin();
        let factor = 2.0;

        let scaled = line.scale(center, factor);

        assert_eq!(scaled.start_point(), Point2D::new(0.0, 0.0));
        assert_eq!(scaled.end_point(), Point2D::new(4.0, 4.0));
    }

    #[test]
    fn test_scale_from_midpoint() {
        let line = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();
        let scaled = line.scale_from_midpoint(2.0);

        // 中点は変わらず、線分の長さは2倍になる
        assert_eq!(scaled.midpoint(), line.midpoint());
        assert!((scaled.length() - line.length() * 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_extend() {
        let line = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)).unwrap();
        let extended = line.extend(1.0, 2.0).unwrap();

        assert_eq!(extended.start_point(), Point2D::new(-1.0, 0.0));
        assert_eq!(extended.end_point(), Point2D::new(3.0, 0.0));
    }

    #[test]
    fn test_trim() {
        let line = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();
        let trimmed = line.trim(0.25, 0.75).unwrap();

        assert_eq!(trimmed.start_point(), Point2D::new(1.0, 0.0));
        assert_eq!(trimmed.end_point(), Point2D::new(3.0, 0.0));
    }

    #[test]
    fn test_bulk_transform_endpoints_consistency() {
        // 2端点一括変換の整合性テスト
        let line = LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0)).unwrap();
        let translation = Vector2D::new(5.0, 10.0);

        // 一括変換結果
        let bulk_transformed = line.translate(translation);
        let bulk_endpoints = [bulk_transformed.start_point(), bulk_transformed.end_point()];

        // 個別変換結果（比較用）
        let original_endpoints = [line.start_point(), line.end_point()];
        let individual_endpoints: Vec<Point2D<f64>> = original_endpoints
            .iter()
            .map(|p| p.translate(translation))
            .collect();

        // 2端点すべてが個別変換と一致することを確認
        for (bulk_endpoint, &individual_endpoint) in
            bulk_endpoints.iter().zip(individual_endpoints.iter())
        {
            assert!((bulk_endpoint.x() - individual_endpoint.x()).abs() < 1e-10);
            assert!((bulk_endpoint.y() - individual_endpoint.y()).abs() < 1e-10);
        }
    }
}
