//! Plane3D テストファイル

use crate::{Direction3D, Plane3D, Point3D, Vector3D};

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants::TOLERANCE_F64;
    use approx::{assert_abs_diff_eq, assert_relative_eq};

    #[test]
    fn test_from_point_and_normal() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);

        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        assert_eq!(plane.point(), point);
        assert_relative_eq!(plane.normal().x(), 0.0);
        assert_relative_eq!(plane.normal().y(), 0.0);
        assert_relative_eq!(plane.normal().z(), 1.0);
    }

    #[test]
    fn test_from_point_and_normal_normalization() {
        let point = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(3.0, 4.0, 0.0); // length = 5.0

        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        // 法線ベクトルが正規化されているかチェック
        assert_relative_eq!(plane.normal().length(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(plane.normal().x(), 0.6); // 3/5
        assert_relative_eq!(plane.normal().y(), 0.8); // 4/5
        assert_relative_eq!(plane.normal().z(), 0.0);
    }

    #[test]
    fn test_from_point_and_normal_zero_normal() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::zero();

        let result = Plane3D::from_point_and_normal(point, normal);

        assert!(result.is_none());
    }

    #[test]
    fn test_from_three_points() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(0.0, 1.0, 0.0);

        let plane = Plane3D::from_three_points(p1, p2, p3).unwrap();

        // XY平面になるはず
        assert_relative_eq!(plane.normal().x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(plane.normal().y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(plane.normal().z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_from_three_points_collinear() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 0.0, 0.0);
        let p3 = Point3D::new(2.0, 0.0, 0.0); // 一直線上

        let result = Plane3D::from_three_points(p1, p2, p3);

        assert!(result.is_none());
    }

    #[test]
    fn test_coordinate_planes() {
        // XY平面
        let xy_plane = Plane3D::xy_plane(5.0);
        assert_eq!(xy_plane.origin(), Point3D::new(0.0, 0.0, 5.0));
        assert_eq!(
            xy_plane.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );

        // XZ平面
        let xz_plane = Plane3D::xz_plane(3.0);
        assert_eq!(xz_plane.origin(), Point3D::new(0.0, 3.0, 0.0));
        assert_eq!(
            xz_plane.normal(),
            Direction3D::from_vector(Vector3D::unit_y()).unwrap()
        );

        // YZ平面
        let yz_plane = Plane3D::yz_plane(2.0);
        assert_eq!(yz_plane.origin(), Point3D::new(2.0, 0.0, 0.0));
        assert_eq!(
            yz_plane.normal(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap()
        );
    }

    #[test]
    fn test_contains_point() {
        let plane = Plane3D::xy_plane(5.0);

        // 平面上の点
        assert!(plane.contains_point(Point3D::new(1.0, 2.0, 5.0), 1e-10));
        assert!(plane.contains_point(Point3D::new(-3.0, 7.0, 5.0), 1e-10));

        // 平面外の点
        assert!(!plane.contains_point(Point3D::new(1.0, 2.0, 5.1), 1e-10));
        assert!(!plane.contains_point(Point3D::new(1.0, 2.0, 4.9), 1e-10));

        // 許容誤差内の点
        assert!(plane.contains_point(Point3D::new(1.0, 2.0, 5.05), 0.1));
    }

    #[test]
    fn test_distance_to_point() {
        let plane = Plane3D::xy_plane(3.0);

        // 平面上の点
        assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 3.0)), 0.0);

        // 法線方向の点（正の距離）
        assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 5.0)), 2.0);

        // 法線逆方向の点（負の距離）
        assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 1.0)), -2.0);
    }

    #[test]
    fn test_project_point() {
        let plane = Plane3D::xy_plane(2.0);

        let point = Point3D::new(3.0, 4.0, 7.0);
        let projected = plane.project_point(point);

        assert_relative_eq!(projected.x(), 3.0);
        assert_relative_eq!(projected.y(), 4.0);
        assert_relative_eq!(projected.z(), 2.0);

        // 投影点は平面上にあるはず
        assert!(plane.contains_point(projected, 1e-10));
    }

    #[test]
    fn test_equation_coefficients() {
        let plane = Plane3D::xy_plane(3.0);
        let (a, b, c, d) = plane.equation_coefficients();

        // z = 3 なので、0x + 0y + 1z - 3 = 0
        assert_relative_eq!(a, 0.0);
        assert_relative_eq!(b, 0.0);
        assert_relative_eq!(c, 1.0);
        assert_relative_eq!(d, -3.0);

        // 平面上の点で方程式をチェック
        let test_point = Point3D::new(5.0, 7.0, 3.0);
        let result = a * test_point.x() + b * test_point.y() + c * test_point.z() + d;
        assert_relative_eq!(result, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_is_valid() {
        // 正常な平面
        let valid_plane = Plane3D::xy_plane(0.0);
        assert!(valid_plane.is_valid());

        // 正規化されていない法線で作成された平面は、from_point_and_normal で自動正規化されるため、
        // 直接的に不正な平面を作ることは難しい。代わりに長さが非常に短い法線をテスト
        let point = Point3D::new(0.0, 0.0, 0.0);
        let very_small_normal = Vector3D::new(1e-15, 0.0, 0.0);

        // 非常に小さい法線ベクトルでも正規化は成功するが...
        if let Some(plane) = Plane3D::from_point_and_normal(point, very_small_normal) {
            // 作成された平面の法線は正規化されているはず
            assert!(plane.is_valid());
        }
    }

    #[test]
    fn test_default() {
        let plane = Plane3D::<f64>::default();
        assert_eq!(plane.origin(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(
            plane.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );
    }

    #[test]
    fn test_xy_constant() {
        let plane = Plane3D::<f64>::xy();
        assert_eq!(plane.origin(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(
            plane.normal(),
            Direction3D::from_vector(Vector3D::unit_z()).unwrap()
        );
    }

    #[test]
    fn test_display() {
        let plane = Plane3D::xy_plane(1.0);
        let display_str = format!("{}", plane);

        assert!(display_str.contains("Plane3D"));
        assert!(display_str.contains("origin"));
        assert!(display_str.contains("normal"));
    }

    #[test]
    fn test_step_plane_creation() {
        // XY平面をZ軸法線、X軸方向でU軸として作成
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys = Plane3D::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // UV座標(1, 1)がワールド座標(1, 1, 0)になることを確認
        let world_point = plane_sys.local_to_world(1.0, 1.0);
        assert_abs_diff_eq!(world_point.x(), 1.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(world_point.y(), 1.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(world_point.z(), 0.0, epsilon = TOLERANCE_F64);
    }

    #[test]
    fn test_three_point_construction() {
        // 3点から平面座標系を作成
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let point_u = Point3D::new(1.0, 0.0, 0.0);
        let point_v = Point3D::new(0.0, 1.0, 0.0);

        let plane_sys = Plane3D::from_three_points(origin, point_u, point_v).unwrap();

        // Z軸が法線になることを確認
        let normal = plane_sys.normal();
        assert_abs_diff_eq!(normal.x(), 0.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(normal.y(), 0.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(normal.z(), 1.0, epsilon = TOLERANCE_F64);
    }

    #[test]
    fn test_coordinate_system_orthogonality() {
        // 座標系の直交性検証
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys = Plane3D::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // U軸とV軸の直交性確認
        let u_axis = plane_sys.u_axis;
        let v_axis = plane_sys.v_axis;
        let normal_axis = plane_sys.normal();

        let dot_uv = u_axis.as_vector().dot(&v_axis.as_vector());
        let dot_un = u_axis.as_vector().dot(&normal_axis.as_vector());
        let dot_vn = v_axis.as_vector().dot(&normal_axis.as_vector());

        assert_abs_diff_eq!(dot_uv, 0.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(dot_un, 0.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(dot_vn, 0.0, epsilon = TOLERANCE_F64);

        // 正規化確認
        assert_abs_diff_eq!(u_axis.as_vector().length(), 1.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(v_axis.as_vector().length(), 1.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(
            normal_axis.as_vector().length(),
            1.0,
            epsilon = TOLERANCE_F64
        );
    }

    #[test]
    fn test_world_to_local_conversion() {
        // ワールド座標⇔ローカル座標変換テスト
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let plane_sys = Plane3D::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // 平面上の点のテスト
        let test_point = Point3D::new(2.0, 3.0, 0.0);
        let (u, v, distance) = plane_sys.world_to_local(test_point);

        assert_abs_diff_eq!(u, 2.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(v, 3.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(distance, 0.0, epsilon = TOLERANCE_F64);

        // 逆変換の確認
        let reconstructed = plane_sys.local_to_world(u, v);
        assert_abs_diff_eq!(reconstructed.x(), test_point.x(), epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(reconstructed.y(), test_point.y(), epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(reconstructed.z(), test_point.z(), epsilon = TOLERANCE_F64);
    }

    #[test]
    fn test_gram_schmidt_orthogonalization() {
        // グラム・シュミット正規直交化のテスト（U軸が法線と平行でない場合）
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let u_direction = Vector3D::new(1.0, 1.0, 0.5); // 法線と平行でない

        let plane_sys = Plane3D::from_origin_and_axes(origin, normal, u_direction).unwrap();

        // 結果の座標系が直交系になることを確認
        let u_axis = plane_sys.u_axis;
        let v_axis = plane_sys.v_axis;
        let normal_axis = plane_sys.normal();

        // 直交性確認
        let dot_uv = u_axis.as_vector().dot(&v_axis.as_vector());
        let dot_un = u_axis.as_vector().dot(&normal_axis.as_vector());
        let dot_vn = v_axis.as_vector().dot(&normal_axis.as_vector());

        assert_abs_diff_eq!(dot_uv, 0.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(dot_un, 0.0, epsilon = TOLERANCE_F64);
        assert_abs_diff_eq!(dot_vn, 0.0, epsilon = TOLERANCE_F64);
    }

    #[test]
    fn test_error_cases() {
        // エラーケースのテスト
        let origin = Point3D::new(0.0, 0.0, 0.0);

        // ゼロベクトル法線
        let zero_normal = Vector3D::new(0.0, 0.0, 0.0);
        let u_direction = Vector3D::new(1.0, 0.0, 0.0);

        let result = Plane3D::from_origin_and_axes(origin, zero_normal, u_direction);
        assert!(result.is_none());

        // ゼロベクトルU軸
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let zero_u = Vector3D::new(0.0, 0.0, 0.0);

        let result = Plane3D::from_origin_and_axes(origin, normal, zero_u);
        assert!(result.is_none());

        // U軸が法線と平行（直交成分がゼロ）
        let parallel_u = Vector3D::new(0.0, 0.0, 1.0);

        let result = Plane3D::from_origin_and_axes(origin, normal, parallel_u);
        assert!(result.is_none());
    }
}
