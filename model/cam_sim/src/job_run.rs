//! ジョブ 1 つの計算（ツールパス生成・切削シミュレーション）と、成果物のバイト列への書き出し
//!
//! 入力の値から成果物のバイト列を作る。

use std::io::{Read, Write};

use cam_algorithms::{CamSolverError, CamSolverInput, ProductSurface, solve_toolpath};
use cam_core::{
    ArtifactHeaderV1, ArtifactKind, BinaryFormatError, CoordinateFrame, LengthUnit, ToolPath,
    read_toolpath_artifact_v1, write_toolpath_payload_v1,
};
use geo_algorithms::TessellationLimits;

use crate::preflight::{PreflightFailure, preflight_cutting_simulation};
use crate::simulation_artifact::write_cutting_simulation_artifact_v1;
use crate::{CuttingSimulationInput, CuttingSimulator};

/// ツールパス生成の点 1 つあたりに使うメモリの見積もり（バイト）
///
/// 生成の間はパスの点（24 バイト）と線分（120 バイト）を持ち、成果物のバイト列（線分 1 本あたり約 62 バイト。長さを
/// 数えてから確保する）を作る間は線分とバイト列を持つ。多い側に余裕を足して 250 バイトとする。
const TOOLPATH_BYTES_PER_POINT: u64 = 250;

/// ツールパス生成に使ってよいメモリの上限（バイト。8 GB。点の数に直し、超えたら止める）
pub const TOOLPATH_MEMORY_LIMIT_BYTES: u64 = 8_000_000_000;

/// 点 `points` 個のツールパスの生成に使うメモリの見積もり（バイト）
pub fn toolpath_memory_bytes(points: u64) -> u64 {
    points.saturating_mul(TOOLPATH_BYTES_PER_POINT)
}

/// メモリの上限 `memory_bytes` に収まる、ツールパスの点の数
fn toolpath_point_limit(memory_bytes: u64) -> usize {
    usize::try_from(memory_bytes / TOOLPATH_BYTES_PER_POINT).unwrap_or(usize::MAX)
}

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

    /// 入力を読めなかった失敗か（読んでいる途中のファイルの読み込みの誤り。中身の誤りではない）
    pub fn is_unreadable_input(&self) -> bool {
        self.code == "artifact-io-failed"
    }

    /// ツールパスの点の数がメモリの上限を超えるため止めた失敗か
    pub fn is_too_large(&self) -> bool {
        self.code == CamSolverError::TooManyPoints.code()
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

/// ツールパスを生成し、ツールパスの成果物のバイト列（`ArtifactKind::ToolPath`）を返す（点の数がメモリの上限を
/// 超えるなら、使い切る前に止める）
pub fn run_cam_solver(input: &CamSolverInput) -> Result<Vec<u8>, JobRunFailure> {
    let point_limit = toolpath_point_limit(TOOLPATH_MEMORY_LIMIT_BYTES);
    let toolpath = solve_toolpath(input, point_limit).map_err(|error| {
        let limit = match error {
            CamSolverError::TooManyPoints => format!(
                " (point limit {point_limit} for the memory limit of {TOOLPATH_MEMORY_LIMIT_BYTES} bytes)"
            ),
            _ => String::new(),
        };
        JobRunFailure::new(error.code(), format!("cam process failed: {error}{limit}"))
    })?;
    encode_toolpath_artifact(&toolpath, input.units, input.coordinate_frame).map_err(|error| {
        JobRunFailure::new(
            "artifact-error",
            format!("failed to build cam artifact bytes: {}", error),
        )
    })
}

/// ツールパスの成果物 `toolpath_artifact` を読みながら組み立てて切削シミュレーションし、成果物を返す（成果物の
/// バイト列を丸ごとメモリに持たなくてよいように、読み手を受け取る）
///
/// 実行の前に、ツールパスと入力の組み合わせを確かめる（Preflight）。最後の時点の素材を製品形状と比べ、削り込み・削り
/// 残しを成果物に含める。
pub fn run_cutting_simulation<R: Read>(
    input: &CuttingSimulationInput,
    toolpath_artifact: &mut R,
) -> Result<CuttingSimulationRun, JobRunFailure> {
    let (toolpath_header, toolpath) =
        read_toolpath_artifact_v1(toolpath_artifact).map_err(|error| {
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
    // 本体を別に作ってから写すと、大きなツールパスで本体の分だけメモリが余計に要るため、1 つのバイト列に書き、
    // 本体の長さを後からヘッダーに書き直す（ヘッダーの長さは、本体の長さによらない）
    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
    header.unit = unit;
    header.frame = frame;
    // 先に長さだけを数えて、ちょうどの大きさを確保する（伸ばしながら書くと、伸ばす瞬間に古い領域と新しい領域を
    // 同時に持ち、ツールパスのバイト列の 3 倍までメモリを使うため）
    let mut counter = ByteCounter(0);
    header.write_to(&mut counter)?;
    let header_len = counter.0;
    write_toolpath_payload_v1(&mut counter, toolpath)?;
    let mut bytes = Vec::with_capacity(counter.0);
    header.write_to(&mut bytes)?;
    write_toolpath_payload_v1(&mut bytes, toolpath)?;
    header.payload_len = (bytes.len() - header_len) as u64;
    let mut head = Vec::with_capacity(header_len);
    header.write_to(&mut head)?;
    bytes[..header_len].copy_from_slice(&head);
    Ok(bytes)
}

/// 書いたバイトの数だけを数える書き先
struct ByteCounter(usize);

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// 成果物の読み込みの失敗の分類
pub(crate) fn classify_artifact_read_error(error: &BinaryFormatError) -> &'static str {
    match error {
        BinaryFormatError::ArtifactKindMismatch { .. } => "kind-mismatch",
        BinaryFormatError::UnsupportedMajorVersion { .. }
        | BinaryFormatError::UnsupportedMinorVersion { .. }
        | BinaryFormatError::ConverterRequired { .. } => "version-mismatch",
        // 途中で切れたファイルは中身の誤り、それ以外の読み込みの誤りは読めなかったこと
        BinaryFormatError::Io(io) if io.kind() != std::io::ErrorKind::UnexpectedEof => {
            "artifact-io-failed"
        }
        _ => "artifact-read-failed",
    }
}

#[cfg(test)]
mod tests {
    use super::classify_artifact_read_error;
    use cam_core::BinaryFormatError;
    use std::io::{Error, ErrorKind};

    #[test]
    fn a_cut_off_file_is_invalid_input_and_other_read_errors_are_unreadable_input() {
        let cut_off = BinaryFormatError::Io(Error::from(ErrorKind::UnexpectedEof));
        assert_eq!(
            classify_artifact_read_error(&cut_off),
            "artifact-read-failed"
        );
        let unreadable = BinaryFormatError::Io(Error::from(ErrorKind::PermissionDenied));
        assert_eq!(
            classify_artifact_read_error(&unreadable),
            "artifact-io-failed"
        );
    }
}
