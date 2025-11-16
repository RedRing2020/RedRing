//! Point2D Core Traits Implementation
//!
//! Foundation Pattern に基づく Point2D の Core traits 実装
//! 統一された3つのCore機能（Constructor/Properties/Measure）を提供

use crate::Point2D;
use analysis::linalg::vector::Vector2;
use geo_foundation::{
    core::point_core_traits::{Point2DConstructor, Point2DCore, Point2DMeasure, Point2DProperties},
    Scalar,
};

// ============================================================================
// Point2DConstructor トレイト実装
// ============================================================================

impl<T: Scalar> Point2DConstructor<T> for Point2D<T> {
    /// 基本コンストラクタ
    fn new(x: T, y: T) -> Self {
        Point2D::new(x, y)
    }

    /// 原点作成
    fn origin() -> Self {
        Point2D::origin()
    }

    /// タプルから作成
    fn from_tuple(coords: (T, T)) -> Self {
        Point2D::from_tuple(coords)
    }

    /// Analysis Vector2から作成
    fn from_analysis_vector(vector: &Vector2<T>) -> Self {
        Point2D::new(vector.x(), vector.y())
    }

    /// 別の点からコピー作成
    fn from_point(other: &Self) -> Self {
        *other
    }
}

// ============================================================================
// Point2DProperties トレイト実装
// ============================================================================

impl<T: Scalar> Point2DProperties<T> for Point2D<T> {
    /// X座標取得
    fn x(&self) -> T {
        self.x()
    }

    /// Y座標取得
    fn y(&self) -> T {
        self.y()
    }

    /// 座標を配列として取得
    fn coords(&self) -> [T; 2] {
        self.coords()
    }

    /// 座標をタプルとして取得
    fn to_tuple(&self) -> (T, T) {
        self.to_tuple()
    }

    /// Analysis Vector2へ変換
    fn to_analysis_vector(&self) -> Vector2<T> {
        Vector2::new(self.x(), self.y())
    }
}

// ============================================================================
// Point2DMeasure トレイト実装
// ============================================================================

impl<T: Scalar> Point2DMeasure<T> for Point2D<T> {
    /// 他の点までの距離
    fn distance_to(&self, other: &Self) -> T {
        self.distance_to(other)
    }

    /// 他の点までの距離の二乗（高速版）
    fn distance_squared_to(&self, other: &Self) -> T {
        self.distance_squared_to(other)
    }

    /// 原点からの距離（ノルム）
    fn distance_from_origin(&self) -> T {
        let origin = Point2D::origin();
        self.distance_to(&origin)
    }

    /// 原点からの距離の二乗（高速版）
    fn norm_squared(&self) -> T {
        let origin = Point2D::origin();
        self.distance_squared_to(&origin)
    }
}

// ============================================================================
// Point2DCore 統合トレイト実装（自動実装）
// ============================================================================

impl<T: Scalar> Point2DCore<T> for Point2D<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2d_constructor() {
        // 基本コンストラクタ
        let p1 = Point2D::<f64>::new(3.0, 4.0);
        assert_eq!(p1.x(), 3.0);
        assert_eq!(p1.y(), 4.0);

        // 原点作成
        let origin = Point2D::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);

        // タプルから作成
        let p2 = Point2D::<f64>::from_tuple((1.0, 2.0));
        assert_eq!(p2.x(), 1.0);
        assert_eq!(p2.y(), 2.0);

        // Analysis Vectorから作成
        let vector = Vector2::new(5.0, 6.0);
        let p3 = Point2D::<f64>::from_analysis_vector(&vector);
        assert_eq!(p3.x(), 5.0);
        assert_eq!(p3.y(), 6.0);

        // 別の点からコピー作成
        let p4 = Point2D::<f64>::from_point(&p1);
        assert_eq!(p4.x(), 3.0);
        assert_eq!(p4.y(), 4.0);
    }

    #[test]
    fn test_point2d_properties() {
        let p = Point2D::<f64>::new(3.0, 4.0);

        // 座標取得
        assert_eq!(p.x(), 3.0);
        assert_eq!(p.y(), 4.0);

        // 配列として取得
        assert_eq!(p.coords(), [3.0, 4.0]);

        // タプルとして取得
        assert_eq!(p.to_tuple(), (3.0, 4.0));

        // Analysis Vectorへ変換
        let vector = p.to_analysis_vector();
        assert_eq!(vector.x(), 3.0);
        assert_eq!(vector.y(), 4.0);

        // 位置（自分自身）
        let position = p.position();
        assert_eq!(position.x(), 3.0);
        assert_eq!(position.y(), 4.0);

        // 次元情報
        assert_eq!(p.dimension(), 0);
    }

    #[test]
    fn test_point2d_measure() {
        let p1 = Point2D::<f64>::new(0.0, 0.0);
        let p2 = Point2D::<f64>::new(3.0, 4.0);

        // 距離計算
        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.distance_squared_to(&p2), 25.0);

        // 原点からの距離
        assert_eq!(p2.distance_from_origin(), 5.0);
        assert_eq!(p2.norm_squared(), 25.0);

        // 面積・長さ（Pointは0/None）
        assert_eq!(p1.area(), None);
        assert_eq!(p1.length(), None);
    }

    #[test]
    fn test_point2d_core_integration() {
        // Point2DCore traitとして統合利用できることを確認
        fn use_point_core<P: Point2DCore<f64>>(point: &P) -> f64 {
            point.distance_from_origin()
        }

        let p = Point2D::<f64>::new(3.0, 4.0);
        assert_eq!(use_point_core(&p), 5.0);
    }
}
