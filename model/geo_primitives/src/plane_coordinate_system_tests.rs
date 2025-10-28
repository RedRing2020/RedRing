//! Plane3DCoordinateSystem テストスイート
//!
//! **作成日: 2025年10月28日**
//! **最終更新: 2025年10月29日**

#[cfg(test)]
mod tests {
    use crate::{Plane3D, Plane3DCoordinateSystem, Point3D, Vector3D};
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_step_plane_creation() {
        // XY平面をZ軸法線、X軸方向でU軸として作成
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // UV座標(1, 1)がワールド座標(1, 1, 0)になることを確認
        let world_point = plane_sys.local_to_world(1.0, 1.0);
        assert_abs_diff_eq!(world_point.x(), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(world_point.y(), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(world_point.z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_cad_sketch_workflow() {
        // 傾いた平面でのスケッチシミュレーション
        let origin = Point3D::new(0.0, 0.0, 1.0);
        let normal = Vector3D::new(1.0, 1.0, 1.0); // 45度傾斜
        let u_direction = Vector3D::new(1.0, -1.0, 0.0);

        let plane_sys =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // 2Dスケッチパターン（正方形）
        let sketch_pattern = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

        // 3D座標に変換
        let world_points = plane_sys.pattern_to_3d(&sketch_pattern);

        // すべての点が同一平面上にあることを確認
        for point in &world_points {
            let (_, _, distance) = plane_sys.world_to_local(*point);
            assert_abs_diff_eq!(distance, 0.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_three_point_construction() {
        // 3点から平面座標系を作成
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let point_u = Point3D::new(1.0, 0.0, 0.0);
        let point_v = Point3D::new(0.0, 1.0, 0.0);

        let plane_sys =
            Plane3DCoordinateSystem::from_three_points(origin, point_u, point_v).unwrap();

        // Z軸が法線になることを確認
        let normal = plane_sys.normal();
        assert_abs_diff_eq!(normal.x(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(normal.y(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(normal.z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_coordinate_system_orthogonality() {
        // 座標系の直交性検証
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // U軸とV軸の直交性確認
        let u_axis = plane_sys.u_axis();
        let v_axis = plane_sys.v_axis();
        let normal_axis = plane_sys.normal();

        let dot_uv = u_axis.as_vector().dot(&v_axis.as_vector());
        let dot_un = u_axis.as_vector().dot(&normal_axis.as_vector());
        let dot_vn = v_axis.as_vector().dot(&normal_axis.as_vector());

        assert_abs_diff_eq!(dot_uv, 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(dot_un, 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(dot_vn, 0.0, epsilon = 1e-10);

        // 正規化確認
        assert_abs_diff_eq!(u_axis.as_vector().length(), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(v_axis.as_vector().length(), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(normal_axis.as_vector().length(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_world_to_local_conversion() {
        // ワールド座標⇔ローカル座標変換テスト
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // 平面上の点のテスト
        let test_point = Point3D::new(2.0, 3.0, 0.0);
        let (u, v, distance) = plane_sys.world_to_local(test_point);

        assert_abs_diff_eq!(u, 2.0, epsilon = 1e-10);
        assert_abs_diff_eq!(v, 3.0, epsilon = 1e-10);
        assert_abs_diff_eq!(distance, 0.0, epsilon = 1e-10);

        // 逆変換の確認
        let reconstructed = plane_sys.local_to_world(u, v);
        assert_abs_diff_eq!(reconstructed.x(), test_point.x(), epsilon = 1e-10);
        assert_abs_diff_eq!(reconstructed.y(), test_point.y(), epsilon = 1e-10);
        assert_abs_diff_eq!(reconstructed.z(), test_point.z(), epsilon = 1e-10);
    }

    #[test]
    fn test_transformation_matrix() {
        // 変換マトリックステスト
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap();

        let matrix = plane_sys.transformation_matrix();

        // マトリックスの最後の行は[0, 0, 0, 1]
        assert_abs_diff_eq!(matrix[3][0], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[3][1], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[3][2], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[3][3], 1.0, epsilon = 1e-10);

        // 原点の位置確認
        assert_abs_diff_eq!(matrix[0][3], 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[1][3], 2.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[2][3], 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_simple_plane_conversion() {
        // 従来のPlane3Dとの相互変換テスト
        let original_plane = Plane3D::from_point_and_normal(
            Point3D::new(1.0, 2.0, 3.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        // PlaneCoordinateSystemに変換
        let coord_system =
            Plane3DCoordinateSystem::from_simple_plane(&original_plane, None).unwrap();

        // 逆変換
        let reconstructed_plane = coord_system.to_simple_plane();

        // 原点と法線が保持されていることを確認
        let orig_point = original_plane.point();
        let recon_point = reconstructed_plane.point();

        assert_abs_diff_eq!(orig_point.x(), recon_point.x(), epsilon = 1e-10);
        assert_abs_diff_eq!(orig_point.y(), recon_point.y(), epsilon = 1e-10);
        assert_abs_diff_eq!(orig_point.z(), recon_point.z(), epsilon = 1e-10);

        let orig_normal = original_plane.normal();
        let recon_normal = reconstructed_plane.normal();

        assert_abs_diff_eq!(orig_normal.x(), recon_normal.x(), epsilon = 1e-10);
        assert_abs_diff_eq!(orig_normal.y(), recon_normal.y(), epsilon = 1e-10);
        assert_abs_diff_eq!(orig_normal.z(), recon_normal.z(), epsilon = 1e-10);
    }

    #[test]
    fn test_gram_schmidt_orthogonalization() {
        // グラム・シュミット正規直交化のテスト（U軸が法線と平行でない場合）
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 1.0, 0.5); // 法線と平行でない

        let plane_sys =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // 結果の座標系が直交系になることを確認
        let u_axis = plane_sys.u_axis();
        let v_axis = plane_sys.v_axis();
        let normal_axis = plane_sys.normal();

        // 直交性確認
        let dot_uv = u_axis.as_vector().dot(&v_axis.as_vector());
        let dot_un = u_axis.as_vector().dot(&normal_axis.as_vector());
        let dot_vn = v_axis.as_vector().dot(&normal_axis.as_vector());

        assert_abs_diff_eq!(dot_uv, 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(dot_un, 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(dot_vn, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_in_plane_rotation() {
        // 平面内回転のテスト
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // 90度回転
        let angle = std::f64::consts::PI / 2.0;
        let rotated_sys = plane_sys.rotate_in_plane(angle, (0.0, 0.0));

        // 回転後のU軸がY軸方向になることを確認
        let new_u = rotated_sys.u_axis();
        assert_abs_diff_eq!(new_u.x(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(new_u.y(), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(new_u.z(), 0.0, epsilon = 1e-10);

        // 法線は変わらないことを確認
        let new_normal = rotated_sys.normal();
        assert_abs_diff_eq!(new_normal.x(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(new_normal.y(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(new_normal.z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_error_cases() {
        // エラーケースのテスト
        let origin = Point3D::new(0.0, 0.0, 0.0);

        // ゼロベクトル法線
        let zero_normal = Vector3D::new(0.0, 0.0, 0.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let result =
            Plane3DCoordinateSystem::from_origin_and_axes(origin, zero_normal, u_direction);
        assert!(result.is_none());

        // ゼロベクトルU軸
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let zero_u = Vector3D::new(0.0, 0.0, 0.0);

        let result = Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, zero_u);
        assert!(result.is_none());

        // U軸が法線と平行（直交成分がゼロ）
        let parallel_u = Vector3D::new(0.0, 0.0, 1.0);

        let result = Plane3DCoordinateSystem::from_origin_and_axes(origin, normal, parallel_u);
        assert!(result.is_none());
    }
}
