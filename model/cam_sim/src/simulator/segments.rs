use cam_core::{ArcDirection, PathGeometry, PathSegment, ToolPath};

use crate::error::SimulationError;

/// Arc を polyline 近似する際のデフォルト弦誤差上限（mm）。
pub(super) const DEFAULT_ARC_CHORD_TOLERANCE_MM: f64 = 0.1;

/// Arc を polyline 近似する際の最小分割数。
const MIN_ARC_DIVISIONS: usize = 4;

impl<T: geo_algorithms::Scalar> super::CuttingSimulator<T> {
    /// 線分長を `f64` で計算する（距離ベース間隔計算用）。
    pub(super) fn segment_length(&self, segment: &geo_algorithms::LineSegment3D<T>) -> f64 {
        let dx = segment.end().x() - segment.start().x();
        let dy = segment.end().y() - segment.start().y();
        let dz = segment.end().z() - segment.start().z();
        (dx * dx + dy * dy + dz * dz).sqrt().to_f64()
    }

    /// ToolPathから線分セグメントを抽出し、切削フラグ付きで返す。
    ///
    /// Arc は polyline 近似して展開する。
    pub(super) fn collect_line_segments(
        &self,
        toolpath: &ToolPath<T>,
    ) -> Result<Vec<(geo_algorithms::LineSegment3D<T>, bool)>, SimulationError> {
        let mut out = Vec::new();

        for segment in &toolpath.approach_segments {
            self.push_segment(segment, false, &mut out)?;
        }

        for contour in &toolpath.contour_levels {
            for segment in &contour.segments {
                self.push_segment(segment, segment.is_cutting(), &mut out)?;
            }
        }

        for segment in &toolpath.retract_segments {
            self.push_segment(segment, false, &mut out)?;
        }

        Ok(out)
    }

    /// PathSegmentをLineSegmentへ変換して出力に追加する。
    fn push_segment(
        &self,
        segment: &PathSegment<T>,
        is_cutting: bool,
        out: &mut Vec<(geo_algorithms::LineSegment3D<T>, bool)>,
    ) -> Result<(), SimulationError> {
        match segment.geometry {
            PathGeometry::Line { end } => {
                if let Some(line_segment) = geo_algorithms::LineSegment3D::new(segment.start, end) {
                    out.push((line_segment, is_cutting));
                }
                Ok(())
            }
            PathGeometry::Arc {
                end,
                center,
                direction,
            } => {
                for line in arc_to_polyline(
                    segment.start,
                    end,
                    center,
                    direction,
                    self.arc_chord_tolerance_mm,
                ) {
                    out.push((line, is_cutting));
                }
                Ok(())
            }
        }
    }
}

/// Arc セグメントを polyline 近似して LineSegment3D の Vec に変換する。
///
/// XY 平面上の円弧を想定し、Z は始点・終点間で線形補間する（ヘリカル対応）。
/// 弦誤差 `chord_tolerance_mm` に基づいて分割数を決定する。
fn arc_to_polyline<T: geo_algorithms::Scalar>(
    start: geo_algorithms::Point3D<T>,
    end: geo_algorithms::Point3D<T>,
    center: geo_algorithms::Point3D<T>,
    direction: ArcDirection,
    chord_tolerance_mm: f64,
) -> Vec<geo_algorithms::LineSegment3D<T>> {
    let dx1 = (start.x() - center.x()).to_f64();
    let dy1 = (start.y() - center.y()).to_f64();
    let dx2 = (end.x() - center.x()).to_f64();
    let dy2 = (end.y() - center.y()).to_f64();

    let r = (dx1 * dx1 + dy1 * dy1).sqrt();
    if r < f64::EPSILON {
        return Vec::new();
    }

    let angle_start = dy1.atan2(dx1);
    let angle_end = dy2.atan2(dx2);

    // 方向に従い弧角を [0, 2π] の範囲で計算する（0 は全周として扱う）。
    let sweep = match direction {
        ArcDirection::CounterClockwise => {
            let mut s = angle_end - angle_start;
            if s < f64::EPSILON {
                s += std::f64::consts::TAU;
            }
            s
        }
        ArcDirection::Clockwise => {
            let mut s = angle_start - angle_end;
            if s < f64::EPSILON {
                s += std::f64::consts::TAU;
            }
            s
        }
    };

    let n = if chord_tolerance_mm > f64::EPSILON {
        let cos_val = (1.0 - chord_tolerance_mm / r).clamp(-1.0, 1.0);
        let half_angle = cos_val.acos();
        let divisions = (sweep / (2.0 * half_angle)).ceil() as usize;
        divisions.max(MIN_ARC_DIVISIONS)
    } else {
        MIN_ARC_DIVISIONS
    };

    let cx = center.x().to_f64();
    let cy = center.y().to_f64();
    let z_start = start.z().to_f64();
    let z_end = end.z().to_f64();

    let sign = match direction {
        ArcDirection::CounterClockwise => 1.0_f64,
        ArcDirection::Clockwise => -1.0_f64,
    };
    let angle_step = sign * sweep / n as f64;

    let mut segments = Vec::with_capacity(n);
    let mut prev = start;
    for i in 1..=n {
        let angle = angle_start + angle_step * i as f64;
        let next = geo_algorithms::Point3D::new(
            T::from_f64(cx + r * angle.cos()),
            T::from_f64(cy + r * angle.sin()),
            T::from_f64(z_start + (z_end - z_start) * (i as f64 / n as f64)),
        );
        if let Some(line) = geo_algorithms::LineSegment3D::new(prev, next) {
            segments.push(line);
        }
        prev = next;
    }
    segments
}
