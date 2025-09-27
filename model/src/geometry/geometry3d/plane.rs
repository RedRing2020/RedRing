use super::point::Point;
use super::direction::Direction;
use super::vector::Vector;
use crate::geometry3d::kind::SurfaceKind;
use crate::geometry3d::surface::surface_trait::Surface;

/// Represents an analytic plane in 3D space.
/// Defined by a point and a normal direction.
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    point: Point,           // A point on the plane
    normal: Direction,      // Unit normal vector
}

impl Surface for Plane {
    fn kind(&self) -> SurfaceKind {
        SurfaceKind::Plane
    }
}

impl Plane {
    /// Creates a new plane from a point and a normal direction.
    pub fn new(point: Point, normal: Direction) -> Self {
        Self { point, normal }
    }

    /// Returns the reference point on the plane.
    pub fn point(&self) -> Point {
        self.point
    }

    /// Returns the unit normal direction of the plane.
    pub fn normal(&self) -> Direction {
        self.normal
    }

    /// Computes the signed distance from a point to the plane.
    pub fn signed_distance_to(&self, target: &Point) -> f64 {
        let v = Vector::between(&self.point, target);
        self.normal.as_vector().dot(&v)
    }

    /// Computes the absolute (unsigned) distance from a point to the plane.
    pub fn distance_to(&self, target: &Point) -> f64 {
        self.signed_distance_to(target).abs()
    }

    /// Projects a point orthogonally onto the plane.
    pub fn project_point(&self, target: &Point) -> Point {
        let d = self.signed_distance_to(target);
        let offset = self.normal.as_vector().scale(-d);
        target.translate(&offset)
    }

    /// Returns true if the point lies on the plane (within tolerance).
    pub fn contains_point(&self, target: &Point, tolerance: f64) -> bool {
        self.distance_to(target) <= tolerance
    }
}