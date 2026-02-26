use cam_core::{PathGeometry, PathSegment, ToolPath};

use crate::error::SimulationError;

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
    /// ArcはPhase 1aの対象外のためエラーとする。
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
            PathGeometry::Arc { .. } => Err(SimulationError::UnsupportedGeometry),
        }
    }
}
