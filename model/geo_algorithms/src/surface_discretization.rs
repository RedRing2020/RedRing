//! NURBS 曲面集合の三角形への離散化
//!
//! 適応パラメータ分割で初期グリッドを作り、各セルの中心・辺中点で弦誤差を検証する。
//! 許容弦誤差を満たさないセルはパラメータ区間を二分して再検証し、
//! 反復上限内に収束しなければ失敗を返す。
//!
//! 各セルは短い方の対角線で 2 三角形に分割する。三角形の形状品質（アスペクト比）は制御しない。
//! 頂点法線は曲面の法線とし、法線が定まらない点（退化した点）では、その頂点を含む三角形の法線の平均とする。

use std::fmt;

use geo_contracts::{NurbsSurface3DEvaluation, TriangleMesh3DConstructor, TriangleMeshError};

use crate::{
    AdaptiveTessellationSettings, NurbsSurface3D, NurbsSurfaceAdaptiveTessellation, Point3D,
    TriangleMesh3D, Vector3D,
};

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

/// 離散化の失敗
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceTessellationError {
    /// 曲面 `surface_index` の弦誤差が反復上限内に許容値へ収束しない（理由）
    ConvergenceFailure {
        surface_index: usize,
        reason: String,
    },
    /// 三角形メッシュを作れない
    InvalidMesh(TriangleMeshError),
}

impl fmt::Display for SurfaceTessellationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConvergenceFailure {
                surface_index,
                reason,
            } => write!(f, "surface[{surface_index}]: {reason}"),
            Self::InvalidMesh(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for SurfaceTessellationError {}

/// NURBS 曲面集合を許容弦誤差内の三角形メッシュ（頂点法線つき）へ離散化する。
///
/// # Errors
///
/// 曲面の弦誤差が反復上限内に収束しない場合、三角形メッシュを作れない場合。
pub fn tessellate_surfaces(
    surfaces: &[NurbsSurface3D<f64>],
    chord_tolerance: f64,
    limits: TessellationLimits,
) -> Result<TriangleMesh3D<f64>, SurfaceTessellationError> {
    let mut vertices = Vec::new();
    let mut normals: Vec<Option<Vector3D<f64>>> = Vec::new();
    let mut indices = Vec::new();

    for (surface_index, surface) in surfaces.iter().enumerate() {
        let (u_params, v_params) =
            converge_param_grid(surface, chord_tolerance, limits).map_err(|reason| {
                SurfaceTessellationError::ConvergenceFailure {
                    surface_index,
                    reason,
                }
            })?;

        let base = vertices.len();
        let v_count = v_params.len();
        for &u in &u_params {
            for &v in &v_params {
                vertices.push(evaluate(surface, u, v));
                normals.push(surface.normal_at(u, v).map(|normal| normal.to_vector()));
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

    let normals = complete_normals(&vertices, &indices, normals);
    TriangleMesh3D::with_normals(vertices, indices, normals)
        .map_err(SurfaceTessellationError::InvalidMesh)
}

/// 法線が定まらない頂点の法線を、その頂点を含む三角形の法線の平均で補う
fn complete_normals(
    vertices: &[Point3D<f64>],
    indices: &[[usize; 3]],
    normals: Vec<Option<Vector3D<f64>>>,
) -> Vec<Vector3D<f64>> {
    let mut sums = vec![Vector3D::new(0.0, 0.0, 0.0); vertices.len()];
    if normals.iter().any(Option::is_none) {
        for &[a, b, c] in indices {
            let normal = Vector3D::from_points(vertices[a], vertices[b])
                .cross(Vector3D::from_points(vertices[a], vertices[c]));
            for index in [a, b, c] {
                sums[index] = sums[index] + normal;
            }
        }
    }
    normals
        .into_iter()
        .zip(sums)
        .map(|(normal, sum)| normal.unwrap_or_else(|| sum.normalize()))
        .collect()
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
    surface
        .point_at_uv(u, v)
        .expect("sample parameter should be within the surface domain")
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
            2,
            2,
            knots.clone(),
            knots,
            &[
                vec![
                    Point3D::new(0.0, 0.0, 80.0),
                    Point3D::new(0.0, 20.0, 80.0),
                    Point3D::new(0.0, 40.0, 80.0),
                ],
                vec![
                    Point3D::new(20.0, 0.0, 80.0),
                    Point3D::new(20.0, 20.0, 100.0),
                    Point3D::new(20.0, 40.0, 80.0),
                ],
                vec![
                    Point3D::new(40.0, 0.0, 80.0),
                    Point3D::new(40.0, 20.0, 80.0),
                    Point3D::new(40.0, 40.0, 80.0),
                ],
            ],
            None,
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
        assert!(matches!(
            tessellate_surfaces(&[dome()], CHORD_TOLERANCE, limits(required - 1)).unwrap_err(),
            SurfaceTessellationError::ConvergenceFailure {
                surface_index: 0,
                ..
            }
        ));
    }

    #[test]
    fn vertex_normals_are_the_surface_normals() {
        use geo_contracts::TriangleMesh3DProperties;

        let surface = dome();
        let mesh = tessellate_surfaces(
            std::slice::from_ref(&surface),
            0.1,
            TessellationLimits::default(),
        )
        .unwrap();
        let normals = mesh
            .normals()
            .expect("tessellated meshes have vertex normals");
        assert_eq!(normals.len(), mesh.vertex_count());
        for normal in normals {
            assert!((normal.length() - 1.0).abs() < 1e-9);
        }
        // 三角形の向き（頂点の並び）と頂点法線の向きがそろう
        for &[a, b, c] in mesh.indices() {
            let vertices = mesh.vertices();
            let face = Vector3D::from_points(vertices[a], vertices[b])
                .cross(Vector3D::from_points(vertices[a], vertices[c]));
            assert!(face.dot(normals[a]) > 0.0);
        }
        // ドームの頂上（u = v = 0.5）の法線は +Z か -Z
        let top = surface.normal_at(0.5, 0.5).unwrap().to_vector();
        assert!(top.x().abs() < 1e-9 && top.y().abs() < 1e-9);
    }
}
