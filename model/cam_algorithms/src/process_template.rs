//! 工程テンプレートの solver 適用（設計: CAM_ALGORITHMS_DESIGN.md §9）
//!
//! 工程テンプレートとオペレーション定義から、精度プロファイル・オペレーション種別・
//! 加工ステージ・加工範囲を解決し、solver 入力へ適用する。
//!
//! 優先順位（`tolerance_profile` / `operation_type` 共通）:
//!
//! 1. オペレーション定義の明示指定
//! 2. 工程テンプレートの既定値
//! 3. solver の既定値（初期値は未設定。未設定のまま解決できなければ失敗）
//!
//! テンプレート適用段階の失敗は内部分類（`TemplateFailure::code`）を持ち、
//! Job Manager へは solver 失敗分類の `invalid_input` に集約して伝達する。

use std::fmt::{Display, Formatter};

use cam_core::{CoordinateFrame, LengthUnit, ToleranceProfile, Tool};
use geo_contracts::default_distance_tolerance;

use crate::solver::{
    CamSolverError, CamSolverInput, OperationSpec, ScanlineParams, SolverGeometry,
};
use crate::tessellation::TessellationLimits;

/// オペレーション種別（`operation_type`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// `contour_offset`: 等高線オフセット加工
    ContourOffset,
    /// `rest_machining`: 等高残加工
    RestMachining,
    /// `scanline`: スキャン加工
    Scanline,
    /// `surface_follow`: 面沿い加工
    SurfaceFollow,
}

impl OperationType {
    pub fn token(self) -> &'static str {
        match self {
            Self::ContourOffset => "contour_offset",
            Self::RestMachining => "rest_machining",
            Self::Scanline => "scanline",
            Self::SurfaceFollow => "surface_follow",
        }
    }

    /// canonical token から解決する。別名 token は受理しない。
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "contour_offset" => Some(Self::ContourOffset),
            "rest_machining" => Some(Self::RestMachining),
            "scanline" => Some(Self::Scanline),
            "surface_follow" => Some(Self::SurfaceFollow),
            _ => None,
        }
    }
}

/// 加工ステージ（`machining_stage`）
///
/// 実行順序を強制する状態ではなくタグ分類として扱い、`operation_type` とは直交する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachiningStage {
    /// `rough`: 荒加工
    Rough,
    /// `semi_finish`: 中加工
    SemiFinish,
    /// `finish`: 仕上げ
    Finish,
}

impl MachiningStage {
    pub fn token(self) -> &'static str {
        match self {
            Self::Rough => "rough",
            Self::SemiFinish => "semi_finish",
            Self::Finish => "finish",
        }
    }

    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "rough" => Some(Self::Rough),
            "semi_finish" => Some(Self::SemiFinish),
            "finish" => Some(Self::Finish),
            _ => None,
        }
    }
}

/// 矩形の加工範囲（`boundary_mode = rectangle`、ワーク局所座標・mm）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectangleBoundary {
    pub rect_min: [f64; 2],
    pub rect_max: [f64; 2],
}

impl RectangleBoundary {
    /// `x_min < x_max` かつ `y_min < y_max`（有限値）であることを検証する。
    pub fn validate(&self) -> Result<(), TemplateFailure> {
        let [x_min, y_min] = self.rect_min;
        let [x_max, y_max] = self.rect_max;
        let finite = [x_min, y_min, x_max, y_max].iter().all(|v| v.is_finite());
        if finite && x_min < x_max && y_min < y_max {
            Ok(())
        } else {
            Err(TemplateFailure::InvalidRectangleBoundary {
                rect_min: self.rect_min,
                rect_max: self.rect_max,
            })
        }
    }

    /// XY 範囲 [min, max] でクリップした範囲を返す。
    ///
    /// クリップ結果の幅・高さのいずれかが距離トレランス以下（辺や角で接するだけ、
    /// または重ならない）の場合は、走査できる面積を持たないため `None` とする。
    pub fn clip_to(&self, min: [f64; 2], max: [f64; 2]) -> Option<([f64; 2], [f64; 2])> {
        let clipped_min = [self.rect_min[0].max(min[0]), self.rect_min[1].max(min[1])];
        let clipped_max = [self.rect_max[0].min(max[0]), self.rect_max[1].min(max[1])];
        let tolerance = default_distance_tolerance::<f64>();
        let has_area = clipped_max[0] - clipped_min[0] > tolerance
            && clipped_max[1] - clipped_min[1] > tolerance;
        has_area.then_some((clipped_min, clipped_max))
    }
}

/// 加工範囲（`boundary_mode`）
///
/// `edge_projected_2d` は後続 Step で追加する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MachiningBoundary {
    Rectangle(RectangleBoundary),
}

/// 工程テンプレート（既定値の供給元）
///
/// token は外部入力をそのまま保持し、解決時に検証する。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcessTemplate {
    pub template_id: String,
    pub default_operation_type: Option<String>,
    pub default_tolerance_profile: Option<String>,
}

/// 工程テンプレート内のオペレーション定義
#[derive(Debug, Clone, PartialEq)]
pub struct OperationDefinition {
    pub operation_id: String,
    /// `machining_stage` token（必須）
    pub machining_stage: String,
    /// `operation_type` token（未指定ならテンプレート/solver 既定値）
    pub operation_type: Option<String>,
    /// `tolerance_profile` token（未指定ならテンプレート/solver 既定値）
    pub tolerance_profile: Option<String>,
    /// 加工範囲（未指定なら形状の XY 範囲全体）
    pub boundary: Option<MachiningBoundary>,
}

/// solver 既定値（優先順位の最下位）。初期値はいずれも未設定。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SolverDefaults {
    pub operation_type: Option<OperationType>,
    pub tolerance_profile: Option<ToleranceProfile>,
}

/// 解決済みオペレーション
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedOperation {
    pub template_id: String,
    pub operation_id: String,
    pub machining_stage: MachiningStage,
    pub operation_type: OperationType,
    pub tolerance_profile: ToleranceProfile,
    pub boundary: Option<MachiningBoundary>,
}

impl ResolvedOperation {
    /// 精度プロファイルから決まる形状離散化の許容弦誤差（mm）
    pub fn chord_tolerance(&self) -> f64 {
        self.tolerance_profile.tolerance_mm()
    }
}

/// テンプレート適用段階の失敗（内部分類）
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateFailure {
    InvalidToleranceProfile(String),
    MissingToleranceProfile,
    InvalidOperationType(String),
    MissingOperationType,
    InvalidMachiningStage(String),
    InvalidRectangleBoundary {
        rect_min: [f64; 2],
        rect_max: [f64; 2],
    },
    OperationBoundaryOutOfDomain,
    UnsupportedOperationType(OperationType),
}

impl TemplateFailure {
    /// 内部分類コード
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidToleranceProfile(_) => "invalid_tolerance_profile",
            Self::MissingToleranceProfile => "missing_tolerance_profile",
            Self::InvalidOperationType(_) => "invalid_operation_type",
            Self::MissingOperationType => "missing_operation_type",
            Self::InvalidMachiningStage(_) => "invalid_machining_stage",
            Self::InvalidRectangleBoundary { .. } => "invalid_rectangle_boundary",
            Self::OperationBoundaryOutOfDomain => "operation_boundary_out_of_domain",
            Self::UnsupportedOperationType(_) => "unsupported_operation_type",
        }
    }
}

impl Display for TemplateFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToleranceProfile(token) => {
                write!(f, "{}: unknown token '{}'", self.code(), token)
            }
            Self::InvalidOperationType(token) => {
                write!(f, "{}: unknown token '{}'", self.code(), token)
            }
            Self::InvalidMachiningStage(token) => {
                write!(f, "{}: unknown token '{}'", self.code(), token)
            }
            Self::InvalidRectangleBoundary { rect_min, rect_max } => write!(
                f,
                "{}: rect_min={:?} and rect_max={:?} must be finite and satisfy x_min < x_max and y_min < y_max",
                self.code(),
                rect_min,
                rect_max
            ),
            Self::UnsupportedOperationType(operation_type) => write!(
                f,
                "{}: '{}' is not supported by the solver yet",
                self.code(),
                operation_type.token()
            ),
            Self::MissingToleranceProfile
            | Self::MissingOperationType
            | Self::OperationBoundaryOutOfDomain => write!(f, "{}", self.code()),
        }
    }
}

/// 内部分類は Job Manager へ公開せず、すべて `invalid_input` に集約する。
impl From<TemplateFailure> for CamSolverError {
    fn from(value: TemplateFailure) -> Self {
        Self::InvalidInput(value.to_string())
    }
}

/// 優先順位に従ってオペレーション定義を解決する。
pub fn resolve_operation(
    template: &ProcessTemplate,
    operation: &OperationDefinition,
    defaults: SolverDefaults,
) -> Result<ResolvedOperation, TemplateFailure> {
    let machining_stage = MachiningStage::from_token(&operation.machining_stage)
        .ok_or_else(|| TemplateFailure::InvalidMachiningStage(operation.machining_stage.clone()))?;

    let tolerance_profile = resolve_token(
        operation.tolerance_profile.as_deref(),
        template.default_tolerance_profile.as_deref(),
        defaults.tolerance_profile,
        ToleranceProfile::from_token,
    )
    .map_err(|token| TemplateFailure::InvalidToleranceProfile(token.to_string()))?
    .ok_or(TemplateFailure::MissingToleranceProfile)?;

    let operation_type = resolve_token(
        operation.operation_type.as_deref(),
        template.default_operation_type.as_deref(),
        defaults.operation_type,
        OperationType::from_token,
    )
    .map_err(|token| TemplateFailure::InvalidOperationType(token.to_string()))?
    .ok_or(TemplateFailure::MissingOperationType)?;

    if let Some(MachiningBoundary::Rectangle(rectangle)) = &operation.boundary {
        rectangle.validate()?;
    }

    Ok(ResolvedOperation {
        template_id: template.template_id.clone(),
        operation_id: operation.operation_id.clone(),
        machining_stage,
        operation_type,
        tolerance_profile,
        boundary: operation.boundary,
    })
}

/// 明示指定 → テンプレート既定値 → solver 既定値 の順に最初の指定を採用する。
///
/// 採用した token が未知の場合はその token を返して失敗させる（下位の既定値へは落とさない）。
fn resolve_token<'a, T: Copy>(
    explicit: Option<&'a str>,
    template_default: Option<&'a str>,
    solver_default: Option<T>,
    parse: fn(&str) -> Option<T>,
) -> Result<Option<T>, &'a str> {
    match explicit.or(template_default) {
        Some(token) => parse(token).map(Some).ok_or(token),
        None => Ok(solver_default),
    }
}

/// 解決済みオペレーションを solver 入力へ適用する。
///
/// 現在 solver が生成できるのは `scanline` のみで、それ以外は `unsupported_operation_type`。
pub fn build_solver_input(
    resolved: &ResolvedOperation,
    tool: Tool<f64>,
    geometry: SolverGeometry,
    scanline: ScanlineParams,
    tessellation_limits: TessellationLimits,
) -> Result<CamSolverInput, TemplateFailure> {
    let operation = match resolved.operation_type {
        OperationType::Scanline => OperationSpec::Scanline(scanline),
        other => return Err(TemplateFailure::UnsupportedOperationType(other)),
    };

    Ok(CamSolverInput {
        operation_id: resolved.operation_id.clone(),
        tool,
        geometry,
        operation,
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: resolved.chord_tolerance(),
        tessellation_limits,
        boundary: resolved.boundary,
    })
}

#[cfg(test)]
mod tests {
    use cam_core::{ToleranceProfile, Tool};
    use geo_algorithms::{Point3D, TriangleMesh3D};

    use super::*;
    use crate::solver::{CamSolverError, ScanlineParams, SolverGeometry};
    use crate::tessellation::TessellationLimits;

    fn template(profile: Option<&str>, operation_type: Option<&str>) -> ProcessTemplate {
        ProcessTemplate {
            template_id: "tpl-1".to_string(),
            default_operation_type: operation_type.map(str::to_string),
            default_tolerance_profile: profile.map(str::to_string),
        }
    }

    fn operation(profile: Option<&str>, operation_type: Option<&str>) -> OperationDefinition {
        OperationDefinition {
            operation_id: "op-1".to_string(),
            machining_stage: "finish".to_string(),
            operation_type: operation_type.map(str::to_string),
            tolerance_profile: profile.map(str::to_string),
            boundary: None,
        }
    }

    fn solver_defaults(profile: Option<ToleranceProfile>) -> SolverDefaults {
        SolverDefaults {
            operation_type: Some(OperationType::Scanline),
            tolerance_profile: profile,
        }
    }

    #[test]
    fn tolerance_profile_priority_is_operation_then_template_then_solver() {
        let defaults = solver_defaults(Some(ToleranceProfile::PressRough));

        let explicit = resolve_operation(
            &template(Some("press_rough"), None),
            &operation(Some("mold_finish"), None),
            defaults,
        )
        .unwrap();
        assert_eq!(explicit.tolerance_profile, ToleranceProfile::MoldFinish);

        let from_template = resolve_operation(
            &template(Some("mold_finish"), None),
            &operation(None, None),
            defaults,
        )
        .unwrap();
        assert_eq!(
            from_template.tolerance_profile,
            ToleranceProfile::MoldFinish
        );

        let from_solver =
            resolve_operation(&template(None, None), &operation(None, None), defaults).unwrap();
        assert_eq!(from_solver.tolerance_profile, ToleranceProfile::PressRough);
    }

    #[test]
    fn operation_type_priority_is_operation_then_template_then_solver() {
        let defaults = solver_defaults(Some(ToleranceProfile::PressRough));
        let explicit = resolve_operation(
            &template(None, Some("contour_offset")),
            &operation(None, Some("surface_follow")),
            defaults,
        )
        .unwrap();
        assert_eq!(explicit.operation_type, OperationType::SurfaceFollow);

        let from_template = resolve_operation(
            &template(None, Some("contour_offset")),
            &operation(None, None),
            defaults,
        )
        .unwrap();
        assert_eq!(from_template.operation_type, OperationType::ContourOffset);

        let from_solver =
            resolve_operation(&template(None, None), &operation(None, None), defaults).unwrap();
        assert_eq!(from_solver.operation_type, OperationType::Scanline);
    }

    #[test]
    fn missing_tolerance_profile_without_solver_default() {
        let failure = resolve_operation(
            &template(None, Some("scanline")),
            &operation(None, None),
            SolverDefaults::default(),
        )
        .unwrap_err();
        assert_eq!(failure, TemplateFailure::MissingToleranceProfile);
    }

    #[test]
    fn missing_operation_type_without_solver_default() {
        let failure = resolve_operation(
            &template(Some("press_rough"), None),
            &operation(None, None),
            SolverDefaults::default(),
        )
        .unwrap_err();
        assert_eq!(failure, TemplateFailure::MissingOperationType);
    }

    #[test]
    fn unknown_tokens_are_rejected_without_falling_back() {
        let defaults = solver_defaults(Some(ToleranceProfile::PressRough));
        // 明示指定の未知 token は、テンプレート既定値へ落とさずに失敗する
        let profile = resolve_operation(
            &template(Some("press_rough"), None),
            &operation(Some("Mold_Finish"), None),
            defaults,
        )
        .unwrap_err();
        assert_eq!(profile.code(), "invalid_tolerance_profile");

        let operation_type = resolve_operation(
            &template(None, Some("scan")),
            &operation(None, None),
            defaults,
        )
        .unwrap_err();
        assert_eq!(operation_type.code(), "invalid_operation_type");

        let mut invalid_stage = operation(None, None);
        invalid_stage.machining_stage = "finishing".to_string();
        let stage = resolve_operation(&template(None, None), &invalid_stage, defaults).unwrap_err();
        assert_eq!(stage.code(), "invalid_machining_stage");
    }

    #[test]
    fn stage_and_operation_type_are_orthogonal() {
        // finish でも rest_machining を受理し、stage から operation_type を推論しない
        let defaults = solver_defaults(Some(ToleranceProfile::MoldFinish));
        let resolved = resolve_operation(
            &template(None, None),
            &operation(None, Some("rest_machining")),
            defaults,
        )
        .unwrap();
        assert_eq!(resolved.machining_stage, MachiningStage::Finish);
        assert_eq!(resolved.operation_type, OperationType::RestMachining);
    }

    #[test]
    fn degenerate_rectangle_is_rejected() {
        let defaults = solver_defaults(Some(ToleranceProfile::PressRough));
        for (rect_min, rect_max) in [
            ([0.0, 0.0], [0.0, 10.0]),
            ([5.0, 0.0], [1.0, 10.0]),
            ([0.0, 0.0], [10.0, f64::NAN]),
        ] {
            let mut definition = operation(None, None);
            definition.boundary = Some(MachiningBoundary::Rectangle(RectangleBoundary {
                rect_min,
                rect_max,
            }));
            let failure =
                resolve_operation(&template(None, None), &definition, defaults).unwrap_err();
            assert_eq!(failure.code(), "invalid_rectangle_boundary");
            assert!(failure.to_string().contains("must be finite"), "{failure}");
        }
    }

    #[test]
    fn template_failures_map_to_invalid_input_with_internal_code() {
        let error: CamSolverError = TemplateFailure::MissingToleranceProfile.into();
        assert_eq!(error.code(), "invalid_input");
        assert!(error.reason().starts_with("missing_tolerance_profile"));
    }

    fn flat_mesh() -> SolverGeometry {
        SolverGeometry::TriangleMesh(
            TriangleMesh3D::new(
                vec![
                    Point3D::new(0.0, 0.0, 0.0),
                    Point3D::new(10.0, 0.0, 0.0),
                    Point3D::new(10.0, 10.0, 0.0),
                ],
                vec![[0, 1, 2]],
            )
            .unwrap(),
        )
    }

    fn scanline_params() -> ScanlineParams {
        ScanlineParams {
            stepover: 1.0,
            sample_pitch: 1.0,
            feed_rate: 1000.0,
            clearance_height: 5.0,
        }
    }

    #[test]
    fn build_solver_input_applies_profile_tolerance() {
        for (profile, expected) in [
            ("press_rough", ToleranceProfile::PressRough.tolerance_mm()),
            ("mold_finish", ToleranceProfile::MoldFinish.tolerance_mm()),
        ] {
            let resolved = resolve_operation(
                &template(Some(profile), Some("scanline")),
                &operation(None, None),
                SolverDefaults::default(),
            )
            .unwrap();
            let input = build_solver_input(
                &resolved,
                Tool::ball_end_mill("BEM2".to_string(), 2.0, 10.0),
                flat_mesh(),
                scanline_params(),
                TessellationLimits::default(),
            )
            .unwrap();
            assert_eq!(input.chord_tolerance, expected);
            assert_eq!(input.operation_id, "op-1");
        }
    }

    #[test]
    fn build_solver_input_rejects_operations_the_solver_cannot_generate() {
        let resolved = resolve_operation(
            &template(Some("press_rough"), Some("contour_offset")),
            &operation(None, None),
            SolverDefaults::default(),
        )
        .unwrap();
        let failure = build_solver_input(
            &resolved,
            Tool::ball_end_mill("BEM2".to_string(), 2.0, 10.0),
            flat_mesh(),
            scanline_params(),
            TessellationLimits::default(),
        )
        .unwrap_err();
        assert_eq!(
            failure,
            TemplateFailure::UnsupportedOperationType(OperationType::ContourOffset)
        );
    }
}
