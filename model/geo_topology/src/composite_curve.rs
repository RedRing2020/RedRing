//! CompositeCurve3D: 複数セグメントから構成される複合曲線
//!
//! 命名は Solidworks 系の CompositeCurve に合わせる
//! （Rhino: PolyCurve、ISO STEP: CompositeCurve）

use crate::{Point3D, TopoArc3D, TopoLineSegment3D, Vector3D};
use geo_contracts::Scalar;

/// 複合曲線を構成する個別セグメント
#[derive(Clone, Debug)]
pub enum CurveSegment3D<T: Scalar> {
    /// 始点から終点を持つ線分セグメント
    Line(TopoLineSegment3D<T>),
    /// 3D円弧セグメント
    Arc(TopoArc3D<T>),
    // 将来: Nurbs(NurbsCurve3D<T>)  ← geo_nurbs 依存方針確定後に追加
}

impl<T: Scalar> CurveSegment3D<T> {
    /// セグメント始点を返す
    pub fn start(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.start(),
            Self::Arc(arc) => arc.start_point(),
        }
    }

    /// セグメント終点を返す
    pub fn end(&self) -> Point3D<T> {
        match self {
            Self::Line(seg) => seg.end(),
            Self::Arc(arc) => arc.end_point(),
        }
    }

    /// セグメント長を返す
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

/// 端点で連結された複数セグメントからなる複合曲線
///
/// 保持する不変条件:
/// - 全セグメントが連続している（N番目終点 == N+1番目始点）
/// - セグメント数は1以上
/// - セグメントは順序付き
#[derive(Clone, Debug)]
pub struct CompositeCurve3D<T: Scalar> {
    segments: Vec<CurveSegment3D<T>>,
}

impl<T: Scalar> CompositeCurve3D<T> {
    /// セグメント列から複合曲線を生成
    ///
    /// 次の場合は None を返す:
    /// - segments が空
    /// - セグメントが連続していない（N番目終点 != N+1番目始点）
    pub fn new(segments: Vec<CurveSegment3D<T>>) -> Option<Self> {
        if segments.is_empty() {
            return None;
        }

        // 連続性検証: N番目終点とN+1番目始点が一致すること
        for i in 0..segments.len() - 1 {
            let end = segments[i].end();
            let next_start = segments[i + 1].start();

            // 接続判定は明示的な座標比較で行う
            // （トレランス考慮は構築処理ではなく検証層の責務）
            if end.x() != next_start.x() || end.y() != next_start.y() || end.z() != next_start.z() {
                return None;
            }
        }

        Some(CompositeCurve3D { segments })
    }

    /// セグメント列を返す
    pub fn segments(&self) -> &[CurveSegment3D<T>] {
        &self.segments
    }

    /// 複合曲線全体の始点を返す
    pub fn start_point(&self) -> Point3D<T> {
        self.segments[0].start()
    }

    /// 複合曲線全体の終点を返す
    pub fn end_point(&self) -> Point3D<T> {
        self.segments[self.segments.len() - 1].end()
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
    fn composite_curve_single_segment() {
        let seg = TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let composite = CompositeCurve3D::new(vec![CurveSegment3D::Line(seg)]).unwrap();

        assert_eq!(composite.start_point(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(composite.end_point(), Point3D::new(1.0, 0.0, 0.0));
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

        let composite = CompositeCurve3D::new(vec![
            CurveSegment3D::Line(seg1),
            CurveSegment3D::Line(seg2),
            CurveSegment3D::Line(seg3),
        ])
        .unwrap();

        assert!(composite.is_closed());
    }
}
