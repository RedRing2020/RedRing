//! 稜線が参照する母曲線

use crate::{Point3D, TopoArc3D, TopoEllipseArc3D, TopoLineSegment3D, TopoNurbsCurve3D};
use geo_contracts::{
    Arc3DEndpoint, Arc3DEvaluation, EllipseArc3DDerived, EllipseArc3DEndpoint,
    EllipseArc3DEvaluation, InfiniteLine3DEvaluation, LineSegment3DProperties,
    LineSegment3DSupportLine, NurbsCurve3DDerived, NurbsCurve3DEvaluation, NurbsCurve3DProperties,
    Scalar, default_kernel_numerical_zero_tolerance,
};

/// 稜線が参照する母曲線
#[derive(Debug, Clone)]
pub enum CurveRef<T: Scalar> {
    Line(TopoLineSegment3D<T>),
    Arc(TopoArc3D<T>),
    EllipseArc(TopoEllipseArc3D<T>),
    Nurbs(TopoNurbsCurve3D<T>),
}

impl<T: Scalar> CurveRef<T> {
    pub(crate) fn nurbs_parameter_tolerance_for_domain(u_min: T, u_max: T) -> T {
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

    pub fn ideal_start_point(&self) -> Point3D<T> {
        match self {
            Self::Line(line) => line.start_point(),
            Self::Arc(arc) => <TopoArc3D<T> as Arc3DEndpoint<T>>::start_point(arc),
            Self::EllipseArc(arc) => {
                <TopoEllipseArc3D<T> as EllipseArc3DEndpoint<T>>::start_point(arc)
            }
            Self::Nurbs(curve) => {
                let (u_min, _) =
                    <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(curve);
                <TopoNurbsCurve3D<T> as NurbsCurve3DEvaluation<T>>::point_at_parameter(curve, u_min)
                    .expect("the end of the parameter domain should be evaluable")
            }
        }
    }

    pub fn ideal_end_point(&self) -> Point3D<T> {
        match self {
            Self::Line(line) => line.end_point(),
            Self::Arc(arc) => <TopoArc3D<T> as Arc3DEndpoint<T>>::end_point(arc),
            Self::EllipseArc(arc) => {
                <TopoEllipseArc3D<T> as EllipseArc3DEndpoint<T>>::end_point(arc)
            }
            Self::Nurbs(curve) => {
                let (_, u_max) =
                    <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(curve);
                <TopoNurbsCurve3D<T> as NurbsCurve3DEvaluation<T>>::point_at_parameter(curve, u_max)
                    .expect("the end of the parameter domain should be evaluable")
            }
        }
    }

    /// 母曲線の native parameter semantics をそのまま使って評価点を返す
    ///
    /// topology 側では family 横断で parameter を再正規化しない。
    /// 現在の variant では Line / Arc / EllipseArc のいずれも bounded curve として
    /// ideal start/end と整合する parameter domain を使う。
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        match self {
            Self::Line(line) => line.support_line().point_at_parameter(t),
            Self::Arc(arc) => <TopoArc3D<T> as Arc3DEvaluation<T>>::point_at_parameter(arc, t),
            Self::EllipseArc(arc) => {
                <TopoEllipseArc3D<T> as EllipseArc3DEvaluation<T>>::point_at_parameter(arc, t)
            }
            Self::Nurbs(curve) => {
                let (u_min, u_max) =
                    <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(curve);
                let tolerance = Self::nurbs_parameter_tolerance_for_domain(u_min, u_max);
                let lower_bound = u_min - tolerance;
                let upper_bound = u_max + tolerance;
                let is_out_of_domain = t < lower_bound || t > upper_bound;
                assert!(
                    !is_out_of_domain,
                    "CurveRef::point_at_parameter received out-of-domain NURBS parameter: t={:?}, domain=[{:?}, {:?}], tolerance={:?}, accepted_bounds=[{:?}, {:?}]",
                    t, u_min, u_max, tolerance, lower_bound, upper_bound
                );

                let adjusted_t = if t < u_min {
                    u_min
                } else if t > u_max {
                    u_max
                } else {
                    t
                };

                <TopoNurbsCurve3D<T> as NurbsCurve3DEvaluation<T>>::point_at_parameter(
                    curve, adjusted_t,
                )
                .expect("a parameter clamped to the domain should be evaluable")
            }
        }
    }

    pub fn length(&self) -> T {
        match self {
            Self::Line(line) => line.length(),
            Self::Arc(arc) => <TopoArc3D<T> as geo_contracts::Arc3DDerived<T>>::length(arc),
            Self::EllipseArc(arc) => <TopoEllipseArc3D<T> as EllipseArc3DDerived<T>>::length(arc),
            Self::Nurbs(curve) => <TopoNurbsCurve3D<T> as NurbsCurve3DDerived<T>>::length(curve),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::NurbsCurve3DConstructor;

    #[test]
    fn curve_ref_nurbs_supports_ideal_endpoints_and_native_parameter_evaluation() {
        let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            1,
            vec![2.0, 2.0, 3.0, 3.0],
            &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)],
            None,
        )
        .unwrap();

        let curve_ref = CurveRef::Nurbs(curve);

        let start = curve_ref.ideal_start_point();
        let end = curve_ref.ideal_end_point();

        assert!(start.distance_to(Point3D::new(0.0, 0.0, 0.0)) < 1.0e-12);
        assert!(end.distance_to(Point3D::new(2.0, 0.0, 0.0)) < 1.0e-12);

        let mid = curve_ref.point_at_parameter(2.5);
        assert!(mid.distance_to(Point3D::new(1.0, 0.0, 0.0)) < 1.0e-9);
    }
}
