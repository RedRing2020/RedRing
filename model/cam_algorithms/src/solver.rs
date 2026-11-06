//! CAM ソルバー入力契約と実行入口（設計: CAM_ALGORITHMS_DESIGN.md §8.2-§8.5）

use geo_contracts::{
    Bounded, TriangleMesh3DConstructor, TriangleMesh3DProperties, default_distance_tolerance,
};
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

use crate::inverse_offset::{CutterShape, MeshTriangles};
use crate::inverse_offset_shape::InverseOffsetShape;
use crate::process_template::{TemplateFailure, validate_rectangle_boundary};
use crate::scan_frame::{ScanFrame, ScanRegion};
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
    /// 走査線の角度が有限値でない
    AngleNotFinite,
}

impl ScanlineInputError {
    /// 失敗理由
    pub fn reason(self) -> &'static str {
        match self {
            Self::StepoverNotPositive => "scanline stepover must be positive and finite",
            Self::StepoverExceedsToolDiameter => "scanline stepover must not exceed tool diameter",
            Self::FeedRateNotPositive => "scanline feed_rate must be positive and finite",
            Self::AngleNotFinite => "scanline angle must be finite",
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
///
/// 走査線に角度がある場合は、形状を走査線の向きの座標へ回して逆オフセット形状を作り、0° の走査線で経路を作って、
/// 点を加工の座標へ戻す（`ScanFrame`）。工具の中心の移動範囲と加工範囲は加工の座標の長方形とし、走査線の向きの座標へ
/// 回して走査する。
pub fn solve_toolpath(input: &CamSolverInput) -> Result<ToolPath<f64>, CamSolverError> {
    let OperationSpec::Scanline(params) = input.operation;
    let frame = ScanFrame::from_degrees(params.angle_degrees);
    let (cutter, work_bounds) = build_cutter(input, frame)?;
    let region = machining_region(
        &cutter,
        work_bounds,
        frame,
        input.boundary,
        params.tool_range,
    )?;
    // 三角形メッシュは三角形そのものが製品形状のため、トレランスをすべて経路に使う（曲面の三角形への分割の誤差との
    // 配分は後続）
    generate_scanline_toolpath(
        &cutter,
        &input.tool.id,
        &params,
        (&region, frame),
        input.chord_tolerance,
    )
}

/// 入力を確かめ、ツールパス生成と同じ形状と工具で逆オフセット形状を作る（加工の座標。走査線の角度によらない）
///
/// # Errors
///
/// 入力の誤り、形状の三角形への分割の失敗、未対応の工具。
pub fn build_inverse_offset_shape(
    input: &CamSolverInput,
) -> Result<InverseOffsetShape, CamSolverError> {
    build_cutter(input, ScanFrame::from_degrees(0.0)).map(|(cutter, _)| cutter)
}

/// XY の範囲（min, max）
type XyBounds = ([f64; 2], [f64; 2]);

/// 入力を確かめ、形状を走査線の向きの座標 `frame` へ回して逆オフセット形状を作る。加工の座標での形状の XY 範囲
/// （min, max）も返す
fn build_cutter(
    input: &CamSolverInput,
    frame: ScanFrame,
) -> Result<(InverseOffsetShape, XyBounds), CamSolverError> {
    input.validate()?;
    let mesh = input
        .geometry
        .triangle_mesh(input.chord_tolerance, input.tessellation_limits)?;
    let shape = cutter_shape(&input.tool)?;
    if frame.is_identity() {
        let cutter = InverseOffsetShape::new(&mesh, shape)?;
        let work_bounds = cutter.xy_bounds();
        return Ok((cutter, work_bounds));
    }
    // 逆オフセット形状の XY 範囲と同じく、面積のない三角形を除いた範囲
    let triangles = MeshTriangles::new(&mesh)?;
    let work_bounds = (triangles.xy_min, triangles.xy_max);
    let vertices = mesh
        .vertices()
        .iter()
        .map(|vertex| {
            let [x, y] = frame.to_scan([vertex.x(), vertex.y()]);
            Point3D::new(x, y, vertex.z())
        })
        .collect();
    let rotated = TriangleMesh3D::new(vertices, mesh.indices().to_vec())
        .map_err(|error| invalid(&error.to_string()))?;
    Ok((InverseOffsetShape::new(&rotated, shape)?, work_bounds))
}

/// 経路生成の範囲を、走査線の向きの座標 `frame` で決める（`cutter` は `frame` へ回した形状から作ったもの、
/// `work_bounds` は加工の座標での形状の XY 範囲）。
///
/// 範囲は加工の座標の長方形とし、走査線の向きの座標へ回して、その座標で工具が形状に触れ得る範囲（形状の XY 範囲を
/// 工具半径だけ広げた領域、以下 reach）で切り取る（reach 外は接触し得ないため、過大な矩形でもサンプル数を形状規模に
/// 抑える）。加工範囲の指定がなければ、加工の座標での形状の XY 範囲を工具の中心の移動範囲 `tool_range` で決める。
/// 矩形指定で、切り取った範囲が面積を持たない（重ならない、または辺・角で接するだけ）場合は、経路生成まで進めて
/// `no_solution` にせず、入力契約違反の `operation_boundary_out_of_domain` とする。
fn machining_region(
    cutter: &InverseOffsetShape,
    (work_min, work_max): XyBounds,
    frame: ScanFrame,
    boundary: Option<MachiningBoundary>,
    tool_range: ScanlineToolRange,
) -> Result<ScanRegion, CamSolverError> {
    let (xy_min, xy_max) = cutter.xy_bounds();
    let r = cutter.shape().radius();
    let reach_min = [xy_min[0] - r, xy_min[1] - r];
    let reach_max = [xy_max[0] + r, xy_max[1] + r];
    match boundary {
        None => {
            let (min, max) = match tool_range {
                ScanlineToolRange::ExtendByToolRadius => (
                    [work_min[0] - r, work_min[1] - r],
                    [work_max[0] + r, work_max[1] + r],
                ),
                ScanlineToolRange::GeometryExtent => (work_min, work_max),
            };
            Ok(ScanRegion::clipped(
                &rectangle_corners(min, max, frame),
                reach_min,
                reach_max,
            ))
        }
        Some(MachiningBoundary::Rectangle(rectangle)) => {
            clip_rectangle_boundary(&rectangle, frame, reach_min, reach_max)
                .ok_or_else(|| TemplateFailure::OperationBoundaryOutOfDomain.into())
        }
    }
}

/// 加工の座標の長方形 [min, max] の角を、走査線の向きの座標 `frame` へ回したもの（順に並べる）
fn rectangle_corners(min: [f64; 2], max: [f64; 2], frame: ScanFrame) -> [[f64; 2]; 4] {
    [min, [max[0], min[1]], max, [min[0], max[1]]].map(|corner| frame.to_scan(corner))
}

fn validate_scanline(params: &ScanlineParams, tool: &Tool<f64>) -> Result<(), CamSolverError> {
    let error = if !is_positive_finite(params.stepover) {
        ScanlineInputError::StepoverNotPositive
    } else if params.stepover > tool.diameter() {
        ScanlineInputError::StepoverExceedsToolDiameter
    } else if !is_positive_finite(params.feed_rate) {
        ScanlineInputError::FeedRateNotPositive
    } else if !params.angle_degrees.is_finite() {
        ScanlineInputError::AngleNotFinite
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

/// 矩形の加工範囲を走査線の向きの座標 `frame` へ回し、その座標の XY 範囲 [min, max] で切り取った範囲を返す。
///
/// 切り取った範囲の幅・高さのいずれかが距離トレランス以下か、面積が距離トレランスの 2 乗以下（辺や角で接するだけ、
/// または重ならない）の場合は、走査できる面積を持たないため `None` とする。
fn clip_rectangle_boundary(
    rectangle: &RectangleBoundary,
    frame: ScanFrame,
    min: [f64; 2],
    max: [f64; 2],
) -> Option<ScanRegion> {
    let corners = rectangle_corners(rectangle.rect_min, rectangle.rect_max, frame);
    let region = ScanRegion::clipped(&corners, min, max);
    let tolerance = default_distance_tolerance::<f64>();
    let [width, height] = region.extents()?;
    let has_area = width > tolerance && height > tolerance && region.area() > tolerance * tolerance;
    has_area.then_some(region)
}
