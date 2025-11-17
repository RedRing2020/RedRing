//! 数値積分手法
//!
//! 弧長計算や一般的な数値積分機能を提供する。
//! 台形公式やシンプソン公式など、様々な積分手法を実装。

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

#[cfg(test)]
mod tests {
    use super::*;

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
        // 直線の弧長：r'(t) = (1, 0) でt∈[0,1]
        let line_derivative = |_t: f64| TestVector2D { x: 1.0, y: 0.0 };
        let arc_length = newton_arc_length(line_derivative, 0.0, 1.0, 100);

        // 理論値: 1.0
        assert!((arc_length - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_newton_arc_length_circle() {
        // 半径1の円の弧長：r'(t) = (-sin(t), cos(t))
        let circle_derivative = |t: f64| TestVector2D {
            x: -t.sin(),
            y: t.cos(),
        };

        // 半周の弧長を計算（理論値: π）
        let arc_length = newton_arc_length(circle_derivative, 0.0, std::f64::consts::PI, 1000);
        assert!((arc_length - std::f64::consts::PI).abs() < 1e-3);
    }

    #[test]
    fn test_trapezoidal_rule_quadratic() {
        // ∫₀¹ x² dx = 1/3 を計算
        let result = trapezoidal_rule(|x| x * x, 0.0, 1.0, 1000);
        assert!((result - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_trapezoidal_rule_sine() {
        // ∫₀π sin(x) dx = 2 を計算
        let result = trapezoidal_rule(|x| x.sin(), 0.0, std::f64::consts::PI, 10000);
        assert!((result - 2.0).abs() < 1e-4);
    }
}
