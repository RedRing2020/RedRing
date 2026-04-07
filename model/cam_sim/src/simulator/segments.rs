use cam_core::{ArcDirection, PathGeometry, PathSegment, ToolPath};
use geo_algorithms::{
    CircularArcDirection, CircularArcPolylineOptions, LineSegment3D, Scalar,
    circular_arc_to_polyline,
};

use crate::error::SimulationError;

/// ToolPathから線分セグメントを抽出し、切削フラグ付きで返す。
///
/// Arc は polyline 近似して展開する。
pub fn collect_toolpath_line_segments<T: Scalar>(
    toolpath: &ToolPath<T>,
    arc_chord_tolerance_mm: f64,
) -> Vec<(LineSegment3D<T>, bool)> {
    collect_toolpath_line_segments_with_arc_options(
        toolpath,
        CircularArcPolylineOptions::simulation_default()
            .with_chord_tolerance_mm(arc_chord_tolerance_mm),
    )
}

/// ToolPathから線分セグメントを抽出し、切削フラグ付きで返す。
///
/// Arc の polyline 近似設定を呼び出し側から明示したい用途向けの入口。
pub fn collect_toolpath_line_segments_with_arc_options<T: Scalar>(
    toolpath: &ToolPath<T>,
    arc_options: CircularArcPolylineOptions,
) -> Vec<(LineSegment3D<T>, bool)> {
    let mut out = Vec::new();

    for segment in &toolpath.approach_segments {
        push_path_segment_lines(segment, false, arc_options, &mut out);
    }

    for contour in &toolpath.contour_levels {
        for segment in &contour.segments {
            push_path_segment_lines(segment, segment.is_cutting(), arc_options, &mut out);
        }
    }

    for segment in &toolpath.retract_segments {
        push_path_segment_lines(segment, false, arc_options, &mut out);
    }

    out
}

fn push_path_segment_lines<T: Scalar>(
    segment: &PathSegment<T>,
    is_cutting: bool,
    arc_options: CircularArcPolylineOptions,
    out: &mut Vec<(LineSegment3D<T>, bool)>,
) {
    match segment.geometry {
        PathGeometry::Line { end } => {
            if let Some(line_segment) = LineSegment3D::new(segment.start, end) {
                out.push((line_segment, is_cutting));
            }
        }
        PathGeometry::Arc {
            end,
            center,
            direction,
        } => {
            let direction = match direction {
                ArcDirection::Clockwise => CircularArcDirection::Clockwise,
                ArcDirection::CounterClockwise => CircularArcDirection::CounterClockwise,
            };
            for line in circular_arc_to_polyline(segment.start, end, center, direction, arc_options)
            {
                out.push((line, is_cutting));
            }
        }
    }
}

impl<T: Scalar> super::CuttingSimulator<T> {
    /// 線分長を `f64` で計算する（距離ベース間隔計算用）。
    pub(super) fn segment_length(&self, segment: &LineSegment3D<T>) -> f64 {
        let start = segment.start();
        let end = segment.end();
        start.distance_to(&end).to_f64()
    }

    /// ToolPathから線分セグメントを抽出し、切削フラグ付きで返す。
    ///
    /// Arc は polyline 近似して展開する。
    pub(super) fn collect_line_segments(
        &self,
        toolpath: &ToolPath<T>,
    ) -> Result<Vec<(LineSegment3D<T>, bool)>, SimulationError> {
        Ok(collect_toolpath_line_segments(
            toolpath,
            self.arc_chord_tolerance_mm,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_toolpath_line_segments, collect_toolpath_line_segments_with_arc_options};
    use cam_core::{
        ArcDirection, ContourLevelPath, CuttingDirection, PathSegment, SegmentType, ToolPath,
    };
    use geo_algorithms::{
        CircularArcPolylineOptions, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM, Point3D,
    };

    #[test]
    fn collect_toolpath_line_segments_expands_arc_and_keeps_flags() {
        let cutting_arc = PathSegment {
            start: Point3D::new(1.0_f64, 0.0, 0.0),
            geometry: cam_core::PathGeometry::Arc {
                end: Point3D::new(0.0_f64, 1.0, 0.0),
                center: Point3D::new(0.0_f64, 0.0, 0.0),
                direction: ArcDirection::CounterClockwise,
            },
            segment_type: SegmentType::Cutting { feed_rate: 100.0 },
            ext_attributes: Vec::new(),
        };

        let toolpath = ToolPath::new(
            "tool".to_string(),
            CuttingDirection::Down,
            Vec::new(),
            vec![ContourLevelPath::new(0, 0.0, vec![cutting_arc])],
            Vec::new(),
        );

        let segments =
            collect_toolpath_line_segments(&toolpath, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM);

        assert!(!segments.is_empty());
        assert!(segments.iter().all(|(_, is_cutting)| *is_cutting));
    }

    #[test]
    fn collect_toolpath_line_segments_with_arc_options_accepts_display_default() {
        let cutting_arc = PathSegment {
            start: Point3D::new(1.0_f64, 0.0, 0.0),
            geometry: cam_core::PathGeometry::Arc {
                end: Point3D::new(0.0_f64, 1.0, 0.0),
                center: Point3D::new(0.0_f64, 0.0, 0.0),
                direction: ArcDirection::CounterClockwise,
            },
            segment_type: SegmentType::Cutting { feed_rate: 100.0 },
            ext_attributes: Vec::new(),
        };

        let toolpath = ToolPath::new(
            "tool".to_string(),
            CuttingDirection::Down,
            Vec::new(),
            vec![ContourLevelPath::new(0, 0.0, vec![cutting_arc])],
            Vec::new(),
        );

        let segments = collect_toolpath_line_segments_with_arc_options(
            &toolpath,
            CircularArcPolylineOptions::default(),
        );

        assert!(!segments.is_empty());
        assert!(segments.iter().all(|(_, is_cutting)| *is_cutting));
    }
}
