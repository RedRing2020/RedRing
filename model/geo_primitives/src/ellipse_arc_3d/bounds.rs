//! EllipseArc3D の Bounds 実装

use crate::EllipseArc3D;
use geo_contracts::{
    Angle, Bounded, EllipseArc3DEndpoint, EllipseArc3DEvaluation, EllipseArc3DProperties,
    EllipseArc3DTrimRange, Scalar,
};
use geo_core::Aabb3D;

/// 座標 `c + a·u·cos t + b·v·sin t` が極値となるパラメータ角
fn candidate_extreme_angles<T: Scalar>(major_component: T, minor_component: T) -> [T; 2] {
    let angle = minor_component.atan2(major_component);
    [angle, angle + T::PI]
}

/// 両端点と、角度範囲内で x・y・z が極値となる点を包む
impl<T: Scalar> Bounded<T> for EllipseArc3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let u = self.major_axis_direction().as_vector() * self.semi_major_axis();
        let v = self.minor_axis_direction().as_vector() * self.semi_minor_axis();

        let mut points = vec![self.start_point(), self.end_point()];
        for angle in candidate_extreme_angles(u.x(), v.x())
            .into_iter()
            .chain(candidate_extreme_angles(u.y(), v.y()))
            .chain(candidate_extreme_angles(u.z(), v.z()))
        {
            let angle = Angle::from_radians(angle);
            if self.contains_angle(angle) {
                points.push(self.point_at_angle(angle));
            }
        }

        Aabb3D::from_points(&points)
    }
}
