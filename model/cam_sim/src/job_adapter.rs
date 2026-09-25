use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;

use cam_algorithms::{CamSolverError, solve_toolpath};
use cam_core::{
    ArtifactHeaderV1, ArtifactKind, BinaryFormatError, CoordinateFrame, LengthUnit, Tool, ToolPath,
    read_toolpath_artifact_v1, write_toolpath_payload_v1,
};
#[cfg(any(test, debug_assertions))]
use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType};
use geo_algorithms::{Aabb3D, Point3D};
use job_runtime::{
    JobExecutionResult, JobExecutor, JobRecord, JobStatus, JobType, RefFactory, RefParser,
};

use crate::solver_input::{CamSolverInputProvider, validate_cam_input_ref};
use crate::{HybridGateConfig, HybridGateMetrics, run_hybrid_gate_case_with_config};

/// cam_sim から JobManager へ接続するアダプタ
///
/// CamProcess は `CamSolverInputProvider` で `InputRef` を solver 入力へ解決し、
/// `cam_algorithms::solve_toolpath` の結果を `toolpath` artifact として返す。
#[derive(Clone, Default)]
pub struct CamJobExecutorAdapter {
    input_provider: Option<Arc<dyn CamSolverInputProvider>>,
}

impl std::fmt::Debug for CamJobExecutorAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CamJobExecutorAdapter")
            .field("input_provider_configured", &self.input_provider.is_some())
            .finish()
    }
}

const JOB_SIM_DEFAULT_WORK_MIN: (f64, f64, f64) = (0.0, 0.0, 0.0);
const JOB_SIM_DEFAULT_WORK_MAX: (f64, f64, f64) = (100.0, 100.0, 100.0);
const JOB_SIM_DEFAULT_MAX_DEPTH: usize = 4;
const JOB_SIM_DEFAULT_SAMPLE_PITCH: f64 = 2.0;

impl CamJobExecutorAdapter {
    /// solver 入力 provider を接続したアダプタを作成する。
    pub fn with_input_provider(provider: Arc<dyn CamSolverInputProvider>) -> Self {
        Self {
            input_provider: Some(provider),
        }
    }

    fn run_cam_process(&self, job: &JobRecord) -> JobExecutionResult {
        if let Err(err) = validate_cam_input_ref(&job.spec.input_ref) {
            return cam_process_failure(
                job,
                &CamSolverError::InvalidInput(format!(
                    "invalid input_ref for cam process: {}: {}",
                    job.spec.input_ref, err
                )),
                10,
            );
        }

        let resolved = self
            .input_provider
            .as_ref()
            .and_then(|provider| provider.resolve(&job.spec.input_ref));
        let Some(input) = resolved else {
            return self.run_cam_process_without_solver_input(job);
        };

        let started = Instant::now();
        let toolpath = match solve_toolpath(&input) {
            Ok(toolpath) => toolpath,
            Err(err) => return cam_process_failure(job, &err, elapsed_millis_since(started)),
        };

        match encode_toolpath_artifact(&toolpath, input.units, input.coordinate_frame) {
            Ok(bytes) => cam_process_success(job, bytes, elapsed_millis_since(started)),
            Err(err) => cam_artifact_error(job, &err),
        }
    }

    /// provider に solver 入力が無い場合の導線。
    ///
    /// 本番ビルドでは `invalid_input` とし、テスト/デバッグビルドでは
    /// 参照 suffix に応じた固定 artifact と失敗分類を返す。
    fn run_cam_process_without_solver_input(&self, job: &JobRecord) -> JobExecutionResult {
        #[cfg(not(any(test, debug_assertions)))]
        {
            cam_process_failure(
                job,
                &CamSolverError::InvalidInput(format!(
                    "solver input is not registered for input_ref: {}",
                    job.spec.input_ref
                )),
                10,
            )
        }

        #[cfg(any(test, debug_assertions))]
        {
            if let Some(err) = classify_fixture_cam_process_failure(&job.spec.input_ref) {
                return cam_process_failure(job, &err, 10);
            }

            match build_cam_process_artifact_bytes(&job.spec.input_ref) {
                Ok(bytes) => cam_process_success(job, bytes, 200),
                Err(err) => cam_artifact_error(job, &err),
            }
        }
    }

    fn run_cutting_simulation(
        &self,
        job: &JobRecord,
        input_artifact_bytes: Option<&[u8]>,
    ) -> JobExecutionResult {
        let Some((cam_job_id_from_ref, result_suffix)) = parse_cam_result_ref(&job.spec.input_ref)
        else {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("sim", job.id.0, "invalid")),
                error: Some(format!(
                    "invalid input_ref for cutting simulation: {}",
                    job.spec.input_ref
                )),
            };
        };

        let Some(parent_job_id) = job.parent_job_id else {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("sim", job.id.0, "invalid")),
                error: Some("cutting simulation requires parent_job_id".to_string()),
            };
        };

        if parent_job_id.0 != cam_job_id_from_ref {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("sim", job.id.0, "invalid")),
                error: Some(format!(
                    "input_ref cam job id mismatch: parent_job_id={}, input_ref={}",
                    parent_job_id.0, job.spec.input_ref
                )),
            };
        }

        if result_suffix != "ok" {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("sim", job.id.0, "invalid")),
                error: Some(format!(
                    "invalid cam result_ref suffix for cutting simulation: {}",
                    result_suffix
                )),
            };
        }

        let Some(input_artifact_bytes) = input_artifact_bytes else {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("sim", job.id.0, "missing-input")),
                error: Some("missing input artifact bytes for cutting simulation".to_string()),
            };
        };

        let mut cursor = Cursor::new(input_artifact_bytes);
        let (_, toolpath) = match read_toolpath_artifact_v1(&mut cursor) {
            Ok(value) => value,
            Err(err) => {
                let log_ref = format!(
                    "log://sim/{}/{}",
                    job.id.0,
                    classify_artifact_read_error(&err)
                );
                return JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: 20,
                    result_ref: None,
                    artifact_bytes: None,
                    log_ref: Some(log_ref),
                    error: Some(format!(
                        "failed to read cutting simulation input artifact: {}",
                        err
                    )),
                };
            }
        };

        let work_bounds = Aabb3D::new(
            Point3D::new(
                JOB_SIM_DEFAULT_WORK_MIN.0,
                JOB_SIM_DEFAULT_WORK_MIN.1,
                JOB_SIM_DEFAULT_WORK_MIN.2,
            ),
            Point3D::new(
                JOB_SIM_DEFAULT_WORK_MAX.0,
                JOB_SIM_DEFAULT_WORK_MAX.1,
                JOB_SIM_DEFAULT_WORK_MAX.2,
            ),
        );
        let tool = Tool::flat_end_mill("sim-tool".to_string(), 10.0, 30.0);
        let hybrid_config = HybridGateConfig {
            bounds: work_bounds,
            octree_depth: JOB_SIM_DEFAULT_MAX_DEPTH,
            sample_pitch: JOB_SIM_DEFAULT_SAMPLE_PITCH,
        };

        let started = Instant::now();
        let hybrid_metrics = match run_hybrid_gate_case_with_config(&toolpath, &tool, hybrid_config)
        {
            Ok(metrics) => metrics,
            Err(err) => {
                return JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: started.elapsed().as_millis().max(1) as u64,
                    result_ref: None,
                    artifact_bytes: None,
                    log_ref: Some(build_log_ref("sim", job.id.0, "sim-failure")),
                    error: Some(format!("failed to run cutting simulation: {}", err)),
                };
            }
        };
        let elapsed_millis = started.elapsed().as_millis().max(1) as u64;

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis,
            result_ref: Some(build_result_ref("sim", job.id.0, "ok")),
            artifact_bytes: None,
            log_ref: Some(format_hybrid_success_log_ref(job.id.0, &hybrid_metrics)),
            error: None,
        }
    }

    fn run_nc_post_from_cam(
        &self,
        job: &JobRecord,
        input_artifact_bytes: Option<&[u8]>,
    ) -> JobExecutionResult {
        let Some((cam_job_id_from_ref, result_suffix)) = parse_cam_result_ref(&job.spec.input_ref)
        else {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("nc-post", job.id.0, "invalid")),
                error: Some(format!(
                    "invalid input_ref for nc post from cam: {}",
                    job.spec.input_ref
                )),
            };
        };

        let Some(parent_job_id) = job.parent_job_id else {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("nc-post", job.id.0, "invalid")),
                error: Some("nc post from cam requires parent_job_id".to_string()),
            };
        };

        if parent_job_id.0 != cam_job_id_from_ref {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("nc-post", job.id.0, "invalid")),
                error: Some(format!(
                    "input_ref cam job id mismatch: parent_job_id={}, input_ref={}",
                    parent_job_id.0, job.spec.input_ref
                )),
            };
        }

        if result_suffix != "ok" {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("nc-post", job.id.0, "invalid")),
                error: Some(format!(
                    "invalid cam result_ref suffix for nc post from cam: {}",
                    result_suffix
                )),
            };
        }

        let Some(input_artifact_bytes) = input_artifact_bytes else {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(build_log_ref("nc-post", job.id.0, "missing-input")),
                error: Some("missing input artifact bytes for nc post from cam".to_string()),
            };
        };

        let mut cursor = Cursor::new(input_artifact_bytes);
        if let Err(err) = read_toolpath_artifact_v1(&mut cursor) {
            let log_ref = format!(
                "log://nc-post/{}/{}",
                job.id.0,
                classify_artifact_read_error(&err)
            );
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 20,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(log_ref),
                error: Some(format!("failed to read artifact binary: {}", err)),
            };
        }

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 240,
            result_ref: Some(build_result_ref("nc-post", job.id.0, "ok")),
            artifact_bytes: None,
            log_ref: Some(build_log_ref("nc-post", job.id.0, "ok")),
            error: None,
        }
    }
}

fn cam_process_success(
    job: &JobRecord,
    artifact_bytes: Vec<u8>,
    elapsed_millis: u64,
) -> JobExecutionResult {
    JobExecutionResult {
        status: JobStatus::Succeeded,
        elapsed_millis,
        result_ref: Some(build_result_ref("cam", job.id.0, "ok")),
        artifact_bytes: Some(artifact_bytes),
        log_ref: Some(build_log_ref("cam", job.id.0, "ok")),
        error: None,
    }
}

fn cam_process_failure(
    job: &JobRecord,
    error: &CamSolverError,
    elapsed_millis: u64,
) -> JobExecutionResult {
    JobExecutionResult {
        status: JobStatus::Failed,
        elapsed_millis,
        result_ref: None,
        artifact_bytes: None,
        log_ref: Some(build_log_ref("cam", job.id.0, error.code())),
        error: Some(format!("cam process failed: {}", error)),
    }
}

fn cam_artifact_error(job: &JobRecord, error: &BinaryFormatError) -> JobExecutionResult {
    JobExecutionResult {
        status: JobStatus::Failed,
        elapsed_millis: 20,
        result_ref: None,
        artifact_bytes: None,
        log_ref: Some(build_log_ref("cam", job.id.0, "artifact-error")),
        error: Some(format!("failed to build cam artifact bytes: {}", error)),
    }
}

fn elapsed_millis_since(started: Instant) -> u64 {
    started.elapsed().as_millis().max(1) as u64
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

/// テスト/デバッグ用の固定失敗分類（solver 入力未登録時のみ参照）
#[cfg(any(test, debug_assertions))]
fn classify_fixture_cam_process_failure(input_ref: &str) -> Option<CamSolverError> {
    if input_ref.ends_with("/invalid-input") {
        return Some(CamSolverError::InvalidInput(
            "required input fields are missing or malformed".to_string(),
        ));
    }

    if input_ref.ends_with("/no-solution") {
        return Some(CamSolverError::NoSolution(
            "no feasible toolpath can be constructed under current geometry constraints"
                .to_string(),
        ));
    }

    if input_ref.ends_with("/convergence-failure") {
        return Some(CamSolverError::ConvergenceFailure(
            "iterative solver did not converge within configured tolerance".to_string(),
        ));
    }

    None
}

fn format_hybrid_success_log_ref(job_id: u64, metrics: &HybridGateMetrics) -> String {
    build_log_ref(
        "sim",
        job_id,
        &format!(
            "ok/hybrid-gap-{:.4}-boundary-{:.4}-ratio-{:.3}",
            metrics.gap, metrics.boundary_disagreement_rate, metrics.elapsed_ratio
        ),
    )
}

#[cfg(any(test, debug_assertions))]
fn build_cam_process_artifact_bytes(input_ref: &str) -> Result<Vec<u8>, BinaryFormatError> {
    if input_ref.ends_with("/kind-mismatch") {
        return make_interference_artifact_bytes();
    }

    if input_ref.ends_with("/version-mismatch") {
        return make_toolpath_artifact_bytes(999);
    }

    if input_ref.ends_with("/artifact-read-failed") {
        return Ok(vec![0_u8, 1, 2, 3]);
    }

    if input_ref.ends_with("/sim-failure") {
        return make_empty_toolpath_artifact_bytes();
    }

    make_toolpath_artifact_bytes(cam_core::FORMAT_VERSION_MINOR_V1)
}

fn parse_cam_result_ref(input_ref: &str) -> Option<(u64, String)> {
    let (job_id, suffix) = RefParser::parse_result_job_ref(input_ref, "cam").ok()?;
    Some((job_id, suffix))
}

fn build_result_ref(domain: &str, job_id: u64, suffix: &str) -> String {
    let suffix_segments = split_path_segments(suffix);
    RefFactory::result(domain, job_id, suffix_segments.as_slice())
        .unwrap_or_else(|_| format!("result://{}/{}/{}", domain, job_id, suffix))
}

fn build_log_ref(domain: &str, job_id: u64, suffix: &str) -> String {
    let suffix_segments = split_path_segments(suffix);
    RefFactory::log(domain, job_id, suffix_segments.as_slice())
        .unwrap_or_else(|_| format!("log://{}/{}/{}", domain, job_id, suffix))
}

fn split_path_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn classify_artifact_read_error(error: &BinaryFormatError) -> &'static str {
    match error {
        BinaryFormatError::ArtifactKindMismatch { .. } => "kind-mismatch",
        BinaryFormatError::UnsupportedMajorVersion { .. }
        | BinaryFormatError::UnsupportedMinorVersion { .. }
        | BinaryFormatError::ConverterRequired { .. } => "version-mismatch",
        _ => "artifact-read-failed",
    }
}

#[cfg(any(test, debug_assertions))]
fn make_empty_toolpath_artifact_bytes() -> Result<Vec<u8>, BinaryFormatError> {
    let toolpath = ToolPath::new(
        "sim-src".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![],
        vec![],
    );

    let mut payload = Vec::new();
    write_toolpath_payload_v1(&mut payload, &toolpath)?;

    let header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64);
    let mut bytes = Vec::new();
    header.write_to(&mut bytes)?;
    bytes.extend_from_slice(&payload);

    Ok(bytes)
}

#[cfg(any(test, debug_assertions))]
fn make_toolpath_artifact_bytes(version_minor: u16) -> Result<Vec<u8>, BinaryFormatError> {
    let toolpath = ToolPath::new(
        "nc-post-src".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            vec![PathSegment::new_line(
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(1.0, 0.0, 0.0),
                SegmentType::Cutting { feed_rate: 1.0 },
            )],
        )],
        vec![],
    );

    let mut payload = Vec::new();
    write_toolpath_payload_v1(&mut payload, &toolpath)?;

    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64);
    header.version_minor = version_minor;

    let mut bytes = Vec::new();
    header.write_to(&mut bytes)?;
    bytes.extend_from_slice(&payload);

    Ok(bytes)
}

#[cfg(any(test, debug_assertions))]
fn make_interference_artifact_bytes() -> Result<Vec<u8>, BinaryFormatError> {
    let interference = cam_core::InterferencePayload {
        events: vec![cam_core::InterferenceEvent {
            sample_index: 0,
            tool_id: "nc-post-src".to_string(),
            position: Point3D::new(0.0, 0.0, 0.0),
            normal: Point3D::new(0.0, 0.0, 1.0),
            penetration_depth: 0.1,
            kind: cam_core::InterferenceKind::Tool,
        }],
    };

    let mut payload = Vec::new();
    cam_core::write_interference_payload_v1(&mut payload, &interference)?;

    let header = ArtifactHeaderV1::new(ArtifactKind::Interference, payload.len() as u64);
    let mut bytes = Vec::new();
    header.write_to(&mut bytes)?;
    bytes.extend_from_slice(&payload);

    Ok(bytes)
}

impl JobExecutor for CamJobExecutorAdapter {
    fn execute(&self, job: &JobRecord, input_artifact_bytes: Option<&[u8]>) -> JobExecutionResult {
        match job.spec.job_type {
            JobType::CamProcess => self.run_cam_process(job),
            JobType::CuttingSimulation => self.run_cutting_simulation(job, input_artifact_bytes),
            JobType::NcPostFromCam => self.run_nc_post_from_cam(job, input_artifact_bytes),
        }
    }
}
