//! CompositeCurve3D: 複数セグメントから構成される複合曲線
//!
//! 命名は Solidworks 系の CompositeCurve に合わせる
//! （Rhino: PolyCurve、ISO STEP: CompositeCurve）

use crate::{Point3D, TopoArc3D, TopoLineSegment3D, TopoNurbsCurve3D, Vector3D};
use geo_contracts::Scalar;

/// 複合曲線を構成する個別セグメント
#[derive(Clone, Debug)]
pub enum CurveSegment3D<T: Scalar> {
    /// 始点から終点を持つ線分セグメント
    Line(TopoLineSegment3D<T>),
    /// 3D円弧セグメント
    Arc(TopoArc3D<T>),
    /// 3D NURBS曲線セグメント（native parameter semantics）
    Nurbs(TopoNurbsCurve3D<T>),
}

impl<T: Scalar> CurveSegment3D<T> {
    /// セグメントの ideal start endpoint を返す
    pub fn start(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.start(),
            Self::Arc(arc) => arc.start_point(),
            Self::Nurbs(curve) => {
                let (u_min, _) = curve.parameter_domain();
                let p = curve.evaluate_at(u_min);
                Point3D::new(p.x(), p.y(), p.z())
            }
        }
    }

    /// セグメントの ideal end endpoint を返す
    pub fn end(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.end(),
            Self::Arc(arc) => arc.end_point(),
            Self::Nurbs(curve) => {
                let (_, u_max) = curve.parameter_domain();
                let p = curve.evaluate_at(u_max);
                Point3D::new(p.x(), p.y(), p.z())
            }
        }
    }

    /// topology 接続に使う拘束始点を返す
    pub fn constraint_start(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.constraint_start_point(),
            Self::Arc(arc) => arc.start_point(),
            Self::Nurbs(_) => self.start(),
        }
    }

    /// topology 接続に使う拘束終点を返す
    pub fn constraint_end(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.constraint_end_point(),
            Self::Arc(arc) => arc.end_point(),
            Self::Nurbs(_) => self.end(),
        }
    }

    /// セグメント長を返す
    pub fn length(&self) -> T {
        match self {
            Self::Line(seg) => {
                let vec = Vector3D::from_points(&seg.start(), &seg.end());
                vec.magnitude()
            }
            Self::Arc(arc) => arc.length(),
            Self::Nurbs(curve) => curve.approximate_length(100),
        }
    }
}

/// 端点で連結された複数セグメントからなる複合曲線
///
/// 保持する不変条件:
/// - 全セグメントが拘束端点で連続している（N番目拘束終点 == N+1番目拘束始点）
/// - セグメント数は1以上
/// - セグメントは順序付き
#[derive(Clone, Debug)]
pub struct CompositeCurve3D<T: Scalar> {
    segments: Vec<CurveSegment3D<T>>,
}

impl<T: Scalar> CompositeCurve3D<T> {
    fn points_are_connected(start: Point3D<T>, end: Point3D<T>, tolerance: T) -> bool {
        start.distance_to(&end) <= tolerance
    }

    /// セグメント列から複合曲線を生成
    ///
    /// 次の場合は None を返す:
    /// - segments が空
    /// - セグメントが拘束端点で連続していない
    pub fn new(segments: Vec<CurveSegment3D<T>>) -> Option<Self> {
        Self::new_with_tolerance(segments, T::ZERO)
    }

    /// 許容誤差付きでセグメント列から複合曲線を生成
    pub fn new_with_tolerance(segments: Vec<CurveSegment3D<T>>, tolerance: T) -> Option<Self> {
        if segments.is_empty() {
            return None;
        }

        // 連続性検証: N番目終点とN+1番目始点が許容差内で接続すること
        for i in 0..segments.len() - 1 {
            let end = segments[i].constraint_end();
            let next_start = segments[i + 1].constraint_start();

            if !Self::points_are_connected(end, next_start, tolerance) {
                return None;
            }
        }

        Some(CompositeCurve3D { segments })
    }

    /// セグメント列を返す
    pub fn segments(&self) -> &[CurveSegment3D<T>] {
        &self.segments
    }

    /// 複合曲線全体の ideal 始点を返す
    pub fn ideal_start_point(&self) -> Point3D<T> {
        self.segments[0].start()
    }

    /// 複合曲線全体の ideal 終点を返す
    pub fn ideal_end_point(&self) -> Point3D<T> {
        self.segments[self.segments.len() - 1].end()
    }

    /// 複合曲線全体の拘束始点を返す
    pub fn constraint_start_point(&self) -> Point3D<T> {
        self.segments[0].constraint_start()
    }

    /// 複合曲線全体の拘束終点を返す
    pub fn constraint_end_point(&self) -> Point3D<T> {
        self.segments[self.segments.len() - 1].constraint_end()
    }

    /// 複合曲線全体の始点を返す
    ///
    /// CompositeCurve の public endpoint は topology 接続に使う拘束端点を正本とする。
    pub fn start_point(&self) -> Point3D<T> {
        self.constraint_start_point()
    }

    /// 複合曲線全体の終点を返す
    ///
    /// CompositeCurve の public endpoint は topology 接続に使う拘束端点を正本とする。
    pub fn end_point(&self) -> Point3D<T> {
        self.constraint_end_point()
    }

    /// 全セグメント長の合計を返す
    pub fn total_length(&self) -> T {
        self.segments
            .iter()
            .map(|seg| seg.length())
            .fold(T::ZERO, |acc, len| acc + len)
    }

    /// 閉曲線かどうかを返す（始点 == 終点）
    pub fn is_closed(&self) -> bool {
        self.is_closed_with_tolerance(T::ZERO)
    }

    /// 許容誤差付きで閉曲線かどうかを返す
    pub fn is_closed_with_tolerance(&self, tolerance: T) -> bool {
        let start = self.constraint_start_point();
        let end = self.constraint_end_point();
        Self::points_are_connected(start, end, tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::NurbsCurve3DConstructor;

    #[test]
    fn curve_segment_line_start_end() {
        let seg = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let curve_seg = CurveSegment3D::Line(seg);

        assert_eq!(curve_seg.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(curve_seg.end(), Point3D::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn curve_segment_line_length() {
        let seg = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(3.0, 4.0, 0.0))
            .unwrap();
        let curve_seg = CurveSegment3D::Line(seg);

        assert_eq!(curve_seg.length(), 5.0);
    }

    #[test]
    fn curve_segment_line_constraint_endpoints_follow_constraint_points() {
        let support_line = crate::TopoInfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
        )
        .unwrap();
        let seg = TopoLineSegment3D::from_support_line_and_constraint_points(
            support_line,
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(2.0, 1.0, 0.0),
        )
        .unwrap();
        let curve_seg = CurveSegment3D::Line(seg);

        assert_eq!(curve_seg.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(curve_seg.end(), Point3D::new(2.0, 0.0, 0.0));
        assert_eq!(curve_seg.constraint_start(), Point3D::new(0.0, 1.0, 0.0));
        assert_eq!(curve_seg.constraint_end(), Point3D::new(2.0, 1.0, 0.0));
    }

    #[test]
    fn curve_segment_nurbs_start_end_and_constraint_endpoints() {
        let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (2.0, 0.0, 0.0),
        )
        .unwrap();

        let segment = CurveSegment3D::Nurbs(curve);

        assert_eq!(segment.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(segment.end(), Point3D::new(2.0, 0.0, 0.0));
        assert_eq!(segment.constraint_start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(segment.constraint_end(), Point3D::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn composite_curve_single_segment() {
        let seg = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let composite = CompositeCurve3D::new(vec![CurveSegment3D::Line(seg)]).unwrap();

        assert_eq!(composite.start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.end_point(), Point3D::new(1.0, 0.0, 0.0));
        assert_eq!(composite.ideal_start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.ideal_end_point(), Point3D::new(1.0, 0.0, 0.0));
        assert_eq!(composite.segments().len(), 1);
    }

    #[test]
    fn composite_curve_two_segments_connected() {
        let seg1 = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let seg2 = TopoLineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0))
            .unwrap();
        let composite =
            CompositeCurve3D::new(vec![CurveSegment3D::Line(seg1), CurveSegment3D::Line(seg2)])
                .unwrap();

        assert_eq!(composite.start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.end_point(), Point3D::new(1.0, 1.0, 0.0));
        assert_eq!(composite.segments().len(), 2);
        assert_eq!(composite.total_length(), 2.0);
    }

    #[test]
    fn composite_curve_accepts_nurbs_segment() {
        let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
            (0.0, 0.0, 0.0),
            (2.0, 0.0, 0.0),
        )
        .unwrap();

        let composite = CompositeCurve3D::new(vec![CurveSegment3D::Nurbs(curve)]).unwrap();

        assert_eq!(composite.ideal_start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.ideal_end_point(), Point3D::new(2.0, 0.0, 0.0));
        assert_eq!(composite.start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.end_point(), Point3D::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn composite_curve_disconnected_segments_fails() {
        let seg1 = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let seg2 = TopoLineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0))
            .unwrap();

        assert!(CompositeCurve3D::new(vec![
            CurveSegment3D::Line(seg1),
            CurveSegment3D::Line(seg2),
        ])
        .is_none());
    }

    #[test]
    fn composite_curve_tolerance_allows_near_connected_segments() {
        let seg1 = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let seg2 = TopoLineSegment3D::new(
            Point3D::new(1.0, 0.0, 1.0e-4),
            Point3D::new(2.0, 0.0, 1.0e-4),
        )
        .unwrap();

        assert!(CompositeCurve3D::new(vec![
            CurveSegment3D::Line(seg1),
            CurveSegment3D::Line(seg2),
        ])
        .is_none());

        assert!(CompositeCurve3D::new_with_tolerance(
            vec![
                CurveSegment3D::Line(
                    TopoLineSegment3D::new(
                        Point3D::new(0.0, 0.0, 0.0),
                        Point3D::new(1.0, 0.0, 0.0),
                    )
                    .unwrap()
                ),
                CurveSegment3D::Line(
                    TopoLineSegment3D::new(
                        Point3D::new(1.0, 0.0, 1.0e-4),
                        Point3D::new(2.0, 0.0, 1.0e-4),
                    )
                    .unwrap(),
                ),
            ],
            1.0e-3,
        )
        .is_some());
    }

    #[test]
    fn composite_curve_connectivity_uses_constraint_endpoints_for_line_segments() {
        let support_line_a = crate::TopoInfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let support_line_b = crate::TopoInfiniteLine3D::from_two_points(
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
        )
        .unwrap();
        let seg1 = TopoLineSegment3D::from_support_line_and_constraint_points(
            support_line_a,
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        )
        .unwrap();
        let seg2 = TopoLineSegment3D::from_support_line_and_constraint_points(
            support_line_b,
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(2.0, 1.0, 0.0),
        )
        .unwrap();

        let composite =
            CompositeCurve3D::new(vec![CurveSegment3D::Line(seg1), CurveSegment3D::Line(seg2)])
                .unwrap();

        assert_eq!(composite.start_point(), Point3D::new(0.0, 1.0, 0.0));
        assert_eq!(composite.end_point(), Point3D::new(2.0, 1.0, 0.0));
        assert_eq!(composite.ideal_start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.ideal_end_point(), Point3D::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn composite_curve_empty_fails() {
        assert!(CompositeCurve3D::<f64>::new(vec![]).is_none());
    }

    #[test]
    fn composite_curve_is_closed() {
        let seg1 = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let seg2 = TopoLineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0))
            .unwrap();
        let seg3 = TopoLineSegment3D::new(Point3D::new(0.0, 1.0, 0.0), Point3D::new(0.0, 0.0, 0.0))
            .unwrap();

        let composite = CompositeCurve3D::new_with_tolerance(
            vec![
                CurveSegment3D::Line(seg1),
                CurveSegment3D::Line(seg2),
                CurveSegment3D::Line(seg3),
            ],
            1.0e-9,
        )
        .unwrap();

        assert!(composite.is_closed());
    }

    #[test]
    fn composite_curve_closed_with_tolerance() {
        let seg1 = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let seg2 =
            TopoLineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(0.0, 0.0, 5.0e-4))
                .unwrap();

        let composite = CompositeCurve3D::new_with_tolerance(
            vec![CurveSegment3D::Line(seg1), CurveSegment3D::Line(seg2)],
            1.0e-3,
        )
        .unwrap();

        assert!(!composite.is_closed());
        assert!(composite.is_closed_with_tolerance(1.0e-3));
    }
}
