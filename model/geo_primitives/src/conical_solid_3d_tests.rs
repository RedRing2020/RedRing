//! ConicalSolid3D 包括的テストスイート
//!
//! 円錐ソリッドの全機能をテストする統合テスト
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

#[cfg(test)]
mod tests {
    use crate::{ConicalSolid3D, Point3D, Vector3D};
    use approx::assert_relative_eq;
    use geo_foundation::{ExtensionFoundation, PrimitiveKind};

    // ========================================================================
    // Constructor Tests
    // ========================================================================

    #[test]
    fn test_new_standard() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0, 10.0).unwrap();

        assert_eq!(cone.center(), Point3D::new(1.0, 2.0, 3.0));
        assert_relative_eq!(cone.radius(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(cone.height(), 10.0, epsilon = 1e-10);
        assert_relative_eq!(cone.axis().z(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(cone.ref_direction().x(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_new_at_origin() {
        let cone = ConicalSolid3D::new_at_origin(3.0, 6.0).unwrap();

        assert_eq!(cone.center(), Point3D::new(0.0, 0.0, 0.0));
        assert_relative_eq!(cone.radius(), 3.0, epsilon = 1e-10);
        assert_relative_eq!(cone.height(), 6.0, epsilon = 1e-10);
    }

    #[test]
    fn test_new_with_custom_axes() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(1.0, 1.0, 0.0); // 斜めの軸
        let ref_direction = Vector3D::new(0.0, 0.0, 1.0);

        let cone = ConicalSolid3D::new(center, axis, ref_direction, 2.0, 4.0).unwrap();

        // 軸と参照方向が正規化されているかチェック
        assert_relative_eq!(cone.axis().as_vector().length(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(
            cone.ref_direction().as_vector().length(),
            1.0,
            epsilon = 1e-10
        );

        // 軸と参照方向が直交しているかチェック
        let dot_product = cone
            .axis()
            .as_vector()
            .dot(&cone.ref_direction().as_vector());
        assert_relative_eq!(dot_product, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_from_apex_and_base() {
        let apex = Point3D::new(0.0, 0.0, 10.0);
        let base_center = Point3D::new(0.0, 0.0, 0.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);

        let cone =
            ConicalSolid3D::from_apex_and_base(apex, base_center, ref_direction, 5.0).unwrap();

        assert_eq!(cone.center(), base_center);
        assert_relative_eq!(cone.height(), 10.0, epsilon = 1e-10);
        assert_relative_eq!(cone.radius(), 5.0, epsilon = 1e-10);
        assert_eq!(cone.apex(), apex);
    }

    #[test]
    fn test_invalid_constructors() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);

        // ゼロ半径
        assert!(ConicalSolid3D::new(center, axis, ref_direction, 0.0, 5.0).is_none());

        // 負の半径
        assert!(ConicalSolid3D::new(center, axis, ref_direction, -1.0, 5.0).is_none());

        // ゼロ高さ
        assert!(ConicalSolid3D::new(center, axis, ref_direction, 5.0, 0.0).is_none());

        // 負の高さ
        assert!(ConicalSolid3D::new(center, axis, ref_direction, 5.0, -1.0).is_none());

        // ゼロ軸ベクトル
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        assert!(ConicalSolid3D::new(center, zero_axis, ref_direction, 5.0, 10.0).is_none());

        // ゼロ参照方向
        let zero_ref = Vector3D::new(0.0, 0.0, 0.0);
        assert!(ConicalSolid3D::new(center, axis, zero_ref, 5.0, 10.0).is_none());
    }

    // ========================================================================
    // Geometric Property Tests
    // ========================================================================

    #[test]
    fn test_volume_calculation() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 3.0, 4.0).unwrap();

        let volume = cone.volume();
        let expected_volume = std::f64::consts::PI * 9.0 * 4.0 / 3.0; // π * r² * h / 3
        assert_relative_eq!(volume, expected_volume, epsilon = 1e-10);
    }

    #[test]
    fn test_surface_area_calculation() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 3.0, 4.0).unwrap();

        let surface_area = cone.surface_area();
        let slant_height = (9.0 + 16.0_f64).sqrt(); // √(r² + h²)
        let expected_area = std::f64::consts::PI * 3.0 * (3.0 + slant_height);
        assert_relative_eq!(surface_area, expected_area, epsilon = 1e-10);
    }

    #[test]
    fn test_apex_calculation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let cone = ConicalSolid3D::new_standard(center, 5.0, 10.0).unwrap();

        let apex = cone.apex();
        assert_relative_eq!(apex.x(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(apex.y(), 2.0, epsilon = 1e-10);
        assert_relative_eq!(apex.z(), 13.0, epsilon = 1e-10); // 3 + 10
    }

    #[test]
    fn test_derived_y_axis() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, 10.0).unwrap();

        let y_axis = cone.derived_y_axis();
        // axis = (0, 0, 1), ref_direction = (1, 0, 0)
        // y_axis = axis × ref_direction = (0, 0, 1) × (1, 0, 0) = (0, 1, 0)
        assert_relative_eq!(y_axis.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(y_axis.y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(y_axis.z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_bounding_box() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, 10.0).unwrap();

        let bbox = cone.bounding_box();

        // 底面: center ± radius in x,y directions
        // 頂点: (0, 0, 10)
        assert_relative_eq!(bbox.min().x(), -5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().y(), -5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().y(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.min().z(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(bbox.max().z(), 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_is_valid() {
        let valid_cone =
            ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, 10.0).unwrap();
        assert!(valid_cone.is_valid());
    }

    // ========================================================================
    // Foundation Pattern Tests
    // ========================================================================

    #[test]
    fn test_extension_foundation() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0, 10.0).unwrap();

        // primitive_kind のテスト
        assert_eq!(cone.primitive_kind(), PrimitiveKind::Cone);

        // measure (体積) のテスト
        assert!(cone.measure().is_some());
        let volume = cone.measure().unwrap();
        let expected_volume = std::f64::consts::PI * 25.0 * 10.0 / 3.0;
        assert_relative_eq!(volume, expected_volume, epsilon = 1e-10);

        // bounding_box のテスト
        let bbox = cone.bounding_box();
        assert_eq!(bbox, cone.bounding_box());
    }

    // ========================================================================
    // Transform Tests
    // ========================================================================

    #[test]
    fn test_translate_transform() {
        let cone = ConicalSolid3D::new_at_origin(5.0, 10.0).unwrap();
        let offset = Vector3D::new(10.0, 20.0, 30.0);

        let translated = cone.translate(&offset);

        assert_eq!(translated.center(), Point3D::new(10.0, 20.0, 30.0));
        assert_relative_eq!(translated.radius(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(translated.height(), 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_scale_uniform_transform() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0, 10.0).unwrap();

        let scaled = cone.scale_uniform(2.0).unwrap();

        // 変換後の値をチェック
        assert_eq!(scaled.center(), Point3D::new(1.0, 2.0, 3.0)); // centerは変わらない
        assert_relative_eq!(scaled.radius(), 10.0, epsilon = 1e-10);
        assert_relative_eq!(scaled.height(), 20.0, epsilon = 1e-10);
    }

    // ========================================================================
    // Extension Function Tests
    // ========================================================================

    #[test]
    fn test_contains_point() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, 10.0).unwrap();

        // 底面中心（含まれる）
        assert!(cone.contains_point(Point3D::new(0.0, 0.0, 0.0)));

        // 底面円周上（含まれる）
        assert!(cone.contains_point(Point3D::new(5.0, 0.0, 0.0)));

        // 頂点（含まれる）
        assert!(cone.contains_point(Point3D::new(0.0, 0.0, 10.0)));

        // 中間の高さの内部点（含まれる）
        assert!(cone.contains_point(Point3D::new(1.0, 0.0, 5.0)));

        // 外部点（含まれない）
        assert!(!cone.contains_point(Point3D::new(10.0, 0.0, 0.0)));
        assert!(!cone.contains_point(Point3D::new(0.0, 0.0, -1.0)));
        assert!(!cone.contains_point(Point3D::new(0.0, 0.0, 15.0)));
    }

    // TODO: 将来実装予定
    #[ignore]
    #[test]
    fn test_contains_point_with_tolerance() {
        let _cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, 10.0).unwrap();

        // tolerance付きの包含テスト - 将来実装
        // assert!(cone.contains_point_with_tolerance(
        //     Point3D::new(5.01, 0.0, 0.0),
        //     0.02
        // ));
        // assert!(cone.contains_point_with_tolerance(
        //     Point3D::new(0.0, 0.0, 10.01),
        //     0.02
        // ));
    }

    #[test]
    fn test_radius_at_height() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 10.0, 20.0).unwrap();

        // 底面での半径
        assert_relative_eq!(cone.radius_at_height(0.0).unwrap(), 10.0, epsilon = 1e-10);

        // 中間での半径
        assert_relative_eq!(cone.radius_at_height(10.0).unwrap(), 5.0, epsilon = 1e-10);

        // 頂点での半径
        assert_relative_eq!(cone.radius_at_height(20.0).unwrap(), 0.0, epsilon = 1e-10);

        // 範囲外
        assert!(cone.radius_at_height(-1.0).is_none());
        assert!(cone.radius_at_height(25.0).is_none());
    }

    // TODO: 将来実装予定
    #[ignore]
    #[test]
    fn test_point_on_base_circle() {
        let _cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 3.0, 6.0).unwrap();

        // 底面円周上の点を取得 - 将来実装
        // let point_0 = cone.point_on_base_circle(0.0);
        // assert_relative_eq!(point_0.x(), 3.0, epsilon = 1e-10);
        // assert_relative_eq!(point_0.y(), 0.0, epsilon = 1e-10);

        // let point_90 = cone.point_on_base_circle(std::f64::consts::PI / 2.0);
        // assert_relative_eq!(point_90.x(), 0.0, epsilon = 1e-10);
        // assert_relative_eq!(point_90.y(), 3.0, epsilon = 1e-10);
    }

    // TODO: 将来実装予定
    #[ignore]
    #[test]
    fn test_point_on_generatrix() {
        let _cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 4.0, 8.0).unwrap();

        // 母線上の点を取得 - 将来実装
        // let base_point = cone.point_on_generatrix(0.0, 0.0);
        // assert_relative_eq!(base_point.x(), 4.0, epsilon = 1e-10);

        // let apex_point = cone.point_on_generatrix(0.0, 1.0);
        // assert_eq!(apex_point, cone.apex());

        // let mid_point = cone.point_on_generatrix(0.0, 0.5);
        // assert_relative_eq!(mid_point.x(), 2.0, epsilon = 1e-10);
        // assert_relative_eq!(mid_point.z(), 4.0, epsilon = 1e-10);
    }

    #[test]
    fn test_projections() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0, 10.0).unwrap();

        // XY平面への投影
        let projection_points = cone.project_to_xy();

        // 投影された点の数をチェック（4つの底面点 + 1つの頂点 = 5点）
        assert_eq!(projection_points.len(), 5);

        // 投影にはapex（頂点）が含まれている
        let apex = cone.apex();
        assert!(
            projection_points.contains(&apex),
            "Should contain the apex point"
        );
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_very_small_cone() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 0.001, 0.001).unwrap();

        assert!(cone.is_valid());
        assert!(cone.volume() > 0.0);
        assert!(cone.surface_area() > 0.0);
    }

    #[test]
    fn test_very_large_cone() {
        let cone =
            ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1000.0, 1000.0).unwrap();

        assert!(cone.is_valid());
        assert!(cone.volume() > 0.0);
        assert!(cone.surface_area() > 0.0);
    }

    #[test]
    fn test_display_format() {
        let cone = ConicalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0, 10.0).unwrap();

        let display_str = format!("{}", cone);
        assert!(display_str.contains("ConicalSolid3D"));
        assert!(display_str.contains("center"));
        assert!(display_str.contains("axis"));
        assert!(display_str.contains("radius"));
        assert!(display_str.contains("height"));
    }

    // ========================================================================
    // Compatibility Tests
    // ========================================================================

    #[test]
    fn test_backward_compatibility_alias() {
        // Cone3D エイリアスのテスト
        let cone: crate::Cone3D<f64> = ConicalSolid3D::new_at_origin(5.0, 10.0).unwrap();
        assert_relative_eq!(cone.radius(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(cone.height(), 10.0, epsilon = 1e-10);
    }
}
