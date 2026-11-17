//! B-rep の表の中身を取り出し、取り出したものから表を戻す（保存して開き直すため）
//!
//! 中身は、番号を表の中の位置（`usize`）で持つ。パラメータ曲線は持たず、戻すときに面を足す処理で作り直す。
//! 戻す口は、信用できない入力として、頂点・稜線・面・シェル・立体を足す処理で作り直し、確かめる。

use super::{Brep, BrepError, EdgeIndex, EdgeUse, FaceIndex, ShellIndex, VertexIndex};
use crate::{CurveRef, Point3D, SurfaceRef, TopologyToleranceSettings};
use geo_contracts::Scalar;

/// B-rep の表の中身（番号は、表の中の位置）
#[derive(Debug, Clone)]
pub struct BrepRecords<T: Scalar> {
    /// 表の全体の許容誤差
    pub tolerance: T,
    pub vertices: Vec<VertexRecord<T>>,
    pub edges: Vec<EdgeRecord<T>>,
    /// 面（ループは、面の順に、面の中の順で並ぶ）
    pub faces: Vec<FaceRecord<T>>,
    /// シェルの面
    pub shells: Vec<Vec<usize>>,
    /// 立体のシェル
    pub solids: Vec<Vec<usize>>,
}

/// 頂点の中身
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexRecord<T: Scalar> {
    pub point: Point3D<T>,
    pub tolerance: T,
}

/// 稜線の中身
#[derive(Debug, Clone)]
pub struct EdgeRecord<T: Scalar> {
    pub curve: CurveRef<T>,
    pub parameter_range: (T, T),
    pub start: usize,
    pub end: usize,
    pub tolerance: T,
}

/// 面の中身
#[derive(Debug, Clone)]
pub struct FaceRecord<T: Scalar> {
    pub surface: SurfaceRef<T>,
    pub same_sense: bool,
    /// ループ（稜線の使用の並び。稜線の番号と、稜線の向きに沿うか）
    pub loops: Vec<Vec<(usize, bool)>>,
}

impl<T: Scalar> Brep<T> {
    /// 表の中身
    pub fn records(&self) -> BrepRecords<T> {
        BrepRecords {
            tolerance: self.tolerance,
            vertices: self
                .vertices
                .iter()
                .map(|vertex| VertexRecord {
                    point: vertex.point,
                    tolerance: vertex.tolerance,
                })
                .collect(),
            edges: self
                .edges
                .iter()
                .map(|edge| EdgeRecord {
                    curve: edge.curve.clone(),
                    parameter_range: edge.parameter_range,
                    start: edge.start.0,
                    end: edge.end.0,
                    tolerance: edge.tolerance,
                })
                .collect(),
            faces: self
                .faces
                .iter()
                .map(|face| FaceRecord {
                    surface: face.surface.clone(),
                    same_sense: face.same_sense,
                    loops: face
                        .loops
                        .iter()
                        .map(|&loop_index| {
                            self.loops[loop_index.0]
                                .uses
                                .iter()
                                .map(|edge_use| (edge_use.edge.0, edge_use.forward))
                                .collect()
                        })
                        .collect(),
                })
                .collect(),
            shells: self
                .shells
                .iter()
                .map(|shell| shell.faces.iter().map(|face| face.0).collect())
                .collect(),
            solids: self
                .solids
                .iter()
                .map(|solid| solid.shells.iter().map(|shell| shell.0).collect())
                .collect(),
        }
    }

    /// 中身 `records` から表を戻す（取り出した表と、同じ番号・向き・稜線の使用の並び・パラメータ曲線になる）
    ///
    /// 頂点・稜線・面・シェル・立体を、この順に足す処理で作り直す。足す処理が確かめるのは、番号・許容誤差・点・
    /// 区間と、稜線をパラメータ曲線で表せるかまで。稜線の端と頂点の一致・ループのつながりは確かめないので、
    /// 表の確かめ（`validate`）は呼び出し元が行う（取り出した表が確かめに合わなければ、戻した表も合わない）。
    ///
    /// # Errors
    ///
    /// 足す処理が誤りを返すとき（許容誤差・点・区間が正しくない、表にない番号を指す、稜線をパラメータ曲線で
    /// 表せない、空のループ・シェル・立体、シェル・立体に同じ要素が 2 回あるなど）。
    pub fn from_records(records: BrepRecords<T>) -> Result<Self, BrepError> {
        let mut brep = Self::new(TopologyToleranceSettings::new(records.tolerance))?;
        for vertex in records.vertices {
            brep.add_vertex(vertex.point, vertex.tolerance)?;
        }
        for edge in records.edges {
            brep.add_edge(
                VertexIndex(edge.start),
                VertexIndex(edge.end),
                edge.curve,
                edge.parameter_range,
                edge.tolerance,
            )?;
        }
        for face in records.faces {
            let loops = face
                .loops
                .into_iter()
                .map(|uses| {
                    uses.into_iter()
                        .map(|(edge, forward)| EdgeUse {
                            edge: EdgeIndex(edge),
                            forward,
                        })
                        .collect()
                })
                .collect();
            brep.add_face(face.surface, face.same_sense, loops)?;
        }
        for faces in records.shells {
            brep.add_shell(faces.into_iter().map(FaceIndex).collect())?;
        }
        for shells in records.solids {
            brep.add_solid(shells.into_iter().map(ShellIndex).collect())?;
        }
        Ok(brep)
    }
}
