//! Geometric Topology
//!
//! Provides minimal topology structures for intersection and modeling operations.
//! Currently includes:
//! - CompositeCurve3D: Multiple curve segments connected at endpoints
//! - CurveSegment3D: Individual curve segment (Line or Arc)
//!
//! Future extensions (depends on #205):
//! - Vertex: Point topology
//! - Edge: Curve topology
//! - Face: Surface topology
//! - HalfEdge: Directed edge with adjacency information

pub mod composite_curve;

pub use composite_curve::{CompositeCurve3D, CurveSegment3D};
pub use geo_core::{Point3D, Vector3D};

pub type TopoArc3D<T> = geo_primitives::Arc3D<T>;
pub type TopoLineSegment3D<T> = geo_primitives::LineSegment3D<T>;
