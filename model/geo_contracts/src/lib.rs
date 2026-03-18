//! geo_contracts - shape contract definitions.
//!
//! This crate will host geometry-facing contracts that are shared across
//! `geo_primitives`, `geo_nurbs`, and higher-level algorithm crates.

pub mod classification;
pub mod entity;
pub mod geometry;

pub use analysis::abstract_types::{Angle, Scalar};
pub use classification::{DimensionClass, GeometryPrimitive, PrimitiveKind};
// Phase 1 では arc/circle 契約のみ先行公開し、以降のPRで段階展開する。
pub use entity::{EntityDisplayProperties, EntityIdentity, LineEntity3DProperties};
pub use geometry::core::{
    Arc2DConstructor, Arc2DCore, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DCore,
    Arc3DMeasure, Arc3DProperties, Circle2DConstructor, Circle2DCore, Circle2DMeasure,
    Circle2DProperties, Circle3DConstructor, Circle3DCore, Circle3DMeasure, Circle3DProperties,
    Point2DConstructor, Point2DCore, Point2DMeasure, Point2DProperties, Point3DConstructor,
    Point3DCore, Point3DMeasure, Point3DProperties, Vector2DConstructor, Vector2DCore,
    Vector2DMeasure, Vector2DProperties, Vector3DConstructor, Vector3DCore, Vector3DMeasure,
    Vector3DProperties,
};
pub use geometry::core::{
    NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DMeasure, NurbsCurve2DProperties,
    NurbsCurve3DConstructor, NurbsCurve3DCore, NurbsCurve3DMeasure, NurbsCurve3DProperties,
    NurbsSurface3DConstructor, NurbsSurface3DCore, NurbsSurface3DMeasure, NurbsSurface3DProperties,
    Triangle2DConstructor, Triangle2DCore, Triangle2DMeasure, Triangle2DProperties,
    Triangle3DConstructor, Triangle3DCore, Triangle3DMeasure, Triangle3DProperties,
};
