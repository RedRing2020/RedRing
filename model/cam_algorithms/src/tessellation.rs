//! NURBS 曲面集合の三角形離散化（逆オフセットの前処理）
//!
//! 適応パラメータ分割で初期格子を作り、各セルを検証して必要な区間を再分割する。
//!
//! - 弦誤差: セル中心・辺中点で許容弦誤差を満たすか（満たさない区間は二分）
//! - 形状品質（任意）: 三角形アスペクト比と最大辺長の上限（超える方向の区間を等分）
//!
//! テンソル積格子を保つため、再分割はパラメータ区間単位で行い、T 字接続は発生しない。
//! 各セルは短い方の対角線で 2 三角形に分割する。
//! 反復上限・頂点数上限内に条件を満たせなければ `convergence_failure` を返す。

use geo_algorithms::adaptive_tessellation::{
    AdaptiveTessellationSettings, NurbsSurfaceAdaptiveTessellation,
};
use geo_algorithms::{NurbsSurface3D, Point3D, TriangleMesh3D};
use geo_contracts::default_kernel_numerical_zero_tolerance;

use crate::solver::CamSolverError;

/// 離散化の上限・品質条件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TessellationLimits {
    /// 初期適応分割の最大深さ
    pub max_subdivisions: u32,
    /// 検証後の再分割反復回数の上限
    pub max_refinement_iterations: u32,
    /// 1 曲面あたりの最大頂点数
    pub max_vertices_per_surface: usize,
    /// 三角形アスペクト比（`triangle_aspect_ratio`）の上限。`None` なら制御しない
    pub max_aspect_ratio: Option<f64>,
    /// 三角形の辺長（3D）の上限。`None` なら制御しない
    pub max_edge_length: Option<f64>,
}

impl Default for TessellationLimits {
    fn default() -> Self {
        Self {
            max_subdivisions: 12,
            max_refinement_iterations: 6,
            max_vertices_per_surface: 1_000_000,
            max_aspect_ratio: None,
            max_edge_length: None,
        }
    }
}

/// 三角形のアスペクト比 = 最長辺 / (2√3 × 内接円半径)
///
/// 正三角形で 1、細長いほど大きい。面積が数値的にゼロの三角形は `f64::INFINITY`。
pub fn triangle_aspect_ratio(a: Point3D<f64>, b: Point3D<f64>, c: Point3D<f64>) -> f64 {
    let (la, lb, lc) = (distance(b, c), distance(c, a), distance(a, b));
    let semi_perimeter = 0.5 * (la + lb + lc);
    let area_squared =
        semi_perimeter * (semi_perimeter - la) * (semi_perimeter - lb) * (semi_perimeter - lc);
    let area = area_squared.max(0.0).sqrt();
    if area <= default_kernel_numerical_zero_tolerance::<f64>() {
        return f64::INFINITY;
    }
    let inradius = area / semi_perimeter;
    la.max(lb).max(lc) / (2.0 * 3.0_f64.sqrt() * inradius)
}

/// NURBS 曲面集合を離散化条件を満たす三角形メッシュへ離散化する。
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
                let cell = Cell::new(vertices[p00], vertices[p10], vertices[p01], vertices[p11]);
                if cell.uses_main_diagonal() {
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

/// 検証と再分割を繰り返す。
///
/// 再分割は最大 `max_refinement_iterations` 回で、各再分割後の格子を必ず検証する。
/// 上限回数の再分割後も条件を満たさなければ失敗とし、それ以上は再分割しない。
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
                    "vertex budget {} exceeded before satisfying chord tolerance {chord_tolerance} and quality limits",
                    limits.max_vertices_per_surface
                ),
                refinements,
            });
        }

        let inspection = inspect_cells(surface, &u_params, &v_params, chord_tolerance, &limits);
        if !inspection.needs_refinement() {
            return match inspection.unfixable_aspect_ratio {
                Some(ratio) => Err(ConvergenceFailure {
                    reason: format!(
                        "triangle aspect ratio {ratio:.2} exceeds {:?} due to parameterization shear",
                        limits.max_aspect_ratio
                    ),
                    refinements,
                }),
                None => Ok((u_params, v_params)),
            };
        }

        if refinements >= limits.max_refinement_iterations {
            return Err(ConvergenceFailure {
                reason: format!(
                    "chord tolerance {chord_tolerance} and quality limits were not satisfied within {} refinement iterations",
                    limits.max_refinement_iterations
                ),
                refinements,
            });
        }

        u_params = subdivide(&u_params, &inspection.u_factors);
        v_params = subdivide(&v_params, &inspection.v_factors);
        refinements += 1;
    }
}

/// 格子セル（隅 p00, p10, p01, p11）
struct Cell {
    p00: Point3D<f64>,
    p10: Point3D<f64>,
    p01: Point3D<f64>,
    p11: Point3D<f64>,
}

impl Cell {
    fn new(p00: Point3D<f64>, p10: Point3D<f64>, p01: Point3D<f64>, p11: Point3D<f64>) -> Self {
        Self { p00, p10, p01, p11 }
    }

    /// 短い方の対角線で分割する（主対角線 p00-p11 が短いか同じなら true）
    fn uses_main_diagonal(&self) -> bool {
        distance(self.p00, self.p11) <= distance(self.p10, self.p01)
    }

    /// 採用する対角線の中点
    fn diagonal_midpoint(&self) -> Point3D<f64> {
        if self.uses_main_diagonal() {
            midpoint(self.p00, self.p11)
        } else {
            midpoint(self.p10, self.p01)
        }
    }

    /// 採用する対角線の長さ
    fn diagonal_length(&self) -> f64 {
        distance(self.p00, self.p11).min(distance(self.p10, self.p01))
    }

    /// u 方向・v 方向の辺長（3D、対辺の長い方）
    fn side_lengths(&self) -> (f64, f64) {
        let u_length = distance(self.p00, self.p10).max(distance(self.p01, self.p11));
        let v_length = distance(self.p00, self.p01).max(distance(self.p10, self.p11));
        (u_length, v_length)
    }

    /// 分割後 2 三角形のアスペクト比の最大値
    fn max_triangle_aspect_ratio(&self) -> f64 {
        let (a, b) = if self.uses_main_diagonal() {
            (
                triangle_aspect_ratio(self.p00, self.p10, self.p11),
                triangle_aspect_ratio(self.p00, self.p11, self.p01),
            )
        } else {
            (
                triangle_aspect_ratio(self.p00, self.p10, self.p01),
                triangle_aspect_ratio(self.p10, self.p11, self.p01),
            )
        };
        a.max(b)
    }
}

/// セル検証の結果（区間ごとの分割数）
struct CellInspection {
    /// u 区間ごとの分割数（1 なら分割しない）
    u_factors: Vec<usize>,
    /// v 区間ごとの分割数（1 なら分割しない）
    v_factors: Vec<usize>,
    /// 辺長比が均衡していても上限を超える（せん断による）アスペクト比の最大値
    unfixable_aspect_ratio: Option<f64>,
}

impl CellInspection {
    fn needs_refinement(&self) -> bool {
        self.u_factors
            .iter()
            .chain(self.v_factors.iter())
            .any(|&factor| factor > 1)
    }
}

/// 各セルを検証し、弦誤差・品質条件を満たすための区間分割数を求める。
fn inspect_cells(
    surface: &NurbsSurface3D<f64>,
    u_params: &[f64],
    v_params: &[f64],
    chord_tolerance: f64,
    limits: &TessellationLimits,
) -> CellInspection {
    let mut inspection = CellInspection {
        u_factors: vec![1; u_params.len() - 1],
        v_factors: vec![1; v_params.len() - 1],
        unfixable_aspect_ratio: None,
    };

    for i in 0..u_params.len() - 1 {
        let (u0, u1) = (u_params[i], u_params[i + 1]);
        let um = 0.5 * (u0 + u1);
        for j in 0..v_params.len() - 1 {
            let (v0, v1) = (v_params[j], v_params[j + 1]);
            let vm = 0.5 * (v0 + v1);
            let cell = Cell::new(
                evaluate(surface, u0, v0),
                evaluate(surface, u1, v0),
                evaluate(surface, u0, v1),
                evaluate(surface, u1, v1),
            );

            // 弦誤差: 辺中点と、分割に用いる対角線の中点（セル中心）
            let u_edge_error = distance(evaluate(surface, um, v0), midpoint(cell.p00, cell.p10));
            let v_edge_error = distance(evaluate(surface, u0, vm), midpoint(cell.p00, cell.p01));
            let center_error = distance(evaluate(surface, um, vm), cell.diagonal_midpoint());
            let mut u_factor = 1;
            let mut v_factor = 1;
            if u_edge_error > chord_tolerance || center_error > chord_tolerance {
                u_factor = 2;
            }
            if v_edge_error > chord_tolerance || center_error > chord_tolerance {
                v_factor = 2;
            }

            let (u_length, v_length) = cell.side_lengths();

            // 最大辺長: 辺が上限を超える方向を等分。対角線（三角形の辺）が超える場合は長い方の辺を二分し、
            // 次の反復で再検証する
            if let Some(max_length) = limits.max_edge_length {
                u_factor = u_factor.max(division_count(u_length, max_length));
                v_factor = v_factor.max(division_count(v_length, max_length));
                if cell.diagonal_length() > max_length {
                    if u_length >= v_length {
                        u_factor = u_factor.max(2);
                    } else {
                        v_factor = v_factor.max(2);
                    }
                }
            }

            // アスペクト比: 長い方向を短い方向と同程度になるよう等分する。
            // 辺長比 2 未満の長方形セルの三角形アスペクト比は約 1.49 以下のため、辺長比 2 未満で
            // 上限を超えるのはせん断が原因であり、パラメータ区間の等分では改善しない。
            if let Some(max_ratio) = limits.max_aspect_ratio {
                let ratio = cell.max_triangle_aspect_ratio();
                if ratio > max_ratio {
                    match side_ratio_split(u_length, v_length) {
                        Some(SideSplit::U(factor)) => u_factor = u_factor.max(factor),
                        Some(SideSplit::V(factor)) => v_factor = v_factor.max(factor),
                        None => {
                            let worst = inspection.unfixable_aspect_ratio.unwrap_or(0.0);
                            inspection.unfixable_aspect_ratio = Some(worst.max(ratio));
                        }
                    }
                }
            }

            inspection.u_factors[i] = inspection.u_factors[i].max(u_factor);
            inspection.v_factors[j] = inspection.v_factors[j].max(v_factor);
        }
    }

    inspection
}

/// 辺長比を均すための分割方向と分割数
enum SideSplit {
    U(usize),
    V(usize),
}

/// 長い方の辺を短い方の辺長程度に等分する分割数（`floor(長辺 / 短辺)`）。
/// 辺長比が 2 未満、または辺長が数値的にゼロなら `None`。
fn side_ratio_split(u_length: f64, v_length: f64) -> Option<SideSplit> {
    let zero = default_kernel_numerical_zero_tolerance::<f64>();
    if u_length <= zero || v_length <= zero {
        return None;
    }
    let (ratio, is_u_longer) = if u_length >= v_length {
        (u_length / v_length, true)
    } else {
        (v_length / u_length, false)
    };
    let factor = ratio.floor();
    if !(factor.is_finite() && factor >= 2.0) {
        return None;
    }
    let factor = factor as usize;
    Some(if is_u_longer {
        SideSplit::U(factor)
    } else {
        SideSplit::V(factor)
    })
}

/// `length` を `max_length` 以下の区間に分けるのに必要な分割数（最小 1）
fn division_count(length: f64, max_length: f64) -> usize {
    let count = (length / max_length).ceil();
    if count.is_finite() && count > 1.0 {
        count as usize
    } else {
        1
    }
}

/// 各区間をパラメータ空間で `factors[i]` 等分する。
fn subdivide(params: &[f64], factors: &[usize]) -> Vec<f64> {
    let extra: usize = factors.iter().map(|&factor| factor - 1).sum();
    let mut refined = Vec::with_capacity(params.len() + extra);
    for (i, &factor) in factors.iter().enumerate() {
        let (start, end) = (params[i], params[i + 1]);
        for k in 0..factor {
            refined.push(start + (end - start) * k as f64 / factor as f64);
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
            ..TessellationLimits::default()
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
