//! geo_topology - 幾何トポロジー基盤クレート
//!
//! 交差判定・モデリングで利用する最小のトポロジー構造を提供する。
//! 現在の提供:
//! - CompositeCurve3D: 端点で連結された複合曲線
//! - CurveSegment3D: 個別セグメント（Line / Arc）
//!
//! 将来拡張（#205 連携）:
//! - Vertex: 点トポロジー
//! - Edge: 曲線トポロジー
//! - Face: 面トポロジー
//! - HalfEdge: 隣接情報を持つ有向エッジ

pub mod composite_curve;
pub mod topology_core;
pub mod wire;

pub use composite_curve::{CompositeCurve3D, CurveSegment3D};
pub use geo_core::{Point3D, Vector3D};
pub use topology_core::{CurveRef, Edge, TopoId, Vertex};
pub use wire::Wire;

pub type TopoArc3D<T> = geo_primitives::Arc3D<T>;
pub type TopoEllipseArc3D<T> = geo_primitives::EllipseArc3D<T>;
pub type TopoInfiniteLine3D<T> = geo_primitives::InfiniteLine3D<T>;
pub type TopoLineSegment3D<T> = geo_primitives::LineSegment3D<T>;
