//! `NurbsSurface3D` の拡張機能（適応分割）

use crate::{NurbsSurface3D, Scalar};
use geo_contracts::NurbsSurface3DProperties;

use crate::adaptive_tessellation::{
    adaptive_params_axis, AdaptiveParamGrid, AdaptiveTessellationSettings,
    NurbsSurfaceAdaptiveTessellation,
};

impl<T: Scalar> NurbsSurface3D<T> {
    fn chord_error_u(&self, u0: T, u1: T, v_samples: &[T]) -> T {
        let mid = (u0 + u1) / (T::ONE + T::ONE);
        let mut max_err = T::ZERO;

        for &v in v_samples {
            let p0 = self.point_and_partials(u0, v).0;
            let p1 = self.point_and_partials(u1, v).0;
            let pm = self.point_and_partials(mid, v).0;
            let err = pm.distance_to(p0.midpoint(p1));
            if err > max_err {
                max_err = err;
            }
        }

        max_err
    }

    fn chord_error_v(&self, v0: T, v1: T, u_samples: &[T]) -> T {
        let mid = (v0 + v1) / (T::ONE + T::ONE);
        let mut max_err = T::ZERO;

        for &u in u_samples {
            let p0 = self.point_and_partials(u, v0).0;
            let p1 = self.point_and_partials(u, v1).0;
            let pm = self.point_and_partials(u, mid).0;
            let err = pm.distance_to(p0.midpoint(p1));
            if err > max_err {
                max_err = err;
            }
        }

        max_err
    }
}

impl<T: Scalar> NurbsSurfaceAdaptiveTessellation<T> for NurbsSurface3D<T> {
    fn adaptive_params_surface(
        &self,
        settings: &AdaptiveTessellationSettings<T>,
    ) -> AdaptiveParamGrid<T> {
        let ((u_min, u_max), (v_min, v_max)) = self.parameter_domain();
        let u_middle = (u_min + u_max) / (T::ONE + T::ONE);
        let v_middle = (v_min + v_max) / (T::ONE + T::ONE);

        let v_samples = vec![v_min, v_middle, v_max];
        let u_samples = vec![u_min, u_middle, u_max];

        let u_params = adaptive_params_axis(u_min, u_max, settings, |a, b| {
            self.chord_error_u(a, b, &v_samples)
        });

        let v_params = adaptive_params_axis(v_min, v_max, settings, |a, b| {
            self.chord_error_v(a, b, &u_samples)
        });

        AdaptiveParamGrid { u_params, v_params }
    }
}
