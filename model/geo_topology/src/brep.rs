//! B-rep の表（頂点・稜線・ループ・面）
//!
//! 要素は種類ごとの配列に置き、型つきの番号（配列の位置）で引く。番号は、その `Brep` の中だけで意味を持つ。
//! 2 つの面は 1 本の稜線を共有し、ループは稜線の使用（どの稜線を、どちら向きに使うか）の並びで持つ。
//! 継ぎ目の稜線は、1 つの面で 2 回使う（設計: BREP_AND_STEP_IMPORT_DESIGN.md 5 章。シェル・立体は、使う段で足す）。

#[cfg(test)]
mod tests;

use crate::parameter_curve::unwrap;
use crate::{
    CurveRef, ParameterCurve, Point2D, Point3D, SurfaceRef, TopoNurbsCurve3D,
    TopologyToleranceSettings,
};
use geo_contracts::{NurbsCurve3DProperties, Scalar};

macro_rules! element_index {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub(crate) usize);
    };
}

element_index!(
    /// 頂点の番号
    VertexIndex
);
element_index!(
    /// 稜線の番号
    EdgeIndex
);
element_index!(
    /// ループの番号
    LoopIndex
);
element_index!(
    /// 面の番号
    FaceIndex
);

/// 頂点
#[derive(Debug, Clone)]
pub struct BrepVertex<T: Scalar> {
    point: Point3D<T>,
}

impl<T: Scalar> BrepVertex<T> {
    pub fn point(&self) -> Point3D<T> {
        self.point
    }
}

/// 稜線（母曲線と、その区間。始点の頂点は区間の小さいほうの端、終点の頂点は大きいほうの端）
#[derive(Debug, Clone)]
pub struct BrepEdge<T: Scalar> {
    curve: CurveRef<T>,
    parameter_range: (T, T),
    start: VertexIndex,
    end: VertexIndex,
    uses: Vec<EdgeUseLocation>,
}

impl<T: Scalar> BrepEdge<T> {
    pub fn curve(&self) -> &CurveRef<T> {
        &self.curve
    }

    /// 母曲線の native parameter の区間 `[t0, t1]`（`t0 < t1`）
    pub fn parameter_range(&self) -> (T, T) {
        self.parameter_range
    }

    pub fn start(&self) -> VertexIndex {
        self.start
    }

    pub fn end(&self) -> VertexIndex {
        self.end
    }

    /// この稜線を使うループの位置（使った順）
    pub fn uses(&self) -> &[EdgeUseLocation] {
        &self.uses
    }
}

/// 稜線の使用（ループの中で、どの稜線を、どちら向きに使うか）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeUse {
    pub edge: EdgeIndex,
    /// 稜線の向き（始点から終点）に沿って使うか
    pub forward: bool,
}

/// 稜線の使用の位置（どの面の、どのループの、何番目か）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeUseLocation {
    pub face: FaceIndex,
    pub loop_index: LoopIndex,
    pub position: usize,
}

/// ループ（稜線の使用の並び。面の外向きの法線の側から見て、面がループの進む向きの左にある）
#[derive(Debug, Clone)]
pub struct BrepLoop<T: Scalar> {
    face: FaceIndex,
    uses: Vec<EdgeUse>,
    parameter_curves: Vec<ParameterCurve<T>>,
}

impl<T: Scalar> BrepLoop<T> {
    pub fn face(&self) -> FaceIndex {
        self.face
    }

    pub fn uses(&self) -> &[EdgeUse] {
        &self.uses
    }

    /// 稜線の使用ごとのパラメータ曲線（`uses` と同じ順。ループの中で (u, v) がつながる）
    pub fn parameter_curves(&self) -> &[ParameterCurve<T>] {
        &self.parameter_curves
    }
}

/// 面（曲面と、それを囲むループ。ループのない面は、閉じた曲面の全体）
#[derive(Debug, Clone)]
pub struct BrepFace<T: Scalar> {
    surface: SurfaceRef<T>,
    loops: Vec<LoopIndex>,
    same_sense: bool,
}

impl<T: Scalar> BrepFace<T> {
    pub fn surface(&self) -> &SurfaceRef<T> {
        &self.surface
    }

    pub fn loops(&self) -> &[LoopIndex] {
        &self.loops
    }

    /// 面の外向きの法線が、曲面の法線と同じ向きか
    pub fn same_sense(&self) -> bool {
        self.same_sense
    }
}

/// B-rep の表に要素を足せない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrepError {
    /// 表にない頂点
    UnknownVertex,
    /// 表にない稜線
    UnknownEdge,
    /// 区間が空、NURBS の区間が母曲線の範囲の外、または円・楕円の区間が 1 周を超える
    InvalidParameterRange,
    /// 稜線の使用のないループ
    EmptyLoop,
    /// 稜線を、面の曲面の上のパラメータ曲線で表せない
    NoParameterCurve,
}

impl std::fmt::Display for BrepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::UnknownVertex => "unknown vertex",
            Self::UnknownEdge => "unknown edge",
            Self::InvalidParameterRange => "invalid parameter range",
            Self::EmptyLoop => "empty loop",
            Self::NoParameterCurve => "no parameter curve",
        };
        f.write_str(text)
    }
}

impl std::error::Error for BrepError {}

/// B-rep の表
#[derive(Debug, Clone)]
pub struct Brep<T: Scalar> {
    vertices: Vec<BrepVertex<T>>,
    edges: Vec<BrepEdge<T>>,
    loops: Vec<BrepLoop<T>>,
    faces: Vec<BrepFace<T>>,
}

impl<T: Scalar> Default for Brep<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Scalar> Brep<T> {
    /// 空の表
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            loops: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, point: Point3D<T>) -> VertexIndex {
        self.vertices.push(BrepVertex { point });
        VertexIndex(self.vertices.len() - 1)
    }

    /// 頂点 `start` から `end` へ、母曲線 `curve` の区間 `parameter_range` の稜線を足す
    ///
    /// NURBS の区間は、母曲線の範囲の外へ許容幅を超えて出ていれば足さず、端の微小なずれは範囲に収める。
    ///
    /// # Errors
    ///
    /// 頂点が表にない、区間が空、NURBS の区間が母曲線の範囲の外、または円・楕円の区間が 1 周を超えるとき。
    pub fn add_edge(
        &mut self,
        start: VertexIndex,
        end: VertexIndex,
        curve: CurveRef<T>,
        parameter_range: (T, T),
    ) -> Result<EdgeIndex, BrepError> {
        if start.0 >= self.vertices.len() || end.0 >= self.vertices.len() {
            return Err(BrepError::UnknownVertex);
        }
        let mut range = parameter_range;
        if let CurveRef::Nurbs(nurbs) = &curve {
            let (u_min, u_max) =
                <TopoNurbsCurve3D<T> as NurbsCurve3DProperties<T>>::parameter_domain(nurbs);
            let tolerance = CurveRef::<T>::nurbs_parameter_tolerance_for_domain(u_min, u_max);
            if range.0 < u_min - tolerance || range.1 > u_max + tolerance {
                return Err(BrepError::InvalidParameterRange);
            }
            if range.0 < u_min {
                range.0 = u_min;
            }
            if range.1 > u_max {
                range.1 = u_max;
            }
        }
        if range.0 >= range.1 {
            return Err(BrepError::InvalidParameterRange);
        }
        // 円・楕円の区間は 1 周まで（角度の丸めの分は許す）
        if matches!(curve, CurveRef::Circle(_) | CurveRef::Ellipse(_))
            && range.1 - range.0
                > T::TAU + CurveRef::<T>::nurbs_parameter_tolerance_for_domain(range.0, range.1)
        {
            return Err(BrepError::InvalidParameterRange);
        }
        self.edges.push(BrepEdge {
            curve,
            parameter_range: range,
            start,
            end,
            uses: Vec::new(),
        });
        Ok(EdgeIndex(self.edges.len() - 1))
    }

    /// 曲面 `surface` の上に、ループ `loops`（稜線の使用の並びの並び）で囲まれた面を足す
    ///
    /// 稜線の使用ごとに、曲面の上のパラメータ曲線を作る。パラメータ曲線を曲面の上で評価した点と稜線の点の
    /// 離れは、`tolerances` の距離の許容誤差の中に収める。周期的な曲面では、ループの中で前の使用の終わりに
    /// つながるように、周期の分だけ動かす。
    ///
    /// # Errors
    ///
    /// 稜線が表にない、稜線の使用のないループがある、または稜線をパラメータ曲線で表せないとき。
    pub fn add_face(
        &mut self,
        surface: SurfaceRef<T>,
        same_sense: bool,
        loops: Vec<Vec<EdgeUse>>,
        tolerances: TopologyToleranceSettings<T>,
    ) -> Result<FaceIndex, BrepError> {
        let mut loop_curves = Vec::with_capacity(loops.len());
        for uses in &loops {
            if uses.is_empty() {
                return Err(BrepError::EmptyLoop);
            }
            if uses
                .iter()
                .any(|edge_use| edge_use.edge.0 >= self.edges.len())
            {
                return Err(BrepError::UnknownEdge);
            }
            loop_curves.push(self.loop_parameter_curves(
                &surface,
                same_sense,
                uses,
                tolerances.distance_tolerance,
            )?);
        }
        let face = FaceIndex(self.faces.len());
        let mut loop_indices = Vec::with_capacity(loops.len());
        for (uses, parameter_curves) in loops.into_iter().zip(loop_curves) {
            let loop_index = LoopIndex(self.loops.len());
            for (position, edge_use) in uses.iter().enumerate() {
                self.edges[edge_use.edge.0].uses.push(EdgeUseLocation {
                    face,
                    loop_index,
                    position,
                });
            }
            self.loops.push(BrepLoop {
                face,
                uses,
                parameter_curves,
            });
            loop_indices.push(loop_index);
        }
        self.faces.push(BrepFace {
            surface,
            loops: loop_indices,
            same_sense,
        });
        Ok(face)
    }

    /// ループ `uses` の稜線の使用ごとのパラメータ曲線（ループの中でつながるように、周期の分だけ動かす）
    ///
    /// 球の極・円錐の頂点では、隣り合う使用の u が本当に違う（極は (u, v) では v が一定の線になる）。そこでは、
    /// 面がループの左にある向きに極の線に沿って進むように、次の使用の u を選ぶ（面の外向きの法線が曲面の
    /// 法線と同じなら、上の極では u の負の向き、下の極では正の向き）。
    fn loop_parameter_curves(
        &self,
        surface: &SurfaceRef<T>,
        same_sense: bool,
        uses: &[EdgeUse],
        tolerance: T,
    ) -> Result<Vec<ParameterCurve<T>>, BrepError> {
        let (u_period, v_period) = surface.periods();
        let mut curves: Vec<ParameterCurve<T>> = Vec::with_capacity(uses.len());
        // 前の使用の終わりの (u, v)・点と、前の使用の中ほどの v
        let mut previous: Option<(Point2D<T>, Point3D<T>, T)> = None;
        for &edge_use in uses {
            let edge = self.edge(edge_use.edge);
            let curve =
                ParameterCurve::exact(surface, &edge.curve, edge.parameter_range, tolerance)
                    .ok_or(BrepError::NoParameterCurve)?;
            let (t_start, t_end) = use_parameters(edge, edge_use);
            let start = curve.point_at(t_start).ok_or(BrepError::NoParameterCurve)?;
            let curve = match previous {
                Some((end, end_point, middle_v)) => {
                    let u = match (surface.singular_v(end_point, tolerance), u_period) {
                        (Some(pole_v), Some(period)) => {
                            let decreasing = (pole_v > middle_v) == same_sense;
                            shift_to_side(start.x(), end.x(), period, decreasing)
                        }
                        _ => unwrap(start.x(), end.x(), u_period),
                    };
                    curve.shifted(
                        u - start.x(),
                        unwrap(start.y(), end.y(), v_period) - start.y(),
                    )
                }
                None => curve,
            };
            let middle = curve
                .point_at((t_start + t_end) / T::from_f64(2.0))
                .ok_or(BrepError::NoParameterCurve)?;
            previous = Some((
                curve.point_at(t_end).ok_or(BrepError::NoParameterCurve)?,
                edge.curve.point_at_parameter(t_end),
                middle.y(),
            ));
            curves.push(curve);
        }
        Ok(curves)
    }

    /// 番号は、この表から得たもの（表から得ていない番号では、パニックする）
    pub fn vertex(&self, index: VertexIndex) -> &BrepVertex<T> {
        &self.vertices[index.0]
    }

    pub fn edge(&self, index: EdgeIndex) -> &BrepEdge<T> {
        &self.edges[index.0]
    }

    pub fn face_loop(&self, index: LoopIndex) -> &BrepLoop<T> {
        &self.loops[index.0]
    }

    pub fn face(&self, index: FaceIndex) -> &BrepFace<T> {
        &self.faces[index.0]
    }

    /// 稜線の使用の、進む向きの始めの頂点
    pub fn use_start(&self, edge_use: EdgeUse) -> VertexIndex {
        let edge = self.edge(edge_use.edge);
        if edge_use.forward {
            edge.start
        } else {
            edge.end
        }
    }

    /// 稜線の使用の、進む向きの終わりの頂点
    pub fn use_end(&self, edge_use: EdgeUse) -> VertexIndex {
        let edge = self.edge(edge_use.edge);
        if edge_use.forward {
            edge.end
        } else {
            edge.start
        }
    }

    pub(crate) fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub(crate) fn loop_count(&self) -> usize {
        self.loops.len()
    }

    pub(crate) fn face_count(&self) -> usize {
        self.faces.len()
    }
}

/// 稜線の使用の、進む向きの始めと終わりの、母曲線のパラメータ
pub(crate) fn use_parameters<T: Scalar>(edge: &BrepEdge<T>, edge_use: EdgeUse) -> (T, T) {
    let (t0, t1) = edge.parameter_range;
    if edge_use.forward { (t0, t1) } else { (t1, t0) }
}

/// `value` を周期 `period` の分だけ動かして、`reference` の片側（`decreasing` なら `(reference - period,
/// reference]`、そうでなければ `[reference, reference + period)`）に置いた値
pub(crate) fn shift_to_side<T: Scalar>(value: T, reference: T, period: T, decreasing: bool) -> T {
    let turns = (reference - value) / period;
    if decreasing {
        value + turns.floor() * period
    } else {
        value - (-turns).floor() * period
    }
}
