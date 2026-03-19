//! ConicalSurface3D - Intersection Tests
//!
//! 3次元円錐サーフェスの交差判定テスト

#[cfg(test)]
mod tests {
    use crate::{ConicalSurface3D, InfiniteLine3D, Plane3D, Point3D, Vector3D};
    use geo_contracts::{BasicIntersection, MultipleIntersection, SelfIntersection};

    const TOLERANCE: f64 = 1e-10;

    fn create_test_cone() -> ConicalSurface3D<f64> {
        // 原点中心、Z軸方向、半径1.0、半頂角45度の円錐サーフェス
        ConicalSurface3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            std::f64::consts::PI / 4.0, // 45度
        )
        .unwrap()
    }

    #[test]
    fn test_plane_intersection() {
        let cone = create_test_cone();
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        let result = cone.intersection_with(&plane, TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_line_intersection_outside() {
        let cone = create_test_cone();
        let line = InfiniteLine3D::new(Point3D::new(10.0, 0.0, 0.0), Vector3D::new(0.0, 0.0, 1.0))
            .unwrap();

        let result = cone.intersection_with(&line, TOLERANCE);
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_intersections_plane() {
        let cone = create_test_cone();
        let plane = Plane3D::from_point_and_normal(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        let results = cone.intersections_with(&plane, TOLERANCE);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_self_intersection() {
        let cone = create_test_cone();

        let results = cone.self_intersections(TOLERANCE);
        // 単一の円錐サーフェスは自己交差しない
        assert!(results.is_empty());
    }

    #[test]
    fn test_intersection_point_on_surface() {
        let cone = create_test_cone();
        let point = Point3D::new(1.0, 0.0, 0.0);

        let result = cone.intersection_with(&point, TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_intersection_point_outside() {
        let cone = create_test_cone();
        let point = Point3D::new(5.0, 0.0, 0.0);

        let result = cone.intersection_with(&point, TOLERANCE);
        assert!(result.is_none());
    }
}
