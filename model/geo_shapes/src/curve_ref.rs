//! 稜線が参照する母曲線（稜線は、母曲線と、その native parameter の区間で持つ）

use geo_contracts::{
    Circle3DEvaluation, Ellipse3DEvaluation, InfiniteLine3DEvaluation, NurbsCurve3DEvaluation,
    NurbsCurve3DProperties, Scalar, default_kernel_numerical_zero_tolerance,
};
use geo_core::Point3D;
use geo_nurbs::NurbsCurve3D;

/// 稜線が参照する母曲線
#[derive(Debug, Clone)]
pub enum CurveRef<T: Scalar> {
    /// 無限の直線（パラメータは、基準点からの向きつきの距離）
    Line(geo_primitives::InfiniteLine3D<T>),
    /// 円（パラメータは、角度 0 の方向からの角度）
    Circle(geo_primitives::Circle3D<T>),
    /// 楕円（パラメータは、長軸の方向からの角度）
    Ellipse(geo_primitives::Ellipse3D<T>),
    /// NURBS 曲線（パラメータは、ノットベクトルの値）
    Nurbs(NurbsCurve3D<T>),
}

impl<T: Scalar> CurveRef<T> {
    /// NURBS の定義域の端で、区間のわずかなはみ出しを許す幅
    pub fn nurbs_parameter_tolerance_for_domain(u_min: T, u_max: T) -> T {
        let domain_span = (u_max - u_min).abs();
        let magnitude = u_min.abs().max(u_max.abs()).max(domain_span).max(T::ONE);
        let scaled_epsilon = T::EPSILON * T::from_f64(16.0) * magnitude;
        let minimum_tolerance = default_kernel_numerical_zero_tolerance::<T>();
        if scaled_epsilon > minimum_tolerance {
            scaled_epsilon
        } else {
            minimum_tolerance
        }
    }

    /// 母曲線の native parameter `t` の点
    ///
    /// NURBS では、定義域の端のわずかなはみ出しは端に収め、それより外の `t` ではパニックする（稜線の区間は、
    /// 表に足すときに定義域の中に収めてある）。
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        match self {
            Self::Line(line) => line.point_at_parameter(t),
            Self::Circle(circle) => circle.point_at_parameter(t),
            Self::Ellipse(ellipse) => ellipse.point_at_parameter(t),
            Self::Nurbs(curve) => {
                let (u_min, u_max) =
                    <NurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(curve);
                let tolerance = Self::nurbs_parameter_tolerance_for_domain(u_min, u_max);
                assert!(
                    t >= u_min - tolerance && t <= u_max + tolerance,
                    "CurveRef::point_at_parameter received out-of-domain NURBS parameter: t={:?}, domain=[{:?}, {:?}]",
                    t,
                    u_min,
                    u_max
                );
                let clamped = t.max(u_min).min(u_max);
                <NurbsCurve3D<T> as NurbsCurve3DEvaluation<T>>::point_at_parameter(curve, clamped)
                    .expect("a parameter clamped to the domain should be evaluable")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::{Circle3DConstructor, NurbsCurve3DConstructor};

    #[test]
    fn a_nurbs_curve_is_evaluated_with_its_knot_values() {
        let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            1,
            vec![2.0, 2.0, 3.0, 3.0],
            &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)],
            None,
        )
        .unwrap();
        let curve_ref = CurveRef::Nurbs(curve);

        assert!(
            curve_ref
                .point_at_parameter(2.0)
                .distance_to(Point3D::new(0.0, 0.0, 0.0))
                < 1.0e-12
        );
        assert!(
            curve_ref
                .point_at_parameter(2.5)
                .distance_to(Point3D::new(1.0, 0.0, 0.0))
                < 1.0e-9
        );
        assert!(
            curve_ref
                .point_at_parameter(3.0)
                .distance_to(Point3D::new(2.0, 0.0, 0.0))
                < 1.0e-12
        );
    }

    #[test]
    fn a_circle_is_evaluated_with_its_angle() {
        let circle =
            geo_primitives::Circle3D::new_xy_plane(Point3D::new(1.0, 0.0, 0.0), 2.0).unwrap();
        let curve_ref = CurveRef::Circle(circle);

        let start = curve_ref.point_at_parameter(0.0);
        let quarter = curve_ref.point_at_parameter(std::f64::consts::FRAC_PI_2);
        assert!((start.distance_to(Point3D::new(1.0, 0.0, 0.0)) - 2.0).abs() < 1.0e-12);
        assert!(start.distance_to(quarter) > 2.8);
    }
}
