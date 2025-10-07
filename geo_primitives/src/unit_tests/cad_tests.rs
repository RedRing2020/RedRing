#[cfg(test)]
mod cad_tests {
    use crate::cad_ellipse::CadEllipse;
    use crate::cad_ellipse_arc::CadEllipseArc;
    use crate::cad_direction::CadDirection;
    use crate::{CadPoint, CadVector};

    // CadEllipse tests
    #[test]
    fn test_cad_ellipse_basic() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let major = CadVector::new(2.0, 0.0, 0.0);
        let minor = CadVector::new(0.0, 1.0, 0.0);

        let ellipse = CadEllipse::new(center, major, minor, 2.0, 1.0).unwrap();

        assert_eq!(ellipse.major_radius(), 2.0);
        assert_eq!(ellipse.minor_radius(), 1.0);
    }

    #[test]
    fn test_cad_ellipse_orthogonal_check() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let major = CadVector::new(1.0, 0.0, 0.0);
        let minor = CadVector::new(1.0, 1.0, 0.0); // 非直交

        let ellipse = CadEllipse::new(center, major, minor, 1.0, 1.0);
        assert!(ellipse.is_none()); // 軸が直交していないため作成失敗
    }

    // CadEllipseArc tests
    #[test]
    fn test_cad_ellipse_arc_basic() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let major = CadVector::new(1.0, 0.0, 0.0);
        let minor = CadVector::new(0.0, 1.0, 0.0);

        let arc = CadEllipseArc::new(
            center, major, minor,
            1.0, 1.0,
            0.0, std::f64::consts::PI / 2.0
        ).unwrap();

        assert_eq!(arc.start_angle(), 0.0);
        assert!((arc.end_angle() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_cad_ellipse_arc_invalid_angles() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let major = CadVector::new(1.0, 0.0, 0.0);
        let minor = CadVector::new(0.0, 1.0, 0.0);

        // 終了角度が開始角度より小さい場合
        let arc = CadEllipseArc::new(
            center, major, minor,
            1.0, 1.0,
            std::f64::consts::PI, 0.0
        );

        assert!(arc.is_none()); // 不正な角度範囲のため作成失敗
    }

    // CadDirection tests
    #[test]
    fn test_cad_direction() {
        let v = CadVector::new(3.0, 4.0, 0.0);
        let dir = CadDirection::from_vector(v).unwrap();
        
        // 正規化されているか確認
        let norm = (dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z()).sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_orthonormal_basis() {
        let dir = CadDirection::from_vector(CadVector::new(0.0, 0.0, 1.0)).unwrap();
        let (u, v) = dir.orthonormal_basis();
        
        // 直交性確認
        assert!(u.dot(&v).abs() < 1e-10);
        // 正規化確認
        assert!((u.norm() - 1.0).abs() < 1e-10);
        assert!((v.norm() - 1.0).abs() < 1e-10);
    }

    // CadVector trait tests
    #[test]
    fn test_cad_vector_operations() {
        let v1 = CadVector::new(1.0, 0.0, 0.0);
        let v2 = CadVector::new(0.0, 1.0, 0.0);

        let cross = v1.cross(&v2);
        assert!((cross.z() - 1.0).abs() < 1e-10);

        let dot = v1.dot(&v2);
        assert!(dot.abs() < 1e-10);
    }

    #[test]
    fn test_cad_vector_norm() {
        let v = CadVector::new(3.0, 4.0, 0.0);
        assert!((v.norm() - 5.0).abs() < 1e-10);
    }
}
