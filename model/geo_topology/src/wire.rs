//! Wire: Edge 連続列

use crate::topology_validator::TopologyValidator;
use crate::{Edge, TopoId, TopologyToleranceSettings};
use geo_contracts::Scalar;

/// 連続した Edge 列
#[derive(Debug, Clone)]
pub struct Wire<T: Scalar> {
    id: TopoId,
    edges: Vec<Edge<T>>,
}

impl<T: Scalar> Wire<T> {
    /// 連続性を検証した上で Wire を生成する
    pub fn new(edges: Vec<Edge<T>>, tolerances: TopologyToleranceSettings<T>) -> Option<Self> {
        if edges.is_empty() {
            return None;
        }

        let wire = Self {
            id: TopoId::new(),
            edges,
        };

        if wire.is_continuous(tolerances) {
            Some(wire)
        } else {
            None
        }
    }

    pub fn new_with_tolerances(
        edges: Vec<Edge<T>>,
        edge_tolerance: T,
        shared_tolerance: T,
    ) -> Option<Self> {
        Self::new(
            edges,
            TopologyToleranceSettings::new(edge_tolerance, shared_tolerance),
        )
    }

    pub fn id(&self) -> TopoId {
        self.id
    }

    pub fn edges(&self) -> &[Edge<T>] {
        &self.edges
    }

    /// 各 Edge の局所整合と、shared vertex を介した連続性を検証
    pub fn is_continuous(&self, tolerances: TopologyToleranceSettings<T>) -> bool {
        if self.edges.is_empty() {
            return false;
        }

        TopologyValidator::new(tolerances)
            .validate_wire(self)
            .is_valid()
    }

    pub fn is_continuous_with_tolerances(&self, edge_tolerance: T, shared_tolerance: T) -> bool {
        self.is_continuous(TopologyToleranceSettings::new(
            edge_tolerance,
            shared_tolerance,
        ))
    }

    /// 閉ループかを判定
    pub fn is_closed(&self, tolerances: TopologyToleranceSettings<T>) -> bool {
        self.is_closed_with_tolerance(tolerances.shared_tolerance)
    }

    pub fn is_closed_with_tolerance(&self, shared_tolerance: T) -> bool {
        let first = self.edges.first().expect("wire has at least one edge");
        let last = self.edges.last().expect("wire has at least one edge");
        first
            .oriented_constraint_start_point()
            .distance_to(&last.oriented_constraint_end_point())
            <= shared_tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CurveRef, Point3D, TopoInfiniteLine3D, TopoLineSegment3D, Vertex};
    use std::sync::Arc;

    fn make_edge(start: Point3D<f64>, end: Point3D<f64>) -> Edge<f64> {
        let v0 = Arc::new(Vertex::new(start));
        let v1 = Arc::new(Vertex::new(end));
        let line = TopoLineSegment3D::new(start, end).unwrap();
        Edge::new(
            v0,
            v1,
            CurveRef::Line(line),
            (line.start_param(), line.end_param()),
        )
        .unwrap()
    }

    fn make_offset_support_edge(start: Point3D<f64>, end: Point3D<f64>) -> Edge<f64> {
        let support_start = Point3D::new(start.x(), start.y(), 0.0);
        let support_end = Point3D::new(end.x(), end.y(), 0.0);
        let support_line = TopoInfiniteLine3D::from_two_points(support_start, support_end).unwrap();
        let line =
            TopoLineSegment3D::from_support_line_and_constraint_points(support_line, start, end)
                .unwrap();

        Edge::new(
            Arc::new(Vertex::new(start)),
            Arc::new(Vertex::new(end)),
            CurveRef::Line(line),
            (line.start_param(), line.end_param()),
        )
        .unwrap()
    }

    #[test]
    fn wire_continuous_edges() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(1.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0));

        let tolerances = TopologyToleranceSettings::new(1e-9, 1e-9);
        let wire = Wire::new(vec![e1, e2], tolerances).unwrap();
        assert!(wire.is_continuous(tolerances));
        assert!(!wire.is_closed(tolerances));
    }

    #[test]
    fn wire_disconnected_edges_fails() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(3.0, 0.0, 0.0), Point3D::new(4.0, 0.0, 0.0));

        assert!(Wire::new_with_tolerances(vec![e1, e2], 1e-9, 1e-9).is_none());
    }

    #[test]
    fn wire_closed_loop() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0));
        let e3 = make_edge(Point3D::new(1.0, 1.0, 0.0), Point3D::new(0.0, 0.0, 0.0));

        let tolerances = TopologyToleranceSettings::new(1e-9, 1e-9);
        let wire = Wire::new(vec![e1, e2, e3], tolerances).unwrap();
        assert!(wire.is_closed(tolerances));
    }

    #[test]
    fn wire_uses_separate_edge_and_shared_tolerances() {
        let start = Point3D::new(0.0, 0.0, 0.1);
        let middle = Point3D::new(1.0, 0.0, 0.1);
        let end = Point3D::new(2.0, 0.0, 0.1);

        let e1 = make_offset_support_edge(start, middle);
        let e2 = make_offset_support_edge(middle, end);

        assert!(Wire::new_with_tolerances(vec![e1.clone(), e2.clone()], 0.31, 1e-9).is_some());
        assert!(Wire::new_with_tolerances(vec![e1, e2], 0.29, 1e-9).is_none());
    }
}
