//! EllipseArc2D の Bounds 実装

use crate::EllipseArc2D;
use geo_contracts::{
    Angle, Bounded, EllipseArc2DEndpoint, EllipseArc2DEvaluation, EllipseArc2DProperties,
    EllipseArc2DTrimRange, Scalar,
};
use geo_core::Aabb2D;

/// 座標 `c + a·u·cos t + b·v·sin t` が極値となるパラメータ角
fn candidate_extreme_angles<T: Scalar>(major_component: T, minor_component: T) -> [T; 2] {
    let angle = minor_component.atan2(major_component);
    [angle, angle + T::PI]
}

/// 両端点と、角度範囲内で x・y が極値となる点を包む
impl<T: Scalar> Bounded<T> for EllipseArc2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let rotation = self.rotation().to_radians();
        let (cos_r, sin_r) = (rotation.cos(), rotation.sin());
        let (a, b) = (self.semi_major_axis(), self.semi_minor_axis());

        // x(t) = cx + a·cos r·cos t − b·sin r·sin t、y(t) = cy + a·sin r·cos t + b·cos r·sin t
        let mut points = vec![self.start_point(), self.end_point()];
        for angle in candidate_extreme_angles(a * cos_r, -b * sin_r)
            .into_iter()
            .chain(candidate_extreme_angles(a * sin_r, b * cos_r))
        {
            let angle = Angle::from_radians(angle);
            if self.contains_angle(angle) {
                points.push(self.point_at_angle(angle));
            }
        }

        Aabb2D::from_points(&points)
    }
}

#[cfg(test)]
mod tests {
    use crate::{EllipseArc2D, Point2D};
    use analysis::test_constants::TOLERANCE_F64;
    use geo_contracts::{
        default_distance_tolerance, Angle, Bounded, EllipseArc2DConstructor, EllipseArc2DEvaluation,
    };

    #[test]
    fn aabb_contains_every_point_of_rotated_arc() {
        let arc = EllipseArc2D::new(
            Point2D::new(1.0_f64, 2.0),
            3.0,
            1.0,
            Angle::from_degrees(30.0),
            Angle::from_degrees(20.0),
            Angle::from_degrees(200.0),
        )
        .unwrap();
        let aabb = arc.aabb().unwrap();

        let tolerance = TOLERANCE_F64;
        // 10000 分割の点の間隔では、端の値の取りこぼしは距離トレランスより小さい
        let touch_tolerance = default_distance_tolerance::<f64>();
        let samples = 10_000;
        let mut touches = [false; 4];
        for i in 0..=samples {
            let p = arc.point_at_parameter(f64::from(i) / f64::from(samples));
            assert!(p.x() >= aabb.min_point().x() - tolerance);
            assert!(p.x() <= aabb.max_point().x() + tolerance);
            assert!(p.y() >= aabb.min_point().y() - tolerance);
            assert!(p.y() <= aabb.max_point().y() + tolerance);
            touches[0] |= (p.x() - aabb.min_point().x()).abs() < touch_tolerance;
            touches[1] |= (p.x() - aabb.max_point().x()).abs() < touch_tolerance;
            touches[2] |= (p.y() - aabb.min_point().y()).abs() < touch_tolerance;
            touches[3] |= (p.y() - aabb.max_point().y()).abs() < touch_tolerance;
        }
        // 境界ボックスは弧に接する（必要以上に大きくない）
        assert!(touches.iter().all(|&t| t));
    }
}
