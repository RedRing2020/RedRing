//! CAM ソルバー入力契約と実行入口（設計: CAM_ALGORITHMS_DESIGN.md §8.2-§8.5）

use geo_contracts::{Bounded, TriangleMesh3DProperties, default_distance_tolerance};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};

use cam_core::{
    CoordinateFrame, LengthUnit, MachiningBoundary, OperationSpec, RectangleBoundary,
    ScanlineParams, ScanlineToolRange, Tool, ToolPath,
};
use geo_algorithms::{
    Aabb3D, NurbsSurface3D, Point3D, SurfaceTessellationError, TessellationLimits, TriangleMesh3D,
    tessellate_surfaces,
};

use crate::inverse_offset::CutterShape;
use crate::inverse_offset_shape::InverseOffsetShape;
use crate::process_template::{TemplateFailure, validate_rectangle_boundary};
use crate::scanline::generate_scanline_toolpath;

/// solver の失敗分類。
///
/// Job Manager へは `code()` の分類のみを伝達し、幾何計算の内部状態は公開しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CamSolverError {
    /// 入力契約エラー（必須項目欠落、値域外、未対応の組み合わせ）
    InvalidInput(String),
    /// 走査線の条件の誤り（入力契約エラーの 1 つ）
    InvalidScanline(ScanlineInputError),
    /// 入力契約は妥当だが、幾何制約下で有効経路を構築できない
    NoSolution(String),
    /// 入力契約は妥当だが、反復解法が収束条件を満たさない
    ConvergenceFailure(String),
}

impl CamSolverError {
    /// 失敗分類の契約キー
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) | Self::InvalidScanline(_) => "invalid_input",
            Self::NoSolution(_) => "no_solution",
            Self::ConvergenceFailure(_) => "convergence_failure",
        }
    }

    /// 失敗理由
    pub fn reason(&self) -> &str {
        match self {
            Self::InvalidInput(reason)
            | Self::NoSolution(reason)
            | Self::ConvergenceFailure(reason) => reason,
            Self::InvalidScanline(error) => error.reason(),
        }
    }
}

/// 走査線の条件（`ScanlineParams`）の誤り
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanlineInputError {
    /// ピックフィードが正の有限値でない
    StepoverNotPositive,
    /// ピックフィードが工具径を超える
    StepoverExceedsToolDiameter,
    /// 送りが正の有限値でない
    FeedRateNotPositive,
}

impl ScanlineInputError {
    /// 失敗理由
    pub fn reason(self) -> &'static str {
        match self {
            Self::StepoverNotPositive => "scanline stepover must be positive and finite",
            Self::StepoverExceedsToolDiameter => "scanline stepover must not exceed tool diameter",
            Self::FeedRateNotPositive => "scanline feed_rate must be positive and finite",
        }
    }
}

impl Display for CamSolverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.reason())
    }
}

impl Error for CamSolverError {}

/// solver が受理する形状種別（`geometry_kind`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryKind {
    /// `nurbs_surface_set`: NURBS 曲面集合（solver 内で三角形へ離散化）
    NurbsSurfaceSet,
    /// `triangle_mesh`: 離散化済み三角形メッシュ（STL 等）
    TriangleMesh,
}

impl GeometryKind {
    /// 契約上の canonical token
    pub fn token(self) -> &'static str {
        match self {
            Self::NurbsSurfaceSet => "nurbs_surface_set",
            Self::TriangleMesh => "triangle_mesh",
        }
    }
}

/// solver 入力形状
#[derive(Debug, Clone)]
pub enum SolverGeometry {
    NurbsSurfaceSet(Vec<NurbsSurface3D<f64>>),
    TriangleMesh(TriangleMesh3D<f64>),
}

impl SolverGeometry {
    pub fn kind(&self) -> GeometryKind {
        match self {
            Self::NurbsSurfaceSet(_) => GeometryKind::NurbsSurfaceSet,
            Self::TriangleMesh(_) => GeometryKind::TriangleMesh,
        }
    }

    /// 形状を囲む境界ボックス（形状が空の場合は `None`）
    ///
    /// 曲面は制御点を包む境界ボックス（曲面は制御点の凸包に含まれる）とする。
    pub fn aabb(&self) -> Option<Aabb3D<f64>> {
        match self {
            Self::TriangleMesh(mesh) => mesh.aabb(),
            Self::NurbsSurfaceSet(surfaces) => {
                let corners: Vec<Point3D<f64>> = surfaces
                    .iter()
                    .filter_map(Bounded::aabb)
                    .flat_map(|aabb| [aabb.min(), aabb.max()])
                    .collect();
                Aabb3D::from_points(&corners)
            }
        }
    }

    /// 形状の三角形メッシュ（曲面は弦の許容誤差 `chord_tolerance` で三角形に分ける）
    ///
    /// # Errors
    ///
    /// 曲面の三角形への分割が収束しない場合。
    pub fn triangle_mesh(
        &self,
        chord_tolerance: f64,
        limits: TessellationLimits,
    ) -> Result<Cow<'_, TriangleMesh3D<f64>>, CamSolverError> {
        match self {
            Self::TriangleMesh(mesh) => Ok(Cow::Borrowed(mesh)),
            Self::NurbsSurfaceSet(surfaces) => {
                tessellate_surfaces(surfaces, chord_tolerance, limits)
                    .map(Cow::Owned)
                    .map_err(CamSolverError::from)
            }
        }
    }
}

impl From<SurfaceTessellationError> for CamSolverError {
    fn from(value: SurfaceTessellationError) -> Self {
        match value {
            SurfaceTessellationError::ConvergenceFailure { .. } => {
                Self::ConvergenceFailure(value.to_string())
            }
            SurfaceTessellationError::InvalidMesh(error) => Self::InvalidInput(error.to_string()),
        }
    }
}

/// solver 入力契約（`InputRef` の参照先 payload）
#[derive(Debug, Clone)]
pub struct CamSolverInput {
    pub operation_id: String,
    pub tool: Tool<f64>,
    pub geometry: SolverGeometry,
    pub operation: OperationSpec,
    pub units: LengthUnit,
    pub coordinate_frame: CoordinateFrame,
    /// 形状離散化の許容弦誤差（mm）
    pub chord_tolerance: f64,
    /// 形状離散化の反復上限
    pub tessellation_limits: TessellationLimits,
    /// 加工範囲（未指定なら形状の XY 範囲全体）
    pub boundary: Option<MachiningBoundary>,
}

impl CamSolverInput {
    /// 入力契約を検証する。違反はすべて `invalid_input` に分類する。
    pub fn validate(&self) -> Result<(), CamSolverError> {
        if self.operation_id.trim().is_empty() {
            return Err(invalid("operation_id is required"));
        }
        if self.tool.id.trim().is_empty() {
            return Err(invalid("tool_id is required"));
        }
        if !is_positive_finite(self.tool.radius()) {
            return Err(invalid("tool radius must be positive and finite"));
        }
        cutter_shape(&self.tool)?;
        if !is_positive_finite(self.chord_tolerance) {
            return Err(invalid("chord_tolerance must be positive and finite"));
        }

        match self.operation {
            OperationSpec::Scanline(params) => validate_scanline(&params, &self.tool)?,
        }

        if let Some(MachiningBoundary::Rectangle(rectangle)) = &self.boundary {
            validate_rectangle_boundary(rectangle)?;
        }

        match &self.geometry {
            SolverGeometry::NurbsSurfaceSet(surfaces) if surfaces.is_empty() => Err(invalid(
                "nurbs_surface_set must contain at least one surface",
            )),
            SolverGeometry::TriangleMesh(mesh) if mesh.is_empty() || !mesh.is_valid() => {
                Err(invalid("triangle_mesh must be non-empty and valid"))
            }
            _ => Ok(()),
        }
    }
}

/// 入力契約から ToolPath を生成する。
///
/// 1. 入力契約検証（`invalid_input`）
/// 2. 形状離散化（収束しなければ `convergence_failure`）
/// 3. 逆オフセットによる CL 算出と経路化（経路が得られなければ `no_solution`）
pub fn solve_toolpath(input: &CamSolverInput) -> Result<ToolPath<f64>, CamSolverError> {
    let cutter = build_inverse_offset_shape(input)?;
    let OperationSpec::Scanline(params) = &input.operation;
    let region = machining_region(&cutter, input.boundary, params.tool_range)?;

    match input.operation {
        OperationSpec::Scanline(params) => {
            // 三角形メッシュは三角形そのものが製品形状のため、トレランスをすべて経路に使う（曲面の三角形への分割の
            // 誤差との配分は後続）
            generate_scanline_toolpath(
                &cutter,
                &input.tool.id,
                &params,
                region,
                input.chord_tolerance,
            )
        }
    }
}

/// 入力を確かめ、ツールパス生成と同じ形状と工具で逆オフセット形状を作る
///
/// # Errors
///
/// 入力の誤り、形状の三角形への分割の失敗、未対応の工具。
pub fn build_inverse_offset_shape(
    input: &CamSolverInput,
) -> Result<InverseOffsetShape, CamSolverError> {
    input.validate()?;
    let mesh = input
        .geometry
        .triangle_mesh(input.chord_tolerance, input.tessellation_limits)?;
    InverseOffsetShape::new(&mesh, cutter_shape(&input.tool)?)
}

/// 経路生成の XY 範囲を決める。
///
/// 加工範囲の指定がなければ形状の XY 範囲とする。矩形指定は、工具が形状に触れ得る範囲
/// （形状の XY 範囲を工具半径だけ広げた領域、以下 reach）でクリップした範囲を走査する
/// （reach 外は接触し得ないため、過大な矩形でもサンプル数を形状規模に抑える）。
/// クリップ結果が面積を持たない（重ならない、または辺・角で接するだけ）場合は、
/// 経路生成まで進めて `no_solution` にせず、入力契約違反の `operation_boundary_out_of_domain` とする。
fn machining_region(
    cutter: &InverseOffsetShape,
    boundary: Option<MachiningBoundary>,
    tool_range: ScanlineToolRange,
) -> Result<([f64; 2], [f64; 2]), CamSolverError> {
    let (xy_min, xy_max) = cutter.xy_bounds();
    let r = cutter.shape().radius();
    let reach_min = [xy_min[0] - r, xy_min[1] - r];
    let reach_max = [xy_max[0] + r, xy_max[1] + r];
    match boundary {
        None => match tool_range {
            ScanlineToolRange::ExtendByToolRadius => Ok((reach_min, reach_max)),
            ScanlineToolRange::GeometryExtent => Ok((xy_min, xy_max)),
        },
        Some(MachiningBoundary::Rectangle(rectangle)) => {
            clip_rectangle_boundary(&rectangle, reach_min, reach_max)
                .ok_or_else(|| TemplateFailure::OperationBoundaryOutOfDomain.into())
        }
    }
}

fn validate_scanline(params: &ScanlineParams, tool: &Tool<f64>) -> Result<(), CamSolverError> {
    let error = if !is_positive_finite(params.stepover) {
        ScanlineInputError::StepoverNotPositive
    } else if params.stepover > tool.diameter() {
        ScanlineInputError::StepoverExceedsToolDiameter
    } else if !is_positive_finite(params.feed_rate) {
        ScanlineInputError::FeedRateNotPositive
    } else {
        return Ok(());
    };
    Err(CamSolverError::InvalidScanline(error))
}

/// 工具種別を逆オフセット形状へ対応付ける。未対応工具は `invalid_input`。
fn cutter_shape(tool: &Tool<f64>) -> Result<CutterShape, CamSolverError> {
    let radius = tool.radius();
    if tool.is_ball_end_mill() {
        Ok(CutterShape::Ball { radius })
    } else if tool.is_flat_end_mill() {
        Ok(CutterShape::Flat { radius })
    } else {
        Err(invalid(
            "inverse offset solver supports ball and flat end mills only",
        ))
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn invalid(reason: &str) -> CamSolverError {
    CamSolverError::InvalidInput(reason.to_string())
}

/// 矩形の加工範囲を XY 範囲 [min, max] でクリップした範囲を返す。
///
/// クリップ結果の幅・高さのいずれかが距離トレランス以下（辺や角で接するだけ、
/// または重ならない）の場合は、走査できる面積を持たないため `None` とする。
fn clip_rectangle_boundary(
    rectangle: &RectangleBoundary,
    min: [f64; 2],
    max: [f64; 2],
) -> Option<([f64; 2], [f64; 2])> {
    let clipped_min = [
        rectangle.rect_min[0].max(min[0]),
        rectangle.rect_min[1].max(min[1]),
    ];
    let clipped_max = [
        rectangle.rect_max[0].min(max[0]),
        rectangle.rect_max[1].min(max[1]),
    ];
    let tolerance = default_distance_tolerance::<f64>();
    let has_area =
        clipped_max[0] - clipped_min[0] > tolerance && clipped_max[1] - clipped_min[1] > tolerance;
    has_area.then_some((clipped_min, clipped_max))
}
