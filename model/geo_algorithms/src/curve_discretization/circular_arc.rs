use crate::{LineSegment3D, Point3D, Scalar};

/// 円弧を polyline 近似する際のデフォルト弦誤差上限（mm）。
pub const DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM: f64 = 0.1;

/// 汎用用途での円弧 1 セグメントあたりの最大角度ステップ。
pub const DEFAULT_CIRCULAR_ARC_MAX_ANGLE_STEP_RAD: f64 = std::f64::consts::PI / 12.0;

/// 円弧を polyline 近似する際の最小分割数。
const MIN_CIRCULAR_ARC_DIVISIONS: usize = 4;

/// 汎用用途での円弧 polyline 近似の最大分割数。
pub const DEFAULT_MAX_CIRCULAR_ARC_DIVISIONS: usize = 128;

/// XY 平面上の円弧方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircularArcDirection {
    Clockwise,
    CounterClockwise,
}

/// 円弧を polyline 近似する際の設定。
///
/// 将来 `ellipse_arc` や `composite_curve` にも同系統の options 型を並べる前提で、
/// shape family ごとに閉じた設定型として扱う。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircularArcPolylineOptions {
    chord_tolerance_mm: f64,
    max_angle_step_rad: Option<f64>,
    min_divisions: usize,
    max_divisions: Option<usize>,
}

impl CircularArcPolylineOptions {
    pub const fn new(chord_tolerance_mm: f64) -> Self {
        Self {
            chord_tolerance_mm,
            max_angle_step_rad: Some(DEFAULT_CIRCULAR_ARC_MAX_ANGLE_STEP_RAD),
            min_divisions: MIN_CIRCULAR_ARC_DIVISIONS,
            max_divisions: Some(DEFAULT_MAX_CIRCULAR_ARC_DIVISIONS),
        }
    }

    /// CAM シミュレーションのように chord tolerance を正本とし、
    /// 追加の角度制約や上限分割を持ち込まない既定値。
    pub const fn simulation_default() -> Self {
        Self {
            chord_tolerance_mm: DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM,
            max_angle_step_rad: None,
            min_divisions: MIN_CIRCULAR_ARC_DIVISIONS,
            max_divisions: None,
        }
    }

    pub const fn chord_tolerance_mm(&self) -> f64 {
        self.chord_tolerance_mm
    }

    pub const fn with_chord_tolerance_mm(mut self, chord_tolerance_mm: f64) -> Self {
        self.chord_tolerance_mm = chord_tolerance_mm;
        self
    }

    pub const fn max_angle_step_rad(&self) -> Option<f64> {
        self.max_angle_step_rad
    }

    pub const fn with_max_angle_step_rad(mut self, max_angle_step_rad: f64) -> Self {
        self.max_angle_step_rad = Some(max_angle_step_rad);
        self
    }

    pub const fn without_max_angle_step(mut self) -> Self {
        self.max_angle_step_rad = None;
        self
    }

    pub const fn min_divisions(&self) -> usize {
        self.min_divisions
    }

    pub const fn with_min_divisions(mut self, min_divisions: usize) -> Self {
        self.min_divisions = min_divisions;
        self
    }

    pub const fn max_divisions(&self) -> Option<usize> {
        self.max_divisions
    }

    pub const fn with_max_divisions(mut self, max_divisions: usize) -> Self {
        self.max_divisions = Some(max_divisions);
        self
    }

    pub const fn without_max_divisions(mut self) -> Self {
        self.max_divisions = None;
        self
    }
}

impl Default for CircularArcPolylineOptions {
    fn default() -> Self {
        Self::new(DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM)
    }
}

/// XY 平面上の円弧を polyline 近似して線分列へ展開する。
///
/// Z は始点・終点間で線形補間するため、ヘリカルな円弧にも利用できる。
pub fn circular_arc_to_polyline<T: Scalar>(
    start: Point3D<T>,
    end: Point3D<T>,
    center: Point3D<T>,
    direction: CircularArcDirection,
    options: CircularArcPolylineOptions,
) -> Vec<LineSegment3D<T>> {
    let chord_tolerance_mm = options.chord_tolerance_mm();
    let dx1 = (start.x() - center.x()).to_f64();
    let dy1 = (start.y() - center.y()).to_f64();
    let dx2 = (end.x() - center.x()).to_f64();
    let dy2 = (end.y() - center.y()).to_f64();

    let radius = (dx1 * dx1 + dy1 * dy1).sqrt();
    if radius < f64::EPSILON {
        return Vec::new();
    }

    let angle_start = dy1.atan2(dx1);
    let angle_end = dy2.atan2(dx2);

    let sweep = match direction {
        CircularArcDirection::CounterClockwise => {
            let mut value = angle_end - angle_start;
            if value < f64::EPSILON {
                value += std::f64::consts::TAU;
            }
            value
        }
        CircularArcDirection::Clockwise => {
            let mut value = angle_start - angle_end;
            if value < f64::EPSILON {
                value += std::f64::consts::TAU;
            }
            value
        }
    };

    let chord_based_divisions = if chord_tolerance_mm > f64::EPSILON {
        let cos_val = (1.0 - chord_tolerance_mm / radius).clamp(-1.0, 1.0);
        let half_angle = cos_val.acos();
        if half_angle > f64::EPSILON {
            (sweep / (2.0 * half_angle)).ceil() as usize
        } else {
            1
        }
    } else {
        1
    };

    let angle_based_divisions = match options.max_angle_step_rad() {
        Some(max_angle_step_rad) if max_angle_step_rad > f64::EPSILON => {
            (sweep / max_angle_step_rad).ceil() as usize
        }
        _ => 1,
    };

    let min_divisions = options.min_divisions().max(MIN_CIRCULAR_ARC_DIVISIONS);
    let mut divisions = chord_based_divisions
        .max(angle_based_divisions)
        .max(min_divisions);

    if let Some(max_divisions) = options.max_divisions() {
        divisions = divisions.min(max_divisions.max(min_divisions));
    }

    let cx = center.x().to_f64();
    let cy = center.y().to_f64();
    let z_start = start.z().to_f64();
    let z_end = end.z().to_f64();
    let direction_sign = match direction {
        CircularArcDirection::CounterClockwise => 1.0_f64,
        CircularArcDirection::Clockwise => -1.0_f64,
    };
    let angle_step = direction_sign * sweep / divisions as f64;

    let mut segments = Vec::with_capacity(divisions);
    let mut prev = start;
    for index in 1..=divisions {
        let next = if index == divisions {
            end
        } else {
            let angle = angle_start + angle_step * index as f64;
            Point3D::new(
                T::from_f64(cx + radius * angle.cos()),
                T::from_f64(cy + radius * angle.sin()),
                T::from_f64(z_start + (z_end - z_start) * (index as f64 / divisions as f64)),
            )
        };
        if let Some(line) = LineSegment3D::new(prev, next) {
            segments.push(line);
        }
        prev = next;
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::{
        circular_arc_to_polyline, CircularArcDirection, CircularArcPolylineOptions,
        DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM, DEFAULT_CIRCULAR_ARC_MAX_ANGLE_STEP_RAD,
        DEFAULT_MAX_CIRCULAR_ARC_DIVISIONS,
    };
    use crate::Point3D;

    #[test]
    fn circular_arc_to_polyline_preserves_endpoints() {
        let start = Point3D::new(1.0_f64, 0.0, 0.0);
        let end = Point3D::new(0.0_f64, 1.0, 1.0);
        let center = Point3D::new(0.0_f64, 0.0, 0.0);

        let segments = circular_arc_to_polyline(
            start,
            end,
            center,
            CircularArcDirection::CounterClockwise,
            CircularArcPolylineOptions::default(),
        );

        assert!(!segments.is_empty());
        assert_eq!(segments.first().unwrap().constraint_start_point(), start);
        assert_eq!(segments.last().unwrap().constraint_end_point(), end);
    }

    #[test]
    fn circular_arc_polyline_options_default_uses_current_tolerance() {
        assert_eq!(
            CircularArcPolylineOptions::default().chord_tolerance_mm(),
            DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM,
        );
        assert_eq!(
            CircularArcPolylineOptions::default().max_angle_step_rad(),
            Some(DEFAULT_CIRCULAR_ARC_MAX_ANGLE_STEP_RAD),
        );
        assert_eq!(
            CircularArcPolylineOptions::default().max_divisions(),
            Some(DEFAULT_MAX_CIRCULAR_ARC_DIVISIONS),
        );
    }

    #[test]
    fn circular_arc_polyline_options_simulation_default_disables_angle_and_cap() {
        assert_eq!(
            CircularArcPolylineOptions::simulation_default().max_angle_step_rad(),
            None,
        );
        assert_eq!(
            CircularArcPolylineOptions::simulation_default().max_divisions(),
            None,
        );
    }

    #[test]
    fn circular_arc_to_polyline_respects_max_divisions_cap() {
        let start = Point3D::new(100.0_f64, 0.0, 0.0);
        let end = Point3D::new(-100.0_f64, 0.0, 0.0);
        let center = Point3D::new(0.0_f64, 0.0, 0.0);

        let segments = circular_arc_to_polyline(
            start,
            end,
            center,
            CircularArcDirection::CounterClockwise,
            CircularArcPolylineOptions::simulation_default()
                .with_chord_tolerance_mm(0.001)
                .with_max_divisions(12),
        );

        assert_eq!(segments.len(), 12);
    }

    #[test]
    fn circular_arc_to_polyline_respects_max_angle_step() {
        let start = Point3D::new(1.0_f64, 0.0, 0.0);
        let end = Point3D::new(0.0_f64, 1.0, 0.0);
        let center = Point3D::new(0.0_f64, 0.0, 0.0);

        let segments = circular_arc_to_polyline(
            start,
            end,
            center,
            CircularArcDirection::CounterClockwise,
            CircularArcPolylineOptions::simulation_default()
                .with_max_angle_step_rad(std::f64::consts::FRAC_PI_4)
                .without_max_divisions(),
        );

        assert_eq!(segments.len(), 4);
    }
}
