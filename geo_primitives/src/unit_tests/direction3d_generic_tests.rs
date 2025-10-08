//! Direction3D型のテスト
//!
//! Direction3D<T>のジェネリック実装の動作確認とDirection2D<T>との一貫性テスト

#[cfg(test)]
mod tests {
    use crate::geometry3d::{Direction3D, Direction3DF32, Direction3DF64, Vector};
    use geo_foundation::abstract_types::{
        geometry::{Direction, Direction3D as Direction3DTrait},
        Scalar,
    };

    #[test]
    fn test_direction3d_basic_creation() {
        // f64版の基本作成
        let dir_x = Direction3D::<f64>::positive_x();
        assert_eq!(dir_x.x(), 1.0);
        assert_eq!(dir_x.y(), 0.0);
        assert_eq!(dir_x.z(), 0.0);

        let dir_y = Direction3D::<f64>::positive_y();
        assert_eq!(dir_y.x(), 0.0);
        assert_eq!(dir_y.y(), 1.0);
        assert_eq!(dir_y.z(), 0.0);

        let dir_z = Direction3D::<f64>::positive_z();
        assert_eq!(dir_z.x(), 0.0);
        assert_eq!(dir_z.y(), 0.0);
        assert_eq!(dir_z.z(), 1.0);
    }

    #[test]
    fn test_direction3d_f32_support() {
        // f32版の基本作成
        let dir_x = Direction3D::<f32>::positive_x();
        assert_eq!(dir_x.x(), 1.0f32);
        assert_eq!(dir_x.y(), 0.0f32);
        assert_eq!(dir_x.z(), 0.0f32);

        let dir_neg_z = Direction3D::<f32>::negative_z();
        assert_eq!(dir_neg_z.x(), 0.0f32);
        assert_eq!(dir_neg_z.y(), 0.0f32);
        assert_eq!(dir_neg_z.z(), -1.0f32);
    }

    #[test]
    fn test_direction3d_from_vector() {
        // Vectorからの作成
        let vec = Vector::<f64>::new(3.0, 4.0, 0.0);
        let dir = Direction3D::from_vector(vec).unwrap();

        // 正規化されているか確認
        let len = (dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z()).sqrt();
        assert!((len - 1.0).abs() < f64::TOLERANCE);

        // 方向が正しいか確認
        assert!((dir.x() - 0.6).abs() < f64::TOLERANCE);
        assert!((dir.y() - 0.8).abs() < f64::TOLERANCE);
        assert!((dir.z() - 0.0).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_direction3d_operations() {
        let dir1 = Direction3D::<f64>::positive_x();
        let dir2 = Direction3D::<f64>::positive_y();

        // 内積
        let dot = dir1.dot(&dir2);
        assert!((dot - 0.0).abs() < f64::TOLERANCE);

        // 外積
        let cross = dir1.cross(&dir2);
        assert!((cross.x() - 0.0).abs() < f64::TOLERANCE);
        assert!((cross.y() - 0.0).abs() < f64::TOLERANCE);
        assert!((cross.z() - 1.0).abs() < f64::TOLERANCE);

        // 逆方向
        let reversed = dir1.reverse();
        assert_eq!(reversed.x(), -1.0);
        assert_eq!(reversed.y(), 0.0);
        assert_eq!(reversed.z(), 0.0);
    }

    #[test]
    fn test_direction3d_parallel_perpendicular() {
        let dir1 = Direction3D::<f64>::positive_x();
        let dir2 = Direction3D::<f64>::negative_x();
        let dir3 = Direction3D::<f64>::positive_y();

        // 平行性チェック
        assert!(dir1.is_parallel(&dir2, f64::TOLERANCE));
        assert!(!dir1.is_parallel(&dir3, f64::TOLERANCE));

        // 垂直性チェック
        assert!(dir1.is_perpendicular(&dir3, f64::TOLERANCE));
        assert!(!dir1.is_perpendicular(&dir2, f64::TOLERANCE));

        // 同方向チェック
        assert!(dir1.is_same_direction(&dir1, f64::TOLERANCE));
        assert!(!dir1.is_same_direction(&dir2, f64::TOLERANCE));

        // 逆方向チェック
        assert!(dir1.is_opposite_direction(&dir2, f64::TOLERANCE));
        assert!(!dir1.is_opposite_direction(&dir3, f64::TOLERANCE));
    }

    #[test]
    fn test_direction3d_trait_methods() {
        let dir = Direction3D::<f64>::positive_x();

        // 垂直ベクトルの取得
        let perp = dir.any_perpendicular();
        assert!(dir.is_perpendicular(&perp, f64::TOLERANCE));

        // 正規直交基底の構築
        let (u, v, w) = dir.build_orthonormal_basis();
        assert!(u.is_same_direction(&dir, f64::TOLERANCE));
        assert!(u.is_perpendicular(&v, f64::TOLERANCE));
        assert!(u.is_perpendicular(&w, f64::TOLERANCE));
        assert!(v.is_perpendicular(&w, f64::TOLERANCE));

        // 軸メソッド
        let x_axis = Direction3D::<f64>::x_axis();
        let y_axis = Direction3D::<f64>::y_axis();
        let z_axis = Direction3D::<f64>::z_axis();

        assert!(x_axis.is_same_direction(&Direction3D::positive_x(), f64::TOLERANCE));
        assert!(y_axis.is_same_direction(&Direction3D::positive_y(), f64::TOLERANCE));
        assert!(z_axis.is_same_direction(&Direction3D::positive_z(), f64::TOLERANCE));
    }

    #[test]
    fn test_direction3d_type_aliases() {
        // 型エイリアスが正しく機能することを確認
        let dir_f64: Direction3DF64 = Direction3D::positive_x();
        let dir_f32: Direction3DF32 = Direction3D::positive_x();

        assert_eq!(dir_f64.x(), 1.0f64);
        assert_eq!(dir_f32.x(), 1.0f32);

        // サイズ確認
        assert_eq!(std::mem::size_of::<Direction3DF64>(), 24); // f64 * 3
        assert_eq!(std::mem::size_of::<Direction3DF32>(), 12); // f32 * 3
    }

    #[test]
    fn test_direction3d_f32_f64_consistency() {
        // f32とf64の一貫性確認
        let vec_f64 = Vector::<f64>::new(1.0, 1.0, 1.0);
        let vec_f32 = Vector::<f32>::new(1.0, 1.0, 1.0);

        let dir_f64 = Direction3D::from_vector(vec_f64).unwrap();
        let dir_f32 = Direction3D::from_vector(vec_f32).unwrap();

        // 正規化結果の一貫性（f32精度内で）
        let expected = 1.0 / 3.0_f64.sqrt();
        assert!((dir_f64.x() - expected).abs() < f64::TOLERANCE);
        assert!((dir_f32.x() as f64 - expected).abs() < 0.001); // f32精度

        assert!((dir_f64.y() - expected).abs() < f64::TOLERANCE);
        assert!((dir_f32.y() as f64 - expected).abs() < 0.001);

        assert!((dir_f64.z() - expected).abs() < f64::TOLERANCE);
        assert!((dir_f32.z() as f64 - expected).abs() < 0.001);
    }

    #[test]
    fn test_direction3d_to_vector_roundtrip() {
        // Vector -> Direction3D -> Vector の往復確認
        let original_vec = Vector::<f64>::new(2.0, 3.0, 6.0);
        let dir = Direction3D::from_vector(original_vec).unwrap();
        let converted_vec = dir.to_vector();

        // 正規化されたベクトルが返されるか確認
        let length = converted_vec.norm();
        assert!((length - 1.0).abs() < f64::TOLERANCE);

        // 方向が同じか確認
        let normalized_original = original_vec.normalize().unwrap();
        assert!((converted_vec.x() - normalized_original.x()).abs() < f64::TOLERANCE);
        assert!((converted_vec.y() - normalized_original.y()).abs() < f64::TOLERANCE);
        assert!((converted_vec.z() - normalized_original.z()).abs() < f64::TOLERANCE);
    }
}
