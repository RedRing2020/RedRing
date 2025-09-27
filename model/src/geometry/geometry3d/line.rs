use super::point::Point;
use super::direction::Direction;
use super::vector::Vector;
use crate::geom3::kind::CurveKind;
use crate::geom3::curve::curve_trait::Curve3;

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    origin: Point,
    direction: Direction,
    start: Point,
    end: Point,
}

impl Curve3 for Line {
    fn kind(&self) -> CurveKind3 {
        CurveKind3::Line
    }
}

impl Line {
    pub fn new(origin: Point, direction: Direction, start: Point, end: Point) -> Self {
        Self { origin, direction, start, end }
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn start(&self) -> Point {
        self.start
    }

    pub fn end(&self) -> Point {
        self.end
    }

    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }

    pub fn vector(&self) -> Vector {
        Vector::between(&self.start, &self.end)
    }

    pub fn interpolate(&self, t: f64) -> Point {
        let v = self.vector().scale(t);
        self.start.translate(&v)
    }

    pub fn is_aligned(&self) -> bool {
        if let Some(trimmed_dir) = Direction::from_vector(self.vector()) {
            trimmed_dir == self.direction
        } else {
            false
        }
    }

    pub fn is_trimmed(&self) -> bool {
        self.start != self.origin || self.end != self.origin
    }
}