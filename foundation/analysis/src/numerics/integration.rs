//! 数値積分手法
//!
//! 弧長計算や一般的な数値積分機能を提供する。
//! 台形公式やシンプソン公式など、様々な積分手法を実装。

use crate::Scalar;

/// 曲線の弧長を数値積分で近似計算（汎用ベクトル向け）
///
/// パラメトリック曲線 r(t) の弧長を台形公式により数値積分で計算。
/// 任意のベクトル型に対応するため、NormedVectorトレイトを使用。
///
/// # Arguments
/// * `evaluate` - パラメータ t に対する曲線の導関数 r'(t)
/// * `start` - 積分開始パラメータ
/// * `end` - 積分終了パラメータ
/// * `steps` - 分割数（精度に影響）
///
/// # Returns
/// 弧長の近似値
///
/// # Example
/// ```rust
/// use analysis::numerics::integration::{newton_arc_length, NormedVector};
///
/// // 2D円の弧長計算の例
/// #[derive(Clone)]
/// struct Vec2D { x: f64, y: f64 }
///
/// impl NormedVector for Vec2D {
///     fn norm(&self) -> f64 {
///         (self.x * self.x + self.y * self.y).sqrt()
///     }
/// }
///
/// // 半径1の円の導関数：r'(t) = (-sin(t), cos(t))
/// let circle_derivative = |t: f64| Vec2D {
///     x: -t.sin(),
///     y: t.cos()
/// };
///
/// // 半周の弧長を計算（理論値: π）
/// let arc_length = newton_arc_length(circle_derivative, 0.0, std::f64::consts::PI, 1000);
/// assert!((arc_length - std::f64::consts::PI).abs() < 1e-3);
/// ```
pub fn newton_arc_length<F, V>(evaluate: F, start: f64, end: f64, steps: usize) -> f64
where
    F: Fn(f64) -> V,
    V: NormedVector,
{
    let mut length = 0.0;
    let dt = (end - start) / steps as f64;

    for i in 0..steps {
        let t0 = start + i as f64 * dt;
        let t1 = t0 + dt;

        let v0 = evaluate(t0);
        let v1 = evaluate(t1);

        // ベクトルの大きさを計算（台形公式）
        length += 0.5 * (v0.norm() + v1.norm()) * dt;
    }

    length
}

/// 弧長計算で使用するベクトルの共通インターフェース
///
/// 異なるベクトル実装に対して統一的なノルム計算を提供。
/// 2D、3D、N次元ベクトルなど、任意の次元に対応可能。
///
/// # Example Implementation
/// ```rust
/// use analysis::numerics::integration::NormedVector;
///
/// struct Vector3D { x: f64, y: f64, z: f64 }
///
/// impl NormedVector for Vector3D {
///     fn norm(&self) -> f64 {
///         (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
///     }
/// }
/// ```
pub trait NormedVector {
    /// ベクトルのノルム（大きさ）を計算
    fn norm(&self) -> f64;
}

/// 台形公式による一般的な数値積分
///
/// 関数 f(x) を区間 [a, b] で台形公式により数値積分する。
///
/// # Arguments
/// * `f` - 積分したい関数
/// * `a` - 積分開始点
/// * `b` - 積分終了点
/// * `n` - 分割数（精度に影響）
///
/// # Returns
/// 積分の近似値
///
/// # Example
/// ```rust
/// use analysis::numerics::integration::trapezoidal_rule;
///
/// // ∫₀¹ x² dx = 1/3 を計算
/// let result = trapezoidal_rule(|x| x * x, 0.0, 1.0, 1000);
/// assert!((result - 1.0/3.0).abs() < 1e-6);
/// ```
pub fn trapezoidal_rule<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    let h = (b - a) / n as f64;
    let mut sum = 0.5 * (f(a) + f(b));

    for i in 1..n {
        let x = a + i as f64 * h;
        sum += f(x);
    }

    sum * h
}

/// 2 等分した結果との差が `tolerance` に収まるまで区間を分割する、5 点ガウス・ルジャンドル求積
///
/// 区間 `[a, b]` の 5 点の求積と、2 等分した 2 区間の求積の和との差が `tolerance` 以下なら後者を返す。
/// そうでなければ、それぞれの半分の区間を `tolerance` の半分で同じように積分する。分割の深さが `max_depth` に
/// 達した区間は、その時点の 2 区間の求積の和を返す。
pub fn adaptive_gauss_legendre<T: Scalar>(
    f: impl Fn(T) -> T,
    a: T,
    b: T,
    tolerance: T,
    max_depth: usize,
) -> T {
    adaptive_gauss_legendre_by_ref(&f, a, b, tolerance, max_depth)
}

fn adaptive_gauss_legendre_by_ref<T: Scalar>(
    f: &impl Fn(T) -> T,
    a: T,
    b: T,
    tolerance: T,
    depth: usize,
) -> T {
    let whole = gauss_legendre_5(f, a, b);
    let middle = (a + b) / (T::ONE + T::ONE);
    let halves = gauss_legendre_5(f, a, middle) + gauss_legendre_5(f, middle, b);
    if depth == 0 || (whole - halves).abs() <= tolerance {
        return halves;
    }
    let half_tolerance = tolerance / (T::ONE + T::ONE);
    adaptive_gauss_legendre_by_ref(f, a, middle, half_tolerance, depth - 1)
        + adaptive_gauss_legendre_by_ref(f, middle, b, half_tolerance, depth - 1)
}

/// 5 点ガウス・ルジャンドル求積
fn gauss_legendre_5<T: Scalar>(f: &impl Fn(T) -> T, a: T, b: T) -> T {
    const NODES: [f64; 5] = [
        0.0,
        -0.538_469_310_105_683_1,
        0.538_469_310_105_683_1,
        -0.906_179_845_938_664,
        0.906_179_845_938_664,
    ];
    const WEIGHTS: [f64; 5] = [
        0.568_888_888_888_888_9,
        0.478_628_670_499_366_5,
        0.478_628_670_499_366_5,
        0.236_926_885_056_189_1,
        0.236_926_885_056_189_1,
    ];
    let half = (b - a) / (T::ONE + T::ONE);
    let middle = (a + b) / (T::ONE + T::ONE);
    let mut sum = T::ZERO;
    for (node, weight) in NODES.iter().zip(WEIGHTS) {
        sum += T::from_f64(weight) * f(middle + half * T::from_f64(*node));
    }
    sum * half
}

#[cfg(test)]
mod tests {
    use super::{adaptive_gauss_legendre, newton_arc_length, trapezoidal_rule, NormedVector};
    use crate::consts::test_constants::{
        INTEGRATION_TOLERANCE, INTEGRATION_TOLERANCE_LOOSE, INTEGRATION_TOLERANCE_STRICT,
        TOLERANCE_F64,
    };

    #[derive(Clone)]
    struct TestVector2D {
        x: f64,
        y: f64,
    }

    impl NormedVector for TestVector2D {
        fn norm(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
    }

    #[test]
    fn test_newton_arc_length_line() {
        let line_derivative = |_t: f64| TestVector2D { x: 1.0, y: 0.0 };
        let arc_length = newton_arc_length(line_derivative, 0.0, 1.0, 100);

        assert!((arc_length - 1.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_newton_arc_length_circle() {
        let circle_derivative = |t: f64| TestVector2D {
            x: -t.sin(),
            y: t.cos(),
        };

        let arc_length = newton_arc_length(circle_derivative, 0.0, std::f64::consts::PI, 1000);
        assert!((arc_length - std::f64::consts::PI).abs() < INTEGRATION_TOLERANCE_LOOSE);
    }

    #[test]
    fn test_trapezoidal_rule_quadratic() {
        let result = trapezoidal_rule(|x| x * x, 0.0, 1.0, 1000);
        assert!((result - 1.0 / 3.0).abs() < INTEGRATION_TOLERANCE_STRICT);
    }

    #[test]
    fn test_trapezoidal_rule_sine() {
        let result = trapezoidal_rule(|x| x.sin(), 0.0, std::f64::consts::PI, 10000);
        assert!((result - 2.0).abs() < INTEGRATION_TOLERANCE);
    }

    #[test]
    fn adaptive_gauss_legendre_integrates_smooth_functions() {
        // ∫₀^π sin x dx = 2、∫₀¹ e^x dx = e - 1
        let sine = adaptive_gauss_legendre(f64::sin, 0.0, std::f64::consts::PI, 1e-12, 20);
        assert!((sine - 2.0).abs() < 1e-12);
        let exponential = adaptive_gauss_legendre(f64::exp, 0.0, 1.0, 1e-12, 20);
        assert!((exponential - (std::f64::consts::E - 1.0)).abs() < 1e-12);
    }
}
