//! Point3D Core Traits Implementation
//!
//! Foundation Pattern に基づく Point3D の Core traits 実装
//! 統一された3つのCore機能（Constructor/Properties/Measure）を提供

use crate::Point3D;
use analysis::linalg::vector::Vector3;
use geo_foundation::{
    core::point_core_traits::{Point3DConstructor, Point3DCore, Point3DMeasure, Point3DProperties},
    Scalar,
};

// ============================================================================
// Point3DConstructor トレイト実装
// ============================================================================

impl<T: Scalar> Point3DConstructor<T> for Point3D<T> {
    /// 基本コンストラクタ
    fn new(x: T, y: T, z: T) -> Self {
        Point3D::new(x, y, z)
    }

    /// 原点作成
    fn origin() -> Self {
        Point3D::origin()
    }

    /// タプルから作成
    fn from_tuple(coords: (T, T, T)) -> Self {
        Point3D::from_tuple(coords)
    }

    /// Analysis Vector3から作成
    fn from_analysis_vector(vector: &Vector3<T>) -> Self {
        Point3D::new(vector.x(), vector.y(), vector.z())
    }

    /// 別の点からコピー作成
    fn from_point(other: &Self) -> Self {
        *other
    }
}

// ============================================================================
// Point3DProperties トレイト実装
// ============================================================================

impl<T: Scalar> Point3DProperties<T> for Point3D<T> {
    /// X座標取得
    fn x(&self) -> T {
        self.x()
    }

    /// Y座標取得
    fn y(&self) -> T {
        self.y()
    }

    /// Z座標取得
    fn z(&self) -> T {
        self.z()
    }

    /// 座標を配列として取得
    fn coords(&self) -> [T; 3] {
        self.coords()
    }

    /// 座標をタプルとして取得
    fn to_tuple(&self) -> (T, T, T) {
        (self.x(), self.y(), self.z())
    }

    /// Analysis Vector3へ変換
    fn to_analysis_vector(&self) -> Vector3<T> {
        Vector3::new(self.x(), self.y(), self.z())
    }
}

// ============================================================================
// Point3DMeasure トレイト実装
// ============================================================================

impl<T: Scalar> Point3DMeasure<T> for Point3D<T> {
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
        let origin = Point3D::origin();
        self.distance_to(&origin)
    }

    /// 原点からの距離の二乗（高速版）
    fn norm_squared(&self) -> T {
        let origin = Point3D::origin();
        self.distance_squared_to(&origin)
    }
}

// ============================================================================
// Point3DCore 統合トレイト実装（自動実装）
// ============================================================================

impl<T: Scalar> Point3DCore<T> for Point3D<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3d_constructor() {
        // 基本コンストラクタ
        let p1 = Point3D::<f64>::new(3.0, 4.0, 5.0);
        assert_eq!(p1.x(), 3.0);
        assert_eq!(p1.y(), 4.0);
        assert_eq!(p1.z(), 5.0);

        // 原点作成
        let origin = Point3D::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);

        // タプルから作成
        let p2 = Point3D::<f64>::from_tuple((1.0, 2.0, 3.0));
        assert_eq!(p2.x(), 1.0);
        assert_eq!(p2.y(), 2.0);
        assert_eq!(p2.z(), 3.0);

        // Analysis Vectorから作成
        let vector = Vector3::new(5.0, 6.0, 7.0);
        let p3 = Point3D::<f64>::from_analysis_vector(&vector);
        assert_eq!(p3.x(), 5.0);
        assert_eq!(p3.y(), 6.0);
        assert_eq!(p3.z(), 7.0);

        // 別の点からコピー作成
        let p4 = Point3D::<f64>::from_point(&p1);
        assert_eq!(p4.x(), 3.0);
        assert_eq!(p4.y(), 4.0);
        assert_eq!(p4.z(), 5.0);
    }

    #[test]
    fn test_point3d_properties() {
        let p = Point3D::<f64>::new(3.0, 4.0, 5.0);

        // 座標取得
        assert_eq!(p.x(), 3.0);
        assert_eq!(p.y(), 4.0);
        assert_eq!(p.z(), 5.0);

        // 配列として取得
        assert_eq!(p.coords(), [3.0, 4.0, 5.0]);

        // タプルとして取得
        assert_eq!(p.to_tuple(), (3.0, 4.0, 5.0));

        // Analysis Vectorへ変換
        let vector = p.to_analysis_vector();
        assert_eq!(vector.x(), 3.0);
        assert_eq!(vector.y(), 4.0);
        assert_eq!(vector.z(), 5.0);

        // 位置（自分自身）
        let position = p.position();
        assert_eq!(position.x(), 3.0);
        assert_eq!(position.y(), 4.0);
        assert_eq!(position.z(), 5.0);

        // 次元情報
        assert_eq!(p.dimension(), 0);
    }

    #[test]
    fn test_point3d_measure() {
        let p1 = Point3D::<f64>::new(0.0, 0.0, 0.0);
        let p2 = Point3D::<f64>::new(3.0, 4.0, 0.0); // 3-4-5三角形のxy平面版

        // 距離計算
        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.distance_squared_to(&p2), 25.0);

        // 原点からの距離
        assert_eq!(p2.distance_from_origin(), 5.0);
        assert_eq!(p2.norm_squared(), 25.0);

        // 3D距離テスト
        let p3 = Point3D::<f64>::new(2.0, 3.0, 6.0);
        let distance = p3.distance_from_origin();
        // 2^2 + 3^2 + 6^2 = 4 + 9 + 36 = 49 -> sqrt(49) = 7.0
        assert_eq!(distance, 7.0);

        // 面積・体積・長さ（Pointは0/None）
        assert_eq!(p1.area(), None);
        assert_eq!(p1.volume(), None);
        assert_eq!(p1.length(), None);
    }

    #[test]
    fn test_point3d_core_integration() {
        // Point3DCore traitとして統合利用できることを確認
        fn use_point_core<P: Point3DCore<f64>>(point: &P) -> f64 {
            point.distance_from_origin()
        }

        let p = Point3D::<f64>::new(3.0, 4.0, 0.0);
        assert_eq!(use_point_core(&p), 5.0);
    }

    #[test]
    fn test_point3d_analysis_vector_roundtrip() {
        // Point3D <-> Analysis Vector3 の相互変換テスト
        let original_point = Point3D::<f64>::new(1.5, 2.5, 3.5);

        // Point3D -> Vector3
        let vector = original_point.to_analysis_vector();
        assert_eq!(vector.x(), 1.5);
        assert_eq!(vector.y(), 2.5);
        assert_eq!(vector.z(), 3.5);

        // Vector3 -> Point3D
        let converted_point = Point3D::<f64>::from_analysis_vector(&vector);
        assert_eq!(converted_point.x(), 1.5);
        assert_eq!(converted_point.y(), 2.5);
        assert_eq!(converted_point.z(), 3.5);

        // 往復変換で一致することを確認
        assert_eq!(original_point.x(), converted_point.x());
        assert_eq!(original_point.y(), converted_point.y());
        assert_eq!(original_point.z(), converted_point.z());
    }
}
