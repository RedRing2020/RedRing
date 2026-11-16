//! B-rep の表の確かめ（稜線と頂点の整合・ループのつながり・多様体でない稜線・境界のない面）

use crate::{
    Brep, BrepEdge, CurveRef, EdgeIndex, FaceIndex, LoopIndex, Point3D, TopologyToleranceSettings,
};
use geo_contracts::{LineSegment3DSupportLine, Scalar};

/// 稜線 1 本の確かめの結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeValidationReport {
    /// 頂点が、母曲線の拘束の端点と合う
    pub binding_consistent: bool,
    /// 母曲線の理想の端点が、拘束の端点と合う
    pub ideal_endpoint_consistent: bool,
    /// 区間の端で評価した点が、拘束の端点と合う
    pub evaluated_endpoint_consistent: bool,
}

impl EdgeValidationReport {
    pub fn is_valid(&self) -> bool {
        self.binding_consistent
            && self.ideal_endpoint_consistent
            && self.evaluated_endpoint_consistent
    }
}

/// B-rep の表の確かめの結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrepValidationReport {
    /// 稜線ごとの結果（稜線の番号の順）
    pub edge_reports: Vec<EdgeValidationReport>,
    /// 稜線の使用が、前の使用の終わりの頂点から始まらない（閉じていない）ループ
    pub disconnected_loops: Vec<LoopIndex>,
    /// 3 回以上使う稜線（多様体でない）
    pub non_manifold_edges: Vec<EdgeIndex>,
    /// 2 回使う稜線のうち、2 つの使用が同じ向きのもの（隣り合う面の向きがそろっていない）
    pub inconsistently_oriented_edges: Vec<EdgeIndex>,
    /// ループがないのに、曲面が閉じていない面
    pub faces_without_boundary: Vec<FaceIndex>,
}

impl BrepValidationReport {
    pub fn is_valid(&self) -> bool {
        self.edge_reports.iter().all(EdgeValidationReport::is_valid)
            && self.disconnected_loops.is_empty()
            && self.non_manifold_edges.is_empty()
            && self.inconsistently_oriented_edges.is_empty()
            && self.faces_without_boundary.is_empty()
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

    /// 稜線 `index` の頂点と母曲線の整合を確かめる
    pub fn validate_edge(&self, brep: &Brep<T>, index: EdgeIndex) -> EdgeValidationReport {
        let budget = self.tolerances.resolve_edge_tolerances();
        let edge = brep.edge(index);
        let curve = edge.curve();
        let ends = EdgeEnds::of(edge);
        let (ideal_start, ideal_end) = ends.ideal;
        let (constraint_start, constraint_end) = ends.constraint;
        let close = |a: Point3D<T>, b: Point3D<T>, tolerance: T| a.distance_to(b) <= tolerance;
        let (t0, t1) = edge.parameter_range();

        EdgeValidationReport {
            binding_consistent: close(
                brep.vertex(edge.start()).point(),
                constraint_start,
                budget.bind_tolerance,
            ) && close(
                brep.vertex(edge.end()).point(),
                constraint_end,
                budget.bind_tolerance,
            ),
            ideal_endpoint_consistent: close(ideal_start, constraint_start, budget.ideal_tolerance)
                && close(ideal_end, constraint_end, budget.ideal_tolerance),
            evaluated_endpoint_consistent: close(
                curve.point_at_parameter(t0),
                constraint_start,
                budget.eval_tolerance,
            ) && close(
                curve.point_at_parameter(t1),
                constraint_end,
                budget.eval_tolerance,
            ),
        }
    }

    /// B-rep の表の全体を確かめる
    pub fn validate(&self, brep: &Brep<T>) -> BrepValidationReport {
        let edge_reports = (0..brep.edge_count())
            .map(|edge| self.validate_edge(brep, EdgeIndex(edge)))
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
        let non_manifold_edges = (0..brep.edge_count())
            .map(EdgeIndex)
            .filter(|&edge| brep.edge(edge).uses().len() > 2)
            .collect();
        let inconsistently_oriented_edges = (0..brep.edge_count())
            .map(EdgeIndex)
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
        let faces_without_boundary = (0..brep.face_count())
            .map(FaceIndex)
            .filter(|&face| {
                let face = brep.face(face);
                face.loops().is_empty() && !face.surface().is_closed()
            })
            .collect();

        BrepValidationReport {
            edge_reports,
            disconnected_loops,
            non_manifold_edges,
            inconsistently_oriented_edges,
            faces_without_boundary,
        }
    }
}

/// 稜線の端の点（理想の端点と、頂点と結びつける拘束の端点。どちらも始点と終点）
struct EdgeEnds<T: Scalar> {
    ideal: (Point3D<T>, Point3D<T>),
    constraint: (Point3D<T>, Point3D<T>),
}

impl<T: Scalar> EdgeEnds<T> {
    fn of(edge: &BrepEdge<T>) -> Self {
        let curve = edge.curve();
        let ideal = (curve.ideal_start_point(), curve.ideal_end_point());
        match curve {
            // 線分は、母線の上の理想の端点と別に、拘束の端点を持つ
            CurveRef::Line(line) => Self {
                ideal,
                constraint: (line.constraint_start_point(), line.constraint_end_point()),
            },
            // NURBS 曲線は、区間で母曲線の一部を使えるため、区間の端を端点とする
            CurveRef::Nurbs(_) => {
                let (t0, t1) = edge.parameter_range();
                let ends = (curve.point_at_parameter(t0), curve.point_at_parameter(t1));
                Self {
                    ideal: ends,
                    constraint: ends,
                }
            }
            // 円弧・楕円弧は、それ自体の端点を端点とする
            CurveRef::Arc(_) | CurveRef::EllipseArc(_) => Self {
                ideal,
                constraint: ideal,
            },
        }
    }
}
