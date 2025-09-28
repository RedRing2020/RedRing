//! CurveKind3D: 3次元曲線の分類
//!
//! `geometry::geometry3d` 配下の構造体に対応する分類を網羅的に定義。
//! 実装との整合性、語義的明快さ、将来的な拡張性を考慮。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind3D {
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
