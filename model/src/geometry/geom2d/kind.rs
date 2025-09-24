#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind2 {
    Line,
    Ray,
    InfiniteLine,
    Circle,
    Arc,
    Ellipse,
    EllipticArc,
    NurbsCurve,
    Unknown,
}