//! Ellipse3D - Intersection Tests
//!
//! 3次元楕円の交点計算のテストスイート

#[cfg(test)]
mod tests {
    use crate::{Ellipse3D, Point3D};
    use geo_contracts::{BasicIntersection, SelfIntersection};

    #[test]
    fn test_ellipse3d_point_intersection() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::new(0.0, 0.0, 0.0), 3.0, 2.0).unwrap();

        let center = Point3D::new(0.0, 0.0, 0.0);
        // 楕円の中心は周上の点まで semi_minor 以下の距離
        assert!(ellipse.intersection_with(&center, 2.0).is_some());

        let outside = Point3D::new(10.0, 10.0, 10.0);
        assert!(ellipse.intersection_with(&outside, f64::EPSILON).is_none());
    }

    #[test]
    fn test_ellipse3d_self_intersection() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::new(0.0, 0.0, 0.0), 3.0, 2.0).unwrap();

        let self_intersections = ellipse.self_intersections(f64::EPSILON);
        assert!(self_intersections.is_empty()); // 楕円は自己交差しない
    }
}
