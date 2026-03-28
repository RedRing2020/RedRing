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
