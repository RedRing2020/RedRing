use crate::geometry::geometry3d::circle::Circle;

#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    circle: Circle,
    start_angle: f64,
    end_angle: f64,
}
