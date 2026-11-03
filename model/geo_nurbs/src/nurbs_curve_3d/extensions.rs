//! `NurbsCurve3D` の適応分割

use crate::adaptive_tessellation::{
    adaptive_params_axis, AdaptiveParamList, AdaptiveTessellationSettings,
    NurbsCurveAdaptiveTessellation,
};
use crate::NurbsCurve3D;
use geo_contracts::{NurbsCurve3DProperties, Scalar};
use geo_core::Point3D;

impl<T: Scalar> NurbsCurve3D<T> {
    /// パラメータ区間 `[t0, t1]` の弦と、区間の中点での曲線上の点との距離
    fn chord_error(&self, t0: T, t1: T) -> T {
        let definition = self.definition();
        let point = |t: T| {
            let ([x, y, z], _) = definition.point_and_derivative(t);
            Point3D::new(x, y, z)
        };
        let middle = (t0 + t1) / (T::ONE + T::ONE);
        let chord_middle = point(t0).midpoint(point(t1));
        point(middle).distance_to(chord_middle)
    }
}

impl<T: Scalar> NurbsCurveAdaptiveTessellation<T> for NurbsCurve3D<T> {
    fn adaptive_params_curve(
        &self,
        settings: &AdaptiveTessellationSettings<T>,
    ) -> AdaptiveParamList<T> {
        let (t_min, t_max) = self.parameter_domain();
        let params = adaptive_params_axis(t_min, t_max, settings, |a, b| self.chord_error(a, b));
        AdaptiveParamList { params }
    }
}
