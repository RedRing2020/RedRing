//! Intersection Result Types
//!
//! Provides generalized result types for geometric intersection operations.
//!
//! Architecture:
//! - `IntersectionGeometry`: Describes what was found geometrically
//! - `IntersectionTopology`: Describes the topological relationship
//! - `IntersectionResult`: Complete result combining geometry and topology

use crate::{CompositeCurve3D, InfiniteLine3D, LineSegment3D, Point3D};
use geo_contracts::Scalar;

/// Geometric content of an intersection result
#[derive(Clone, Debug)]
pub enum IntersectionGeometry<T: Scalar> {
    /// No intersection
    None,
    /// Single point intersection
    Point(Point3D<T>),
    /// Multiple isolated points
    Points(Vec<Point3D<T>>),
    /// Infinite line (e.g., plane-plane intersection when not parallel)
    InfiniteLine(InfiniteLine3D<T>),
    /// Line segment (e.g., surface-surface intersection bounded)
    Segment(LineSegment3D<T>),
    /// Composite curve (e.g., multiple connected segments)
    CompositeCurve(CompositeCurve3D<T>),
    /// Complete coincidence (shapes overlap completely)
    Coincident,
}

impl<T: Scalar> IntersectionGeometry<T> {
    /// Get a human-readable description of the geometry type
    pub fn description(&self) -> &'static str {
        match self {
            Self::None => "no intersection",
            Self::Point(_) => "single point",
            Self::Points(_) => "multiple points",
            Self::InfiniteLine(_) => "infinite line",
            Self::Segment(_) => "line segment",
            Self::CompositeCurve(_) => "composite curve",
            Self::Coincident => "coincident (complete overlap)",
        }
    }

    /// Check if the intersection is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Check if the intersection contains multiple geometric elements
    pub fn is_multiple(&self) -> bool {
        matches!(self, Self::Points(_) | Self::CompositeCurve(_))
    }

    /// Get the dimension of the intersection
    /// - -1: Empty
    /// - 0: Point(s)
    /// - 1: Curve(s)
    /// - 2: Surface (Coincident)
    pub fn dimension(&self) -> i32 {
        match self {
            Self::None => -1,
            Self::Point(_) | Self::Points(_) => 0,
            Self::InfiniteLine(_) | Self::Segment(_) | Self::CompositeCurve(_) => 1,
            Self::Coincident => 2,
        }
    }
}

/// Topological classification of the intersection
///
/// Describes how the shapes relate positionally:
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntersectionTopology {
    /// Shapes do not touch
    Disjoint,
    /// Shapes touch at isolated points (e.g., tangent)
    Touching,
    /// Shapes cross (transverse intersection)
    Crossing,
    /// Shapes overlap completely (all points coincide)
    Coincident,
}

impl IntersectionTopology {
    /// Check if shapes intersect (excluding purely disjoint)
    pub fn intersects(&self) -> bool {
        !matches!(self, Self::Disjoint)
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Disjoint => "disjoint",
            Self::Touching => "touching",
            Self::Crossing => "crossing",
            Self::Coincident => "coincident",
        }
    }
}

/// Complete intersection result combining geometry and topology
///
/// # Invariants
///
/// The following rules are maintained by the intersection computation layer:
///
/// 1. **Priority order for classification**: Coincident > Crossing > Touching > Disjoint
/// 2. **Geometry-Topology consistency**:
///    - `Coincident`: geometry must be `Self::Coincident`
///    - `Touching`: geometry dimension must be 0 (point(s))
///    - `Crossing`: geometry dimension must be >= 1
///    - `Disjoint`: geometry must be `Self::None`
/// 3. **Tolerance usage**: `tolerance_used` must match caller's input tolerance
///    (or system default if caller provided none)
#[derive(Clone, Debug)]
pub struct IntersectionResult<T: Scalar> {
    /// The geometric intersection
    pub geometry: IntersectionGeometry<T>,

    /// The topological relationship
    pub topology: IntersectionTopology,

    /// Whether the surfaces are tangent (derivative-aligned but not coincident)
    pub is_tangent: bool,

    /// The tolerance value used in the computation
    pub tolerance_used: T,
}

impl<T: Scalar> IntersectionResult<T> {
    /// Create a new intersection result
    pub fn new(
        geometry: IntersectionGeometry<T>,
        topology: IntersectionTopology,
        is_tangent: bool,
        tolerance_used: T,
    ) -> Self {
        IntersectionResult {
            geometry,
            topology,
            is_tangent,
            tolerance_used,
        }
    }

    /// Create a "no intersection" result
    pub fn disjoint(tolerance: T) -> Self {
        IntersectionResult {
            geometry: IntersectionGeometry::None,
            topology: IntersectionTopology::Disjoint,
            is_tangent: false,
            tolerance_used: tolerance,
        }
    }

    /// Create a "single point intersection" result
    pub fn point(point: Point3D<T>, is_tangent: bool, tolerance: T) -> Self {
        IntersectionResult {
            geometry: IntersectionGeometry::Point(point),
            topology: IntersectionTopology::Crossing,
            is_tangent,
            tolerance_used: tolerance,
        }
    }

    /// Create a "multiple points intersection" result
    pub fn points(points: Vec<Point3D<T>>, is_tangent: bool, tolerance: T) -> Self {
        IntersectionResult {
            geometry: IntersectionGeometry::Points(points),
            topology: if is_tangent {
                IntersectionTopology::Touching
            } else {
                IntersectionTopology::Crossing
            },
            is_tangent,
            tolerance_used: tolerance,
        }
    }

    /// Check if shapes intersect
    pub fn intersects(&self) -> bool {
        self.topology.intersects()
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        format!(
            "{} ({}, tangent: {}, tol: {:?})",
            self.geometry.description(),
            self.topology.description(),
            self.is_tangent,
            self.tolerance_used
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_dimension() {
        assert_eq!(IntersectionGeometry::<f64>::None.dimension(), -1);
        assert_eq!(
            IntersectionGeometry::Point(Point3D::new(0.0, 0.0, 0.0)).dimension(),
            0
        );
    }

    #[test]
    fn topology_intersects() {
        assert!(!IntersectionTopology::Disjoint.intersects());
        assert!(IntersectionTopology::Touching.intersects());
        assert!(IntersectionTopology::Crossing.intersects());
        assert!(IntersectionTopology::Coincident.intersects());
    }

    #[test]
    fn result_disjoint() {
        let result = IntersectionResult::disjoint(1e-9);
        assert!(!result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn result_point() {
        let pt = Point3D::new(1.0, 2.0, 3.0);
        let result = IntersectionResult::point(pt, false, 1e-9);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
    }
}
