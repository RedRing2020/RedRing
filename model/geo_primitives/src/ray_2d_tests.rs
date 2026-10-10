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

    #[test]
    fn test_closest_parameter_and_point() {
        let ray = Ray2D::new(Point2D::new(1.0_f64, 1.0), Vector2D::new(0.0, 1.0)).unwrap();

        // 起点の前方は投影した位置、後方は起点
        assert!((ray.closest_parameter(&Point2D::new(3.0, 4.0)) - 3.0).abs() < 1e-12);
        assert!(
            ray.closest_point(&Point2D::new(3.0, 4.0))
                .distance_to(&Point2D::new(1.0, 4.0))
                < 1e-12
        );
        assert_eq!(ray.closest_parameter(&Point2D::new(3.0, -2.0)), 0.0);
        assert!(
            ray.closest_point(&Point2D::new(3.0, -2.0))
                .distance_to(&Point2D::new(1.0, 1.0))
                < 1e-12
        );

        // parameter_for_point は起点の後方で負の値を返す
        assert!((ray.parameter_for_point(&Point2D::new(3.0, -2.0)) + 3.0).abs() < 1e-12);
    }
}
