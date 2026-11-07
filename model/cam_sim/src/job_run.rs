//! ジョブ 1 つの計算（ツールパス生成・切削シミュレーション）と、成果物のバイト列への書き出し
//!
//! 入力の値から成果物のバイト列を作る。

use std::io::Cursor;

use cam_algorithms::{CamSolverInput, ProductSurface, solve_toolpath};
use cam_core::{
    ArtifactHeaderV1, ArtifactKind, BinaryFormatError, CoordinateFrame, LengthUnit, ToolPath,
    read_toolpath_artifact_v1, write_toolpath_payload_v1,
};
use geo_algorithms::TessellationLimits;

use crate::preflight::{PreflightFailure, preflight_cutting_simulation};
use crate::simulation_artifact::write_cutting_simulation_artifact_v1;
use crate::{CuttingSimulationInput, CuttingSimulator};

/// ジョブの計算の失敗
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobRunFailure {
    /// 失敗の分類（ASCII の短い語。ログの参照の接尾辞に使う）
    pub code: &'static str,
    /// 失敗の理由（英語）
    pub message: String,
}

impl JobRunFailure {
    fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }

    /// 入力の中身の誤り（条件の値・ツールパスの成果物の形式）による失敗か
    pub fn is_invalid_input(&self) -> bool {
        matches!(
            self.code,
            "invalid_input"
                | "preflight-invalid-input"
                | "kind-mismatch"
                | "version-mismatch"
                | "artifact-read-failed"
        )
    }
}

/// 切削シミュレーションの成果物
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuttingSimulationRun {
    /// 成果物のバイト列（`ArtifactKind::CuttingSimulation`）
    pub artifact: Vec<u8>,
    /// 記録したスナップショットの数
    pub snapshot_count: usize,
}

/// ツールパスを生成し、ツールパスの成果物のバイト列（`ArtifactKind::ToolPath`）を返す
pub fn run_cam_solver(input: &CamSolverInput) -> Result<Vec<u8>, JobRunFailure> {
    let toolpath = solve_toolpath(input).map_err(|error| {
        JobRunFailure::new(error.code(), format!("cam process failed: {}", error))
    })?;
    encode_toolpath_artifact(&toolpath, input.units, input.coordinate_frame).map_err(|error| {
        JobRunFailure::new(
            "artifact-error",
            format!("failed to build cam artifact bytes: {}", error),
        )
    })
}

/// ツールパスの成果物のバイト列 `toolpath_artifact` を切削シミュレーションし、成果物を返す
///
/// 実行の前に、ツールパスと入力の組み合わせを確かめる（Preflight）。最後の時点の素材を製品形状と比べ、削り込み・削り
/// 残しを成果物に含める。
pub fn run_cutting_simulation(
    input: &CuttingSimulationInput,
    toolpath_artifact: &[u8],
) -> Result<CuttingSimulationRun, JobRunFailure> {
    let mut cursor = Cursor::new(toolpath_artifact);
    let (toolpath_header, toolpath) = read_toolpath_artifact_v1(&mut cursor).map_err(|error| {
        JobRunFailure::new(
            classify_artifact_read_error(&error),
            format!(
                "failed to read cutting simulation input artifact: {}",
                error
            ),
        )
    })?;

    if let Err(failure) = preflight_cutting_simulation(&toolpath, input) {
        let code = match failure {
            PreflightFailure::NotReady(_) => "preflight-not-ready",
            PreflightFailure::InvalidInput(_) => "preflight-invalid-input",
        };
        return Err(JobRunFailure::new(code, failure.to_string()));
    }

    let mut simulator = CuttingSimulator::new(input.stock, input.snapshot_interval.clone());
    simulator
        .simulate(&toolpath, &input.tool, input.precision)
        .map_err(|error| {
            JobRunFailure::new(
                "sim-failure",
                format!("failed to run cutting simulation: {}", error),
            )
        })?;

    // 製品形状は、ツールパス生成と同じ弦の許容誤差・既定の分割の上限で三角形にする
    let surface = input
        .product
        .triangle_mesh(input.tolerance, TessellationLimits::default())
        .and_then(|mesh| ProductSurface::new(&mesh))
        .map_err(|error| {
            JobRunFailure::new(
                "product-failure",
                format!(
                    "failed to compare the stock with the product shape: {}",
                    error
                ),
            )
        })?;
    simulator.compare_with_product(&surface, input.tolerance);

    let mut artifact = Vec::new();
    simulator
        .artifact()
        .ok_or(BinaryFormatError::InvalidPayload("simulation result"))
        .and_then(|result| {
            write_cutting_simulation_artifact_v1(
                &mut artifact,
                &result,
                toolpath_header.unit,
                toolpath_header.frame,
            )
        })
        .map_err(|error| {
            JobRunFailure::new(
                "artifact-error",
                format!("failed to write cutting simulation artifact: {}", error),
            )
        })?;
    Ok(CuttingSimulationRun {
        artifact,
        snapshot_count: simulator.snapshots().len(),
    })
}

fn encode_toolpath_artifact(
    toolpath: &ToolPath<f64>,
    unit: LengthUnit,
    frame: CoordinateFrame,
) -> Result<Vec<u8>, BinaryFormatError> {
    let mut payload = Vec::new();
    write_toolpath_payload_v1(&mut payload, toolpath)?;
    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64);
    header.unit = unit;
    header.frame = frame;
    let mut bytes = Vec::new();
    header.write_to(&mut bytes)?;
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

/// 成果物の読み込みの失敗の分類
pub(crate) fn classify_artifact_read_error(error: &BinaryFormatError) -> &'static str {
    match error {
        BinaryFormatError::ArtifactKindMismatch { .. } => "kind-mismatch",
        BinaryFormatError::UnsupportedMajorVersion { .. }
        | BinaryFormatError::UnsupportedMinorVersion { .. }
        | BinaryFormatError::ConverterRequired { .. } => "version-mismatch",
        _ => "artifact-read-failed",
    }
}
