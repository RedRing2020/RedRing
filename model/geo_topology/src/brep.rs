//! B-rep の表（頂点・稜線・ループ・面・シェル・立体）
//!
//! 要素は種類ごとの配列に置き、型つきの番号（配列の位置）で引く。番号は、その `Brep` の中だけで意味を持つ。
//! 2 つの面は 1 本の稜線を共有し、ループは稜線の使用（どの稜線を、どちら向きに使うか）の並びで持つ。
//! 継ぎ目の稜線は、1 つの面で 2 回使う（設計: BREP_AND_STEP_IMPORT_DESIGN.md 5 章）。

mod records;
#[cfg(test)]
mod tests;

pub use records::{BrepRecords, EdgeRecord, FaceRecord, VertexRecord};

use crate::parameter_curve::unwrap;
use crate::{
    CurveRef, ParameterCurve, Point2D, Point3D, SurfaceRef, TopoNurbsCurve3D,
    TopologyToleranceSettings,
};
use geo_contracts::{NurbsCurve3DProperties, Scalar};
use std::collections::{HashMap, HashSet};

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
element_index!(
    /// シェルの番号
    ShellIndex
);
element_index!(
    /// 立体の番号
    SolidIndex
);

/// 頂点（許容誤差は、その頂点を使う稜線の許容誤差以上）
#[derive(Debug, Clone)]
pub struct BrepVertex<T: Scalar> {
    point: Point3D<T>,
    tolerance: T,
}

impl<T: Scalar> BrepVertex<T> {
    pub fn point(&self) -> Point3D<T> {
        self.point
    }

    /// 頂点の許容誤差（頂点の点から、この距離の中を同じ頂点とみなす）
    pub fn tolerance(&self) -> T {
        self.tolerance
    }
}

/// 稜線（母曲線と、その区間。始点の頂点は区間の小さいほうの端、終点の頂点は大きいほうの端）
#[derive(Debug, Clone)]
pub struct BrepEdge<T: Scalar> {
    curve: CurveRef<T>,
    parameter_range: (T, T),
    start: VertexIndex,
    end: VertexIndex,
    /// 稜線の許容誤差（稜線を使う面の曲面と、この距離の中で合う）
    tolerance: T,
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

    /// 稜線の許容誤差（稜線を使う面の曲面と、この距離の中で合う）
    pub fn tolerance(&self) -> T {
        self.tolerance
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

/// シェル（外向きにそろえた、つながった面の集まり）
#[derive(Debug, Clone)]
pub struct BrepShell {
    faces: Vec<FaceIndex>,
}

impl BrepShell {
    pub fn faces(&self) -> &[FaceIndex] {
        &self.faces
    }
}

/// 立体（初めのシェルが外側、残りが空洞）
#[derive(Debug, Clone)]
pub struct BrepSolid {
    shells: Vec<ShellIndex>,
}

impl BrepSolid {
    pub fn shells(&self) -> &[ShellIndex] {
        &self.shells
    }
}

/// B-rep の表に要素を足せない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrepError {
    /// 表にない頂点
    UnknownVertex,
    /// 頂点の点が有限の値でない
    InvalidPoint,
    /// 表にない稜線
    UnknownEdge,
    /// 区間が空、NURBS の区間が母曲線の範囲の外、または円・楕円の区間が 1 周を超える
    InvalidParameterRange,
    /// 稜線の使用のないループ
    EmptyLoop,
    /// 稜線を、面の曲面の上のパラメータ曲線で表せない
    NoParameterCurve,
    /// 表の全体の許容誤差が正の有限の値でない、または頂点・稜線の許容誤差が有限の値でない
    InvalidTolerance,
    /// 表にない面
    UnknownFace,
    /// 表にないシェル
    UnknownShell,
    /// 面のないシェル
    EmptyShell,
    /// シェルのない立体
    EmptySolid,
    /// シェルに同じ面が 2 回ある
    DuplicateFace,
    /// 立体に同じシェルが 2 回ある
    DuplicateShell,
}

impl std::fmt::Display for BrepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::UnknownVertex => "unknown vertex",
            Self::InvalidPoint => "invalid point",
            Self::UnknownEdge => "unknown edge",
            Self::InvalidParameterRange => "invalid parameter range",
            Self::EmptyLoop => "empty loop",
            Self::NoParameterCurve => "no parameter curve",
            Self::InvalidTolerance => "invalid tolerance",
            Self::UnknownFace => "unknown face",
            Self::UnknownShell => "unknown shell",
            Self::EmptyShell => "empty shell",
            Self::EmptySolid => "empty solid",
            Self::DuplicateFace => "duplicate face",
            Self::DuplicateShell => "duplicate shell",
        };
        f.write_str(text)
    }
}

impl std::error::Error for BrepError {}

/// B-rep の表
#[derive(Debug, Clone)]
pub struct Brep<T: Scalar> {
    /// 表の全体の許容誤差（頂点・稜線の許容誤差の下限）
    tolerance: T,
    vertices: Vec<BrepVertex<T>>,
    edges: Vec<BrepEdge<T>>,
    loops: Vec<BrepLoop<T>>,
    faces: Vec<BrepFace<T>>,
    shells: Vec<BrepShell>,
    solids: Vec<BrepSolid>,
}

impl<T: Scalar> Brep<T> {
    /// 空の表（`tolerances` の距離の許容誤差を、頂点・稜線の許容誤差の下限にする）
    ///
    /// # Errors
    ///
    /// 許容誤差が正の有限の値でないとき。
    pub fn new(tolerances: TopologyToleranceSettings<T>) -> Result<Self, BrepError> {
        let tolerance = tolerances.distance_tolerance;
        if !(tolerance > T::ZERO && tolerance.is_finite()) {
            return Err(BrepError::InvalidTolerance);
        }
        Ok(Self {
            tolerance,
            vertices: Vec::new(),
            edges: Vec::new(),
            loops: Vec::new(),
            faces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
        })
    }

    /// 許容誤差 `tolerance`（表の全体の許容誤差より小さければ、全体の許容誤差）の頂点を足す
    ///
    /// # Errors
    ///
    /// 許容誤差が有限の値でない（無限大・NaN）、または点が有限の値でないとき。
    pub fn add_vertex(
        &mut self,
        point: Point3D<T>,
        tolerance: T,
    ) -> Result<VertexIndex, BrepError> {
        if !tolerance.is_finite() {
            return Err(BrepError::InvalidTolerance);
        }
        if !(point.x().is_finite() && point.y().is_finite() && point.z().is_finite()) {
            return Err(BrepError::InvalidPoint);
        }
        self.vertices.push(BrepVertex {
            point,
            tolerance: tolerance.max(self.tolerance),
        });
        Ok(VertexIndex(self.vertices.len() - 1))
    }

    /// 頂点 `start` から `end` へ、母曲線 `curve` の区間 `parameter_range` の、許容誤差 `tolerance` の稜線を足す
    ///
    /// 稜線の許容誤差は、表の全体の許容誤差より小さければ全体の許容誤差にする。始点・終点の頂点の許容誤差は、
    /// 稜線の許容誤差より小さければ、稜線の許容誤差まで広げる。NURBS の区間は、母曲線の範囲の外へ許容幅を
    /// 超えて出ていれば足さず、端の微小なずれは範囲に収める。
    ///
    /// # Errors
    ///
    /// 頂点が表にない、許容誤差が有限の値でない、区間が有限の値でないか空、NURBS の区間が母曲線の範囲の外、
    /// または円・楕円の区間が 1 周を超えるとき。
    pub fn add_edge(
        &mut self,
        start: VertexIndex,
        end: VertexIndex,
        curve: CurveRef<T>,
        parameter_range: (T, T),
        tolerance: T,
    ) -> Result<EdgeIndex, BrepError> {
        if start.0 >= self.vertices.len() || end.0 >= self.vertices.len() {
            return Err(BrepError::UnknownVertex);
        }
        if !tolerance.is_finite() {
            return Err(BrepError::InvalidTolerance);
        }
        if !(parameter_range.0.is_finite() && parameter_range.1.is_finite()) {
            return Err(BrepError::InvalidParameterRange);
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
        let tolerance = tolerance.max(self.tolerance);
        for vertex in [start, end] {
            let vertex = &mut self.vertices[vertex.0];
            vertex.tolerance = vertex.tolerance.max(tolerance);
        }
        self.edges.push(BrepEdge {
            curve,
            parameter_range: range,
            start,
            end,
            tolerance,
            uses: Vec::new(),
        });
        Ok(EdgeIndex(self.edges.len() - 1))
    }

    /// 曲面 `surface` の上に、ループ `loops`（稜線の使用の並びの並び）で囲まれた面を足す
    ///
    /// 稜線の使用ごとに、曲面の上のパラメータ曲線を作る。パラメータ曲線を曲面の上で評価した点と稜線の点の
    /// 離れは、稜線の許容誤差の中に収める。周期的な曲面では、ループの中で前の使用の終わりにつながるように、
    /// 周期の分だけ動かす。
    ///
    /// # Errors
    ///
    /// 稜線が表にない、稜線の使用のないループがある、または稜線をパラメータ曲線で表せないとき。
    pub fn add_face(
        &mut self,
        surface: SurfaceRef<T>,
        same_sense: bool,
        loops: Vec<Vec<EdgeUse>>,
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
            loop_curves.push(self.loop_parameter_curves(&surface, same_sense, uses)?);
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
    /// 法線と同じなら、上の極では u の負の向き、下の極では正の向き）。つなぎ目が極かは、前の使用の稜線の
    /// 許容誤差で見分ける（頂点の許容誤差は、ほかの稜線を足すと広がるため使わない）。
    fn loop_parameter_curves(
        &self,
        surface: &SurfaceRef<T>,
        same_sense: bool,
        uses: &[EdgeUse],
    ) -> Result<Vec<ParameterCurve<T>>, BrepError> {
        let (u_period, v_period) = surface.periods();
        let mut curves: Vec<ParameterCurve<T>> = Vec::with_capacity(uses.len());
        // 前の使用の終わりの (u, v)・頂点と、前の使用の中ほどの v・前の使用の稜線の許容誤差
        let mut previous: Option<(Point2D<T>, VertexIndex, T, T)> = None;
        for &edge_use in uses {
            let edge = self.edge(edge_use.edge);
            let curve =
                ParameterCurve::exact(surface, &edge.curve, edge.parameter_range, edge.tolerance)
                    .ok_or(BrepError::NoParameterCurve)?;
            let (t_start, t_end) = use_parameters(edge, edge_use);
            let start = curve.point_at(t_start).ok_or(BrepError::NoParameterCurve)?;
            let curve = match previous {
                Some((end, end_vertex, middle_v, previous_tolerance)) => {
                    let joint = self.vertex(end_vertex).point;
                    let u = match (surface.singular_v(joint, previous_tolerance), u_period) {
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
                self.use_end(edge_use),
                middle.y(),
                edge.tolerance,
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

    pub fn shell(&self, index: ShellIndex) -> &BrepShell {
        &self.shells[index.0]
    }

    pub fn solid(&self, index: SolidIndex) -> &BrepSolid {
        &self.solids[index.0]
    }

    /// 面 `index` を裏返す（面の外向きの法線を逆にする。ループの向きは、そのまま）
    ///
    /// 極を通るループのパラメータ曲線は、面の向きで極の線に沿って進む向きが決まるため、作り直す。
    ///
    /// # Errors
    ///
    /// 面が表にない、またはパラメータ曲線を作り直せないとき（面は変えない）。
    pub fn reverse_face(&mut self, index: FaceIndex) -> Result<(), BrepError> {
        let face = self.faces.get(index.0).ok_or(BrepError::UnknownFace)?;
        let same_sense = !face.same_sense;
        let curves = face
            .loops
            .iter()
            .map(|loop_index| {
                self.loop_parameter_curves(
                    &face.surface,
                    same_sense,
                    &self.loops[loop_index.0].uses,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (&loop_index, parameter_curves) in face.loops.clone().iter().zip(curves) {
            self.loops[loop_index.0].parameter_curves = parameter_curves;
        }
        self.faces[index.0].same_sense = same_sense;
        Ok(())
    }

    /// ループ `index` を逆にする（稜線の使用を逆の順・逆の向きにする）
    ///
    /// # Errors
    ///
    /// パラメータ曲線を作り直せないとき（ループは変えない）。
    pub fn reverse_loop(&mut self, index: LoopIndex) -> Result<(), BrepError> {
        let face_loop = &self.loops[index.0];
        let face = &self.faces[face_loop.face.0];
        let uses: Vec<EdgeUse> = face_loop
            .uses
            .iter()
            .rev()
            .map(|edge_use| EdgeUse {
                edge: edge_use.edge,
                forward: !edge_use.forward,
            })
            .collect();
        let parameter_curves = self.loop_parameter_curves(&face.surface, face.same_sense, &uses)?;
        // 稜線の使用の位置を、逆にした並びの位置にする
        // （継ぎ目の稜線は、同じループに 2 回あるため、稜線ごとに 1 回だけ直す）
        let last = uses.len() - 1;
        let edges: HashSet<EdgeIndex> = uses.iter().map(|edge_use| edge_use.edge).collect();
        for edge in edges {
            let locations = &mut self.edges[edge.0].uses;
            for location in locations.iter_mut() {
                if location.loop_index == index {
                    location.position = last - location.position;
                }
            }
            // 使用の並びは、面を足したときと同じく、ループと位置の順に保つ
            locations.sort_by_key(|location| (location.loop_index.0, location.position));
        }
        let face_loop = &mut self.loops[index.0];
        face_loop.uses = uses;
        face_loop.parameter_curves = parameter_curves;
        Ok(())
    }

    /// 面 `faces` のシェルを足す
    ///
    /// # Errors
    ///
    /// 面がない、表にない面がある、または同じ面が 2 回あるとき。
    pub fn add_shell(&mut self, faces: Vec<FaceIndex>) -> Result<ShellIndex, BrepError> {
        if faces.is_empty() {
            return Err(BrepError::EmptyShell);
        }
        if faces.iter().any(|face| face.0 >= self.faces.len()) {
            return Err(BrepError::UnknownFace);
        }
        if has_duplicates(&faces) {
            return Err(BrepError::DuplicateFace);
        }
        self.shells.push(BrepShell { faces });
        Ok(ShellIndex(self.shells.len() - 1))
    }

    /// シェル `shells`（初めが外側、残りが空洞）の立体を足す
    ///
    /// # Errors
    ///
    /// シェルがない、表にないシェルがある、または同じシェルが 2 回あるとき。
    pub fn add_solid(&mut self, shells: Vec<ShellIndex>) -> Result<SolidIndex, BrepError> {
        if shells.is_empty() {
            return Err(BrepError::EmptySolid);
        }
        if shells.iter().any(|shell| shell.0 >= self.shells.len()) {
            return Err(BrepError::UnknownShell);
        }
        if has_duplicates(&shells) {
            return Err(BrepError::DuplicateShell);
        }
        self.solids.push(BrepSolid { shells });
        Ok(SolidIndex(self.solids.len() - 1))
    }

    /// シェルが閉じているか（シェルの面が使う稜線を、シェルの面がどれも逆向きにちょうど 1 回ずつ使う）
    pub fn is_shell_closed(&self, index: ShellIndex) -> bool {
        // 稜線ごとの、向きに沿う使用と逆向きの使用の数
        let mut counts: HashMap<EdgeIndex, (usize, usize)> = HashMap::new();
        for &face in &self.shell(index).faces {
            for &loop_index in &self.faces[face.0].loops {
                for edge_use in &self.loops[loop_index.0].uses {
                    let count = counts.entry(edge_use.edge).or_default();
                    if edge_use.forward {
                        count.0 += 1;
                    } else {
                        count.1 += 1;
                    }
                }
            }
        }
        counts.values().all(|&count| count == (1, 1))
    }

    /// 要素の並びを、決まった順にそろえる（面はシェルをたどる順、ループは面の順、稜線はループをたどる順、
    /// 頂点は稜線をたどる順。たどれない要素は、元の順で後ろに置く）。同じ表からは、同じ並びになる
    ///
    /// 外で持っている面の番号 `faces` を、並べた後の番号に置き換える。
    pub fn into_canonical_order(self, faces: &mut [FaceIndex]) -> Self {
        let face_order = visit_order(
            self.faces.len(),
            self.shells
                .iter()
                .flat_map(|shell| shell.faces.iter().map(|face| face.0)),
        );
        let loop_order: Vec<usize> = face_order
            .iter()
            .flat_map(|&face| self.faces[face].loops.iter().map(|loop_index| loop_index.0))
            .collect();
        let edge_order = visit_order(
            self.edges.len(),
            loop_order.iter().flat_map(|&loop_index| {
                self.loops[loop_index]
                    .uses
                    .iter()
                    .map(|edge_use| edge_use.edge.0)
            }),
        );
        let vertex_order = visit_order(
            self.vertices.len(),
            edge_order
                .iter()
                .flat_map(|&edge| [self.edges[edge].start.0, self.edges[edge].end.0]),
        );
        let face_map = inverse(&face_order);
        for face in faces.iter_mut() {
            *face = FaceIndex(face_map[face.0]);
        }
        let loop_map = inverse(&loop_order);
        let edge_map = inverse(&edge_order);
        let vertex_map = inverse(&vertex_order);

        let Self {
            tolerance,
            vertices,
            edges,
            loops,
            faces,
            shells,
            solids,
        } = self;
        let mut vertices: Vec<Option<BrepVertex<T>>> = vertices.into_iter().map(Some).collect();
        let mut edges: Vec<Option<BrepEdge<T>>> = edges.into_iter().map(Some).collect();
        let mut loops: Vec<Option<BrepLoop<T>>> = loops.into_iter().map(Some).collect();
        let mut faces: Vec<Option<BrepFace<T>>> = faces.into_iter().map(Some).collect();

        let mut edges_ordered: Vec<BrepEdge<T>> = edge_order
            .iter()
            .filter_map(|&edge| edges[edge].take())
            .map(|mut edge| {
                edge.start = VertexIndex(vertex_map[edge.start.0]);
                edge.end = VertexIndex(vertex_map[edge.end.0]);
                edge.uses.clear();
                edge
            })
            .collect();
        let loops_ordered: Vec<BrepLoop<T>> = loop_order
            .iter()
            .filter_map(|&loop_index| loops[loop_index].take())
            .map(|mut face_loop| {
                face_loop.face = FaceIndex(face_map[face_loop.face.0]);
                for edge_use in &mut face_loop.uses {
                    edge_use.edge = EdgeIndex(edge_map[edge_use.edge.0]);
                }
                face_loop
            })
            .collect();
        // 稜線の使用の位置は、並べた後のループから作り直す
        for (loop_position, face_loop) in loops_ordered.iter().enumerate() {
            for (position, edge_use) in face_loop.uses.iter().enumerate() {
                edges_ordered[edge_use.edge.0].uses.push(EdgeUseLocation {
                    face: face_loop.face,
                    loop_index: LoopIndex(loop_position),
                    position,
                });
            }
        }
        Self {
            tolerance,
            vertices: vertex_order
                .iter()
                .filter_map(|&vertex| vertices[vertex].take())
                .collect(),
            edges: edges_ordered,
            loops: loops_ordered,
            faces: face_order
                .iter()
                .filter_map(|&face| faces[face].take())
                .map(|mut face| {
                    for loop_index in &mut face.loops {
                        *loop_index = LoopIndex(loop_map[loop_index.0]);
                    }
                    face
                })
                .collect(),
            shells: shells
                .into_iter()
                .map(|mut shell| {
                    for face in &mut shell.faces {
                        *face = FaceIndex(face_map[face.0]);
                    }
                    shell
                })
                .collect(),
            solids,
        }
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

    /// 面の数
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// 面の番号（表の面の順）
    pub fn face_indices(&self) -> impl ExactSizeIterator<Item = FaceIndex> + use<T> {
        (0..self.faces.len()).map(FaceIndex)
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

/// 同じ番号が 2 回以上あるか
fn has_duplicates<I: Eq + std::hash::Hash + Copy>(indices: &[I]) -> bool {
    let mut seen = HashSet::with_capacity(indices.len());
    indices.iter().any(|&index| !seen.insert(index))
}

/// `visited` を初めて出た順に並べ、出なかった `0..count` を元の順で後ろに足す
fn visit_order(count: usize, visited: impl Iterator<Item = usize>) -> Vec<usize> {
    let mut seen = vec![false; count];
    let mut order = Vec::with_capacity(count);
    for index in visited.chain(0..count) {
        if !seen[index] {
            seen[index] = true;
            order.push(index);
        }
    }
    order
}

/// 新しい位置から元の位置への並び `order` を、元の位置から新しい位置への並びにする
fn inverse(order: &[usize]) -> Vec<usize> {
    let mut map = vec![0; order.len()];
    for (new_position, &old_position) in order.iter().enumerate() {
        map[old_position] = new_position;
    }
    map
}
