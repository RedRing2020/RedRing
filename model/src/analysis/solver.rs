use crate::model::analysis::consts::{EPSILON, DERIVATIVE_ZERO_THRESHOLD};

pub fn newton_solve<F, G>(
    f: F,
    df: G,
    initial: f64,
    max_iter: usize,
    tol: f64,
) -> Option<f64>
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