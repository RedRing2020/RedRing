//! Wire: Edge 連続列

use crate::{Edge, TopoId};
use geo_contracts::Scalar;

/// 連続した Edge 列
#[derive(Debug, Clone)]
pub struct Wire<T: Scalar> {
    id: TopoId,
    edges: Vec<Edge<T>>,
}

impl<T: Scalar> Wire<T> {
    /// 連続性を検証した上で Wire を生成する
    pub fn new(edges: Vec<Edge<T>>, edge_tolerance: T, shared_tolerance: T) -> Option<Self> {
        if edges.is_empty() {
            return None;
        }

        let wire = Self {
            id: TopoId::new(),
            edges,
        };

        if wire.is_continuous(edge_tolerance, shared_tolerance) {
            Some(wire)
        } else {
            None
        }
    }

    pub fn id(&self) -> TopoId {
        self.id
    }

    pub fn edges(&self) -> &[Edge<T>] {
        &self.edges
    }

    /// 各 Edge の局所整合と、shared vertex を介した連続性を検証
    pub fn is_continuous(&self, edge_tolerance: T, shared_tolerance: T) -> bool {
        if self.edges.is_empty() {
            return false;
        }

        if !self
            .edges
            .iter()
            .all(|edge| edge.is_vertex_binding_consistent(edge_tolerance))
        {
            return false;
        }

        for i in 0..self.edges.len() - 1 {
            let end = self.edges[i].oriented_constraint_end_point();
            let next_start = self.edges[i + 1].oriented_constraint_start_point();
            if end.distance_to(&next_start) > shared_tolerance {
                return false;
            }
        }

        true
    }

    /// 閉ループかを判定
    pub fn is_closed(&self, shared_tolerance: T) -> bool {
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

        let wire = Wire::new(vec![e1, e2], 1e-9, 1e-9).unwrap();
        assert!(wire.is_continuous(1e-9, 1e-9));
        assert!(!wire.is_closed(1e-9));
    }

    #[test]
    fn wire_disconnected_edges_fails() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(3.0, 0.0, 0.0), Point3D::new(4.0, 0.0, 0.0));

        assert!(Wire::new(vec![e1, e2], 1e-9, 1e-9).is_none());
    }

    #[test]
    fn wire_closed_loop() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0));
        let e3 = make_edge(Point3D::new(1.0, 1.0, 0.0), Point3D::new(0.0, 0.0, 0.0));

        let wire = Wire::new(vec![e1, e2, e3], 1e-9, 1e-9).unwrap();
        assert!(wire.is_closed(1e-9));
    }

    #[test]
    fn wire_uses_separate_edge_and_shared_tolerances() {
        let start = Point3D::new(0.0, 0.0, 0.1);
        let middle = Point3D::new(1.0, 0.0, 0.1);
        let end = Point3D::new(2.0, 0.0, 0.1);

        let e1 = make_offset_support_edge(start, middle);
        let e2 = make_offset_support_edge(middle, end);

        assert!(Wire::new(vec![e1.clone(), e2.clone()], 0.31, 1e-9).is_some());
        assert!(Wire::new(vec![e1, e2], 0.29, 1e-9).is_none());
    }
}
