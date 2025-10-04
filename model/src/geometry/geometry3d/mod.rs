pub mod point;
pub mod vector;
pub mod direction;
pub mod infinite_line;
pub mod line;
pub mod circle;
pub mod arc;
pub mod ellipse;
pub mod ellipse_arc;
pub mod nurbs_curve;
pub mod plane;
pub mod curve;

pub use point::Point;
pub use vector::Vector;
pub use direction::Direction;
pub use infinite_line::InfiniteLine;
pub use line::Line;
pub use circle::Circle;
pub use arc::Arc;
pub use ellipse::Ellipse;
pub use ellipse_arc::EllipseArc;
pub use nurbs_curve::NurbsCurve;
pub use plane::Plane;
pub use curve::Curve3;

// Type aliases for dimensional clarity
pub type Point3D = Point;
pub type Vector3D = Vector;
pub type Direction3D = Direction;
