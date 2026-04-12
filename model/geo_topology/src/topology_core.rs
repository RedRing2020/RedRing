//! topo正規形の最小構成要素
//!
//! #408 の最小導入として、Vertex/Edge/CurveRef を提供する。

use crate::tolerance::ResolvedEdgeToleranceSettings;
use crate::{Point3D, TopoArc3D, TopoEllipseArc3D, TopoLineSegment3D, TopoNurbsCurve3D};
use geo_contracts::{
    default_kernel_numerical_zero_tolerance, Arc3DEndpoint, Arc3DEvaluation, EllipseArc3DDerived,
    EllipseArc3DEndpoint, EllipseArc3DEvaluation, NurbsCurve3DProperties, Scalar,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// トポロジー要素の識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopoId(u64);

impl TopoId {
    /// 一意IDを採番する
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// 生のID値を返す
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for TopoId {
    fn default() -> Self {
        Self::new()
    }
}

/// 頂点（Vertex）
#[derive(Debug, Clone)]
pub struct Vertex<T: Scalar> {
    id: TopoId,
    point: Point3D<T>,
}

impl<T: Scalar> Vertex<T> {
    /// 座標から頂点を生成する
    pub fn new(point: Point3D<T>) -> Self {
        Self {
            id: TopoId::new(),
            point,
        }
    }

    pub fn id(&self) -> TopoId {
        self.id
    }

    pub fn point(&self) -> Point3D<T> {
        self.point
    }

    /// 他頂点と同一点かを許容誤差付きで判定する
    pub fn is_coincident(&self, other: &Self, tolerance: T) -> bool {
        self.point.distance_to(&other.point) <= tolerance
    }
}

/// Edge が参照する母曲線
#[derive(Debug, Clone)]
pub enum CurveRef<T: Scalar> {
    Line(TopoLineSegment3D<T>),
    Arc(TopoArc3D<T>),
    EllipseArc(TopoEllipseArc3D<T>),
    Nurbs(TopoNurbsCurve3D<T>),
}

impl<T: Scalar> CurveRef<T> {
    fn nurbs_parameter_tolerance() -> T {
        let scaled_epsilon = T::EPSILON * T::from_f64(16.0);
        let minimum_tolerance = default_kernel_numerical_zero_tolerance::<T>();
        if scaled_epsilon > minimum_tolerance {
            scaled_epsilon
        } else {
            minimum_tolerance
        }
    }

    pub fn ideal_start_point(&self) -> Point3D<T> {
        match self {
            Self::Line(line) => line.start(),
            Self::Arc(arc) => {
                let (x, y, z) = <TopoArc3D<T> as Arc3DEndpoint<T>>::start_point(arc);
                Point3D::new(x, y, z)
            }
            Self::EllipseArc(arc) => {
                let (x, y, z) = <TopoEllipseArc3D<T> as EllipseArc3DEndpoint<T>>::start_point(arc);
                Point3D::new(x, y, z)
            }
            Self::Nurbs(curve) => {
                let (u_min, _) =
                    <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(curve);
                let point = curve.evaluate_at(u_min);
                Point3D::new(point.x(), point.y(), point.z())
            }
        }
    }

    pub fn ideal_end_point(&self) -> Point3D<T> {
        match self {
            Self::Line(line) => line.end(),
            Self::Arc(arc) => {
                let (x, y, z) = <TopoArc3D<T> as Arc3DEndpoint<T>>::end_point(arc);
                Point3D::new(x, y, z)
            }
            Self::EllipseArc(arc) => {
                let (x, y, z) = <TopoEllipseArc3D<T> as EllipseArc3DEndpoint<T>>::end_point(arc);
                Point3D::new(x, y, z)
            }
            Self::Nurbs(curve) => {
                let (_, u_max) =
                    <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(curve);
                let point = curve.evaluate_at(u_max);
                Point3D::new(point.x(), point.y(), point.z())
            }
        }
    }

    /// 母曲線の native parameter semantics をそのまま使って評価点を返す
    ///
    /// topology 側では family 横断で parameter を再正規化しない。
    /// 現在の variant では Line / Arc / EllipseArc のいずれも bounded curve として
    /// ideal start/end と整合する parameter domain を使う。
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        match self {
            Self::Line(line) => line.line().point_at_parameter(t),
            Self::Arc(arc) => {
                let (x, y, z) = <TopoArc3D<T> as Arc3DEvaluation<T>>::point_at_parameter(arc, t);
                Point3D::new(x, y, z)
            }
            Self::EllipseArc(arc) => {
                let (x, y, z) =
                    <TopoEllipseArc3D<T> as EllipseArc3DEvaluation<T>>::point_at_parameter(arc, t);
                Point3D::new(x, y, z)
            }
            Self::Nurbs(curve) => {
                let (u_min, u_max) =
                    <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(curve);
                let tolerance = Self::nurbs_parameter_tolerance();
                let lower_bound = u_min - tolerance;
                let upper_bound = u_max + tolerance;
                let is_out_of_domain = t < lower_bound || t > upper_bound;
                debug_assert!(
                    !is_out_of_domain,
                    "CurveRef::point_at_parameter received out-of-domain NURBS parameter: t={:?}, domain=[{:?}, {:?}], tolerance={:?}, accepted_bounds=[{:?}, {:?}]",
                    t,
                    u_min,
                    u_max,
                    tolerance,
                    lower_bound,
                    upper_bound
                );

                let adjusted_t = if t < u_min {
                    u_min
                } else if t > u_max {
                    u_max
                } else {
                    t
                };

                let point = curve.evaluate_at(adjusted_t);
                Point3D::new(point.x(), point.y(), point.z())
            }
        }
    }

    pub fn length(&self) -> T {
        match self {
            Self::Line(line) => line.length(),
            Self::Arc(arc) => <TopoArc3D<T> as geo_contracts::Arc3DDerived<T>>::length(arc),
            Self::EllipseArc(arc) => <TopoEllipseArc3D<T> as EllipseArc3DDerived<T>>::length(arc),
            Self::Nurbs(curve) => {
                curve.approximate_length(geo_nurbs::constants::CURVE_LENGTH_SUBDIVISIONS)
            }
        }
    }
}

/// 辺（Edge）
#[derive(Debug, Clone)]
pub struct Edge<T: Scalar> {
    id: TopoId,
    start_vertex: Arc<Vertex<T>>,
    end_vertex: Arc<Vertex<T>>,
    curve: CurveRef<T>,
    parameter_range: (T, T),
    same_sense: bool,
}

impl<T: Scalar> Edge<T> {
    fn nurbs_parameter_tolerance() -> T {
        CurveRef::<T>::nurbs_parameter_tolerance()
    }

    /// Edge を生成する
    ///
    /// NURBS の場合は parameter_range を native domain に対して検証する。
    /// 許容幅を超えて domain 外なら `None` を返し、端点近傍の微小ズレは
    /// `[u_min, u_max]` へ吸着したうえで保持する。
    pub fn new(
        start_vertex: Arc<Vertex<T>>,
        end_vertex: Arc<Vertex<T>>,
        curve: CurveRef<T>,
        parameter_range: (T, T),
    ) -> Option<Self> {
        let mut normalized_parameter_range = parameter_range;
        if let CurveRef::Nurbs(nurbs_curve) = &curve {
            let (u_min, u_max) =
                <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(nurbs_curve);
            let tolerance = Self::nurbs_parameter_tolerance();

            if normalized_parameter_range.0 < u_min - tolerance
                || normalized_parameter_range.1 > u_max + tolerance
            {
                return None;
            }

            if normalized_parameter_range.0 < u_min {
                normalized_parameter_range.0 = u_min;
            }
            if normalized_parameter_range.1 > u_max {
                normalized_parameter_range.1 = u_max;
            }
        }

        if normalized_parameter_range.0 >= normalized_parameter_range.1 {
            return None;
        }

        Some(Self {
            id: TopoId::new(),
            start_vertex,
            end_vertex,
            curve,
            parameter_range: normalized_parameter_range,
            same_sense: true,
        })
    }

    pub fn id(&self) -> TopoId {
        self.id
    }

    pub fn start_vertex(&self) -> &Arc<Vertex<T>> {
        &self.start_vertex
    }

    pub fn end_vertex(&self) -> &Arc<Vertex<T>> {
        &self.end_vertex
    }

    pub fn curve(&self) -> &CurveRef<T> {
        &self.curve
    }

    /// 母曲線 native parameter 空間における拘束区間 `[t0, t1]` を返す
    pub fn parameter_range(&self) -> (T, T) {
        self.parameter_range
    }

    pub fn same_sense(&self) -> bool {
        self.same_sense
    }

    /// Edge の向きを反転する（母曲線は変更しない）
    pub fn reverse(&mut self) {
        self.same_sense = !self.same_sense;
        std::mem::swap(&mut self.start_vertex, &mut self.end_vertex);
    }

    /// Edge の始端点（向き適用後）
    pub fn oriented_constraint_start_point(&self) -> Point3D<T> {
        self.start_vertex.point()
    }

    /// Edge の終端点（向き適用後）
    pub fn oriented_constraint_end_point(&self) -> Point3D<T> {
        self.end_vertex.point()
    }

    fn oriented_curve_points(
        &self,
        start: Point3D<T>,
        end: Point3D<T>,
    ) -> (Point3D<T>, Point3D<T>) {
        if self.same_sense {
            (start, end)
        } else {
            (end, start)
        }
    }

    fn oriented_constraint_points(&self) -> (Point3D<T>, Point3D<T>) {
        (
            self.oriented_constraint_start_point(),
            self.oriented_constraint_end_point(),
        )
    }

    fn oriented_ideal_points(&self) -> (Point3D<T>, Point3D<T>) {
        self.oriented_curve_points(self.curve.ideal_start_point(), self.curve.ideal_end_point())
    }

    fn oriented_evaluated_points(&self) -> (Point3D<T>, Point3D<T>) {
        let (t0, t1) = self.parameter_range;
        let evaluated_start = self.curve.point_at_parameter(t0);
        let evaluated_end = self.curve.point_at_parameter(t1);

        self.oriented_curve_points(evaluated_start, evaluated_end)
    }

    /// Edge 局所 parameter `local_t` (`0..=1`) を母曲線 parameter_range へ写像して評価する
    ///
    /// `same_sense=false` の場合は拘束向きに合わせて逆向きに写像する。
    pub fn point_at(&self, local_t: T) -> Option<Point3D<T>> {
        if local_t < T::ZERO || local_t > T::ONE {
            return None;
        }

        let (t0, t1) = self.parameter_range;
        let mapped_t = if self.same_sense {
            t0 + (t1 - t0) * local_t
        } else {
            t1 - (t1 - t0) * local_t
        };

        Some(self.curve.point_at_parameter(mapped_t))
    }

    /// 拘束点と Vertex の整合を確認する
    pub fn is_binding_consistent(&self, bind_tolerance: T) -> bool {
        let (constraint_start, constraint_end) = self.oriented_constraint_points();

        self.start_vertex.point().distance_to(&constraint_start) <= bind_tolerance
            && self.end_vertex.point().distance_to(&constraint_end) <= bind_tolerance
    }

    /// primitive の ideal endpoint と拘束端点の整合を確認する
    pub fn is_ideal_endpoint_consistent(&self, ideal_tolerance: T) -> bool {
        let (constraint_start, constraint_end) = self.oriented_constraint_points();
        let (ideal_start, ideal_end) = self.oriented_ideal_points();

        ideal_start.distance_to(&constraint_start) <= ideal_tolerance
            && ideal_end.distance_to(&constraint_end) <= ideal_tolerance
    }

    /// parameter_range から評価した endpoint と拘束端点の整合を確認する
    pub fn is_evaluated_endpoint_consistent(&self, eval_tolerance: T) -> bool {
        let (constraint_start, constraint_end) = self.oriented_constraint_points();
        let (evaluated_start, evaluated_end) = self.oriented_evaluated_points();

        evaluated_start.distance_to(&constraint_start) <= eval_tolerance
            && evaluated_end.distance_to(&constraint_end) <= eval_tolerance
    }

    /// Edge の局所整合を確認する
    pub(crate) fn is_local_consistent_with_tolerances(
        &self,
        tolerances: ResolvedEdgeToleranceSettings<T>,
    ) -> bool {
        self.is_binding_consistent(tolerances.bind_tolerance)
            && self.is_ideal_endpoint_consistent(tolerances.ideal_tolerance)
            && self.is_evaluated_endpoint_consistent(tolerances.eval_tolerance)
    }

    pub fn is_local_consistent(&self, edge_tolerance: T) -> bool {
        self.is_local_consistent_with_tolerances(ResolvedEdgeToleranceSettings::symmetric(
            edge_tolerance,
        ))
    }

    /// 既存呼び出し向けの互換メソッド
    pub fn is_vertex_binding_consistent(&self, edge_tolerance: T) -> bool {
        self.is_local_consistent(edge_tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TopoInfiniteLine3D, TopoNurbsCurve3D};
    use geo_contracts::NurbsCurve3DConstructor;

    #[test]
    fn edge_new_rejects_invalid_range() {
        let v0 = Arc::new(Vertex::new(Point3D::new(0.0, 0.0, 0.0)));
        let v1 = Arc::new(Vertex::new(Point3D::new(1.0, 0.0, 0.0)));
        let line = TopoLineSegment3D::new(v0.point(), v1.point()).unwrap();

        assert!(Edge::new(v0, v1, CurveRef::Line(line), (1.0, 0.0)).is_none());
    }

    #[test]
    fn edge_reverse_swaps_vertices() {
        let v0 = Arc::new(Vertex::new(Point3D::new(0.0, 0.0, 0.0)));
        let v1 = Arc::new(Vertex::new(Point3D::new(1.0, 0.0, 0.0)));
        let line = TopoLineSegment3D::new(v0.point(), v1.point()).unwrap();
        let mut edge = Edge::new(v0.clone(), v1.clone(), CurveRef::Line(line), (0.0, 1.0)).unwrap();

        edge.reverse();

        assert!(!edge.same_sense());
        assert_eq!(edge.start_vertex().point(), v1.point());
        assert_eq!(edge.end_vertex().point(), v0.point());
    }

    #[test]
    fn edge_local_consistency() {
        let v0 = Arc::new(Vertex::new(Point3D::new(0.0, 0.0, 0.0)));
        let v1 = Arc::new(Vertex::new(Point3D::new(1.0, 0.0, 0.0)));
        let line = TopoLineSegment3D::new(v0.point(), v1.point()).unwrap();
        let edge = Edge::new(v0, v1, CurveRef::Line(line), (0.0, 1.0)).unwrap();

        assert!(edge.is_local_consistent(1e-9));
    }

    #[test]
    fn edge_binding_is_separated_from_curve_consistency_for_offset_constraints() {
        let support_line = TopoInfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
        )
        .unwrap();
        let start = Point3D::new(0.0, 0.0, 0.1);
        let end = Point3D::new(2.0, 0.0, 0.1);
        let line =
            TopoLineSegment3D::from_support_line_and_constraint_points(support_line, start, end)
                .unwrap();
        let edge = Edge::new(
            Arc::new(Vertex::new(start)),
            Arc::new(Vertex::new(end)),
            CurveRef::Line(line),
            (0.0, 2.0),
        )
        .unwrap();

        assert!(edge.is_binding_consistent(1e-9));
        assert!(!edge.is_ideal_endpoint_consistent(1e-9));
        assert!(!edge.is_evaluated_endpoint_consistent(1e-9));
        assert!(edge.is_local_consistent(0.31));
        assert!(!edge.is_local_consistent(0.29));
        assert!(
            edge.is_local_consistent_with_tolerances(ResolvedEdgeToleranceSettings::new(
                0.01, 0.15, 0.15,
            ))
        );
    }

    #[test]
    fn edge_ideal_and_evaluated_consistency_are_separated_by_parameter_range() {
        let start = Point3D::new(0.0, 0.0, 0.0);
        let end = Point3D::new(2.0, 0.0, 0.0);
        let line = TopoLineSegment3D::new(start, end).unwrap();
        let edge = Edge::new(
            Arc::new(Vertex::new(start)),
            Arc::new(Vertex::new(end)),
            CurveRef::Line(line),
            (0.1, 1.9),
        )
        .unwrap();

        assert!(edge.is_binding_consistent(1e-9));
        assert!(edge.is_ideal_endpoint_consistent(1e-9));
        assert!(!edge.is_evaluated_endpoint_consistent(1e-9));
    }

    #[test]
    fn curve_ref_nurbs_supports_ideal_endpoints_and_native_parameter_evaluation() {
        let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            1,
            vec![2.0, 2.0, 3.0, 3.0],
            vec![(0.0, 0.0, 0.0), (2.0, 0.0, 0.0)],
            None,
        )
        .unwrap();

        let curve_ref = CurveRef::Nurbs(curve);

        let start = curve_ref.ideal_start_point();
        let end = curve_ref.ideal_end_point();

        assert!((start.x() - 0.0).abs() < 1.0e-12);
        assert!((start.y() - 0.0).abs() < 1.0e-12);
        assert!((start.z() - 0.0).abs() < 1.0e-12);
        assert!((end.x() - 2.0).abs() < 1.0e-12);
        assert!((end.y() - 0.0).abs() < 1.0e-12);
        assert!((end.z() - 0.0).abs() < 1.0e-12);

        let mid = curve_ref.point_at_parameter(2.5);
        assert!((mid.x() - 1.0).abs() < 1.0e-9);
        assert!((mid.y() - 0.0).abs() < 1.0e-9);
        assert!((mid.z() - 0.0).abs() < 1.0e-9);
    }

    #[test]
    fn edge_with_nurbs_curve_uses_native_parameter_range() {
        let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            1,
            vec![2.0, 2.0, 3.0, 3.0],
            vec![(0.0, 0.0, 0.0), (2.0, 0.0, 0.0)],
            None,
        )
        .unwrap();

        let native_range =
            <TopoNurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::parameter_domain(&curve);

        let edge = Edge::new(
            Arc::new(Vertex::new(Point3D::new(0.0, 0.0, 0.0))),
            Arc::new(Vertex::new(Point3D::new(2.0, 0.0, 0.0))),
            CurveRef::Nurbs(curve),
            native_range,
        )
        .unwrap();

        let (t0, t1) = edge.parameter_range();
        assert!((t0 - 2.0).abs() < 1.0e-12);
        assert!((t1 - 3.0).abs() < 1.0e-12);

        let p = edge.point_at(0.5).unwrap();
        assert!((p.x() - 1.0).abs() < 1.0e-9);
        assert!((p.y() - 0.0).abs() < 1.0e-9);
        assert!((p.z() - 0.0).abs() < 1.0e-9);
    }

    #[test]
    fn edge_new_rejects_nurbs_range_outside_native_domain() {
        let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
            1,
            vec![2.0, 2.0, 3.0, 3.0],
            vec![(0.0, 0.0, 0.0), (2.0, 0.0, 0.0)],
            None,
        )
        .unwrap();

        let edge = Edge::new(
            Arc::new(Vertex::new(Point3D::new(0.0, 0.0, 0.0))),
            Arc::new(Vertex::new(Point3D::new(2.0, 0.0, 0.0))),
            CurveRef::Nurbs(curve),
            (1.0, 2.5),
        );

        assert!(edge.is_none());
    }
}
