//! 交差結果型
//!
//! 幾何交差演算で利用する汎用結果型を提供する。
//!
//! 構成:
//! - `IntersectionGeometry`: 幾何的に何が得られたか
//! - `IntersectionTopology`: 位相的な関係
//! - `IntersectionResult`: 幾何と位相を統合した結果

use crate::{CompositeCurve3D, InfiniteLine3D, LineSegment3D, Point3D};
use geo_contracts::Scalar;

/// 交差結果の幾何内容
#[derive(Clone, Debug)]
pub enum IntersectionGeometry<T: Scalar> {
    /// 交差なし
    None,
    /// 単一点交差
    Point(Point3D<T>),
    /// 複数の孤立点
    Points(Vec<Point3D<T>>),
    /// 無限直線（例: 非平行な平面同士の交差）
    InfiniteLine(InfiniteLine3D<T>),
    /// 線分（例: 有界な面同士の交差）
    Segment(LineSegment3D<T>),
    /// 複合曲線（例: 連結した複数セグメント）
    CompositeCurve(CompositeCurve3D<T>),
    /// 完全一致（形状が全域で重なる）
    Coincident,
}

impl<T: Scalar> IntersectionGeometry<T> {
    /// 幾何タイプの説明文字列を返す
    pub fn description(&self) -> &'static str {
        match self {
            Self::None => "no intersection",
            Self::Point(_) => "single point",
            Self::Points(_) => "multiple points",
            Self::InfiniteLine(_) => "infinite line",
            Self::Segment(_) => "line segment",
            Self::CompositeCurve(_) => "composite curve",
            Self::Coincident => "coincident (complete overlap)",
        }
    }

    /// 交差が空かを返す
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }

    /// 複数要素を含むかを返す
    pub fn is_multiple(&self) -> bool {
        matches!(self, Self::Points(_) | Self::CompositeCurve(_))
    }

    /// 交差集合の次元を返す
    /// - -1: 空集合
    /// - 0: 点
    /// - 1: 曲線
    /// - 2: 面（Coincident）
    pub fn dimension(&self) -> i32 {
        match self {
            Self::None => -1,
            Self::Point(_) | Self::Points(_) => 0,
            Self::InfiniteLine(_) | Self::Segment(_) | Self::CompositeCurve(_) => 1,
            Self::Coincident => 2,
        }
    }
}

/// 交差の位相分類
///
/// 形状間の位置関係を表す。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntersectionTopology {
    /// 形状が接していない
    Disjoint,
    /// 孤立点で接する（例: 接線接触）
    Touching,
    /// 形状が交差する（横断交差）
    Crossing,
    /// 形状が完全に一致する
    Coincident,
}

impl IntersectionTopology {
    /// 交差しているかを返す（Disjoint を除く）
    pub fn intersects(&self) -> bool {
        !matches!(self, Self::Disjoint)
    }

    /// 説明文字列を返す
    pub fn description(&self) -> &'static str {
        match self {
            Self::Disjoint => "disjoint",
            Self::Touching => "touching",
            Self::Crossing => "crossing",
            Self::Coincident => "coincident",
        }
    }
}

/// 幾何と位相を統合した交差結果
///
/// # 不変条件
///
/// 交差計算層では以下の規約を維持する。
///
/// 1. **判定優先順位**: Coincident > Crossing > Touching > Disjoint
/// 2. **幾何と位相の整合**:
///    - `Coincident`: geometry は `Self::Coincident`
///    - `Touching`: geometry の次元は 0（点）
///    - `Crossing`: geometry の次元は 1 以上
///    - `Disjoint`: geometry は `Self::None`
/// 3. **トレランス運用**: `tolerance_used` は呼び出し元入力と一致すること
///    （未指定時はシステム既定値）
#[derive(Clone, Debug)]
pub struct IntersectionResult<T: Scalar> {
    /// 幾何交差結果
    pub geometry: IntersectionGeometry<T>,

    /// 位相的関係
    pub topology: IntersectionTopology,

    /// 接触が接線的か（導関数整合だが一致ではない）
    pub is_tangent: bool,

    /// 計算で使用したトレランス値
    pub tolerance_used: T,
}

impl<T: Scalar> IntersectionResult<T> {
    /// 交差結果を構築する
    pub fn new(
        geometry: IntersectionGeometry<T>,
        topology: IntersectionTopology,
        is_tangent: bool,
        tolerance_used: T,
    ) -> Self {
        IntersectionResult {
            geometry,
            topology,
            is_tangent,
            tolerance_used,
        }
    }

    /// 「交差なし」結果を構築する
    pub fn disjoint(tolerance: T) -> Self {
        IntersectionResult {
            geometry: IntersectionGeometry::None,
            topology: IntersectionTopology::Disjoint,
            is_tangent: false,
            tolerance_used: tolerance,
        }
    }

    /// 「単一点交差」結果を構築する
    pub fn point(point: Point3D<T>, is_tangent: bool, tolerance: T) -> Self {
        IntersectionResult {
            geometry: IntersectionGeometry::Point(point),
            topology: IntersectionTopology::Crossing,
            is_tangent,
            tolerance_used: tolerance,
        }
    }

    /// 「複数点交差」結果を構築する
    pub fn points(points: Vec<Point3D<T>>, is_tangent: bool, tolerance: T) -> Self {
        IntersectionResult {
            geometry: IntersectionGeometry::Points(points),
            topology: if is_tangent {
                IntersectionTopology::Touching
            } else {
                IntersectionTopology::Crossing
            },
            is_tangent,
            tolerance_used: tolerance,
        }
    }

    /// `Option<Point3D<T>>` から変換する互換アダプタ
    ///
    /// 既存 API（`Option<Point3D<T>>` を返す交差関数）から
    /// `IntersectionResult<T>` へ段階移行するための変換規約。
    ///
    /// 変換規約:
    /// - `None`    → `Disjoint`（交差なし）
    /// - `Some(p)` → `Crossing` / 単一点（接線かどうかは呼び出し元が指定）
    pub fn from_option_point(opt: Option<Point3D<T>>, is_tangent: bool, tolerance: T) -> Self {
        match opt {
            None => Self::disjoint(tolerance),
            Some(p) => {
                let topology = if is_tangent {
                    IntersectionTopology::Touching
                } else {
                    IntersectionTopology::Crossing
                };
                IntersectionResult {
                    geometry: IntersectionGeometry::Point(p),
                    topology,
                    is_tangent,
                    tolerance_used: tolerance,
                }
            }
        }
    }

    /// `Vec<Point3D<T>>` から変換する互換アダプタ
    ///
    /// 既存 API（複数点を返す交差関数）からの変換規約。
    ///
    /// 変換規約:
    /// - 空ベクタ   → `Disjoint`
    /// - 1点以上   → `Crossing` または `Touching`（`is_tangent` で制御）
    pub fn from_option_points(points: Vec<Point3D<T>>, is_tangent: bool, tolerance: T) -> Self {
        if points.is_empty() {
            return Self::disjoint(tolerance);
        }
        Self::points(points, is_tangent, tolerance)
    }

    /// 交差しているかを返す
    pub fn intersects(&self) -> bool {
        self.topology.intersects()
    }

    /// 人が読める説明文字列を返す
    pub fn description(&self) -> String {
        format!(
            "{} ({}, tangent: {}, tol: {:?})",
            self.geometry.description(),
            self.topology.description(),
            self.is_tangent,
            self.tolerance_used
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_dimension() {
        assert_eq!(IntersectionGeometry::<f64>::None.dimension(), -1);
        assert_eq!(
            IntersectionGeometry::Point(Point3D::new(0.0, 0.0, 0.0)).dimension(),
            0
        );
    }

    #[test]
    fn topology_intersects() {
        assert!(!IntersectionTopology::Disjoint.intersects());
        assert!(IntersectionTopology::Touching.intersects());
        assert!(IntersectionTopology::Crossing.intersects());
        assert!(IntersectionTopology::Coincident.intersects());
    }

    #[test]
    fn result_disjoint() {
        let result = IntersectionResult::disjoint(1e-9);
        assert!(!result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn result_point() {
        let pt = Point3D::new(1.0, 2.0, 3.0);
        let result = IntersectionResult::point(pt, false, 1e-9);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
    }

    // --- from_option_point 変換規約テスト ---

    #[test]
    fn compat_from_option_point_none_is_disjoint() {
        // None → Disjoint
        let result = IntersectionResult::<f64>::from_option_point(None, false, 1e-9);
        assert!(!result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
        assert!(result.geometry.is_empty());
    }

    #[test]
    fn compat_from_option_point_some_is_crossing() {
        // Some(p)、非接線 → Crossing
        let pt = Point3D::new(1.0, 0.0, 0.0);
        let result = IntersectionResult::from_option_point(Some(pt), false, 1e-9);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
        assert!(!result.is_tangent);
    }

    #[test]
    fn compat_from_option_point_tangent_is_touching() {
        // Some(p)、接線 → Touching
        let pt = Point3D::new(0.0, 1.0, 0.0);
        let result = IntersectionResult::from_option_point(Some(pt), true, 1e-9);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Touching);
        assert!(result.is_tangent);
    }

    // --- from_option_points 変換規約テスト ---

    #[test]
    fn compat_from_option_points_empty_is_disjoint() {
        // 空ベクタ → Disjoint
        let result = IntersectionResult::<f64>::from_option_points(vec![], false, 1e-9);
        assert!(!result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn compat_from_option_points_multi_is_crossing() {
        // 複数点、非接線 → Crossing
        let pts = vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)];
        let result = IntersectionResult::from_option_points(pts, false, 1e-9);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Crossing);
    }

    #[test]
    fn compat_from_option_points_multi_tangent_is_touching() {
        // 複数点、接線 → Touching
        let pts = vec![Point3D::new(0.0, 1.0, 0.0)];
        let result = IntersectionResult::from_option_points(pts, true, 1e-9);
        assert!(result.intersects());
        assert_eq!(result.topology, IntersectionTopology::Touching);
    }
}
