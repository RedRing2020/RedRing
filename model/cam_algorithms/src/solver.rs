//! CAM ソルバー入力契約と実行入口（設計: CAM_ALGORITHMS_DESIGN.md §8.2-§8.5）

use std::error::Error;
use std::fmt::{Display, Formatter};

use cam_core::{CoordinateFrame, LengthUnit, Tool, ToolPath};
use geo_algorithms::{NurbsSurface3D, TriangleMesh3D};

use crate::inverse_offset::{CutterShape, DropCutter};
use crate::process_template::{MachiningBoundary, TemplateFailure};
use crate::scanline::generate_scanline_toolpath;
use crate::tessellation::{TessellationLimits, tessellate_surfaces};

/// solver の失敗分類。
///
/// Job Manager へは `code()` の分類のみを伝達し、幾何計算の内部状態は公開しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CamSolverError {
    /// 入力契約エラー（必須項目欠落、値域外、未対応の組み合わせ）
    InvalidInput(String),
    /// 入力契約は妥当だが、幾何制約下で有効経路を構築できない
    NoSolution(String),
    /// 入力契約は妥当だが、反復解法が収束条件を満たさない
    ConvergenceFailure(String),
}

impl CamSolverError {
    /// 失敗分類の契約キー
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid_input",
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
}

/// スキャン加工（`operation_type = scanline`）パラメータ
///
/// X 方向の片方向走査を Y 方向へ `stepover` ずつ送る。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScanlineParams {
    /// 走査ライン間隔（mm）。工具径以下であること
    pub stepover: f64,
    /// 走査方向の CL サンプリング間隔（mm）
    pub sample_pitch: f64,
    /// 切削送り速度（mm/min）
    pub feed_rate: f64,
    /// CL 最高点から早送り高さまでの逃げ量（mm）
    pub clearance_height: f64,
}

/// オペレーション定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationSpec {
    Scanline(ScanlineParams),
}

impl OperationSpec {
    /// `operation_type` の canonical token
    pub fn operation_type(&self) -> &'static str {
        match self {
            Self::Scanline(_) => "scanline",
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
            rectangle.validate()?;
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
    input.validate()?;

    let tessellated;
    let mesh = match &input.geometry {
        SolverGeometry::TriangleMesh(mesh) => mesh,
        SolverGeometry::NurbsSurfaceSet(surfaces) => {
            tessellated =
                tessellate_surfaces(surfaces, input.chord_tolerance, input.tessellation_limits)?;
            &tessellated
        }
    };

    let cutter = DropCutter::new(mesh, cutter_shape(&input.tool)?)?;
    let region = machining_region(&cutter, input.boundary)?;

    match input.operation {
        OperationSpec::Scanline(params) => {
            generate_scanline_toolpath(&cutter, &input.tool.id, &params, region)
        }
    }
}

/// 経路生成の XY 範囲を決める。
///
/// 加工範囲の指定がなければ形状の XY 範囲とする。矩形指定は、工具が形状に触れ得る範囲
/// （形状の XY 範囲を工具半径だけ広げた領域、以下 reach）と重ならなければ
/// `operation_boundary_out_of_domain`。重なる場合は矩形を reach でクリップした範囲を走査する
/// （reach 外は接触し得ないため、過大な矩形でもサンプル数を形状規模に抑える）。
fn machining_region(
    cutter: &DropCutter,
    boundary: Option<MachiningBoundary>,
) -> Result<([f64; 2], [f64; 2]), CamSolverError> {
    let (xy_min, xy_max) = cutter.xy_bounds();
    match boundary {
        None => Ok((xy_min, xy_max)),
        Some(MachiningBoundary::Rectangle(rectangle)) => {
            let r = cutter.shape().radius();
            let reach_min = [xy_min[0] - r, xy_min[1] - r];
            let reach_max = [xy_max[0] + r, xy_max[1] + r];
            if !rectangle.overlaps(reach_min, reach_max) {
                return Err(TemplateFailure::OperationBoundaryOutOfDomain.into());
            }
            Ok((
                [
                    rectangle.rect_min[0].max(reach_min[0]),
                    rectangle.rect_min[1].max(reach_min[1]),
                ],
                [
                    rectangle.rect_max[0].min(reach_max[0]),
                    rectangle.rect_max[1].min(reach_max[1]),
                ],
            ))
        }
    }
}

fn validate_scanline(params: &ScanlineParams, tool: &Tool<f64>) -> Result<(), CamSolverError> {
    if !is_positive_finite(params.stepover) {
        return Err(invalid("scanline stepover must be positive and finite"));
    }
    if params.stepover > tool.diameter() {
        return Err(invalid("scanline stepover must not exceed tool diameter"));
    }
    if !is_positive_finite(params.sample_pitch) {
        return Err(invalid("scanline sample_pitch must be positive and finite"));
    }
    if !is_positive_finite(params.feed_rate) {
        return Err(invalid("scanline feed_rate must be positive and finite"));
    }
    if !is_positive_finite(params.clearance_height) {
        return Err(invalid(
            "scanline clearance_height must be positive and finite",
        ));
    }
    Ok(())
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
