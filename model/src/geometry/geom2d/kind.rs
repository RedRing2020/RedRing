use super::point::Point2D;
use super::line::Line2D;

pub enum GeometryKind2D {
    Point { position: Point2D},
    Line(Line2D),
    Ray(Ray2D),
    InfiniteLine(InfiniteLine2D),
    Circle { center: Point2D, radius: f64 },
    Arc { center: Point2D, radius: f64, start_angle: f64, end_angle: f64 },
    Ellipse { center: Point2D, major_axis: [f64; 2], major_radius: f64, minor_radius: f64 },
    Nurbs { curve: NurbsCurve2D },
}