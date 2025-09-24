use super::point::Point;
use super::direction::Direction;
use super::vector::Vector;
use super::surface::Surface;

/// Represents an analytic ellipsoid in 3D space.
/// Defined by center point, axis directions, and three radii.
#[derive(Debug, Clone, PartialEq)]
pub struct Ellipsoid {
    center: Point,            // Center of the ellipsoid
    axis_x: Direction,        // Direction of semi-axis a
    axis_y: Direction,        // Direction of semi-axis b (orthogonal to axis_x)
    axis_z: Direction,        // Direction of semi-axis c (orthogonal to both)
    radius_x: f64,            // Length of semi-axis a
    radius_y: f64,            // Length of semi-axis b
    radius_z: f64,            // Length of semi-axis c
}

impl Surface for Ellipsoid {
    fn kind(&self) -> SurfaceKind {
        SurfaceKind::Ellipsoid
    }
}

impl Ellipsoid {
    /// Creates a new ellipsoid from center, three orthogonal directions, and radii.
    pub fn new(
        center: Point,
        axis_x: Direction,
        axis_y: Direction,
        axis_z: Direction,
        radius_x: f64,
        radius_y: f64,
        radius_z: f64,
    ) -> Option<Self> {
        if radius_x <= 0.0 || radius_y <= 0.0 || radius_z <= 0.0 {
            return None;
        }
        // Optional: orthogonality check (can be relaxed in tolerant modeling)
        Some(Self {
            center,
            axis_x,
            axis_y,
            axis_z,
            radius_x,
            radius_y,
            radius_z,
        })
    }

    pub fn center(&self) -> Point {
        self.center
    }

    pub fn axis_x(&self) -> Direction {
        self.axis_x
    }

    pub fn axis_y(&self) -> Direction {
        self.axis_y
    }

    pub fn axis_z(&self) -> Direction {
        self.axis_z
    }

    pub fn radius_x(&self) -> f64 {
        self.radius_x
    }

    pub fn radius_y(&self) -> f64 {
        self.radius_y
    }

    pub fn radius_z(&self) -> f64 {
        self.radius_z
    }

    /// Returns true if the ellipsoid is a sphere.
    pub fn is_sphere(&self) -> bool {
        self.radius_x == self.radius_y && self.radius_y == self.radius_z
    }
}