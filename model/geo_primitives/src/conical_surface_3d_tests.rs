//! ConicalSurface3D の基本機能、拡張機能、Foundation実装のテストスイート

#[cfg(test)]
mod tests {
    use crate::{ConicalSurface3D, Point3D, Vector3D};
    use geo_foundation::{BasicTransform, ExtensionFoundation, PrimitiveKind};
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    fn approx_eq_point(p1: &Point3D<f64>, p2: &Point3D<f64>) -> bool {
        approx_eq(p1.x(), p2.x()) && approx_eq(p1.y(), p2.y()) && approx_eq(p1.z(), p2.z())
    }

    // ========================================================================
    // Core Implementation Tests
    // ========================================================================

    #[test]
    fn test_new_valid_cone_surface() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 2.0;
        let semi_angle = PI / 6.0; // 30度

        let cone = ConicalSurface3D::new(center, axis, ref_direction, radius, semi_angle);
        assert!(cone.is_some());

        let cone = cone.unwrap();
        assert_eq!(cone.center(), center);
        assert_eq!(cone.radius(), radius);
        assert_eq!(cone.semi_angle(), semi_angle);
    }

    #[test]
    fn test_new_invalid_radius() {
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let semi_angle = PI / 6.0;

        // 負の半径
        let cone = ConicalSurface3D::new(center, axis, ref_direction, -1.0, semi_angle);
        assert!(cone.is_none());

        // ゼロの半径
        let cone = ConicalSurface3D::new(center, axis, ref_direction, 0.0, semi_angle);
        assert!(cone.is_none());
    }

    #[test]
    fn test_new_invalid_semi_angle() {
        let center = Point3D::origin();
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let radius = 1.0;

        // 負の半頂角
        let cone = ConicalSurface3D::new(center, axis, ref_direction, radius, -PI / 6.0);
        assert!(cone.is_none());

        // ゼロの半頂角
        let cone = ConicalSurface3D::new(center, axis, ref_direction, radius, 0.0);
        assert!(cone.is_none());

        // 90度以上の半頂角
        let cone = ConicalSurface3D::new(center, axis, ref_direction, radius, PI / 2.0);
        assert!(cone.is_none());
    }

    #[test]
    fn test_new_at_origin() {
        let radius = 1.5;
        let semi_angle = PI / 4.0; // 45度

        let cone = ConicalSurface3D::new_at_origin(radius, semi_angle);
        assert!(cone.is_some());

        let cone = cone.unwrap();
        assert_eq!(cone.center(), Point3D::origin());
        assert_eq!(cone.radius(), radius);
        assert_eq!(cone.semi_angle(), semi_angle);

        // Z軸方向
        let axis_vec = cone.axis().as_vector();
        assert!(approx_eq(axis_vec.z(), 1.0));
        assert!(approx_eq(axis_vec.x(), 0.0));
        assert!(approx_eq(axis_vec.y(), 0.0));

        // X軸方向
        let ref_vec = cone.ref_direction().as_vector();
        assert!(approx_eq(ref_vec.x(), 1.0));
        assert!(approx_eq(ref_vec.y(), 0.0));
        assert!(approx_eq(ref_vec.z(), 0.0));
    }

    #[test]
    fn test_derived_y_axis() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        let y_axis = cone.derived_y_axis();
        let y_vec = y_axis.as_vector();

        // Y軸方向 (0, 1, 0)
        assert!(approx_eq(y_vec.x(), 0.0));
        assert!(approx_eq(y_vec.y(), 1.0));
        assert!(approx_eq(y_vec.z(), 0.0));
    }

    #[test]
    fn test_radius_at_v() {
        let radius = 2.0;
        let semi_angle = PI / 6.0; // tan(30°) = 1/√3
        let cone = ConicalSurface3D::new_at_origin(radius, semi_angle).unwrap();

        // v=0での半径
        assert!(approx_eq(cone.radius_at_v(0.0), radius));

        // v>0での半径
        let v = 1.0;
        let expected_radius = radius + v * semi_angle.tan();
        assert!(approx_eq(cone.radius_at_v(v), expected_radius));

        // v<0での半径
        let v = -1.0;
        let expected_radius = radius + v * semi_angle.tan();
        assert!(approx_eq(cone.radius_at_v(v), expected_radius));
    }

    #[test]
    fn test_point_at_uv() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        // 基準点 (u=0, v=0)
        let point = cone.point_at_uv(0.0, 0.0);
        let expected = Point3D::new(1.0, 0.0, 0.0); // X軸方向に半径分
        assert!(approx_eq_point(&point, &expected));

        // 90度回転 (u=π/2, v=0)
        let point = cone.point_at_uv(PI / 2.0, 0.0);
        let expected = Point3D::new(0.0, 1.0, 0.0); // Y軸方向に半径分
        assert!(approx_eq_point(&point, &expected));

        // 軸方向移動 (u=0, v=1)
        let point = cone.point_at_uv(0.0, 1.0);
        let radius_at_v1 = cone.radius_at_v(1.0);
        let expected = Point3D::new(radius_at_v1, 0.0, 1.0);
        assert!(approx_eq_point(&point, &expected));
    }

    #[test]
    fn test_apex_calculation() {
        let radius = 2.0;
        let semi_angle = PI / 4.0; // 45度, tan(45°) = 1
        let cone = ConicalSurface3D::new_at_origin(radius, semi_angle).unwrap();

        let apex = cone.apex();
        let expected_z = -radius / semi_angle.tan(); // -2.0
        let expected = Point3D::new(0.0, 0.0, expected_z);

        assert!(approx_eq_point(&apex, &expected));
    }

    #[test]
    fn test_normal_at_uv() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        // 基準点での法線
        let normal = cone.normal_at_uv(0.0, 0.0);
        assert!(normal.is_some());

        let normal_vec = normal.unwrap().as_vector();

        // 45度円錐の場合、法線は放射方向と軸方向の合成
        // 正確な値は複雑だが、単位ベクトルであることを確認
        let length = (normal_vec.x() * normal_vec.x()
            + normal_vec.y() * normal_vec.y()
            + normal_vec.z() * normal_vec.z())
        .sqrt();
        assert!(approx_eq(length, 1.0));
    }

    #[test]
    fn test_contains_point() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();
        let tolerance = 1e-6;

        // サーフェス上の点
        let surface_point = cone.point_at_uv(0.0, 0.0);
        assert!(cone.contains_point(&surface_point, tolerance));

        // サーフェス外の点
        let outside_point = Point3D::new(0.5, 0.0, 0.0);
        assert!(!cone.contains_point(&outside_point, tolerance));
    }

    #[test]
    fn test_is_valid() {
        let valid_cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        assert!(valid_cone.is_valid());

        // 無効な円錐の作成テストは new() メソッドで行われる
        // 負の半径での作成
        let invalid_result = ConicalSurface3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            -1.0, // 負の半径
            PI / 6.0,
        );
        assert!(invalid_result.is_none());
    }

    #[test]
    fn test_bounding_box() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        let bbox = cone.bounding_box(-2.0, 2.0);

        // バウンディングボックスの検証
        assert!(bbox.min().x() <= -2.0); // 最大半径を考慮した範囲
        assert!(bbox.max().x() >= 2.0);
        assert!(bbox.min().y() <= -2.0);
        assert!(bbox.max().y() >= 2.0);
        assert!(bbox.min().z() <= -2.0);
        assert!(bbox.max().z() >= 2.0);
    }

    // ========================================================================
    // Foundation Pattern Tests
    // ========================================================================

    #[test]
    fn test_extension_foundation_primitive_kind() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        assert_eq!(cone.primitive_kind(), PrimitiveKind::ConicalSurface);
    }

    #[test]
    fn test_extension_foundation_bounding_box() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        let bbox = cone.bounding_box(-2.0, 2.0);

        // デフォルト範囲での境界ボックス
        assert!(bbox.min().x() < 0.0);
        assert!(bbox.max().x() > 0.0);
        assert!(bbox.min().y() < 0.0);
        assert!(bbox.max().y() > 0.0);
        assert!(bbox.min().z() < 0.0);
        assert!(bbox.max().z() > 0.0);
    }

    #[test]
    fn test_extension_foundation_measure() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();

        // 無限サーフェスの測度は None
        assert!(cone.measure().is_none());
    }

    #[test]
    fn test_surface_area() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        // 範囲を指定した表面積計算
        let area = cone.surface_area(0.0, 1.0, None, None);
        assert!(area > 0.0);

        // 部分範囲での表面積
        let partial_area = cone.surface_area(0.0, 1.0, Some(0.0), Some(PI));
        assert!(partial_area < area); // 半分の範囲なので小さくなる
    }

    // ========================================================================
    // Basic Transform Tests
    // ========================================================================

    #[test]
    fn test_translate() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let translated = cone.translate(translation);
        let expected_center = Point3D::new(1.0, 2.0, 3.0);

        assert_eq!(translated.center(), expected_center);
        assert_eq!(translated.radius(), cone.radius());
        assert_eq!(translated.semi_angle(), cone.semi_angle());
    }

    #[test]
    fn test_rotate_z() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        let angle = PI / 2.0; // 90度

        let rotated = cone.rotate_z(angle);

        // 90度回転後、参照方向はY軸方向になる
        let ref_vec = rotated.ref_direction().as_vector();
        assert!(approx_eq(ref_vec.x(), 0.0));
        assert!(approx_eq(ref_vec.y(), 1.0));
        assert!(approx_eq(ref_vec.z(), 0.0));
    }

    #[test]
    fn test_rotate_axis() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        let axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸
        let angle = PI / 2.0; // 90度

        let rotated = cone.rotate_axis(axis, angle);

        // Z軸周りの90度回転は rotate_z と同じ結果
        let ref_vec = rotated.ref_direction().as_vector();
        assert!(approx_eq(ref_vec.x(), 0.0));
        assert!(approx_eq(ref_vec.y(), 1.0));
        assert!(approx_eq(ref_vec.z(), 0.0));
    }

    #[test]
    fn test_scale_uniform() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();
        let factor = 2.0;

        let scaled = cone.scale_uniform(factor);

        assert_eq!(scaled.radius(), cone.radius() * factor);
        assert_eq!(scaled.semi_angle(), cone.semi_angle()); // 角度は不変

        // 中心も原点から2倍に
        assert_eq!(scaled.center(), Point3D::origin()); // 原点なので変化なし
    }

    #[test]
    fn test_scale_non_uniform() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 6.0).unwrap();

        let scaled = cone.scale_non_uniform(2.0, 3.0, 1.0);

        // XY平均倍率で半径がスケールされる
        let expected_radius = cone.radius() * ((2.0 + 3.0) / 2.0);
        assert!(approx_eq(scaled.radius(), expected_radius));
    }

    #[test]
    fn test_reflect() {
        let cone = ConicalSurface3D::new(
            Point3D::new(1.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            PI / 6.0,
        )
        .unwrap();

        // YZ平面で反射
        let plane_point = Point3D::origin();
        let plane_normal = Vector3D::new(1.0, 0.0, 0.0);

        let reflected = cone.reflect(plane_point, plane_normal);

        // X座標が反転される
        assert!(approx_eq(reflected.center().x(), -1.0));
        assert!(approx_eq(reflected.center().y(), 0.0));
        assert!(approx_eq(reflected.center().z(), 0.0));
    }

    // ========================================================================
    // Extensions Tests
    // ========================================================================

    #[test]
    fn test_closest_point_to() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();
        let query_point = Point3D::new(0.5, 0.0, 0.0);

        let (_closest_point, (u, _v), distance) = cone.closest_point_to(&query_point);

        // 基本的な妥当性チェック
        assert!(distance >= 0.0);
        assert!((0.0..=2.0 * PI).contains(&u));
    }

    #[test]
    fn test_du_dv_vectors() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        let du = cone.du_vector(0.0, 0.0);
        let dv = cone.dv_vector(0.0, 0.0);

        // 偏微分ベクトルがゼロでないことを確認
        assert!(!du.is_zero());
        assert!(!dv.is_zero());

        // 基準点でのdu方向はY軸方向
        assert!(approx_eq(du.x(), 0.0));
        assert!(du.y() > 0.0);
        assert!(approx_eq(du.z(), 0.0));
    }

    #[test]
    fn test_principal_curvatures() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        let (k1, k2) = cone.principal_curvatures(0.0, 0.0);

        // 円錐面の特性：一方向は曲率0（母線方向）
        assert!(approx_eq(k1, 0.0));
        assert!(k2 > 0.0); // 円周方向は正の曲率
    }

    #[test]
    fn test_gaussian_curvature() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        let gaussian_k = cone.gaussian_curvature(0.0, 0.0);

        // 円錐面は可展面なのでガウス曲率は0
        assert!(approx_eq(gaussian_k, 0.0));
    }

    #[test]
    fn test_generatrix_at_u() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        let (point, direction) = cone.generatrix_at_u(0.0);

        // 母線の開始点は円錐表面上
        let expected_point = Point3D::new(1.0, 0.0, 0.0);
        assert!(approx_eq_point(&point, &expected_point));

        // 方向ベクトルは頂点に向かう
        let dir_vec = direction.as_vector();
        assert!(dir_vec.z() < 0.0); // Z軸負方向成分を持つ
    }

    #[test]
    fn test_cross_section_at_v() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        let (center, _axis, radius) = cone.cross_section_at_v(1.0);

        // v=1での断面
        let expected_center = Point3D::new(0.0, 0.0, 1.0);
        assert!(approx_eq_point(&center, &expected_center));

        let expected_radius = cone.radius_at_v(1.0);
        assert!(approx_eq(radius, expected_radius));
    }

    #[test]
    fn test_surface_quality_metrics() {
        let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();

        let metrics = cone.surface_quality_metrics();

        assert!(metrics.is_smooth);
        assert!(metrics.is_developable);
        assert!(metrics.has_singularities); // 頂点が特異点
        assert!(approx_eq(metrics.gaussian_curvature_constant, 0.0));
    }

    // 平面交線のテストは将来の実装で追加予定
    // #[test]
    // fn test_plane_intersection_type() {
    //     let cone = ConicalSurface3D::new_at_origin(1.0, PI / 4.0).unwrap();
    //
    //     // 軸に垂直な平面 -> 楕円
    //     let plane_point = Point3D::new(0.0, 0.0, 1.0);
    //     let plane_normal = crate::Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    //
    //     let intersection_type = cone.plane_intersection_type(&plane_point, &plane_normal);
    //
    //     // 結果の型をチェック（具体的な値は実装依存）
    //     match intersection_type {
    //         crate::PlaneIntersectionType::Ellipse |
    //         crate::PlaneIntersectionType::Parabola |
    //         crate::PlaneIntersectionType::Hyperbola |
    //         crate::PlaneIntersectionType::Point |
    //         crate::PlaneIntersectionType::Line => {
    //             // いずれかの有効な型であることを確認
    //         }
    //     }
    // }
}
