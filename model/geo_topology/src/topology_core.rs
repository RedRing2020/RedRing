//! topo正規形の最小構成要素
//!
//! #408 の最小導入として、Vertex/Edge/CurveRef を提供する。

use crate::{Point3D, TopoArc3D, TopoEllipseArc3D, TopoLineSegment3D};
use geo_contracts::{Arc3DMeasure, EllipseArc3DMeasure, Scalar};
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
}

impl<T: Scalar> CurveRef<T> {
    pub fn start_point(&self) -> Point3D<T> {
        match self {
            Self::Line(line) => line.start(),
            Self::Arc(arc) => {
                let (x, y, z) = <TopoArc3D<T> as Arc3DMeasure<T>>::start_point(arc);
                Point3D::new(x, y, z)
            }
            Self::EllipseArc(arc) => {
                let (x, y, z) = <TopoEllipseArc3D<T> as EllipseArc3DMeasure<T>>::start_point(arc);
                Point3D::new(x, y, z)
            }
        }
    }

    pub fn end_point(&self) -> Point3D<T> {
        match self {
            Self::Line(line) => line.end(),
            Self::Arc(arc) => {
                let (x, y, z) = <TopoArc3D<T> as Arc3DMeasure<T>>::end_point(arc);
                Point3D::new(x, y, z)
            }
            Self::EllipseArc(arc) => {
                let (x, y, z) = <TopoEllipseArc3D<T> as EllipseArc3DMeasure<T>>::end_point(arc);
                Point3D::new(x, y, z)
            }
        }
    }

    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        match self {
            Self::Line(line) => line.line().point_at_parameter(t),
            Self::Arc(arc) => {
                let (x, y, z) = <TopoArc3D<T> as Arc3DMeasure<T>>::point_at_parameter(arc, t);
                Point3D::new(x, y, z)
            }
            Self::EllipseArc(arc) => {
                let (x, y, z) =
                    <TopoEllipseArc3D<T> as EllipseArc3DMeasure<T>>::point_at_parameter(arc, t);
                Point3D::new(x, y, z)
            }
        }
    }

    pub fn length(&self) -> T {
        match self {
            Self::Line(line) => line.length(),
            Self::Arc(arc) => arc.arc_length(),
            Self::EllipseArc(arc) => arc.measure(),
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
    /// Edge を生成する
    pub fn new(
        start_vertex: Arc<Vertex<T>>,
        end_vertex: Arc<Vertex<T>>,
        curve: CurveRef<T>,
        parameter_range: (T, T),
    ) -> Option<Self> {
        if parameter_range.0 >= parameter_range.1 {
            return None;
        }

        Some(Self {
            id: TopoId::new(),
            start_vertex,
            end_vertex,
            curve,
            parameter_range,
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
    pub fn oriented_start_point(&self) -> Point3D<T> {
        self.start_vertex.point()
    }

    /// Edge の終端点（向き適用後）
    pub fn oriented_end_point(&self) -> Point3D<T> {
        self.end_vertex.point()
    }

    /// 曲線評価（0..1 のローカルパラメータ）
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

    /// Edge が保持する頂点と幾何端点の整合を確認する
    pub fn is_vertex_binding_consistent(&self, tolerance: T) -> bool {
        let (t0, t1) = self.parameter_range;
        let curve_start = self.curve.point_at_parameter(t0);
        let curve_end = self.curve.point_at_parameter(t1);

        let (expected_start, expected_end) = if self.same_sense {
            (curve_start, curve_end)
        } else {
            (curve_end, curve_start)
        };

        self.start_vertex.point().distance_to(&expected_start) <= tolerance
            && self.end_vertex.point().distance_to(&expected_end) <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn edge_vertex_binding_consistency() {
        let v0 = Arc::new(Vertex::new(Point3D::new(0.0, 0.0, 0.0)));
        let v1 = Arc::new(Vertex::new(Point3D::new(1.0, 0.0, 0.0)));
        let line = TopoLineSegment3D::new(v0.point(), v1.point()).unwrap();
        let edge = Edge::new(v0, v1, CurveRef::Line(line), (0.0, 1.0)).unwrap();

        assert!(edge.is_vertex_binding_consistent(1e-9));
    }
}
