//! STEP の中間データ: エンティティの表を、幾何と位相の記録にする（設計: BREP_AND_STEP_IMPORT_DESIGN.md 9.2）
//!
//! 幾何の計算をせず、形状の型も使わない（単位の掛け算だけ）。長さは mm、角度は `analysis` の `Angle` にする。
//! 配置はファイルの値のまま持つ。読むのは、次元 3 の文脈の表現に載る立体とシェルと、そこからたどれる要素だけで、
//! ほかの形状（次元 2 の文脈・線や点だけの要素）は飛ばして、飛ばしたことを記録する。読めない要素も、理由を
//! 記録して飛ばす（ほかの要素は読み続ける）。参照は再帰でたどらず、順に読むため、入れ子の深さで止まらない。

mod parameters;
mod product;
mod short_names;
#[cfg(test)]
mod tests;
mod units;

pub use product::{Color, Occurrence, OccurrencePlacement, Product, Transformation};
pub use units::Context;

use crate::syntax::{Record, StepFile};
use analysis::abstract_types::Angle;
use analysis::consts::precision::GEOMETRIC_TOLERANCE;
use parameters::{
    at, boolean, integer, integers, list, optional_reference, part, real, reals, reference,
    references, text,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// B-spline の制御点の数の上限（1 つの曲線・曲面）
const MAX_CONTROL_POINTS: usize = 1_000_000;
/// B-spline の次数の上限
const MAX_DEGREE: i64 = 32;

/// 中間データ
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Records {
    /// ヘッダの `FILE_SCHEMA` のスキーマの名前
    pub schemas: Vec<String>,
    /// 表現の文脈（番号から）
    pub contexts: BTreeMap<u64, Context>,
    /// 立体・シェルを載せた、次元 3 の文脈の形状の表現（番号から）
    pub representations: BTreeMap<u64, Representation>,
    /// 幾何（点・方向・配置・曲線・曲面）
    pub geometry: BTreeMap<u64, Geometry>,
    /// 位相（頂点・稜線・ループ・面・シェル・立体）
    pub topology: BTreeMap<u64, Topology>,
    /// 製品（`PRODUCT_DEFINITION` の番号から）
    pub products: BTreeMap<u64, Product>,
    /// アセンブリの出現（`NEXT_ASSEMBLY_USAGE_OCCURRENCE` の番号から）
    pub occurrences: BTreeMap<u64, Occurrence>,
    /// 変換のない、形状の表現の間の関係（`SHAPE_REPRESENTATION_RELATIONSHIP` の `rep_1`・`rep_2`。ファイルの順）
    pub representation_links: Vec<(u64, u64)>,
    /// 立体・シェルなど、読んだ表現の要素の名前（要素の番号から）
    pub item_names: BTreeMap<u64, String>,
    /// 色（スタイルを割り当てた要素の番号から。面のスタイルの色を優先し、初めに割り当てたものを使う）
    pub colors: BTreeMap<u64, Color>,
    /// 非表示の要素（`INVISIBILITY`。スタイルの割り当ては、その対象の番号）
    pub invisible: BTreeSet<u64>,
    /// 読まなかった・読めなかった要素と、その理由
    pub skipped: Vec<Skipped>,
}

/// 形状の表現（読んだ立体・シェルの番号と、その文脈）
#[derive(Debug, Clone, PartialEq)]
pub struct Representation {
    pub name: String,
    pub context: u64,
    pub items: Vec<u64>,
}

/// 読まなかった・読めなかった要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub id: u64,
    /// エンティティの名前（複合のエンティティは、初めの部分の名前）
    pub name: String,
    pub reason: SkipReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// 次元 2 の文脈の表現（2D の形状）
    TwoDimensional,
    /// 立体・シェルでない、線や点だけの表現の要素（ワイヤーフレームなど）
    NotSolid,
    /// 中間データで扱わない種類のエンティティ
    Unsupported,
    /// 引数の形が規格と合わない
    Malformed,
    /// 参照の先のエンティティがない
    MissingReference,
    /// 単位の違う文脈から共有されている（初めの文脈の単位で読んだ）
    UnitConflict,
    /// B-spline の制御点が多すぎる、または次数が高すぎる
    TooLarge,
    /// 表現の文脈が、次元の分かる幾何の文脈でない
    UnknownContext,
}

/// 点（mm）
pub type Point = [f64; 3];

/// 幾何（番号はファイルの番号。長さは mm）
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Point(Point),
    /// 方向（ファイルの値のまま。大きさを 1 にしない）
    Direction([f64; 3]),
    Vector {
        direction: u64,
        magnitude: f64,
    },
    /// 配置（`AXIS2_PLACEMENT_3D`。軸と基準の方向はないことがある）
    Placement {
        location: u64,
        axis: Option<u64>,
        ref_direction: Option<u64>,
    },
    /// 軸（`AXIS1_PLACEMENT`）
    Axis {
        location: u64,
        axis: Option<u64>,
    },
    Line {
        point: u64,
        vector: u64,
    },
    Circle {
        placement: u64,
        radius: f64,
    },
    Ellipse {
        placement: u64,
        semi_axis_1: f64,
        semi_axis_2: f64,
    },
    Parabola {
        placement: u64,
        focal_distance: f64,
    },
    Hyperbola {
        placement: u64,
        semi_axis: f64,
        semi_imaginary_axis: f64,
    },
    BSplineCurve(BSplineCurve),
    /// 曲面の上の曲線・継ぎ目の曲線（3 次元の曲線を指す。ファイルのパラメータ曲線は使わない）
    SurfaceCurve {
        curve: u64,
    },
    Plane {
        placement: u64,
    },
    CylindricalSurface {
        placement: u64,
        radius: f64,
    },
    ConicalSurface {
        placement: u64,
        radius: f64,
        semi_angle: Angle<f64>,
    },
    SphericalSurface {
        placement: u64,
        radius: f64,
    },
    ToroidalSurface {
        placement: u64,
        major_radius: f64,
        minor_radius: f64,
    },
    BSplineSurface(BSplineSurface),
    SurfaceOfLinearExtrusion {
        curve: u64,
        vector: u64,
    },
    SurfaceOfRevolution {
        curve: u64,
        axis: u64,
    },
    OffsetSurface {
        surface: u64,
        distance: f64,
    },
}

/// B-spline 曲線（ノットは、重複の数とノットの値の組のまま）
#[derive(Debug, Clone, PartialEq)]
pub struct BSplineCurve {
    pub degree: i64,
    pub control_points: Vec<u64>,
    pub closed: Option<bool>,
    pub multiplicities: Vec<i64>,
    pub knots: Vec<f64>,
    /// 有理の曲線の重み（非有理なら `None`）
    pub weights: Option<Vec<f64>>,
}

/// B-spline 曲面（制御点は u 方向の行ごと）
#[derive(Debug, Clone, PartialEq)]
pub struct BSplineSurface {
    pub u_degree: i64,
    pub v_degree: i64,
    pub control_points: Vec<Vec<u64>>,
    pub u_closed: Option<bool>,
    pub v_closed: Option<bool>,
    pub u_multiplicities: Vec<i64>,
    pub v_multiplicities: Vec<i64>,
    pub u_knots: Vec<f64>,
    pub v_knots: Vec<f64>,
    pub weights: Option<Vec<Vec<f64>>>,
}

/// 位相（番号はファイルの番号。向きの旗はファイルの値のまま）
#[derive(Debug, Clone, PartialEq)]
pub enum Topology {
    Vertex {
        point: u64,
    },
    Edge {
        start: u64,
        end: u64,
        curve: u64,
        same_sense: Option<bool>,
    },
    OrientedEdge {
        edge: u64,
        orientation: Option<bool>,
    },
    EdgeLoop {
        edges: Vec<u64>,
    },
    VertexLoop {
        vertex: u64,
    },
    PolyLoop {
        points: Vec<u64>,
    },
    FaceBound {
        bound: u64,
        orientation: Option<bool>,
        outer: bool,
    },
    /// 面（曲面のない `FACE` は `surface` が `None`）
    Face {
        bounds: Vec<u64>,
        surface: Option<u64>,
        same_sense: Option<bool>,
    },
    OrientedFace {
        face: u64,
        orientation: Option<bool>,
    },
    Shell {
        faces: Vec<u64>,
        closed: bool,
    },
    OrientedShell {
        shell: u64,
        orientation: Option<bool>,
    },
    /// 立体（初めが外側のシェル、残りが空洞）
    Solid {
        outer: u64,
        voids: Vec<u64>,
    },
    /// シェルの集まりの面のモデル
    SurfaceModel {
        shells: Vec<u64>,
    },
}

/// 表現の要素のうち、立体・シェル・面の集まりとして読むもの
const SOLID_ITEMS: [&str; 10] = [
    "MANIFOLD_SOLID_BREP",
    "BREP_WITH_VOIDS",
    "FACETED_BREP",
    "SHELL_BASED_SURFACE_MODEL",
    "FACE_BASED_SURFACE_MODEL",
    "CLOSED_SHELL",
    "OPEN_SHELL",
    "CONNECTED_FACE_SET",
    "ORIENTED_CLOSED_SHELL",
    "ORIENTED_OPEN_SHELL",
];

/// 表現の要素のうち、形状ではなく、製品構造で使うため、飛ばしたと記録しないもの
const STRUCTURE_ITEMS: [&str; 4] = [
    "AXIS2_PLACEMENT_3D",
    "MAPPED_ITEM",
    "STYLED_ITEM",
    "OVER_RIDING_STYLED_ITEM",
];

/// エンティティの表を、幾何と位相の記録にする
///
/// 短い名前で書かれたエンティティは、表の中で正式な名前に直してから読む（表を受け取るのは、写さずに直すため）。
/// 読まなかった・読めなかった要素の記録の名前も、直した正式な名前になる（表にない短い名前は、書かれたとおり）。
pub fn read(mut file: StepFile) -> Records {
    to_long_names(&mut file);
    let file = &file;
    let mut reader = Reader {
        file,
        records: Records {
            schemas: file.schemas.clone(),
            ..Records::default()
        },
        units_of: BTreeMap::new(),
        conflicts: BTreeSet::new(),
        queue: VecDeque::new(),
    };
    reader.representations();
    reader.product_structure();
    reader.records
}

/// 短い名前で書かれたエンティティを、正式な名前に直す
fn to_long_names(file: &mut StepFile) {
    for record in file
        .entities
        .values_mut()
        .flat_map(|entity| entity.records.iter_mut())
    {
        if let Some(long) = short_names::long_name(&record.name) {
            record.name = long.to_string();
        }
    }
}

struct Reader<'a> {
    file: &'a StepFile,
    records: Records,
    /// 読んだ（読むと決めた）要素の、文脈の倍率（長さ、角度。単位の違う文脈から共有されていないかを見る）
    units_of: BTreeMap<u64, (f64, f64)>,
    /// 単位の違う文脈から共有されていると記録した要素
    conflicts: BTreeSet<u64>,
    /// 読む要素と、その文脈の倍率（長さ、角度）
    queue: VecDeque<(u64, f64, f64)>,
}

impl Reader<'_> {
    fn skip(&mut self, id: u64, reason: SkipReason) {
        let name = self
            .file
            .entities
            .get(&id)
            .and_then(|entity| entity.records.first())
            .map(|record| record.name.clone())
            .unwrap_or_default();
        self.records.skipped.push(Skipped { id, name, reason });
    }

    /// 形状の表現を探し、文脈の次元と要素の種類で、読む要素を決める
    fn representations(&mut self) {
        let file = self.file;
        for (&id, entity) in &file.entities {
            let Some(record) = entity.records.first() else {
                continue;
            };
            let (name, items, context) = match representation(record) {
                RepresentationRecord::Shape {
                    name,
                    items,
                    context,
                } => (name, items, context),
                RepresentationRecord::Malformed => {
                    self.skip(id, SkipReason::Malformed);
                    continue;
                }
                RepresentationRecord::Other => continue,
            };
            let context_record = match self.context(context) {
                Ok(context) => context,
                Err(reason) => {
                    self.skip(id, reason);
                    continue;
                }
            };
            let mut solids = Vec::new();
            for item in items {
                let Some(item_name) = file
                    .entities
                    .get(&item)
                    .and_then(|entity| entity.records.first())
                    .map(|record| record.name.as_str())
                else {
                    self.skip(item, SkipReason::MissingReference);
                    continue;
                };
                if STRUCTURE_ITEMS.contains(&item_name) {
                    continue;
                }
                match context_record.dimension {
                    Some(3) => {}
                    Some(2) => {
                        self.skip(item, SkipReason::TwoDimensional);
                        continue;
                    }
                    _ => {
                        self.skip(item, SkipReason::UnknownContext);
                        continue;
                    }
                }
                if SOLID_ITEMS.contains(&item_name) {
                    solids.push(item);
                } else {
                    self.skip(item, SkipReason::NotSolid);
                }
            }
            if solids.is_empty() {
                continue;
            }
            for &solid in &solids {
                self.enqueue(
                    solid,
                    context_record.length_to_mm,
                    context_record.angle_to_degrees,
                );
            }
            self.drain();
            self.records.representations.insert(
                id,
                Representation {
                    name,
                    context,
                    items: solids,
                },
            );
        }
    }

    /// 表現の文脈 `context`（読んだものは覚えておく。読めなければ理由）
    fn context(&mut self, context: u64) -> Result<Context, SkipReason> {
        if let Some(read) = self.records.contexts.get(&context) {
            return Ok(read.clone());
        }
        match units::context(self.file, context) {
            Some(read) => {
                self.records.contexts.insert(context, read.clone());
                Ok(read)
            }
            None if self.file.entities.contains_key(&context) => Err(SkipReason::UnknownContext),
            None => Err(SkipReason::MissingReference),
        }
    }

    /// 要素 `id` を、文脈の倍率（長さ `length`・角度 `angle`）で読む待ちに入れる（読むと決めた要素は入れ直さ
    /// ない。単位の違う文脈から共有されていれば、1 回だけ記録する）
    fn enqueue(&mut self, id: u64, length: f64, angle: f64) {
        match self.units_of.get(&id) {
            Some(&(read_length, read_angle)) => {
                if !(same_factor(read_length, length) && same_factor(read_angle, angle))
                    && self.conflicts.insert(id)
                {
                    self.skip(id, SkipReason::UnitConflict);
                }
            }
            None => {
                self.units_of.insert(id, (length, angle));
                self.queue.push_back((id, length, angle));
            }
        }
    }

    /// 待っている要素を、参照をたどりながら順に読む
    fn drain(&mut self) {
        while let Some((id, length, angle)) = self.queue.pop_front() {
            let Some(entity) = self.file.entities.get(&id) else {
                self.skip(id, SkipReason::MissingReference);
                continue;
            };
            let read = match entity.records.as_slice() {
                [record] => self.simple(id, record, length, angle),
                records => self.complex(id, records),
            };
            match read {
                Ok(Some(references)) => {
                    for reference in references {
                        self.enqueue(reference, length, angle);
                    }
                }
                Ok(None) => self.skip(id, SkipReason::Unsupported),
                Err(reason) => self.skip(id, reason),
            }
        }
    }

    fn geometry(&mut self, id: u64, geometry: Geometry) {
        self.records.geometry.insert(id, geometry);
    }

    fn topology(&mut self, id: u64, topology: Topology) {
        self.records.topology.insert(id, topology);
    }

    /// 単純なエンティティを読み、たどる参照を返す（扱わない種類なら `Ok(None)`）
    fn simple(
        &mut self,
        id: u64,
        record: &Record,
        length: f64,
        angle: f64,
    ) -> Result<Option<Vec<u64>>, SkipReason> {
        let malformed = || SkipReason::Malformed;
        let r = |index: usize| at(record, index).and_then(reference).ok_or_else(malformed);
        let len = |index: usize| {
            at(record, index)
                .and_then(real)
                .map(|value| value * length)
                .ok_or_else(malformed)
        };
        let flag = |index: usize| at(record, index).and_then(boolean);
        let refs = |index: usize| at(record, index).and_then(references).ok_or_else(malformed);
        let optional = |index: usize| {
            at(record, index)
                .and_then(optional_reference)
                .ok_or_else(malformed)
        };
        let follow = match record.name.as_str() {
            "CARTESIAN_POINT" => {
                let coordinates = at(record, 1).and_then(reals).ok_or_else(malformed)?;
                let [x, y, z] = coordinates.as_slice() else {
                    return Err(if coordinates.len() == 2 {
                        SkipReason::TwoDimensional
                    } else {
                        SkipReason::Malformed
                    });
                };
                self.geometry(id, Geometry::Point([x * length, y * length, z * length]));
                vec![]
            }
            "DIRECTION" => {
                let ratios = at(record, 1).and_then(reals).ok_or_else(malformed)?;
                let [x, y, z] = ratios.as_slice() else {
                    return Err(if ratios.len() == 2 {
                        SkipReason::TwoDimensional
                    } else {
                        SkipReason::Malformed
                    });
                };
                self.geometry(id, Geometry::Direction([*x, *y, *z]));
                vec![]
            }
            "VECTOR" => {
                let direction = r(1)?;
                self.geometry(
                    id,
                    Geometry::Vector {
                        direction,
                        magnitude: len(2)?,
                    },
                );
                vec![direction]
            }
            "AXIS2_PLACEMENT_3D" => {
                let (location, axis, ref_direction) = (r(1)?, optional(2)?, optional(3)?);
                self.geometry(
                    id,
                    Geometry::Placement {
                        location,
                        axis,
                        ref_direction,
                    },
                );
                [Some(location), axis, ref_direction]
                    .into_iter()
                    .flatten()
                    .collect()
            }
            "AXIS1_PLACEMENT" => {
                let (location, axis) = (r(1)?, optional(2)?);
                self.geometry(id, Geometry::Axis { location, axis });
                [Some(location), axis].into_iter().flatten().collect()
            }
            "LINE" => {
                let (point, vector) = (r(1)?, r(2)?);
                self.geometry(id, Geometry::Line { point, vector });
                vec![point, vector]
            }
            "CIRCLE" => {
                let placement = r(1)?;
                self.geometry(
                    id,
                    Geometry::Circle {
                        placement,
                        radius: len(2)?,
                    },
                );
                vec![placement]
            }
            "ELLIPSE" => {
                let placement = r(1)?;
                self.geometry(
                    id,
                    Geometry::Ellipse {
                        placement,
                        semi_axis_1: len(2)?,
                        semi_axis_2: len(3)?,
                    },
                );
                vec![placement]
            }
            "PARABOLA" => {
                let placement = r(1)?;
                self.geometry(
                    id,
                    Geometry::Parabola {
                        placement,
                        focal_distance: len(2)?,
                    },
                );
                vec![placement]
            }
            "HYPERBOLA" => {
                let placement = r(1)?;
                self.geometry(
                    id,
                    Geometry::Hyperbola {
                        placement,
                        semi_axis: len(2)?,
                        semi_imaginary_axis: len(3)?,
                    },
                );
                vec![placement]
            }
            "B_SPLINE_CURVE_WITH_KNOTS" => {
                // 単純なエンティティは、名前の後ろに曲線の引数（5 つ）、続けてノットの引数が並ぶ
                let curve = b_spline_curve(record, 1, record, 6, None)?;
                let follow = curve.control_points.clone();
                self.geometry(id, Geometry::BSplineCurve(curve));
                follow
            }
            // 曲面の上の曲線とその仲間（引数の並びは同じ）
            "SURFACE_CURVE" | "SEAM_CURVE" | "INTERSECTION_CURVE" | "BOUNDED_SURFACE_CURVE" => {
                let curve = r(1)?;
                self.geometry(id, Geometry::SurfaceCurve { curve });
                vec![curve]
            }
            "PLANE" => {
                let placement = r(1)?;
                self.geometry(id, Geometry::Plane { placement });
                vec![placement]
            }
            "CYLINDRICAL_SURFACE" => {
                let placement = r(1)?;
                self.geometry(
                    id,
                    Geometry::CylindricalSurface {
                        placement,
                        radius: len(2)?,
                    },
                );
                vec![placement]
            }
            "CONICAL_SURFACE" => {
                let placement = r(1)?;
                let semi_angle = at(record, 3).and_then(real).ok_or_else(malformed)?;
                self.geometry(
                    id,
                    Geometry::ConicalSurface {
                        placement,
                        radius: len(2)?,
                        semi_angle: angle_from(semi_angle, angle),
                    },
                );
                vec![placement]
            }
            "SPHERICAL_SURFACE" => {
                let placement = r(1)?;
                self.geometry(
                    id,
                    Geometry::SphericalSurface {
                        placement,
                        radius: len(2)?,
                    },
                );
                vec![placement]
            }
            "TOROIDAL_SURFACE" => {
                let placement = r(1)?;
                self.geometry(
                    id,
                    Geometry::ToroidalSurface {
                        placement,
                        major_radius: len(2)?,
                        minor_radius: len(3)?,
                    },
                );
                vec![placement]
            }
            "B_SPLINE_SURFACE_WITH_KNOTS" => {
                // 単純なエンティティは、名前の後ろに曲面の引数（7 つ）、続けてノットの引数が並ぶ
                let surface = b_spline_surface(record, 1, record, 8, None)?;
                let follow = surface.control_points.iter().flatten().copied().collect();
                self.geometry(id, Geometry::BSplineSurface(surface));
                follow
            }
            "SURFACE_OF_LINEAR_EXTRUSION" => {
                let (curve, vector) = (r(1)?, r(2)?);
                self.geometry(id, Geometry::SurfaceOfLinearExtrusion { curve, vector });
                vec![curve, vector]
            }
            "SURFACE_OF_REVOLUTION" => {
                let (curve, axis) = (r(1)?, r(2)?);
                self.geometry(id, Geometry::SurfaceOfRevolution { curve, axis });
                vec![curve, axis]
            }
            "OFFSET_SURFACE" => {
                let surface = r(1)?;
                self.geometry(
                    id,
                    Geometry::OffsetSurface {
                        surface,
                        distance: len(2)?,
                    },
                );
                vec![surface]
            }
            "VERTEX_POINT" => {
                let point = r(1)?;
                self.topology(id, Topology::Vertex { point });
                vec![point]
            }
            "EDGE_CURVE" => {
                let (start, end, curve) = (r(1)?, r(2)?, r(3)?);
                self.topology(
                    id,
                    Topology::Edge {
                        start,
                        end,
                        curve,
                        same_sense: flag(4),
                    },
                );
                vec![start, end, curve]
            }
            "ORIENTED_EDGE" => {
                let edge = r(3)?;
                self.topology(
                    id,
                    Topology::OrientedEdge {
                        edge,
                        orientation: flag(4),
                    },
                );
                vec![edge]
            }
            "EDGE_LOOP" => {
                let edges = refs(1)?;
                self.topology(
                    id,
                    Topology::EdgeLoop {
                        edges: edges.clone(),
                    },
                );
                edges
            }
            "VERTEX_LOOP" => {
                let vertex = r(1)?;
                self.topology(id, Topology::VertexLoop { vertex });
                vec![vertex]
            }
            "POLY_LOOP" => {
                let points = refs(1)?;
                self.topology(
                    id,
                    Topology::PolyLoop {
                        points: points.clone(),
                    },
                );
                points
            }
            "FACE_BOUND" | "FACE_OUTER_BOUND" => {
                let bound = r(1)?;
                self.topology(
                    id,
                    Topology::FaceBound {
                        bound,
                        orientation: flag(2),
                        outer: record.name == "FACE_OUTER_BOUND",
                    },
                );
                vec![bound]
            }
            "ADVANCED_FACE" | "FACE_SURFACE" => {
                let (bounds, surface) = (refs(1)?, r(2)?);
                let mut follow = bounds.clone();
                follow.push(surface);
                self.topology(
                    id,
                    Topology::Face {
                        bounds,
                        surface: Some(surface),
                        same_sense: flag(3),
                    },
                );
                follow
            }
            "FACE" => {
                let bounds = refs(1)?;
                self.topology(
                    id,
                    Topology::Face {
                        bounds: bounds.clone(),
                        surface: None,
                        same_sense: None,
                    },
                );
                bounds
            }
            "ORIENTED_FACE" => {
                let face = r(2)?;
                self.topology(
                    id,
                    Topology::OrientedFace {
                        face,
                        orientation: flag(3),
                    },
                );
                vec![face]
            }
            "CLOSED_SHELL" | "OPEN_SHELL" | "CONNECTED_FACE_SET" => {
                let faces = refs(1)?;
                self.topology(
                    id,
                    Topology::Shell {
                        faces: faces.clone(),
                        closed: record.name == "CLOSED_SHELL",
                    },
                );
                faces
            }
            "ORIENTED_CLOSED_SHELL" | "ORIENTED_OPEN_SHELL" => {
                let shell = r(2)?;
                self.topology(
                    id,
                    Topology::OrientedShell {
                        shell,
                        orientation: flag(3),
                    },
                );
                vec![shell]
            }
            "MANIFOLD_SOLID_BREP" | "FACETED_BREP" => {
                let outer = r(1)?;
                self.topology(
                    id,
                    Topology::Solid {
                        outer,
                        voids: vec![],
                    },
                );
                vec![outer]
            }
            "BREP_WITH_VOIDS" => {
                let (outer, voids) = (r(1)?, refs(2)?);
                let mut follow = voids.clone();
                follow.push(outer);
                self.topology(id, Topology::Solid { outer, voids });
                follow
            }
            "SHELL_BASED_SURFACE_MODEL" | "FACE_BASED_SURFACE_MODEL" => {
                let shells = refs(1)?;
                self.topology(
                    id,
                    Topology::SurfaceModel {
                        shells: shells.clone(),
                    },
                );
                shells
            }
            _ => return Ok(None),
        };
        Ok(Some(follow))
    }

    /// 複合のエンティティ（有理の B-spline 曲線・曲面）を読む
    fn complex(&mut self, id: u64, records: &[Record]) -> Result<Option<Vec<u64>>, SkipReason> {
        if let (Some(base), Some(knots)) = (
            part(records, "B_SPLINE_CURVE"),
            part(records, "B_SPLINE_CURVE_WITH_KNOTS"),
        ) {
            let weights = part(records, "RATIONAL_B_SPLINE_CURVE")
                .map(|rational| at(rational, 0).and_then(reals).ok_or(SkipReason::Malformed))
                .transpose()?;
            let curve = b_spline_curve(base, 0, knots, 0, weights)?;
            let follow = curve.control_points.clone();
            self.geometry(id, Geometry::BSplineCurve(curve));
            return Ok(Some(follow));
        }
        if let (Some(base), Some(knots)) = (
            part(records, "B_SPLINE_SURFACE"),
            part(records, "B_SPLINE_SURFACE_WITH_KNOTS"),
        ) {
            let weights = part(records, "RATIONAL_B_SPLINE_SURFACE")
                .map(|rational| {
                    at(rational, 0)
                        .and_then(list)
                        .and_then(|rows| rows.iter().map(reals).collect::<Option<Vec<_>>>())
                        .ok_or(SkipReason::Malformed)
                })
                .transpose()?;
            let surface = b_spline_surface(base, 0, knots, 0, weights)?;
            let follow = surface.control_points.iter().flatten().copied().collect();
            self.geometry(id, Geometry::BSplineSurface(surface));
            return Ok(Some(follow));
        }
        Ok(None)
    }
}

/// 倍率 `a` と `b` が同じか（換算の連なりの違いによる丸めは同じとみなす）
fn same_factor(a: f64, b: f64) -> bool {
    (a - b).abs() <= GEOMETRIC_TOLERANCE * a.abs().max(b.abs())
}

/// 文脈の角度の値 `value` の `Angle`（度への倍率が `to_degrees`。ラジアン・度は、そのまま作る）
fn angle_from(value: f64, to_degrees: f64) -> Angle<f64> {
    if to_degrees == units::DEGREES_PER_RADIAN {
        Angle::from_radians(value)
    } else {
        Angle::from_degrees(value * to_degrees)
    }
}

/// 表現のエンティティを見分けた結果
enum RepresentationRecord {
    /// 形状の表現（名前・要素・文脈）
    Shape {
        name: String,
        items: Vec<u64>,
        context: u64,
    },
    /// 形状の表現の名前だが、引数の形が合わない
    Malformed,
    /// 形状の表現でない
    Other,
}

/// 形状の表現の名前・要素・文脈
///
/// パラメータ曲線の表現（`DEFINITIONAL_REPRESENTATION`）、色などの表示の表現（名前に `PRESENTATION` を
/// 含むもの）、製品と表現を結ぶもの（`SHAPE_DEFINITION_REPRESENTATION` など、引数が 2 つのもの）は、形状の
/// 表現として扱わない。
fn representation(record: &Record) -> RepresentationRecord {
    let name = record.name.as_str();
    if !name.ends_with("REPRESENTATION")
        || name == "DEFINITIONAL_REPRESENTATION"
        || name.contains("PRESENTATION_")
        || record.parameters.len() != 3
    {
        return RepresentationRecord::Other;
    }
    match (
        at(record, 1).and_then(references),
        at(record, 2).and_then(reference),
    ) {
        (Some(items), Some(context)) => RepresentationRecord::Shape {
            name: at(record, 0).and_then(text).unwrap_or_default().to_string(),
            items,
            context,
        },
        _ => RepresentationRecord::Malformed,
    }
}

/// B-spline 曲線（曲線の引数は `base` の `base_offset` 番目から、ノットの引数は `knots` の `knot_offset`
/// 番目から並ぶ）
fn b_spline_curve(
    base: &Record,
    base_offset: usize,
    knots: &Record,
    knot_offset: usize,
    weights: Option<Vec<f64>>,
) -> Result<BSplineCurve, SkipReason> {
    let malformed = || SkipReason::Malformed;
    let degree = at(base, base_offset)
        .and_then(integer)
        .ok_or_else(malformed)?;
    let control_points = at(base, base_offset + 1)
        .and_then(references)
        .ok_or_else(malformed)?;
    check_degree(degree)?;
    check_count(control_points.len())?;
    Ok(BSplineCurve {
        degree,
        closed: at(base, base_offset + 3).and_then(boolean),
        control_points,
        multiplicities: at(knots, knot_offset)
            .and_then(integers)
            .ok_or_else(malformed)?,
        knots: at(knots, knot_offset + 1)
            .and_then(reals)
            .ok_or_else(malformed)?,
        weights,
    })
}

/// B-spline 曲面（引数の並びは `b_spline_curve` と同じ考え方）
fn b_spline_surface(
    base: &Record,
    base_offset: usize,
    knots: &Record,
    knot_offset: usize,
    weights: Option<Vec<Vec<f64>>>,
) -> Result<BSplineSurface, SkipReason> {
    let malformed = || SkipReason::Malformed;
    let u_degree = at(base, base_offset)
        .and_then(integer)
        .ok_or_else(malformed)?;
    let v_degree = at(base, base_offset + 1)
        .and_then(integer)
        .ok_or_else(malformed)?;
    let control_points: Vec<Vec<u64>> = at(base, base_offset + 2)
        .and_then(list)
        .and_then(|rows| rows.iter().map(references).collect())
        .ok_or_else(malformed)?;
    check_degree(u_degree)?;
    check_degree(v_degree)?;
    check_count(control_points.iter().map(Vec::len).sum())?;
    let integers_at = |index| at(knots, index).and_then(integers).ok_or_else(malformed);
    let reals_at = |index| at(knots, index).and_then(reals).ok_or_else(malformed);
    Ok(BSplineSurface {
        u_degree,
        v_degree,
        u_closed: at(base, base_offset + 4).and_then(boolean),
        v_closed: at(base, base_offset + 5).and_then(boolean),
        control_points,
        u_multiplicities: integers_at(knot_offset)?,
        v_multiplicities: integers_at(knot_offset + 1)?,
        u_knots: reals_at(knot_offset + 2)?,
        v_knots: reals_at(knot_offset + 3)?,
        weights,
    })
}

/// B-spline の次数（1 より小さければ形が合わない、上限より高ければ大きすぎる）
fn check_degree(degree: i64) -> Result<(), SkipReason> {
    if degree < 1 {
        Err(SkipReason::Malformed)
    } else if degree > MAX_DEGREE {
        Err(SkipReason::TooLarge)
    } else {
        Ok(())
    }
}

/// B-spline の制御点の数の上限
fn check_count(control_points: usize) -> Result<(), SkipReason> {
    if control_points > MAX_CONTROL_POINTS {
        Err(SkipReason::TooLarge)
    } else {
        Ok(())
    }
}
