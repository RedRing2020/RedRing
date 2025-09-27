use super::point::Point;
use super::direction::Direction;
use super::vector::Vector;
use super::surface::Surface;
use super::kind::SurfaceKind;

/// Represents a finite analytic cylinder in 3D space.
/// Defined by base and top points along a unit axis direction.
#[derive(Debug, Clone, PartialEq)]
pub struct Cylinder {
    base: Point,            // Bottom center point
    top: Point,             // Top center point
    axis: Direction,        // Unit direction from base to top
    radius: f64,            // Radius of the cylinder
}

impl Surface for Cylinder {
    fn kind(&self) -> SurfaceKind {
        SurfaceKind::Cylinder
    }
}

impl Cylinder {
    /// Creates a new cylinder from base and top points and radius.
    /// Automatically computes axis direction.
    pub fn new(base: Point, top: Point, radius: f64) -> Option<Self> {
        let axis_vec = Vector::between(&base, &top);
        let axis = Direction::from_vector(axis_vec)?;
        Some(Self { base, top, axis, radius })
    }

    /// Returns the base point.
    pub fn base(&self) -> Point {
        self.base
    }

    /// Returns the top point.
    pub fn top(&self) -> Point {
        self.top
    }

    /// Returns the axis direction (unit vector).
    pub fn axis(&self) -> Direction {
        self.axis
    }

    /// Returns the radius.
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Returns the height of the cylinder.
    pub fn height(&self) -> f64 {
        self.base.distance_to(&self.top)
    }

    /// Returns true if the cylinder is degenerate (zero height).
    pub fn is_degenerate(&self) -> bool {
        self.base == self.top
    }

    /// Returns a reversed cylinder (base/top swapped, axis flipped).
    pub fn reversed(&self) -> Self {
        Self {
            base: self.top,
            top: self.base,
            axis: self.axis.negate(),
            radius: self.radius,
        }
    }
}