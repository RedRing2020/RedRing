//! Classification contracts for geometry primitives.
//!
//! This module defines the classification system that is shared across the workspace.
//! All crates use these types to identify geometric primitives.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Point,
    LineSegment,
    PolyLine,
    BezierCurve,
    NurbsCurve,
    NurbsCurve2D,
    NurbsCurve3D,
    Arc,
    Ray,
    InfiniteLine,
    Circle,
    Ellipse,
    Rectangle,
    Polygon,
    Triangle,
    Sphere,
    SphericalSolid,
    SphericalSurface,
    EllipsoidalSolid,
    EllipsoidalSurface,
    Cylinder,
    CylindricalSolid,
    CylindricalSurface,
    Cone,
    ConicalSolid,
    ConicalSurface,
    TorusSolid,
    TorusSurface,
    Cube,
    Plane,
    TriangleMesh,
    NurbsSurface,
    NurbsSurface3D,
    Group,
    Assembly,
    BBox,
    Vector,
    Mesh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionClass {
    Zero,
    One,
    Two,
    Three,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryPrimitive {
    Basic,
    Curve,
    Surface,
    Solid,
}
