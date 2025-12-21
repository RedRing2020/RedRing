//! ConicalSolid3D collision tests

#[cfg(test)]
mod tests {
    use crate::{
        ConicalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Vector3D,
    };
    use geo_foundation::extensions::BasicCollision;

    /// テスト用の標準的な円錐ソリッドを作成
    /// - 底面中心: (0, 0, 0)
    /// - 軸: Z軸正方向
    /// - 参照方向: X軸正方向
    /// - 底面半径: 1.0
    /// - 高さ: 2.0
    fn create_test_cone() -> ConicalSolid3D<f64> {
        ConicalSolid3D::new(
            Point3D::origin(),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            2.0,
        )
        .unwrap()
    }

    // TODO: 円錐ソリッド内部の点の判定ロジックの改善が必要
    // #[test]
    // fn test_point_inside() {
    //     let cone = create_test_cone();
    //     // 底面近くの中心
    //     let point = Point3D::new(0.0, 0.0, 0.1);
    //     assert!(cone.intersects(&point, 1e-10));
    // }

    #[test]
    fn test_point_on_surface() {
        let cone = create_test_cone();
        // 底面エッジ
        let point = Point3D::new(1.0, 0.0, 0.0);
        assert!(cone.intersects(&point, 1e-6));
    }

    #[test]
    fn test_point_outside() {
        let cone = create_test_cone();
        // 外部の点
        let point = Point3D::new(2.0, 0.0, 0.0);
        assert!(!cone.intersects(&point, 1e-10));
    }

    #[test]
    fn test_point_at_apex() {
        let cone = create_test_cone();
        // 頂点
        let point = Point3D::new(0.0, 0.0, 2.0);
        assert!(cone.intersects(&point, 1e-6));
    }

    // TODO: 線分と円錐ソリッドの正確な距離計算が必要
    // #[test]
    // fn test_line_segment_intersection() {
    //     let cone = create_test_cone();
    //     // 円錐を貫通する線分
    //     let line = LineSegment3D::new(
    //         Point3D::new(0.0, 0.0, -1.0),
    //         Point3D::new(0.0, 0.0, 3.0),
    //     )
    //     .unwrap();
    //     assert!(cone.intersects(&line, 1e-10));
    // }

    #[test]
    fn test_line_segment_no_intersection() {
        let cone = create_test_cone();
        // 外部の線分
        let line = LineSegment3D::new(
            Point3D::new(3.0, 0.0, 0.0),
            Point3D::new(4.0, 0.0, 0.0),
        )
        .unwrap();
        assert!(!cone.intersects(&line, 1e-10));
    }

    // TODO: 光線と円錐ソリッドの正確な距離計算が必要
    // #[test]
    // fn test_ray_intersection() {
    //     let cone = create_test_cone();
    //     // 中心に向かう光線
    //     let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    //     assert!(cone.intersects(&ray, 1e-10));
    // }

    // TODO: 無限直線と円錐ソリッドの正確な距離計算が必要
    // #[test]
    // fn test_infinite_line_intersection() {
    //     let cone = create_test_cone();
    //     // 中心を通る無限直線
    //     let line = InfiniteLine3D::new(
    //         Point3D::new(0.0, 0.0, 1.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     )
    //     .unwrap();
    //     assert!(cone.intersects(&line, 1e-10));
    // }

    #[test]
    fn test_plane_intersection() {
        let cone = create_test_cone();
        // 底面と平行な平面（円錐を横切る）
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 1.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        assert!(cone.intersects(&plane, 1e-10));
    }

    // TODO: 平面と円錐ソリッドの正確な距離計算が必要
    // #[test]
    // fn test_plane_no_intersection() {
    //     let cone = create_test_cone();
    //     // 円錐から離れた平面
    //     let plane = Plane3D::from_point_and_normal(
    //         Point3D::new(0.0, 0.0, 5.0),
    //         Vector3D::new(0.0, 0.0, 1.0),
    //     )
    //     .unwrap();
    //     assert!(!cone.intersects(&plane, 1e-10));
    // }

    #[test]
    fn test_conical_solid_self_intersection() {
        let cone1 = create_test_cone();
        let cone2 = create_test_cone();
        // 同一の円錐ソリッド
        assert!(cone1.intersects(&cone2, 1e-10));
    }

    #[test]
    fn test_conical_solid_no_intersection() {
        let cone1 = create_test_cone();
        // 離れた円錐ソリッド
        let cone2 = ConicalSolid3D::new(
            Point3D::new(5.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            2.0,
        )
        .unwrap();
        assert!(!cone1.intersects(&cone2, 1e-10));
    }

    #[test]
    fn test_distance_to_point_inside() {
        let cone = create_test_cone();
        // 内部の点（中心軸上）
        let point = Point3D::new(0.0, 0.0, 1.0);
        let distance = cone.distance_to(&point);
        // 内部なので距離は小さい（表面までの距離）
        assert!(distance < 1.0);
    }

    #[test]
    fn test_distance_to_point_outside() {
        let cone = create_test_cone();
        // 外部の点
        let point = Point3D::new(3.0, 0.0, 0.0);
        let distance = cone.distance_to(&point);
        // 底面エッジからの距離（約2.0）
        assert!(distance > 1.5);
    }
}
