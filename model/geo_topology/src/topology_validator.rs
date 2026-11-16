//! B-rep の表の確かめ（稜線と頂点の整合・ループのつながり・多様体でない稜線・面の向きの規則）

use crate::brep::use_parameters;
use crate::{
    Brep, EdgeIndex, FaceIndex, LoopIndex, Point2D, SurfaceRef, TopologyToleranceSettings,
};
use geo_contracts::{ConicalSurface3DProperties, Scalar};

/// ループの向きを求めるときに、稜線の使用 1 つから取る点の数
const SAMPLES_PER_USE: usize = 16;

/// B-rep の表の確かめの結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrepValidationReport {
    /// 区間の端の点が、始点・終点の頂点から許容誤差より離れた稜線
    pub edges_off_vertices: Vec<EdgeIndex>,
    /// 稜線の使用が、前の使用の終わりの頂点から始まらない（閉じていない）ループ
    pub disconnected_loops: Vec<LoopIndex>,
    /// 3 回以上使う稜線（多様体でない）
    pub non_manifold_edges: Vec<EdgeIndex>,
    /// 2 回使う稜線のうち、2 つの使用が同じ向きのもの（隣り合う面の向きがそろっていない）
    pub inconsistently_oriented_edges: Vec<EdgeIndex>,
    /// ループがないのに、曲面が閉じていない面
    pub faces_without_boundary: Vec<FaceIndex>,
    /// ループの向きが、面の外向きの法線の側から見て面がループの左にある規則に合わない面（ループの巻き方を
    /// 決められない面を含む）
    pub misoriented_faces: Vec<FaceIndex>,
}

impl BrepValidationReport {
    pub fn is_valid(&self) -> bool {
        self.edges_off_vertices.is_empty()
            && self.disconnected_loops.is_empty()
            && self.non_manifold_edges.is_empty()
            && self.inconsistently_oriented_edges.is_empty()
            && self.faces_without_boundary.is_empty()
            && self.misoriented_faces.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologyValidator<T: Scalar> {
    tolerances: TopologyToleranceSettings<T>,
}

impl<T: Scalar> TopologyValidator<T> {
    pub fn new(tolerances: TopologyToleranceSettings<T>) -> Self {
        Self { tolerances }
    }

    pub fn tolerances(&self) -> TopologyToleranceSettings<T> {
        self.tolerances
    }

    /// B-rep の表の全体を確かめる
    pub fn validate(&self, brep: &Brep<T>) -> BrepValidationReport {
        let tolerance = self.tolerances.distance_tolerance;
        let edges = || (0..brep.edge_count()).map(EdgeIndex);
        let edges_off_vertices = edges()
            .filter(|&index| {
                let edge = brep.edge(index);
                let curve = edge.curve();
                let (t0, t1) = edge.parameter_range();
                let off = |vertex, t| {
                    brep.vertex(vertex)
                        .point()
                        .distance_to(curve.point_at_parameter(t))
                        > tolerance
                };
                off(edge.start(), t0) || off(edge.end(), t1)
            })
            .collect();
        let disconnected_loops = (0..brep.loop_count())
            .map(LoopIndex)
            .filter(|&loop_index| {
                let uses = brep.face_loop(loop_index).uses();
                (0..uses.len()).any(|position| {
                    let next = uses[(position + 1) % uses.len()];
                    brep.use_end(uses[position]) != brep.use_start(next)
                })
            })
            .collect();
        let non_manifold_edges = edges()
            .filter(|&edge| brep.edge(edge).uses().len() > 2)
            .collect();
        let inconsistently_oriented_edges = edges()
            .filter(|&edge| {
                let uses = brep.edge(edge).uses();
                uses.len() == 2 && {
                    let forward = |position: usize| {
                        let location = uses[position];
                        brep.face_loop(location.loop_index).uses()[location.position].forward
                    };
                    forward(0) == forward(1)
                }
            })
            .collect();
        let faces = || (0..brep.face_count()).map(FaceIndex);
        let faces_without_boundary = faces()
            .filter(|&face| {
                let face = brep.face(face);
                face.loops().is_empty() && !face.surface().is_closed()
            })
            .collect();
        let misoriented_faces = faces()
            .filter(|&face| !is_face_oriented(brep, face))
            .collect();

        BrepValidationReport {
            edges_off_vertices,
            disconnected_loops,
            non_manifold_edges,
            inconsistently_oriented_edges,
            faces_without_boundary,
            misoriented_faces,
        }
    }
}

/// ループを (u, v) に写したもの（向きは、面の外向きの法線が曲面の自然な法線と逆なら、逆にしてある）
enum LoopInUv<T: Scalar> {
    /// 周期をまわらないループ（`area` は符号つきの面積。反時計回りが正）
    Plain { area: T },
    /// 周期をまわるループ（`turns_u`・`turns_v` は u・v の周期をまわる回数、`mean_v` は v の平均）
    Winding { turns_u: T, turns_v: T, mean_v: T },
}

/// 面のループが、面の外向きの法線の側から見て面がループの左にある規則に合うか
///
/// (u, v) では、曲面の自然な法線の側から見て、面がループの左にあることは、周期をまわらない外側のループが
/// 反時計回り・内側のループが時計回りになることに当たる。周期をまわるループは、面の側を巻く向きで確かめる。
fn is_face_oriented<T: Scalar>(brep: &Brep<T>, index: FaceIndex) -> bool {
    let face = brep.face(index);
    let surface = face.surface();
    let mut plain_areas = Vec::new();
    let mut windings = Vec::new();
    for &loop_index in face.loops() {
        match loop_in_uv(brep, loop_index, surface, face.same_sense()) {
            Some(LoopInUv::Plain { area }) => plain_areas.push(area),
            Some(LoopInUv::Winding {
                turns_u,
                turns_v,
                mean_v,
            }) => {
                // 1 回より多くまわる、または u・v の両方でまわるループは扱わない
                if turns_u.abs() > T::ONE
                    || turns_v.abs() > T::ONE
                    || (turns_u != T::ZERO && turns_v != T::ZERO)
                {
                    return false;
                }
                windings.push((turns_u, turns_v, mean_v));
            }
            None => return false,
        }
    }

    if windings.is_empty() && surface.is_closed() {
        // 閉じた曲面では、ループで囲んだ部分を除いた残りも面になる（どのループも穴でよい）。外側のループは
        // 1 つまで
        return plain_areas.iter().filter(|&&area| area > T::ZERO).count() <= 1;
    }
    if windings.is_empty() {
        // 面積の最も大きいループが外側（反時計回り）、ほかは穴（時計回り）
        let Some(outer) = (0..plain_areas.len()).max_by(|&a, &b| {
            plain_areas[a]
                .abs()
                .partial_cmp(&plain_areas[b].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            return true;
        };
        return plain_areas
            .iter()
            .enumerate()
            .all(|(position, &area)| (position == outer) == (area > T::ZERO));
    }
    // 周期をまわるループがあれば、周期をまわらないループはどれも穴
    if plain_areas.iter().any(|&area| area >= T::ZERO) {
        return false;
    }
    match surface {
        // u・v の両方に周期があり、どちらの側も有限の面になる。まわる向きが打ち消し合えばよい
        SurfaceRef::Torus(_) => {
            let sum_u = windings
                .iter()
                .fold(T::ZERO, |sum, winding| sum + winding.0);
            let sum_v = windings
                .iter()
                .fold(T::ZERO, |sum, winding| sum + winding.1);
            sum_u == T::ZERO && sum_v == T::ZERO
        }
        SurfaceRef::Cylinder(_) | SurfaceRef::Cone(_) | SurfaceRef::Sphere(_) => {
            match windings.as_slice() {
                // 帯: u の正の向きにまわるループ（面はその上）が、負の向きにまわるループより下
                [first, second] => {
                    let (lower, upper) = if first.0 > T::ZERO {
                        (first, second)
                    } else {
                        (second, first)
                    };
                    lower.0 == T::ONE && upper.0 == -T::ONE && lower.2 < upper.2
                }
                // 極までの面: 面は、ループから見て極の側（球はどちらの極の側でもよい）
                [single] => match surface {
                    SurfaceRef::Sphere(_) => true,
                    SurfaceRef::Cone(cone) => surface
                        .uv_of(cone.apex())
                        .is_some_and(|(_, apex_v)| single.0 * (apex_v - single.2) > T::ZERO),
                    _ => false,
                },
                _ => false,
            }
        }
        SurfaceRef::Plane(_) | SurfaceRef::Nurbs(_) => false,
    }
}

/// ループを、パラメータ曲線に沿って取った点で (u, v) に写す（パラメータ曲線を評価できなければ `None`）
fn loop_in_uv<T: Scalar>(
    brep: &Brep<T>,
    index: LoopIndex,
    surface: &SurfaceRef<T>,
    same_sense: bool,
) -> Option<LoopInUv<T>> {
    let face_loop = brep.face_loop(index);
    let mut points: Vec<Point2D<T>> = Vec::with_capacity(face_loop.uses().len() * SAMPLES_PER_USE);
    // 稜線の使用に沿って進んだ u・v の和（極の線に沿って移る分と、周期の分の移しは含めない）
    let (mut progress_u, mut progress_v) = (T::ZERO, T::ZERO);
    for (edge_use, curve) in face_loop.uses().iter().zip(face_loop.parameter_curves()) {
        let (t_start, t_end) = use_parameters(brep.edge(edge_use.edge), *edge_use);
        for sample in 0..SAMPLES_PER_USE {
            let t =
                t_start + (t_end - t_start) * T::from_f64(sample as f64 / SAMPLES_PER_USE as f64);
            points.push(curve.point_at(t)?);
        }
        let (start, end) = (curve.point_at(t_start)?, curve.point_at(t_end)?);
        progress_u += end.x() - start.x();
        progress_v += end.y() - start.y();
    }
    let sense = if same_sense { T::ONE } else { -T::ONE };
    let (u_period, v_period) = surface.periods();
    let turns =
        |delta: T, period: Option<T>| period.map_or(T::ZERO, |period| (delta / period).round());
    let turns_u = turns(progress_u, u_period);
    let turns_v = turns(progress_v, v_period);
    if turns_u == T::ZERO && turns_v == T::ZERO {
        let area = (0..points.len()).fold(T::ZERO, |sum, position| {
            let a = points[position];
            let b = points[(position + 1) % points.len()];
            sum + (a.x() * b.y() - b.x() * a.y())
        }) / T::from_f64(2.0);
        Some(LoopInUv::Plain { area: area * sense })
    } else {
        let mean_v = points.iter().fold(T::ZERO, |sum, point| sum + point.y())
            / T::from_f64(points.len() as f64);
        Some(LoopInUv::Winding {
            turns_u: turns_u * sense,
            turns_v: turns_v * sense,
            mean_v,
        })
    }
}
