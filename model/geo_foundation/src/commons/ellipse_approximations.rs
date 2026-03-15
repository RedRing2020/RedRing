//! Ellipse approximation and derived metric formulas.

use crate::Scalar;

/// Ramanujan approximation I for ellipse perimeter.
pub fn ellipse_perimeter_ramanujan_i<T: Scalar>(a: T, b: T) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    (a + b)
        * T::PI
        * (T::ONE
            + (T::from_f64(3.0) * h)
                / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
}

/// Ramanujan approximation II for ellipse perimeter.
pub fn ellipse_perimeter_ramanujan_ii<T: Scalar>(a: T, b: T) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    let numerator = T::from_f64(3.0) * h;
    let denominator = T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt();
    (a + b) * T::PI * (T::ONE + numerator / denominator)
}

/// Pade approximation for ellipse perimeter.
pub fn ellipse_perimeter_padé<T: Scalar>(a: T, b: T) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    let term1 = T::from_f64(64.0) - T::from_f64(3.0) * h.powi(2);
    let term2 = T::from_f64(256.0) - T::from_f64(48.0) * h - T::from_f64(21.0) * h.powi(2);
    (a + b) * T::PI * (T::ONE + h * term1 / term2)
}

/// Cantrell approximation for ellipse perimeter.
pub fn ellipse_perimeter_cantrell<T: Scalar>(a: T, b: T) -> T {
    let a_plus_b = a + b;
    let sqrt_ab = (a * b).sqrt();
    T::PI * (T::from_f64(1.5) * a_plus_b - sqrt_ab)
}

/// Series expansion for ellipse perimeter.
pub fn ellipse_circumference_series<T: Scalar>(a: T, b: T, terms: usize) -> T {
    let h = ((a - b) / (a + b)).powi(2);
    let mut sum = T::ONE;
    let mut coefficient = T::ONE;
    let mut h_power = h;

    for n in 1..=terms {
        let n_t = T::from_usize(n);
        coefficient = coefficient * (T::from_f64(2.0) * n_t - T::ONE) / (T::from_f64(2.0) * n_t);
        sum += coefficient.powi(2) * h_power;
        h_power *= h;
    }

    T::PI * (a + b) * sum
}

/// Numerical integration (Simpson) for ellipse perimeter.
pub fn ellipse_circumference_numerical<T: Scalar>(a: T, b: T, n_intervals: usize) -> T {
    let pi_2 = T::PI / T::from_f64(2.0);
    let h = pi_2 / T::from_usize(n_intervals);
    let integrand = |t: T| -> T {
        let sin_t = t.sin();
        let cos_t = t.cos();
        (a.powi(2) * sin_t.powi(2) + b.powi(2) * cos_t.powi(2)).sqrt()
    };

    let mut sum = integrand(T::ZERO) + integrand(pi_2);
    for i in 1..n_intervals {
        let t = T::from_usize(i) * h;
        if i % 2 == 0 {
            sum += T::from_f64(2.0) * integrand(t);
        } else {
            sum += T::from_f64(4.0) * integrand(t);
        }
    }
    T::from_f64(4.0) * h * sum / T::from_f64(3.0)
}

/// Eccentricity of an ellipse.
pub fn ellipse_eccentricity<T: Scalar>(a: T, b: T) -> T {
    (T::ONE - (b / a).powi(2)).sqrt()
}

/// Focal distance from center.
pub fn ellipse_focal_distance<T: Scalar>(a: T, b: T) -> T {
    (a.powi(2) - b.powi(2)).sqrt()
}

/// Foci coordinates assuming center at origin on X axis.
pub fn ellipse_foci<T: Scalar>(a: T, b: T) -> ((T, T), (T, T)) {
    let c = ellipse_focal_distance(a, b);
    ((-c, T::ZERO), (c, T::ZERO))
}
