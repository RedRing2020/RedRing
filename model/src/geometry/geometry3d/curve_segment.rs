use std::sync::Arc;
use super::point::Point;
use super::curve::curve_trait::Curve3;

/// Represents a single segment of a composite curve.
/// Holds a reference to a curve, its trim range, and direction.
#[derive(Debug, Clone)]
pub struct CurveSegment {
    curve: Arc<dyn Curve3>,  // Curve instance (NURBS, Line, Arc, etc.)
    start_param: f64,         // Trim start parameter
    end_param: f64,           // Trim end parameter
    reversed: bool,           // Whether the segment is reversed
}

impl CurveSegment {
    /// Creates a new curve segment.
    pub fn new(
        curve: Arc<dyn Curve3>,
        start_param: f64,
        end_param: f64,
        reversed: bool,
    ) -> Option<Self> {
        if start_param >= end_param {
            return None;
        }
        Some(Self {
            curve,
            start_param,
            end_param,
            reversed,
        })
    }

    /// Returns the underlying curve.
    pub fn curve(&self) -> &Arc<dyn Curve3> {
        &self.curve
    }

    /// Returns the trim range.
    pub fn trim_range(&self) -> (f64, f64) {
        if self.reversed {
            (self.end_param, self.start_param)
        } else {
            (self.start_param, self.end_param)
        }
    }

    /// Returns true if the segment is reversed.
    pub fn is_reversed(&self) -> bool {
        self.reversed
    }

    /// Returns the start point of the segment.
    pub fn start_point(&self) -> Point {
        let (t0, _) = self.trim_range();
        self.curve.evaluate(t0)
    }

    /// Returns the end point of the segment.
    pub fn end_point(&self) -> Point {
        let (_, t1) = self.trim_range();
        self.curve.evaluate(t1)
    }

    /// Returns the length of the segment (approximate or exact).
    pub fn length(&self) -> f64 {
        self.curve.length_between(self.start_param, self.end_param)
    }

    /// Returns a reversed copy of the segment.
    pub fn reversed(&self) -> Self {
        Self {
            curve: Arc::clone(&self.curve),
            start_param: self.start_param,
            end_param: self.end_param,
            reversed: !self.reversed,
        }
    }
}