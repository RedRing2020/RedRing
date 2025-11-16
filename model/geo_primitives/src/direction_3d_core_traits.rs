//! Direction3D Core Traits Implementation
//!
//! Foundation Pattern に基づく Direction3D の Core traits 実装
//! 統一された3つのCore機能（Constructor/Properties/Measure）を提供

use crate::{Direction3D, Vector3D};
use analysis::linalg::vector::Vector3;
use geo_foundation::{
    core::direction_core_traits::{
        Direction3DConstructor, Direction3DCore, Direction3DMeasure, Direction3DProperties,
    },
    Scalar,
};

// ============================================================================
// Direction3DConstructor トレイト実装
// ============================================================================

impl<T: Scalar> Direction3DConstructor<T> for Direction3D<T> {
    /// ベクトルから方向を作成（自動正規化）
    fn from_vector(vector: Vector3<T>) -> Option<Self> {
        let vec3d = Vector3D::new(vector.x(), vector.y(), vector.z());
        Direction3D::from_vector(vec3d)
    }

    /// X、Y、Z成分から方向を作成
    fn new(x: T, y: T, z: T) -> Option<Self> {
        Direction3D::new(x, y, z)
    }

    /// X軸正方向の単位ベクトル
    fn positive_x() -> Self {
        Direction3D::positive_x()
    }

    /// Y軸正方向の単位ベクトル
    fn positive_y() -> Self {
        Direction3D::positive_y()
    }

    /// Z軸正方向の単位ベクトル
    fn positive_z() -> Self {
        Direction3D::positive_z()
    }

    /// X軸負方向の単位ベクトル
    fn negative_x() -> Self {
        Direction3D::positive_x().reverse()
    }

    /// Y軸負方向の単位ベクトル
    fn negative_y() -> Self {
        Direction3D::positive_y().reverse()
    }

    /// Z軸負方向の単位ベクトル
    fn negative_z() -> Self {
        Direction3D::positive_z().reverse()
    }

    /// タプルから作成
    fn from_tuple(components: (T, T, T)) -> Option<Self> {
        Self::new(components.0, components.1, components.2)
    }

    /// Analysis Vector3から作成
    fn from_analysis_vector(vector: &Vector3<T>) -> Option<Self> {
        Self::new(vector.x(), vector.y(), vector.z())
    }
}

// ============================================================================
// Direction3DProperties トレイト実装
// ============================================================================

impl<T: Scalar> Direction3DProperties<T> for Direction3D<T> {
    /// X成分取得（-1.0 ≤ x ≤ 1.0）
    fn x(&self) -> T {
        self.x()
    }

    /// Y成分取得（-1.0 ≤ y ≤ 1.0）
    fn y(&self) -> T {
        self.y()
    }

    /// Z成分取得（-1.0 ≤ z ≤ 1.0）
    fn z(&self) -> T {
        self.z()
    }

    /// 成分を配列として取得
    fn components(&self) -> [T; 3] {
        [self.x(), self.y(), self.z()]
    }

    /// 成分をタプルとして取得
    fn to_tuple(&self) -> (T, T, T) {
        (self.x(), self.y(), self.z())
    }

    /// Analysis Vector3へ変換
    fn to_analysis_vector(&self) -> Vector3<T> {
        Vector3::new(self.x(), self.y(), self.z())
    }

    /// 内部ベクトルを取得
    fn as_vector(&self) -> Vector3<T> {
        self.to_analysis_vector()
    }

    /// 長さ（常に1.0）
    fn length(&self) -> T {
        T::ONE
    }

    /// 正規化済みかどうか（常にtrue）
    fn is_normalized(&self) -> bool {
        true
    }

    /// 形状の次元数（3）
    fn dimension(&self) -> u32 {
        3
    }
}

// ============================================================================
// Direction3DMeasure トレイト実装
// ============================================================================

impl<T: Scalar> Direction3DMeasure<T> for Direction3D<T> {
    /// 他の方向との内積
    fn dot(&self, other: &Self) -> T {
        self.as_vector().dot(&other.as_vector())
    }

    /// 他の方向との角度
    fn angle_to(&self, other: &Self) -> T {
        let dot_product = self.dot(other);
        // -1 ≤ dot_product ≤ 1 の範囲にクランプ
        let clamped = dot_product.max(-T::ONE).min(T::ONE);
        clamped.acos()
    }

    /// 他の方向との外積（結果も正規化される）
    fn cross(&self, other: &Self) -> Self {
        let result_vector = self.as_vector().cross(&other.as_vector());
        Direction3D::from_vector(Vector3D::new(
            result_vector.x(),
            result_vector.y(),
            result_vector.z(),
        ))
        .unwrap_or_else(|| Direction3D::positive_x()) // 平行な場合のフォールバック
    }

    /// 他の方向と平行かどうか
    fn is_parallel_to(&self, other: &Self) -> bool {
        let dot_product = self.dot(other).abs();
        (dot_product - T::ONE).abs() <= T::EPSILON
    }

    /// 他の方向と垂直かどうか
    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.dot(other).abs() <= T::EPSILON
    }

    /// 他の方向と同じ方向かどうか
    fn is_same_direction(&self, other: &Self) -> bool {
        let dot_product = self.dot(other);
        dot_product >= T::ONE - T::EPSILON
    }

    /// 他の方向と反対方向かどうか
    fn is_opposite_direction(&self, other: &Self) -> bool {
        let dot_product = self.dot(other);
        dot_product <= -T::ONE + T::EPSILON
    }

    /// 反転（逆方向）
    fn reverse(&self) -> Self {
        -(*self)
    }

    /// 指定軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let one_minus_cos = T::ONE - cos_angle;

        let ax = axis.x();
        let ay = axis.y();
        let az = axis.z();

        let x = self.x();
        let y = self.y();
        let z = self.z();

        // Rodriguesの回転公式
        let new_x = (cos_angle + ax * ax * one_minus_cos) * x
            + (ax * ay * one_minus_cos - az * sin_angle) * y
            + (ax * az * one_minus_cos + ay * sin_angle) * z;

        let new_y = (ay * ax * one_minus_cos + az * sin_angle) * x
            + (cos_angle + ay * ay * one_minus_cos) * y
            + (ay * az * one_minus_cos - ax * sin_angle) * z;

        let new_z = (az * ax * one_minus_cos - ay * sin_angle) * x
            + (az * ay * one_minus_cos + ax * sin_angle) * y
            + (cos_angle + az * az * one_minus_cos) * z;

        Direction3D::new(new_x, new_y, new_z)
            .expect("回転後のベクトルは正規化可能でなければならない")
    }
}

// ============================================================================
// 統合トレイト実装（自動実装）
// ============================================================================

impl<T: Scalar> Direction3DCore<T> for Direction3D<T> {}

// ============================================================================
// テストスイート
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction3d_constructor() {
        // 基本コンストラクタ
        let dir = Direction3D::new(1.0f64, 0.0, 0.0).unwrap();
        assert_eq!(dir.x(), 1.0);
        assert_eq!(dir.y(), 0.0);
        assert_eq!(dir.z(), 0.0);

        // Analysis Vectorから作成
        let vec = Vector3::new(0.0f64, 1.0, 0.0);
        let dir = Direction3D::from_analysis_vector(&vec).unwrap();
        assert_eq!(dir.y(), 1.0);

        // 単位ベクトル作成
        let x_dir: Direction3D<f64> = Direction3D::positive_x();
        let neg_x_dir: Direction3D<f64> = Direction3D::negative_x();
        assert!(Direction3DMeasure::is_opposite_direction(
            &x_dir, &neg_x_dir
        ));
    }

    #[test]
    fn test_direction3d_properties() {
        let dir = Direction3D::new(1.0f64, 1.0, 1.0).unwrap();

        // 成分取得
        let components = Direction3DProperties::components(&dir);
        assert_eq!(components.len(), 3);

        // ベクトル取得
        let vector = Direction3DProperties::as_vector(&dir);
        assert_eq!(Direction3DProperties::x(&dir), vector.x());
        assert_eq!(Direction3DProperties::y(&dir), vector.y());
        assert_eq!(Direction3DProperties::z(&dir), vector.z());

        // 正規化チェック（Direction3Dは常に正規化されている）
        let length = (Direction3DProperties::x(&dir).powi(2)
            + Direction3DProperties::y(&dir).powi(2)
            + Direction3DProperties::z(&dir).powi(2))
        .sqrt();
        assert!((length - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction3d_measure() {
        let x_dir: Direction3D<f64> = Direction3D::positive_x();
        let y_dir: Direction3D<f64> = Direction3D::positive_y();
        let z_dir: Direction3D<f64> = Direction3D::positive_z();

        // 内積
        assert_eq!(Direction3DMeasure::dot(&x_dir, &x_dir), 1.0);
        assert_eq!(Direction3DMeasure::dot(&x_dir, &y_dir), 0.0);

        // 垂直性チェック
        assert!(Direction3DMeasure::is_perpendicular_to(&x_dir, &y_dir));
        assert!(Direction3DMeasure::is_perpendicular_to(&y_dir, &z_dir));

        // 外積
        let cross_result = Direction3DMeasure::cross(&x_dir, &y_dir);
        assert!(Direction3DMeasure::is_same_direction(&cross_result, &z_dir));

        // 角度計算
        let angle = Direction3DMeasure::angle_to(&x_dir, &y_dir);
        assert!((angle - std::f64::consts::PI / 2.0).abs() < 1e-10);

        // 反転
        let neg_x = Direction3DMeasure::reverse(&x_dir);
        assert!(Direction3DMeasure::is_opposite_direction(&x_dir, &neg_x));
    }

    #[test]
    fn test_direction3d_core_integration() {
        // Core traitの統合テスト（dyn compatibilityの問題でBoxは使用しない）
        let dir = Direction3D::positive_x();

        // Constructorメソッド経由での作成
        let dir2: Direction3D<f64> = Direction3D::from_tuple((0.0, 1.0, 0.0)).unwrap();

        // Measureメソッド経由での計算
        assert!(Direction3DMeasure::is_perpendicular_to(&dir, &dir2));
    }
}
