#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind3 {
    Line,
    Circle,
    Arc,
    Ellipse,
    EllipticArc,
    NurbsCurve,
    CompositeCurve,
    TrimmedCurve,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Plane,
    Sphere,
    Cone,
    Cylinder,
    Ellipsoid,
    Torus,
    NurbsSurface,
    TrimmedSurface,
    Unknown,
}