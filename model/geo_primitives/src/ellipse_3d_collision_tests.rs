//! Ellipse3D - Collision Tests
//!
//! 3次元楕円の衝突判定のテストスイート

#[cfg(test)]
mod tests {
    use crate::{Ellipse3D, LineSegment3D, Point3D, Vector3D};
    use geo_foundation::extensions::BasicCollision;

    #[test]
    fn test_ellipse3d_point_collision() {
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            3.0, // semi_major
            2.0, // semi_minor
            Vector3D::unit_z(),
            Vector3D::unit_x(),
        )
        .unwrap();

        let center = Point3D::new(0.0, 0.0, 0.0);
        // 楕円の中心は内部なので、周上の点までの距離が semi_minor 以下
        assert!(ellipse.intersects(&center, 2.0));

        let outside = Point3D::new(10.0, 10.0, 10.0);
        assert!(!ellipse.intersects(&outside, f64::EPSILON));
    }

    #[test]
    fn test_ellipse3d_line_segment_collision() {
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            3.0,
            2.0,
            Vector3D::unit_z(),
            Vector3D::unit_x(),
        )
        .unwrap();

        // 楕円の長軸上の点を通る線分
        let crossing =
            LineSegment3D::new(Point3D::new(-3.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();
        // 端点が楕円上にある
        assert!(ellipse.intersects(&crossing, 0.1));
    }
}
