//! CompositeCurve3D: A curve composed of multiple connected segments
//!
//! Naming: Follows Solidworks conv ention (Rhino uses PolyCurve, ISO STEP uses CompositeCurve)

use geo_contracts::Scalar;
use geo_primitives::{Arc3D, LineSegment3D, Point3D, Vector3D};

/// Individual curve segment in a composite curve
#[derive(Clone, Debug)]
pub enum CurveSegment3D<T: Scalar> {
    /// Line segment from start to end point
    Line(LineSegment3D<T>),
    /// Arc segment (circular arc in 3D)
    Arc(Arc3D<T>),
    // Future: Nurbs(NurbsCurve3D<T>)  ← add after geo_nurbs dependency is confirmed
}

impl<T: Scalar> CurveSegment3D<T> {
    /// Get the start point of this segment
    pub fn start(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.start(),
            Self::Arc(arc) => arc.start_point(),
        }
    }

    /// Get the end point of this segment
    pub fn end(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.end(),
            Self::Arc(arc) => arc.end_point(),
        }
    }

    /// Get the length of this segment
    pub fn length(&self) -> T {
        match self {
            Self::Line(seg) => {
                let vec = Vector3D::from_points(&seg.start(), &seg.end());
                vec.magnitude()
            }
            Self::Arc(arc) => arc.arc_length(),
        }
    }
}

/// Composite curve: multiple curve segments connected at endpoints
///
/// Invariants maintained:
/// - All segments form a continuous curve (end of segment N == start of segment N+1)
/// - At least 1 segment
/// - Segments are ordered sequentially
#[derive(Clone, Debug)]
pub struct CompositeCurve3D<T: Scalar> {
    segments: Vec<CurveSegment3D<T>>,
}

impl<T: Scalar> CompositeCurve3D<T> {
    /// Create a new composite curve from segments
    ///
    /// Returns None if:
    /// - segments is empty
    /// - segments are not connected (endpoint of N != startpoint of N+1)
    pub fn new(segments: Vec<CurveSegment3D<T>>) -> Option<Self> {
        if segments.is_empty() {
            return None;
        }

        // Verify continuity: end of segment N should equal start of segment N+1
        for i in 0..segments.len() - 1 {
            let end = segments[i].end();
            let next_start = segments[i + 1].start();

            // Use explicit coordinate comparison for connectivity check
            // (tolerance-based checks belong in validation layer, not construction)
            if end.x() != next_start.x() || end.y() != next_start.y() || end.z() != next_start.z() {
                return None;
            }
        }

        Some(CompositeCurve3D { segments })
    }

    /// Get the segments of this composite curve
    pub fn segments(&self) -> &[CurveSegment3D<T>] {
        &self.segments
    }

    /// Get start point of the entire composite curve
    pub fn start_point(&self) -> Point3D<T> {
        self.segments[0].start()
    }

    /// Get end point of the entire composite curve
    pub fn end_point(&self) -> Point3D<T> {
        self.segments[self.segments.len() - 1].end()
    }

    /// Get total length of all segments
    pub fn total_length(&self) -> T {
        self.segments
            .iter()
            .map(|seg| seg.length())
            .fold(T::ZERO, |acc, len| acc + len)
    }

    /// Check if this curve is closed (start == end point)
    pub fn is_closed(&self) -> bool {
        let start = self.start_point();
        let end = self.end_point();
        start.x() == end.x() && start.y() == end.y() && start.z() == end.z()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_segment_line_start_end() {
        let seg =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        let curve_seg = CurveSegment3D::Line(seg);

        assert_eq!(curve_seg.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(curve_seg.end(), Point3D::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn curve_segment_line_length() {
        let seg =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(3.0, 4.0, 0.0)).unwrap();
        let curve_seg = CurveSegment3D::Line(seg);

        assert_eq!(curve_seg.length(), 5.0);
    }

    #[test]
    fn composite_curve_single_segment() {
        let seg =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        let composite = CompositeCurve3D::new(vec![CurveSegment3D::Line(seg)]).unwrap();

        assert_eq!(composite.start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.end_point(), Point3D::new(1.0, 0.0, 0.0));
        assert_eq!(composite.segments().len(), 1);
    }

    #[test]
    fn composite_curve_two_segments_connected() {
        let seg1 =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        let seg2 =
            LineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0)).unwrap();
        let composite =
            CompositeCurve3D::new(vec![CurveSegment3D::Line(seg1), CurveSegment3D::Line(seg2)])
                .unwrap();

        assert_eq!(composite.start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.end_point(), Point3D::new(1.0, 1.0, 0.0));
        assert_eq!(composite.segments().len(), 2);
        assert_eq!(composite.total_length(), 2.0);
    }

    #[test]
    fn composite_curve_disconnected_segments_fails() {
        let seg1 =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        let seg2 =
            LineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();

        assert!(CompositeCurve3D::new(vec![
            CurveSegment3D::Line(seg1),
            CurveSegment3D::Line(seg2),
        ])
        .is_none());
    }

    #[test]
    fn composite_curve_empty_fails() {
        assert!(CompositeCurve3D::<f64>::new(vec![]).is_none());
    }

    #[test]
    fn composite_curve_is_closed() {
        let seg1 =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        let seg2 =
            LineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)).unwrap();
        let seg3 =
            LineSegment3D::new(Point3D::new(0.0, 1.0, 0.0), Point3D::new(0.0, 0.0, 0.0)).unwrap();

        let composite = CompositeCurve3D::new(vec![
            CurveSegment3D::Line(seg1),
            CurveSegment3D::Line(seg2),
            CurveSegment3D::Line(seg3),
        ])
        .unwrap();

        assert!(composite.is_closed());
    }
}
