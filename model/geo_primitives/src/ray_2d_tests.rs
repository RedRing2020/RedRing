//! Ray2D のテスト

#[cfg(test)]
mod tests {
    use crate::{Point2D, Ray2D, Vector2D};

    #[test]
    fn test_contains_point() {
        let ray = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let tolerance = 1e-3;

        assert!(ray.contains_point(&Point2D::new(5.0, 0.0), tolerance));
        assert!(ray.contains_point(&Point2D::new(5.0, 0.5 * tolerance), tolerance));
        assert!(!ray.contains_point(&Point2D::new(5.0, 2.0 * tolerance), tolerance));
        assert!(!ray.contains_point(&Point2D::new(-2.0, 0.0), tolerance));

        // 起点の後方も、起点までの距離が許容誤差以内なら Ray 上とする（Ray3D と同じ）
        assert!(ray.contains_point(&Point2D::new(-0.5 * tolerance, 0.0), tolerance));
        assert!(!ray.contains_point(&Point2D::new(-2.0 * tolerance, 0.0), tolerance));
    }
}
