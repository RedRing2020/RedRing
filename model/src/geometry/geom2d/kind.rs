use super::point::Point2D;
use super::line::Line2D;

pub enum GeometryKind2D {
    Line(Line2D),
    Ray(Ray2D),
    InfiniteLine(InfiniteLine2D),
    Circle(Circle2D),
    Arc(Arc2D),
    Ellipse(Ellipse2D),
    EllipticArc(EllipticArc2D),
    Nurbs(NurbsCurve2D),
}