use super::point::Point;
use super::direction::Direction;

/// Represents an analytic torus in 3D space.
/// Defined by a center point, axis direction, major radius, and minor radius.
#[derive(Debug, Clone, PartialEq)]
pub struct Torus {
    center: Point,          // Center of the torus (center of the hole)
    axis: Direction,        // Axis direction (normal to the torus plane)
    major_radius: f64,      // Distance from center to tube center
    minor_radius: f64,      // Radius of the tube
}

impl Surface for Torus {
    fn kind(&self) -> SurfaceKind {
        SurfaceKind::Torus
    }
}

impl Torus {
    /// Creates a new torus from center, axis, major and minor radius.
    pub fn new(center: Point, axis: Direction, major_radius: f64, minor_radius: f64) -> Option<Self> {
        if major_radius <= 0.0 || minor_radius <= 0.0 || minor_radius >= major_radius {
            return None;
        }
        Some(Self {
            center,
            axis,
            major_radius,
            minor_radius,
        })
    }

    /// Returns the center point.
    pub fn center(&self) -> Point {
        self.center
    }

    /// Returns the axis direction.
    pub fn axis(&self) -> Direction {
        self.axis
    }

    /// Returns the major radius.
    pub fn major_radius(&self) -> f64 {
        self.major_radius
    }

    /// Returns the minor radius.
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius
    }

    /// Returns true if the torus is degenerate (zero or invalid radius).
    pub fn is_degenerate(&self) -> bool {
        self.major_radius <= 0.0 || self.minor_radius <= 0.0 || self.minor_radius >= self.major_radius
    }

    /// Returns the ratio of minor to major radius.
    pub fn thickness_ratio(&self) -> f64 {
        self.minor_radius / self.major_radius
    }
}