use super::point::Point2;
use super::line::Line2;

pub enum GeometryKind2 {
    Line(Line2),
    Ray(Ray2),
    InfiniteLine(InfiniteLine2),
    Circle(Circle2),
    Arc(Arc2),
    Ellipse(Ellipse2),
    EllipticArc(EllipticArc2),
    Nurbs(NurbsCurve2),
}