//! 走査線の向きの座標（走査線を X 軸に沿わせる、Z 軸のまわりの回転）と、その座標での走査の範囲
//!
//! 走査線の角度 θ（X 軸から反時計回り）のパスは、形状を Z 軸のまわりに −θ 回した座標で 0° の走査線として作り、
//! 点を +θ 回して戻す。0° では回転は恒等で、計算は回さない場合と同じになる。

use geo_algorithms::Point3D;
use geo_contracts::default_distance_tolerance;

/// 走査線の向きの座標への回転
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScanFrame {
    cos: f64,
    sin: f64,
}

impl ScanFrame {
    /// 走査線の角度 `degrees`（X 軸から反時計回り）の座標
    pub(crate) fn from_degrees(degrees: f64) -> Self {
        // 0° は、cos と sin がちょうど 1 と 0 になり、回転が恒等になる
        let (sin, cos) = degrees.to_radians().sin_cos();
        Self { cos, sin }
    }

    /// 回転が恒等か（0° の走査線）
    pub(crate) fn is_identity(self) -> bool {
        self.cos == 1.0 && self.sin == 0.0
    }

    /// 加工の座標の XY を、走査線の向きの座標へ
    pub(crate) fn to_scan(self, [x, y]: [f64; 2]) -> [f64; 2] {
        [x * self.cos + y * self.sin, y * self.cos - x * self.sin]
    }

    /// 走査線の向きの座標の点を、加工の座標へ（高さは変えない）
    pub(crate) fn to_work(self, point: Point3D<f64>) -> Point3D<f64> {
        let (x, y) = (point.x(), point.y());
        Point3D::new(
            x * self.cos - y * self.sin,
            y * self.cos + x * self.sin,
            point.z(),
        )
    }
}

/// 走査線の向きの座標での走査の範囲（凸の多角形。角を順に並べる）
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScanRegion {
    corners: Vec<[f64; 2]>,
}

impl ScanRegion {
    /// 軸に平行な長方形 [min, max]
    #[cfg(test)]
    pub(crate) fn rectangle(min: [f64; 2], max: [f64; 2]) -> Self {
        Self {
            corners: vec![min, [max[0], min[1]], max, [min[0], max[1]]],
        }
    }

    /// 凸の多角形 `corners` を、軸に平行な長方形 [min, max] で切り取った範囲（重ならなければ角のない範囲）
    ///
    /// 長方形の辺の上の角は、辺の座標の値をそのまま使う（軸に平行な長方形どうしでは、角は元の座標の値になる）。
    pub(crate) fn clipped(corners: &[[f64; 2]], min: [f64; 2], max: [f64; 2]) -> Self {
        let mut corners = corners.to_vec();
        for axis in 0..2 {
            corners = clip_by_half_plane(&corners, axis, min[axis], true);
            corners = clip_by_half_plane(&corners, axis, max[axis], false);
        }
        Self { corners }
    }

    /// 範囲の XY の幅（X の幅, Y の幅）。角がなければ `None`
    pub(crate) fn extents(&self) -> Option<[f64; 2]> {
        let (min, max) = self.bounds()?;
        Some([max[0] - min[0], max[1] - min[1]])
    }

    /// 範囲の面積
    pub(crate) fn area(&self) -> f64 {
        let twice: f64 = self
            .corners
            .iter()
            .zip(self.corners.iter().cycle().skip(1))
            .map(|(start, end)| start[0] * end[1] - end[0] * start[1])
            .sum();
        0.5 * twice.abs()
    }

    /// 範囲の Y の範囲（min, max）
    pub(crate) fn y_range(&self) -> Option<(f64, f64)> {
        let (min, max) = self.bounds()?;
        Some((min[1], max[1]))
    }

    /// 走査線 y が範囲を通る X の範囲（min, max）。通らなければ `None`
    ///
    /// Y の差が距離トレランス以下の角は、走査線の上にあるとする（回して軸に平行からわずかにずれた辺の上の走査線が、
    /// 1 点にならない）。
    pub(crate) fn x_range_at(&self, y: f64) -> Option<(f64, f64)> {
        let tolerance = default_distance_tolerance::<f64>();
        let mut range: Option<(f64, f64)> = None;
        let mut include = |x: f64| {
            range = Some(range.map_or((x, x), |(low, high)| (low.min(x), high.max(x))));
        };
        for (index, &start) in self.corners.iter().enumerate() {
            let end = self.corners[(index + 1) % self.corners.len()];
            if (start[1] - y).abs() <= tolerance {
                include(start[0]);
            }
            if (start[1] < y && y < end[1]) || (end[1] < y && y < start[1]) {
                let t = (y - start[1]) / (end[1] - start[1]);
                include(start[0] + t * (end[0] - start[0]));
            }
        }
        range
    }

    fn bounds(&self) -> Option<([f64; 2], [f64; 2])> {
        let first = *self.corners.first()?;
        Some(
            self.corners
                .iter()
                .fold((first, first), |(min, max), corner| {
                    (
                        [min[0].min(corner[0]), min[1].min(corner[1])],
                        [max[0].max(corner[0]), max[1].max(corner[1])],
                    )
                }),
        )
    }
}

/// 凸の多角形 `corners` のうち、座標 `axis` が `value` 以上（`keep_above`）か以下の部分
fn clip_by_half_plane(
    corners: &[[f64; 2]],
    axis: usize,
    value: f64,
    keep_above: bool,
) -> Vec<[f64; 2]> {
    let inside = |corner: [f64; 2]| {
        if keep_above {
            corner[axis] >= value
        } else {
            corner[axis] <= value
        }
    };
    let mut clipped = Vec::new();
    for (index, &start) in corners.iter().enumerate() {
        let end = corners[(index + 1) % corners.len()];
        if inside(start) {
            clipped.push(start);
        }
        if inside(start) != inside(end) {
            let t = (value - start[axis]) / (end[axis] - start[axis]);
            let other = 1 - axis;
            let mut crossing = [0.0; 2];
            crossing[axis] = value;
            crossing[other] = start[other] + t * (end[other] - start[other]);
            clipped.push(crossing);
        }
    }
    clipped
}

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::consts::test_constants::TOLERANCE_F64 as EPS;

    #[test]
    fn zero_degrees_is_the_identity() {
        let frame = ScanFrame::from_degrees(0.0);
        assert!(frame.is_identity());
        assert_eq!(frame.to_scan([1.5, -2.25]), [1.5, -2.25]);
        let point = Point3D::new(1.5, -2.25, 3.0);
        assert_eq!(frame.to_work(point), point);
    }

    #[test]
    fn scan_and_work_coordinates_are_inverse_rotations() {
        let frame = ScanFrame::from_degrees(90.0);
        assert!(!frame.is_identity());
        // 90° の走査線は Y 軸に沿う: 加工の座標の +Y が、走査線の向きの座標の +X になる
        let [x, y] = frame.to_scan([0.0, 1.0]);
        assert!((x - 1.0).abs() < EPS && y.abs() < EPS);
        let frame = ScanFrame::from_degrees(30.0);
        let [x, y] = frame.to_scan([2.0, -1.0]);
        let back = frame.to_work(Point3D::new(x, y, 4.0));
        assert!((back.x() - 2.0).abs() < EPS && (back.y() + 1.0).abs() < EPS);
        assert_eq!(back.z(), 4.0);
    }

    #[test]
    fn clipping_axis_aligned_rectangles_keeps_the_coordinates() {
        let region = ScanRegion::clipped(
            &ScanRegion::rectangle([-1.0, 0.5], [3.0, 7.0]).corners,
            [0.0, 0.0],
            [2.0, 5.0],
        );
        assert_eq!(region.extents(), Some([2.0, 4.5]));
        assert_eq!(region.area(), 9.0);
        assert_eq!(region.y_range(), Some((0.5, 5.0)));
        assert_eq!(region.x_range_at(0.5), Some((0.0, 2.0)));
        assert_eq!(region.x_range_at(3.0), Some((0.0, 2.0)));
        assert_eq!(region.x_range_at(5.5), None);
    }

    #[test]
    fn disjoint_rectangles_have_no_corners() {
        let region = ScanRegion::clipped(
            &ScanRegion::rectangle([3.0, 0.0], [4.0, 1.0]).corners,
            [0.0, 0.0],
            [2.0, 1.0],
        );
        assert_eq!(region.extents(), None);
    }

    #[test]
    fn scanlines_cross_a_tilted_square_over_its_width_at_that_height() {
        // 中心 (0, 0)、対角線の半分が 1 のひし形
        let region = ScanRegion::clipped(
            &[[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]],
            [-10.0, -10.0],
            [10.0, 10.0],
        );
        assert_eq!(region.y_range(), Some((-1.0, 1.0)));
        assert_eq!(region.x_range_at(0.0), Some((-1.0, 1.0)));
        assert_eq!(region.x_range_at(0.5), Some((-0.5, 0.5)));
        assert_eq!(region.x_range_at(1.0), Some((0.0, 0.0)));
    }
}
