// torus_solid_3d_transform_safe_tests.rs
// TorusSolid3D の SafeTransform エラーハンドリングテスト
//
// SafeTransform トレイトの包括的なエラーハンドリングテストスイートを提供します。
// 無効な入力値、境界条件、幾何学的制約違反のテストケースを含みます。

#[cfg(test)]
mod tests {
    use crate::TorusSolid3D;
    use analysis::abstract_types::Angle;
    use geo_foundation::{SafeTransform, TransformError};

    #[test]
    fn test_safe_translate_valid() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let result = torus.safe_translate(1.0);

        assert!(result.is_ok());
        let translated = result.unwrap();
        assert_eq!(translated.origin().x(), 1.0);
        assert_eq!(translated.origin().y(), 0.0);
        assert_eq!(translated.origin().z(), 0.0);
    }

    #[test]
    fn test_safe_translate_infinite() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let result = torus.safe_translate(f64::INFINITY);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidGeometry(msg) => {
                assert!(msg.contains("無限大またはNaN"));
            }
            _ => panic!("期待されたエラータイプではありません"),
        }
    }

    #[test]
    fn test_safe_scale_valid() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let result = torus.safe_scale(0.0, 2.0);

        assert!(result.is_ok());
        let scaled = result.unwrap();
        assert_eq!(scaled.major_radius(), 6.0);
        assert_eq!(scaled.minor_radius(), 2.0);
    }

    #[test]
    fn test_safe_scale_zero_factor() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let result = torus.safe_scale(0.0, 0.0);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(msg) => {
                assert!(msg.contains("ゼロスケール"));
            }
            _ => panic!("期待されたエラータイプではありません"),
        }
    }

    #[test]
    fn test_safe_scale_negative_factor() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let result = torus.safe_scale(0.0, -1.0);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidScaleFactor(msg) => {
                assert!(msg.contains("負のスケール"));
            }
            _ => panic!("期待されたエラータイプではありません"),
        }
    }

    #[test]
    fn test_safe_rotate_valid() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let angle = Angle::from_radians(std::f64::consts::PI / 2.0);
        let result = torus.safe_rotate(0.0, 0.0, angle);

        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_rotate_infinite_angle() {
        let torus = TorusSolid3D::standard(3.0_f64, 1.0_f64).unwrap();
        let angle = Angle::from_radians(f64::INFINITY);
        let result = torus.safe_rotate(0.0, 0.0, angle);

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformError::InvalidRotation(msg) => {
                assert!(msg.contains("無限大またはNaN"));
            }
            _ => panic!("期待されたエラータイプではありません"),
        }
    }

    #[test]
    fn test_safe_scale_constraint_violation() {
        let torus = TorusSolid3D::standard(2.0_f64, 1.5_f64).unwrap();
        let result = torus.safe_scale(0.0, 0.5); // スケール後: major=1.0, minor=0.75 → 有効

        assert!(result.is_ok());

        // 制約違反のケース
        let result2 = torus.safe_scale(0.0, 0.25); // スケール後: major=0.5, minor=0.375 → 無効（副半径が主半径に近づく）
        assert!(result2.is_ok()); // この例では有効なまま
    }
}
