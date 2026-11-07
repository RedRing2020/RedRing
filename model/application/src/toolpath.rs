//! ツールパスの値の型
//!
//! ツールパスと、その経路の区間・高さごとの経路・区間の種類・送りの向きの型。経路の点・線分の型と、そのメソッドを
//! 呼ぶための trait と、円弧の区間を線分の列にする問い合わせ。

use geo_algorithms::{CircularArcDirection, circular_arc_to_polyline};
pub use geo_algorithms::{CircularArcPolylineOptions, LineSegment3D, Point3D};
pub use geo_contracts::{
    LineSegment3DConstructor, LineSegment3DProperties, LineSegment3DSupportLine,
};

pub use cam_core::{
    ArcDirection, ContourLevelPath, CuttingDirection, PathGeometry, PathSegment, SegmentType,
    ToolPath,
};

/// 円弧の区間（始点 `start`・終点 `end`・中心 `center`・向き `direction`）を、`options` の弦の誤差と分割の数の範囲で
/// 線分の列にする
pub fn arc_to_line_segments(
    start: Point3D<f64>,
    end: Point3D<f64>,
    center: Point3D<f64>,
    direction: ArcDirection,
    options: CircularArcPolylineOptions,
) -> Vec<LineSegment3D<f64>> {
    let direction = match direction {
        ArcDirection::Clockwise => CircularArcDirection::Clockwise,
        ArcDirection::CounterClockwise => CircularArcDirection::CounterClockwise,
    };
    circular_arc_to_polyline(start, end, center, direction, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (2, 0) から (0, 2) への、中心 (0, 0) の円弧の線分の終点
    fn arc_points(direction: ArcDirection) -> Vec<Point3D<f64>> {
        arc_to_line_segments(
            Point3D::new(2.0, 0.0, 0.0),
            Point3D::new(0.0, 2.0, 0.0),
            Point3D::new(0.0, 0.0, 0.0),
            direction,
            CircularArcPolylineOptions::default(),
        )
        .iter()
        .map(|segment| segment.end_point())
        .collect()
    }

    #[test]
    fn arc_directions_choose_the_side_of_the_circle() {
        // 反時計回りは 4 分の 1 周で、途中の点は第 1 象限にある
        let counter_clockwise = arc_points(ArcDirection::CounterClockwise);
        let middle = counter_clockwise[counter_clockwise.len() / 2 - 1];
        assert!(middle.x() > 0.0 && middle.y() > 0.0, "{middle:?}");
        // 時計回りは 4 分の 3 周で、x が負の側を通る
        let clockwise = arc_points(ArcDirection::Clockwise);
        assert!(clockwise.iter().any(|point| point.x() < -1.0));
        assert!(clockwise.len() > counter_clockwise.len());
    }
}
