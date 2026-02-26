//! `NurbsSurface3D` の拡張機能（適応分割）

use crate::{NurbsSurface3D, Scalar};

use crate::adaptive_tessellation::{
    adaptive_params_axis, AdaptiveParamGrid, AdaptiveTessellationSettings,
    NurbsSurfaceAdaptiveTessellation,
};

impl<T: Scalar> NurbsSurface3D<T> {
    fn chord_error_u(&self, u0: T, u1: T, v_samples: &[T]) -> T {
        let mid = (u0 + u1) / (T::ONE + T::ONE);
        let mut max_err = T::ZERO;

        for &v in v_samples {
            let p0 = self.evaluate_at(u0, v);
            let p1 = self.evaluate_at(u1, v);
            let pm = self.evaluate_at(mid, v);
            let chord_mid = (p0 + p1) / (T::ONE + T::ONE);
            let err = (pm - chord_mid).norm();
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
            let p0 = self.evaluate_at(u, v0);
            let p1 = self.evaluate_at(u, v1);
            let pm = self.evaluate_at(u, mid);
            let chord_mid = (p0 + p1) / (T::ONE + T::ONE);
            let err = (pm - chord_mid).norm();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clamped_knot_vector;
    use geo_foundation::NurbsSurface3DConstructor;

    #[test]
    fn test_surface_adaptive_params_plane() {
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
        ];

        let u_knots = clamped_knot_vector(1, 2);
        let v_knots = clamped_knot_vector(1, 2);

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            None,
            u_knots,
            v_knots,
            1,
            1,
        )
        .unwrap();

        let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
        let grid = surface.adaptive_params_surface(&settings);

        assert!(grid.u_params.len() > settings.min_segments as usize);
        assert!(grid.v_params.len() > settings.min_segments as usize);

        let (u_min, u_max) = surface.parameter_domain().0;
        let (v_min, v_max) = surface.parameter_domain().1;
        assert!((grid.u_params.first().unwrap() - u_min).abs() < 1e-10);
        assert!((grid.u_params.last().unwrap() - u_max).abs() < 1e-10);
        assert!((grid.v_params.first().unwrap() - v_min).abs() < 1e-10);
        assert!((grid.v_params.last().unwrap() - v_max).abs() < 1e-10);
    }
}
