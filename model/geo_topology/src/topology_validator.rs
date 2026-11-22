//! B-rep の表の確かめ（稜線と頂点の整合・ループのつながり・多様体でない稜線・面の向きの規則）

use crate::brep::{shift_to_side, use_parameters};
use crate::{Brep, EdgeIndex, FaceIndex, LoopIndex, Point2D, SurfaceRef};
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

impl<T: Scalar> Brep<T> {
    /// B-rep の表の全体を確かめる（稜線の端の点は、頂点の許容誤差で確かめる。パラメータ曲線と面の曲面は、
    /// 面を足すときに稜線の許容誤差で確かめてある）
    pub fn validate(&self) -> BrepValidationReport {
        let edges = || (0..self.edge_count()).map(EdgeIndex);
        let edges_off_vertices = edges()
            .filter(|&index| {
                let edge = self.edge(index);
                let curve = edge.curve();
                let (t0, t1) = edge.parameter_range();
                // 頂点の許容誤差は、稜線の許容誤差以上
                let off = |vertex, t| {
                    let vertex = self.vertex(vertex);
                    vertex.point().distance_to(curve.point_at_parameter(t)) > vertex.tolerance()
                };
                off(edge.start(), t0) || off(edge.end(), t1)
            })
            .collect();
        let disconnected_loops = (0..self.loop_count())
            .map(LoopIndex)
            .filter(|&loop_index| {
                let uses = self.face_loop(loop_index).uses();
                (0..uses.len()).any(|position| {
                    let next = uses[(position + 1) % uses.len()];
                    self.use_end(uses[position]) != self.use_start(next)
                })
            })
            .collect();
        let non_manifold_edges = edges()
            .filter(|&edge| self.edge(edge).uses().len() > 2)
            .collect();
        let inconsistently_oriented_edges = edges()
            .filter(|&edge| {
                let uses = self.edge(edge).uses();
                uses.len() == 2 && {
                    let forward = |position: usize| {
                        let location = uses[position];
                        self.face_loop(location.loop_index).uses()[location.position].forward
                    };
                    forward(0) == forward(1)
                }
            })
            .collect();
        let faces = || (0..self.face_count()).map(FaceIndex);
        let faces_without_boundary = faces()
            .filter(|&face| {
                let face = self.face(face);
                face.loops().is_empty() && !face.surface().is_closed()
            })
            .collect();
        let misoriented_faces = faces()
            .filter(|&face| !self.is_face_oriented(face))
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

impl<T: Scalar> Brep<T> {
    /// 面 `index` のループが、面の外向きの法線の側から見て面がループの左にある規則に合うか（ループの巻き方を
    /// 決められない面は、合わないとする）
    ///
    /// (u, v) では、曲面の自然な法線の側から見て、面がループの左にあることは、周期をまわらない外側のループが
    /// 反時計回り・内側のループが時計回りになることに当たる。周期をまわるループは、面の側を巻く向きで確かめる。
    pub fn is_face_oriented(&self, index: FaceIndex) -> bool {
        is_face_oriented(self, index)
    }

    /// 面 `index` のループの回り方が、周期的な曲面の上で扱える形か（どのループも周期を 1 回までしかまわらず、
    /// u・v の両方ではまわらず、同じ向きの周期をまわるループが 2 つまで。パラメータ曲線を評価できないループが
    /// あれば、扱えないとする）
    ///
    /// 周期をまわらないループだけの面・極までの面・帯・ループのない面がこれに当たる。面の向きの規則
    /// （`is_face_oriented`）と同じ巻き数の数え方を使う。
    pub fn has_readable_winding(&self, index: FaceIndex) -> bool {
        let face = self.face(index);
        let (mut around_u, mut around_v) = (0, 0);
        for &loop_index in face.loops() {
            match loop_in_uv(self, loop_index, face.surface(), face.same_sense()) {
                Some(LoopInUv::Plain { .. }) => {}
                Some(LoopInUv::Winding {
                    turns_u, turns_v, ..
                }) => {
                    if turns_u.abs() > T::ONE
                        || turns_v.abs() > T::ONE
                        || (turns_u != T::ZERO && turns_v != T::ZERO)
                    {
                        return false;
                    }
                    around_u += usize::from(turns_u != T::ZERO);
                    around_v += usize::from(turns_v != T::ZERO);
                }
                None => return false,
            }
        }
        around_u <= 2 && around_v <= 2
    }
}

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
///
/// 巻き数は、ループの終わりと初めの (u, v) の差（極・頂点の線に沿って移る分を含む）で数える。初めの点が
/// 極・頂点なら、閉じるときの移りも、面を作るときと同じく、面がループの左にある向きに決めて含める。
fn loop_in_uv<T: Scalar>(
    brep: &Brep<T>,
    index: LoopIndex,
    surface: &SurfaceRef<T>,
    same_sense: bool,
) -> Option<LoopInUv<T>> {
    let face_loop = brep.face_loop(index);
    let mut points: Vec<Point2D<T>> = Vec::with_capacity(face_loop.uses().len() * SAMPLES_PER_USE);
    // 最後の使用の終わりの (u, v) と、中ほどの v・稜線の許容誤差
    let mut last = None;
    for (edge_use, curve) in face_loop.uses().iter().zip(face_loop.parameter_curves()) {
        let (t_start, t_end) = use_parameters(brep.edge(edge_use.edge), *edge_use);
        for sample in 0..SAMPLES_PER_USE {
            let t =
                t_start + (t_end - t_start) * T::from_f64(sample as f64 / SAMPLES_PER_USE as f64);
            points.push(curve.point_at(t)?);
        }
        let middle = curve.point_at((t_start + t_end) / T::from_f64(2.0))?;
        last = Some((
            curve.point_at(t_end)?,
            middle.y(),
            brep.edge(edge_use.edge).tolerance(),
        ));
    }
    let (first, (end, last_middle_v, last_tolerance)) = (*points.first()?, last?);
    let sense = if same_sense { T::ONE } else { -T::ONE };
    let (u_period, v_period) = surface.periods();
    // 初めの点が極・頂点なら、閉じるときに極の線に沿って移る先の u（初めの u を、周期の分だけ動かしたもの）。
    // 面を足すときと同じく、つなぎ目の前の使用（最後の使用）の稜線の許容誤差で見分ける
    let first_point = brep
        .vertex(brep.use_start(*face_loop.uses().first()?))
        .point();
    let closing_u = match (surface.singular_v(first_point, last_tolerance), u_period) {
        (Some(pole_v), Some(period)) => {
            let decreasing = (pole_v > last_middle_v) == same_sense;
            shift_to_side(first.x(), end.x(), period, decreasing)
        }
        _ => end.x(),
    };
    let turns =
        |delta: T, period: Option<T>| period.map_or(T::ZERO, |period| (delta / period).round());
    let turns_u = turns(closing_u - first.x(), u_period);
    let turns_v = turns(end.y() - first.y(), v_period);
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
