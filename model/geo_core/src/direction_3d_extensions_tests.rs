//! Direction3D Extensions Tests
//!
//! Direction3D拡張機能のテスト

#[cfg(test)]
mod tests {
    use crate::{Direction3D, Vector3D};
    use geo_contracts::Scalar;

    type TestType = f64;

    #[test]
    fn test_direction_3d_construction() {
        // ベクトルから方向を作成
        let vec = Vector3D::new(1.0, 1.0, 1.0);
        let dir = Direction3D::from_vector(vec).unwrap();

        // 正規化されているかチェック（より緩い許容誤差）
        let length_squared = dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z();
        assert!((length_squared - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction_normalization() {
        // 異なる長さのベクトルから同じ方向を作成
        let vec1 = Vector3D::new(1.0, 0.0, 0.0);
        let vec2 = Vector3D::new(5.0, 0.0, 0.0);

        let dir1 = Direction3D::from_vector(vec1).unwrap();
        let dir2 = Direction3D::from_vector(vec2).unwrap();

        // 両方とも正規化されて同じ方向になる
        assert!((dir1.x() - dir2.x()).abs() < TestType::EPSILON);
        assert!((dir1.y() - dir2.y()).abs() < TestType::EPSILON);
        assert!((dir1.z() - dir2.z()).abs() < TestType::EPSILON);
    }
}
