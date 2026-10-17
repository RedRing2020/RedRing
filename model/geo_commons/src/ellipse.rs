//! 2D 楕円のパラメータの計算
//!
//! 楕円は局所座標（中心が原点、x 軸方向の半軸 `a`、y 軸方向の半軸 `b`）で、パラメータ `θ` の点を
//! `(a cos θ, b sin θ)` とする。`θ` についての式は `x = tan(θ/2)` で 4 次多項式にし、楕円を半周ずつ
//! （`-π/2 <= θ <= π/2` と、点を原点について反転した残りの半周）に分けて `|x| <= 1` の区間で解く。

use analysis::{polynomial_real_roots_in_interval, Scalar};

/// 点 `(u, v)` までの距離が極値になる楕円上のパラメータ（`-π/2 <= θ < 3π/2`）
///
/// 距離の 2 乗の導関数 `(b² - a²) sin θ cos θ + a u sin θ - b v cos θ = 0` の根。
/// 距離が楕円上で一定の場合（円の中心）は根を返さない。
pub fn ellipse_2d_distance_critical_parameters<T: Scalar>(u: T, v: T, a: T, b: T) -> Vec<T> {
    let two = T::ONE + T::ONE;
    half_turn_parameters(u, v, |u, v| {
        let axis_difference = two * (b * b - a * a);
        let au = two * a * u;
        let bv = b * v;
        [-bv, axis_difference + au, T::ZERO, au - axis_difference, bv]
    })
}

/// 点 `(u, v)` を中心とする半径 `r` の円と楕円が交わるパラメータ（`-π/2 <= θ < 3π/2`）
///
/// `|(a cos θ, b sin θ) - (u, v)|² = r²` の根。符号が変わらない接点は返さない。
pub fn ellipse_2d_circle_intersection_parameters<T: Scalar>(
    u: T,
    v: T,
    a: T,
    b: T,
    r: T,
) -> Vec<T> {
    let two = T::ONE + T::ONE;
    let four = two + two;
    half_turn_parameters(u, v, |u, v| {
        let k = u * u + v * v - r * r;
        let aa = a * a;
        let au = two * a * u;
        let bv = four * b * v;
        [
            aa - au + k,
            -bv,
            two * (two * b * b - aa + k),
            -bv,
            aa + au + k,
        ]
    })
}

/// 半周ずつ解いたパラメータ
///
/// `coefficients(u, v)` は、`x = tan(θ/2)` で表した式に `(1 + x²)²` を掛けた 4 次多項式の係数（低次から）。
/// 残りの半周は、点を原点について反転して（`θ - π` の点は `-(a cos θ, b sin θ)` なので）同じ式で解く。
fn half_turn_parameters<T: Scalar>(u: T, v: T, coefficients: impl Fn(T, T) -> [T; 5]) -> Vec<T> {
    let two = T::ONE + T::ONE;
    let to_parameter = |x: T| two * x.atan();
    let mut parameters: Vec<T> =
        polynomial_real_roots_in_interval(&coefficients(u, v), -T::ONE, T::ONE)
            .into_iter()
            .map(to_parameter)
            .collect();
    parameters.extend(
        polynomial_real_roots_in_interval(&coefficients(-u, -v), -T::ONE, T::ONE)
            .into_iter()
            // θ = π/2, 3π/2（-π/2）は前の半周の x = ±1 として求めている
            .filter(|x| -T::ONE < *x && *x < T::ONE)
            .map(|x| to_parameter(x) + T::PI),
    );
    parameters
}

#[cfg(test)]
mod tests {
    use super::{
        ellipse_2d_circle_intersection_parameters, ellipse_2d_distance_critical_parameters,
    };

    fn point(theta: f64, a: f64, b: f64) -> (f64, f64) {
        (a * theta.cos(), b * theta.sin())
    }

    #[test]
    fn critical_parameters_of_an_inner_point_on_the_minor_axis() {
        // 短軸上の内部の点 (0, 0.5): 短軸の両端（θ = ±π/2）と、長軸寄りの 2 点が極値になる
        let (a, b) = (2.0, 1.0);
        let parameters = ellipse_2d_distance_critical_parameters(0.0, 0.5, a, b);
        assert_eq!(parameters.len(), 4, "{parameters:?}");
        let distances: Vec<f64> = parameters
            .iter()
            .map(|theta| {
                let (x, y) = point(*theta, a, b);
                (x * x + (y - 0.5) * (y - 0.5)).sqrt()
            })
            .collect();
        let min = distances.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = distances.iter().cloned().fold(0.0, f64::max);
        assert!((min - 0.5).abs() < 1e-12);
        // 長軸寄りの 2 点で最大（x² = 4(1 - y²) で y = -1/6 のとき）
        assert!(
            (max - (4.0_f64 * (1.0 - 1.0 / 36.0) + (2.0 / 3.0) * (2.0 / 3.0)).sqrt()).abs() < 1e-12
        );
    }

    #[test]
    fn intersection_parameters_lie_on_both_curves() {
        let (a, b) = (2.0, 1.0);
        let (u, v, r) = (1.0, 0.5, 1.2);
        let parameters = ellipse_2d_circle_intersection_parameters(u, v, a, b, r);
        assert_eq!(parameters.len(), 2, "{parameters:?}");
        for theta in parameters {
            let (x, y) = point(theta, a, b);
            assert!((((x - u).powi(2) + (y - v).powi(2)).sqrt() - r).abs() < 1e-12);
        }
    }

    #[test]
    fn four_intersections_when_the_circle_crosses_both_halves() {
        let parameters = ellipse_2d_circle_intersection_parameters(0.0, 0.0, 2.0, 1.0, 1.5);
        assert_eq!(parameters.len(), 4, "{parameters:?}");
    }
}
