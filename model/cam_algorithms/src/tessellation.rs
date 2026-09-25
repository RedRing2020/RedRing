//! NURBS 曲面集合の三角形離散化（逆オフセットの前処理）
//!
//! 適応パラメータ分割で初期グリッドを作り、各セルの中心・辺中点で弦誤差を検証する。
//! 許容弦誤差を満たさないセルはパラメータ区間を二分して再検証し、
//! 反復上限内に収束しなければ `convergence_failure` を返す。

use geo_algorithms::adaptive_tessellation::{
    AdaptiveTessellationSettings, NurbsSurfaceAdaptiveTessellation,
};
use geo_algorithms::{NurbsSurface3D, Point3D, TriangleMesh3D};

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
                indices.push([p00, p10, p11]);
                indices.push([p00, p11, p01]);
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
    let settings = AdaptiveTessellationSettings {
        chord_error: chord_tolerance,
        max_subdivisions: limits.max_subdivisions,
        min_segments: 2,
        min_param_span: 0.0,
    };
    let grid = surface.adaptive_params_surface(&settings);
    let mut u_params = grid.u_params;
    let mut v_params = grid.v_params;

    for _ in 0..=limits.max_refinement_iterations {
        if u_params.len() * v_params.len() > limits.max_vertices_per_surface {
            return Err(format!(
                "vertex budget exceeded before reaching chord tolerance {chord_tolerance}"
            ));
        }

        let (u_split, v_split) =
            find_cells_over_tolerance(surface, &u_params, &v_params, chord_tolerance);
        if u_split.is_empty() && v_split.is_empty() {
            return Ok((u_params, v_params));
        }
        u_params = insert_midpoints(&u_params, &u_split);
        v_params = insert_midpoints(&v_params, &v_split);
    }

    Err(format!(
        "chord error did not converge to {chord_tolerance} within {} refinement iterations",
        limits.max_refinement_iterations
    ))
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
            // セル中心は三角形分割の対角線（p00-p11）上で比較する
            let center_error = distance(evaluate(surface, um, vm), midpoint(p00, p11));

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
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    let dz = a.z() - b.z();
    (dx * dx + dy * dy + dz * dz).sqrt()
}
