use super::point::Point;
use super::direction::Direction;
use super::vector::Vector;
use super::surface::Surface;
use super::kind::SurfaceKind;

/// Represents a finite analytic cone in 3D space.
/// Defined by base and top points along a unit axis direction, and base radius.
#[derive(Debug, Clone, PartialEq)]
pub struct Cone {
    base: Point,            // Center of base circle
    top: Point,             // Apex point
    axis: Direction,        // Unit direction from base to top
    base_radius: f64,       // Radius at base
}

impl Surface for Cone {
    fn kind(&self) -> SurfaceKind {
        SurfaceKind::Cone
    }
}

impl Cone {
    /// Creates a new cone from base and top points and base radius.
    /// Automatically computes axis direction.
    pub fn new(base: Point, top: Point, base_radius: f64) -> Option<Self> {
        let axis_vec = Vector::between(&base, &top);
        let axis = Direction::from_vector(axis_vec)?;
        Some(Self { base, top, axis, base_radius })
    }

    /// Returns the base point.
    pub fn base(&self) -> Point {
        self.base
    }

    /// Returns the apex point.
    pub fn top(&self) -> Point {
        self.top
    }

    /// Returns the axis direction.
    pub fn axis(&self) -> Direction {
        self.axis
    }

    /// Returns the base radius.
    pub fn base_radius(&self) -> f64 {
        self.base_radius
    }

    /// Returns the height of the cone.
    pub fn height(&self) -> f64 {
        self.base.distance_to(&self.top)
    }

    /// Returns the cone angle in radians.
    pub fn angle(&self) -> f64 {
        (self.base_radius / self.height()).atan()
    }

    /// Returns true if the cone is degenerate (zero height or zero radius).
    pub fn is_degenerate(&self) -> bool {
        self.base == self.top || self.base_radius == 0.0
    }

    /// Returns a reversed cone (base/top swapped, axis flipped).
    pub fn reversed(&self) -> Self {
        Self {
            base: self.top,
            top: self.base,
            axis: self.axis.negate(),
            base_radius: self.base_radius,
        }
    }
}