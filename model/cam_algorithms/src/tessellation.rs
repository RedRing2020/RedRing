//! NURBS 曲面集合の三角形離散化（逆オフセットの前処理）
//!
//! 適応パラメータ分割で初期グリッドを作り、各セルの中心・辺中点で弦誤差を検証する。
//! 許容弦誤差を満たさないセルはパラメータ区間を二分して再検証し、
//! 反復上限内に収束しなければ `convergence_failure` を返す。
//!
//! 各セルは短い方の対角線で 2 三角形に分割する。三角形の形状品質（アスペクト比）は制御しない。
//! 逆オフセットは要素ごとに厳密に計算するため、細長三角形でも工具位置は変わらず、
//! 経路生成時間は三角形数が支配的となる（設計: CAM_ALGORITHMS_DESIGN.md §10）。

use geo_algorithms::adaptive_tessellation::{
    AdaptiveTessellationSettings, NurbsSurfaceAdaptiveTessellation,
};
use geo_algorithms::{NurbsSurface3D, Point3D, TriangleMesh3D};
use geo_contracts::default_kernel_numerical_zero_tolerance;

use crate::solver::CamSolverError;

/// 離散化の反復上限
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TessellationLimits {
    /// 初期適応分割の最大深さ
    pub max_subdivisions: u32,
    /// 弦誤差検証後の再分割反復回数の上限
    pub max_refinement_iterations: u32,
    /// 1 曲面あたりの最大頂点数
    pub max_vertices_per_surface: usize,
}

impl Default for TessellationLimits {
    fn default() -> Self {
        Self {
            max_subdivisions: 12,
            max_refinement_iterations: 6,
            max_vertices_per_surface: 1_000_000,
        }
    }
}

/// 三角形のアスペクト比 = 最長辺 / (2√3 × 内接円半径)
///
/// 正三角形で 1、細長いほど大きい。内接円半径が数値的にゼロの三角形は `f64::INFINITY`。
///
/// 面積は外積から求める（Heron の公式は細長三角形で桁落ちするため用いない）。
pub fn triangle_aspect_ratio(a: Point3D<f64>, b: Point3D<f64>, c: Point3D<f64>) -> f64 {
    let (la, lb, lc) = (distance(b, c), distance(c, a), distance(a, b));
    let semi_perimeter = 0.5 * (la + lb + lc);

    let ab = [b.x() - a.x(), b.y() - a.y(), b.z() - a.z()];
    let ac = [c.x() - a.x(), c.y() - a.y(), c.z() - a.z()];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();

    // 退化判定は面積（長さの二乗の次元）ではなく内接円半径（長さの次元）を、固定の長さ閾値
    // （カーネルのゼロ判定トレランス）と比較する。寸法の二乗で効く面積比較より寸法依存を抑えるが、
    // 閾値自体は固定長のため完全なスケール非依存ではない
    // （3 点が一致すると 0 / 0 で NaN になるため、それも退化として扱う）
    let inradius = area / semi_perimeter;
    if inradius.is_nan() || inradius <= default_kernel_numerical_zero_tolerance::<f64>() {
        return f64::INFINITY;
    }
    la.max(lb).max(lc) / (2.0 * 3.0_f64.sqrt() * inradius)
}

/// NURBS 曲面集合を許容弦誤差内の三角形メッシュへ離散化する。
pub fn tessellate_surfaces(
    surfaces: &[NurbsSurface3D<f64>],
    chord_tolerance: f64,
    limits: TessellationLimits,
) -> Result<TriangleMesh3D<f64>, CamSolverError> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (surface_index, surface) in surfaces.iter().enumerate() {
        let (u_params, v_params) =
            converge_param_grid(surface, chord_tolerance, limits).map_err(|reason| {
                CamSolverError::ConvergenceFailure(format!("surface[{surface_index}]: {reason}"))
            })?;

        let base = vertices.len();
        let v_count = v_params.len();
        for &u in &u_params {
            for &v in &v_params {
                vertices.push(evaluate(surface, u, v));
            }
        }
        for i in 0..u_params.len() - 1 {
            for j in 0..v_count - 1 {
                let p00 = base + i * v_count + j;
                let p01 = p00 + 1;
                let p10 = p00 + v_count;
                let p11 = p10 + 1;
                if uses_main_diagonal(vertices[p00], vertices[p10], vertices[p01], vertices[p11]) {
                    indices.push([p00, p10, p11]);
                    indices.push([p00, p11, p01]);
                } else {
                    indices.push([p00, p10, p01]);
                    indices.push([p10, p11, p01]);
                }
            }
        }
    }

    TriangleMesh3D::new(vertices, indices).map_err(CamSolverError::InvalidInput)
}

fn converge_param_grid(
    surface: &NurbsSurface3D<f64>,
    chord_tolerance: f64,
    limits: TessellationLimits,
) -> Result<(Vec<f64>, Vec<f64>), String> {
    converge_param_grid_counted(surface, chord_tolerance, limits).map_err(|failure| {
        format!(
            "{} (after {} refinements)",
            failure.reason, failure.refinements
        )
    })
}

/// 収束失敗の理由と、失敗までに行った再分割回数
#[derive(Debug)]
struct ConvergenceFailure {
    reason: String,
    refinements: u32,
}

/// 弦誤差検証と再分割を繰り返す。
///
/// 再分割は最大 `max_refinement_iterations` 回で、各再分割後の格子を必ず検証する。
/// 上限回数の再分割後も許容値を満たさなければ失敗とし、それ以上は再分割しない。
fn converge_param_grid_counted(
    surface: &NurbsSurface3D<f64>,
    chord_tolerance: f64,
    limits: TessellationLimits,
) -> Result<(Vec<f64>, Vec<f64>), ConvergenceFailure> {
    let settings = AdaptiveTessellationSettings {
        chord_error: chord_tolerance,
        max_subdivisions: limits.max_subdivisions,
        min_segments: 2,
        min_param_span: 0.0,
    };
    let grid = surface.adaptive_params_surface(&settings);
    let mut u_params = grid.u_params;
    let mut v_params = grid.v_params;

    let mut refinements = 0;
    loop {
        if u_params.len() * v_params.len() > limits.max_vertices_per_surface {
            return Err(ConvergenceFailure {
                reason: format!(
                    "vertex budget exceeded before reaching chord tolerance {chord_tolerance}"
                ),
                refinements,
            });
        }

        let (u_split, v_split) =
            find_cells_over_tolerance(surface, &u_params, &v_params, chord_tolerance);
        if u_split.is_empty() && v_split.is_empty() {
            return Ok((u_params, v_params));
        }

        if refinements >= limits.max_refinement_iterations {
            return Err(ConvergenceFailure {
                reason: format!(
                    "chord error did not converge to {chord_tolerance} within {} refinement iterations",
                    limits.max_refinement_iterations
                ),
                refinements,
            });
        }

        u_params = insert_midpoints(&u_params, &u_split);
        v_params = insert_midpoints(&v_params, &v_split);
        refinements += 1;
    }
}

/// 弦誤差が許容値を超えるセルの u/v 区間インデックスを返す。
fn find_cells_over_tolerance(
    surface: &NurbsSurface3D<f64>,
    u_params: &[f64],
    v_params: &[f64],
    tolerance: f64,
) -> (Vec<bool>, Vec<bool>) {
    let mut u_split = vec![false; u_params.len() - 1];
    let mut v_split = vec![false; v_params.len() - 1];
    let mut any = false;

    for i in 0..u_params.len() - 1 {
        let (u0, u1) = (u_params[i], u_params[i + 1]);
        let um = 0.5 * (u0 + u1);
        for j in 0..v_params.len() - 1 {
            let (v0, v1) = (v_params[j], v_params[j + 1]);
            let vm = 0.5 * (v0 + v1);
            let p00 = evaluate(surface, u0, v0);
            let p10 = evaluate(surface, u1, v0);
            let p01 = evaluate(surface, u0, v1);
            let p11 = evaluate(surface, u1, v1);

            let u_edge_error = distance(evaluate(surface, um, v0), midpoint(p00, p10));
            let v_edge_error = distance(evaluate(surface, u0, vm), midpoint(p00, p01));
            // セル中心は三角形分割に用いる（短い方の）対角線上で比較する
            let diagonal_midpoint = if uses_main_diagonal(p00, p10, p01, p11) {
                midpoint(p00, p11)
            } else {
                midpoint(p10, p01)
            };
            let center_error = distance(evaluate(surface, um, vm), diagonal_midpoint);

            if u_edge_error > tolerance || center_error > tolerance {
                u_split[i] = true;
                any = true;
            }
            if v_edge_error > tolerance || center_error > tolerance {
                v_split[j] = true;
                any = true;
            }
        }
    }

    if !any {
        return (Vec::new(), Vec::new());
    }
    (u_split, v_split)
}

fn insert_midpoints(params: &[f64], split: &[bool]) -> Vec<f64> {
    if split.is_empty() {
        return params.to_vec();
    }
    let mut refined = Vec::with_capacity(params.len() * 2);
    for i in 0..params.len() - 1 {
        refined.push(params[i]);
        if split[i] {
            refined.push(0.5 * (params[i] + params[i + 1]));
        }
    }
    refined.push(params[params.len() - 1]);
    refined
}

/// セル（隅 p00, p10, p01, p11）を短い方の対角線で分割するとき、主対角線 p00-p11 を使うか
fn uses_main_diagonal(
    p00: Point3D<f64>,
    p10: Point3D<f64>,
    p01: Point3D<f64>,
    p11: Point3D<f64>,
) -> bool {
    distance_squared(p00, p11) <= distance_squared(p10, p01)
}

fn evaluate(surface: &NurbsSurface3D<f64>, u: f64, v: f64) -> Point3D<f64> {
    let p = surface.evaluate_at(u, v);
    Point3D::new(p.x(), p.y(), p.z())
}

fn midpoint(a: Point3D<f64>, b: Point3D<f64>) -> Point3D<f64> {
    Point3D::new(
        0.5 * (a.x() + b.x()),
        0.5 * (a.y() + b.y()),
        0.5 * (a.z() + b.z()),
    )
}

fn distance(a: Point3D<f64>, b: Point3D<f64>) -> f64 {
    distance_squared(a, b).sqrt()
}

fn distance_squared(a: Point3D<f64>, b: Point3D<f64>) -> f64 {
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    let dz = a.z() - b.z();
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use geo_contracts::NurbsSurface3DConstructor;

    use super::*;

    const CHORD_TOLERANCE: f64 = 0.001;

    fn dome() -> NurbsSurface3D<f64> {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        NurbsSurface3D::new(
            vec![
                vec![(0.0, 0.0, 80.0), (0.0, 20.0, 80.0), (0.0, 40.0, 80.0)],
                vec![(20.0, 0.0, 80.0), (20.0, 20.0, 100.0), (20.0, 40.0, 80.0)],
                vec![(40.0, 0.0, 80.0), (40.0, 20.0, 80.0), (40.0, 40.0, 80.0)],
            ],
            None,
            knots.clone(),
            knots,
            2,
            2,
        )
        .unwrap()
    }

    /// 初期分割を浅くし、収束に再分割が必要な条件にする
    fn limits(max_refinement_iterations: u32) -> TessellationLimits {
        TessellationLimits {
            max_subdivisions: 1,
            max_refinement_iterations,
            max_vertices_per_surface: 1_000_000,
        }
    }

    /// 制限なしで収束に必要な再分割回数
    fn required_refinements() -> u32 {
        let required = (0..64)
            .find(|&max| converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(max)).is_ok())
            .expect("dome should converge within 64 refinements");
        assert!(required >= 2, "test needs a case requiring refinement");
        required
    }

    #[test]
    fn converges_when_limit_equals_required_refinements() {
        let required = required_refinements();
        assert!(converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(required)).is_ok());
    }

    #[test]
    fn fails_without_exceeding_limit_when_one_short() {
        let required = required_refinements();
        let failure = converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(required - 1))
            .unwrap_err();
        // 上限回数ちょうどで打ち切り、上限を超える再分割は行わない
        assert_eq!(failure.refinements, required - 1);
    }

    #[test]
    fn zero_limit_verifies_initial_grid_only() {
        let failure = converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(0)).unwrap_err();
        assert_eq!(failure.refinements, 0);
    }

    #[test]
    fn tessellate_surfaces_reports_convergence_failure_at_boundary() {
        let required = required_refinements();
        assert!(tessellate_surfaces(&[dome()], CHORD_TOLERANCE, limits(required)).is_ok());
        assert_eq!(
            tessellate_surfaces(&[dome()], CHORD_TOLERANCE, limits(required - 1))
                .unwrap_err()
                .code(),
            "convergence_failure"
        );
    }
}
