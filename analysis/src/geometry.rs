//! 幾何特化数値計算モジュール
//!
//! NURBS曲線、弧長計算など、幾何形状に特化した数値計算関数を提供する。
//! 元々model/src/analysis/numeric.rsにあった関数群を独立化。
use crate::DERIVATIVE_ZERO_THRESHOLD;

/// ニュートン法による方程式求解
pub fn newton_solve<F, G>(f: F, df: G, initial: f64, max_iter: usize, tol: f64) -> Option<f64>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    let mut x = initial;
    for _ in 0..max_iter {
        let fx = f(x);
        let dfx = df(x);
        if dfx.abs() < DERIVATIVE_ZERO_THRESHOLD {
            return None;
        }
        let next = x - fx / dfx;
        if (next - x).abs() < tol {
            return Some(next);
        }
        x = next;
    }
    None
}

/// 単調関数 f(x) = y に対する逆関数 x をニュートン法で求める
pub fn newton_inverse<F, G>(
    f: F,
    df: G,
    target: f64,
    initial: f64,
    max_iter: usize,
    tol: f64,
) -> Option<f64>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    let g = |x: f64| f(x) - target;
    newton_solve(g, df, initial, max_iter, tol)
}

/// NURBS/B-splineノットベクトルからスパンインデックスを検索
pub fn find_span(n: usize, degree: usize, u: f64, knots: &[f64]) -> usize {
    if u >= knots[n + 1] {
        return n;
    }
    if u <= knots[degree] {
        return degree;
    }

    let mut low = degree;
    let mut high = n + 1;
    let mut mid = (low + high) / 2;

    while u < knots[mid] || u >= knots[mid + 1] {
        if u < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }

    mid
}

/// B-spline基底関数Nᵢₚ(u)の値を計算
pub fn basis_functions(span: usize, u: f64, degree: usize, knots: &[f64]) -> Vec<f64> {
    let mut n = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];

    n[0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            let temp = n[r] / (right[r + 1] + left[j - r]);
            n[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        n[j] = saved;
    }

    n
}

/// B-spline基底関数の一階導関数 Nᵢₚ′(u) を計算
pub fn basis_function_derivatives(span: usize, u: f64, degree: usize, knots: &[f64]) -> Vec<f64> {
    let mut ders = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];
    let mut ndu = vec![vec![0.0; degree + 1]; degree + 1];

    ndu[0][0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            let temp = ndu[r][j - 1] / (right[r + 1] + left[j - r]);
            ndu[r][j] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        ndu[j][j] = saved;
    }

    #[allow(clippy::needless_range_loop)]
    for j in 0..=degree {
        ders[j] = 0.0;
    }

    for j in 1..=degree {
        let coeff = degree as f64 / (knots[span + j] - knots[span + j - degree]);
        ders[j - 1] = coeff * (ndu[j - 1][degree - 1] - ndu[j][degree - 1]);
    }

    ders
}

/// 曲線の弧長を数値積分で近似計算（model::geometry::Vector向け）
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

        // ベクトルの大きさを計算
        length += 0.5 * (v0.norm() + v1.norm()) * dt;
    }

    length
}

/// 弧長計算で使用するベクトルの共通インターフェース
pub trait NormedVector {
    fn norm(&self) -> f64;
}

// =============================================================================
// 楕円の数値解析機能
// =============================================================================

/// 楕円周長計算の各種近似手法を提供
pub mod ellipse_circumference {
    use std::f64::consts::PI;

    /// ラマヌジャンの近似式による楕円周長計算
    /// 
    /// # 数値解析について
    /// 楕円の正確な周長は第2種完全楕円積分で表現され、解析的解がないため近似が必要です。
    /// 
    /// ## 使用している手法: ラマヌジャンの近似式 (1914)
    /// - 精度: 相対誤差 < 5×10⁻⁵ (離心率0.99でも高精度)
    /// - 計算量: O(1) - 平方根1回のみ
    /// - 適用範囲: すべての楕円形状
    /// 
    /// # Arguments
    /// * `major_radius` - 楕円の長軸半径
    /// * `minor_radius` - 楕円の短軸半径
    /// 
    /// # Returns
    /// 楕円の周長の近似値
    pub fn ramanujan_approximation(major_radius: f64, minor_radius: f64) -> f64 {
        let a = major_radius;
        let b = minor_radius;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    /// 無限級数展開による楕円周長計算（高精度版）
    /// 
    /// より高精度だが計算量が大きい手法。高精度が必要な場合に使用。
    /// 
    /// # Arguments
    /// * `major_radius` - 楕円の長軸半径
    /// * `minor_radius` - 楕円の短軸半径
    /// * `terms` - 級数の項数（精度と計算量のトレードオフ）
    /// 
    /// # Returns
    /// 楕円の周長の高精度近似値
    pub fn series_expansion(major_radius: f64, minor_radius: f64, terms: usize) -> f64 {
        if major_radius == minor_radius {
            return 2.0 * PI * major_radius; // 円の場合
        }
        
        let a = major_radius.max(minor_radius);
        let b = major_radius.min(minor_radius);
        let e_squared = 1.0 - (b * b) / (a * a); // 離心率の二乗
        
        // 第2種完全楕円積分 E(e) の級数展開
        // E(e) = π/2 * [1 - Σ((2n-1)!!/(2n)!)^2 * e^(2n) / (2n-1))]
        let mut e_k = 1.0; // E(e) の値
        let mut coeff = 1.0;
        let mut power = e_squared;
        
        for n in 1..=terms {
            // (2n-1)!! / (2n)!! の計算
            coeff *= (2.0 * n as f64 - 1.0) / (2.0 * n as f64);
            let term = coeff * coeff * power / (2.0 * n as f64 - 1.0);
            e_k -= term;
            power *= e_squared;
            
            // 収束判定
            if term.abs() < 1e-15 {
                break;
            }
        }
        
        // 楕円周長 = 4 * a * E(e)
        4.0 * a * e_k * PI / 2.0
    }

    /// 数値積分による楕円周長計算（最高精度版）
    /// 
    /// ガウス・ルジャンドル求積法による数値積分。最高精度だが計算コスト高。
    /// 
    /// # Arguments
    /// * `major_radius` - 楕円の長軸半径
    /// * `minor_radius` - 楕円の短軸半径
    /// * `n_points` - 積分点数（精度と計算量のトレードオフ）
    /// 
    /// # Returns
    /// 楕円の周長の数値積分値
    pub fn numerical_integration(major_radius: f64, minor_radius: f64, n_points: usize) -> f64 {
        if major_radius == minor_radius {
            return 2.0 * PI * major_radius; // 円の場合
        }
        
        let a = major_radius;
        let b = minor_radius;
        let e_squared = 1.0 - (b * b) / (a * a);
        
        // ガウス・ルジャンドル求積法の実装（簡易版）
        let mut sum = 0.0;
        let dt = PI / (2.0 * n_points as f64);
        
        for i in 0..n_points {
            let t = (i as f64 + 0.5) * dt; // 中点公式
            let integrand = (1.0 - e_squared * t.sin().powi(2)).sqrt();
            sum += integrand * dt;
        }
        
        4.0 * a * sum
    }

    /// 楕円の離心率を計算
    /// 
    /// # Arguments
    /// * `major_radius` - 楕円の長軸半径
    /// * `minor_radius` - 楕円の短軸半径
    /// 
    /// # Returns
    /// 離心率 (0 ≤ e < 1)
    pub fn eccentricity(major_radius: f64, minor_radius: f64) -> f64 {
        if major_radius <= minor_radius {
            0.0
        } else {
            (1.0 - (minor_radius * minor_radius) / (major_radius * major_radius)).sqrt()
        }
    }
}

/// 楕円の様々な幾何学的性質を計算
pub mod ellipse_properties {
    use std::f64::consts::PI;

    /// 楕円の面積を計算
    pub fn area(major_radius: f64, minor_radius: f64) -> f64 {
        PI * major_radius * minor_radius
    }

    /// 楕円の焦点距離を計算
    pub fn focal_distance(major_radius: f64, minor_radius: f64) -> f64 {
        if major_radius <= minor_radius {
            0.0
        } else {
            (major_radius * major_radius - minor_radius * minor_radius).sqrt()
        }
    }

    /// 楕円の焦点座標を計算（中心が原点、長軸がx軸の場合）
    pub fn foci(major_radius: f64, minor_radius: f64) -> (f64, f64) {
        let c = focal_distance(major_radius, minor_radius);
        (c, -c)
    }
}

#[cfg(test)]
mod ellipse_tests {
    use super::ellipse_circumference::*;
    use super::ellipse_properties::*;

    #[test]
    fn test_circle_circumference() {
        let radius = 5.0;
        let expected = 2.0 * std::f64::consts::PI * radius;
        
        // 円の場合、すべての手法で同じ結果になるはず
        assert!((ramanujan_approximation(radius, radius) - expected).abs() < 1e-10);
        assert!((series_expansion(radius, radius, 10) - expected).abs() < 1e-10);
        assert!((numerical_integration(radius, radius, 100) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_ellipse_circumference_consistency() {
        let major = 10.0;
        let minor = 6.0;
        
        let ramanujan = ramanujan_approximation(major, minor);
        let series = series_expansion(major, minor, 20);
        let numerical = numerical_integration(major, minor, 200);
        
        // 各手法の結果が十分近いことを確認
        assert!((ramanujan - series).abs() / series < 1e-4, 
                "Ramanujan vs Series: {} vs {}", ramanujan, series);
        assert!((series - numerical).abs() / numerical < 1e-4,
                "Series vs Numerical: {} vs {}", series, numerical);
        assert!((ramanujan - numerical).abs() / numerical < 1e-4,
                "Ramanujan vs Numerical: {} vs {}", ramanujan, numerical);
    }

    #[test]
    fn test_ellipse_properties() {
        let major = 5.0;
        let minor = 3.0;
        
        assert!((area(major, minor) - std::f64::consts::PI * major * minor).abs() < 1e-10);
        assert!((focal_distance(major, minor) - 4.0).abs() < 1e-10);
        
        let (f1, f2) = foci(major, minor);
        assert!((f1 - 4.0).abs() < 1e-10);
        assert!((f2 - (-4.0)).abs() < 1e-10);
    }
}
