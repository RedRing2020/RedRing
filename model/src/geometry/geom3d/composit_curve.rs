use super::curve_segment::CurveSegment;

/// Represents a composite curve composed of multiple curve segments.
/// Used for trimmed boundaries, sketch loops, and profile definitions.
#[derive(Debug, Clone)]
pub struct CompositeCurve {
    segments: Vec<CurveSegment>,
}

impl Curve3 for CompositeCurve {
    fn kind(&self) -> CurveKind3 {
        CurveKind3::CompositeCurve
    }
}

impl CompositeCurve {
    /// Creates a new composite curve from segments.
    pub fn new(segments: Vec<CurveSegment>) -> Self {
        Self { segments }
    }

    /// Returns the segments.
    pub fn segments(&self) -> &[CurveSegment] {
        &self.segments
    }

    /// Returns true if the curve is closed (start == end within tolerance).
    pub fn is_closed(&self, tolerance: f64) -> bool {
        if self.segments.is_empty() {
            return false;
        }

        let first = self.segments.first().unwrap().start_point();
        let last = self.segments.last().unwrap().end_point();
        first.distance_to(&last) <= tolerance
    }

    /// Returns the total number of segments.
    pub fn num_segments(&self) -> usize {
        self.segments.len()
    }

    /// Returns true if all segments are geometrically connected (G0 continuity).
    pub fn is_g0_continuous(&self, tolerance: f64) -> bool {
        self.segments
            .windows(2)
            .all(|pair| {
                let a = pair[0].end_point();
                let b = pair[1].start_point();
                a.distance_to(&b) <= tolerance
            })
    }
}