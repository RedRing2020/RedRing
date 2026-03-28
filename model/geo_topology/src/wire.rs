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
    pub fn new(edges: Vec<Edge<T>>, tolerance: T) -> Option<Self> {
        if edges.is_empty() {
            return None;
        }

        let wire = Self {
            id: TopoId::new(),
            edges,
        };

        if wire.is_continuous(tolerance) {
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

    /// 端点連続性 + 各 Edge の頂点拘束整合を検証
    pub fn is_continuous(&self, tolerance: T) -> bool {
        if self.edges.is_empty() {
            return false;
        }

        if !self
            .edges
            .iter()
            .all(|edge| edge.is_vertex_binding_consistent(tolerance))
        {
            return false;
        }

        for i in 0..self.edges.len() - 1 {
            let end = self.edges[i].oriented_end_point();
            let next_start = self.edges[i + 1].oriented_start_point();
            if end.distance_to(&next_start) > tolerance {
                return false;
            }
        }

        true
    }

    /// 閉ループかを判定
    pub fn is_closed(&self, tolerance: T) -> bool {
        let first = self.edges.first().expect("wire has at least one edge");
        let last = self.edges.last().expect("wire has at least one edge");
        first
            .oriented_start_point()
            .distance_to(&last.oriented_end_point())
            <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CurveRef, Point3D, TopoLineSegment3D, Vertex};
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

    #[test]
    fn wire_continuous_edges() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(1.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0));

        let wire = Wire::new(vec![e1, e2], 1e-9).unwrap();
        assert!(wire.is_continuous(1e-9));
        assert!(!wire.is_closed(1e-9));
    }

    #[test]
    fn wire_disconnected_edges_fails() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(3.0, 0.0, 0.0), Point3D::new(4.0, 0.0, 0.0));

        assert!(Wire::new(vec![e1, e2], 1e-9).is_none());
    }

    #[test]
    fn wire_closed_loop() {
        let e1 = make_edge(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0));
        let e2 = make_edge(Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0));
        let e3 = make_edge(Point3D::new(1.0, 1.0, 0.0), Point3D::new(0.0, 0.0, 0.0));

        let wire = Wire::new(vec![e1, e2, e3], 1e-9).unwrap();
        assert!(wire.is_closed(1e-9));
    }
}
