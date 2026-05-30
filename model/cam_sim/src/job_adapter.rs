use std::io::Cursor;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, BinaryFormatError, ContourLevelPath, CuttingDirection,
    InterferenceEvent, InterferenceKind, InterferencePayload, PathSegment, SegmentType, Tool,
    ToolPath, read_toolpath_artifact_v1, write_interference_payload_v1, write_toolpath_payload_v1,
};
use geo_algorithms::{Aabb3D, Point3D, octree::VoxelOctree};
use job_runtime::{JobExecutionResult, JobExecutor, JobRecord, JobStatus, JobType};

use crate::{CuttingSimulator, SnapshotInterval};

/// cam_sim から JobManager へ接続する初期アダプタ
#[derive(Debug, Default, Clone, Copy)]
pub struct CamJobExecutorAdapter;

impl CamJobExecutorAdapter {
    fn run_cam_process(&self, job: &JobRecord) -> JobExecutionResult {
        if !job.spec.input_ref.starts_with("input://cam/") {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                log_ref: Some(format!("log://cam/{}/invalid", job.id.0)),
                error: Some(format!(
                    "invalid input_ref for cam process: {}",
                    job.spec.input_ref
                )),
            };
        }

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 200,
            result_ref: Some(format!("result://cam/{}/ok", job.id.0)),
            log_ref: Some(format!("log://cam/{}/ok", job.id.0)),
            error: None,
        }
    }

    fn run_cutting_simulation(&self, job: &JobRecord) -> JobExecutionResult {
        if !job.spec.input_ref.starts_with("result://cam/") {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                log_ref: Some(format!("log://sim/{}/invalid", job.id.0)),
                error: Some(format!(
                    "invalid input_ref for cutting simulation: {}",
                    job.spec.input_ref
                )),
            };
        }

        let toolpath =
            match read_cutting_simulation_toolpath_from_cam_result_ref(&job.spec.input_ref) {
                Ok(toolpath) => toolpath,
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
                        log_ref: Some(log_ref),
                        error: Some(format!(
                            "failed to read cutting simulation input artifact: {}",
                            err
                        )),
                    };
                }
            };

        let mut simulator = CuttingSimulator::new(
            VoxelOctree::new(
                Aabb3D::new(
                    Point3D::new(0.0, 0.0, 0.0),
                    Point3D::new(100.0, 100.0, 100.0),
                ),
                4,
            ),
            SnapshotInterval::default(),
        );
        let tool = Tool::flat_end_mill("sim-tool".to_string(), 10.0, 30.0);

        if let Err(err) = simulator.simulate(&toolpath, &tool) {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 30,
                result_ref: None,
                log_ref: Some(format!("log://sim/{}/sim-failure", job.id.0)),
                error: Some(format!("failed to run cutting simulation: {}", err)),
            };
        }

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 300,
            result_ref: Some(format!("result://sim/{}/ok", job.id.0)),
            log_ref: Some(format!("log://sim/{}/ok", job.id.0)),
            error: None,
        }
    }

    fn run_nc_post_from_cam(&self, job: &JobRecord) -> JobExecutionResult {
        if !job.spec.input_ref.starts_with("result://cam/") {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                log_ref: Some(format!("log://nc-post/{}/invalid", job.id.0)),
                error: Some(format!(
                    "invalid input_ref for nc post from cam: {}",
                    job.spec.input_ref
                )),
            };
        }

        let artifact_bytes = match build_artifact_bytes_from_cam_result_ref(&job.spec.input_ref) {
            Ok(bytes) => bytes,
            Err(err) => {
                return JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: 20,
                    result_ref: None,
                    log_ref: Some(format!("log://nc-post/{}/artifact-error", job.id.0)),
                    error: Some(format!("failed to build artifact bytes: {}", err)),
                };
            }
        };

        let mut cursor = Cursor::new(artifact_bytes);
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
                log_ref: Some(log_ref),
                error: Some(format!("failed to read artifact binary: {}", err)),
            };
        }

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 240,
            result_ref: Some(format!("result://nc-post/{}/ok", job.id.0)),
            log_ref: Some(format!("log://nc-post/{}/ok", job.id.0)),
            error: None,
        }
    }
}

fn build_artifact_bytes_from_cam_result_ref(
    result_ref: &str,
) -> Result<Vec<u8>, BinaryFormatError> {
    if result_ref.ends_with("/kind-mismatch") {
        return make_interference_artifact_bytes();
    }

    if result_ref.ends_with("/version-mismatch") {
        return make_toolpath_artifact_bytes(999);
    }

    make_toolpath_artifact_bytes(cam_core::FORMAT_VERSION_MINOR_V1)
}

fn build_cutting_simulation_input_artifact_bytes(
    result_ref: &str,
) -> Result<Vec<u8>, BinaryFormatError> {
    if result_ref.ends_with("/artifact-read-failed") {
        return Ok(vec![0_u8, 1, 2, 3]);
    }
    if result_ref.ends_with("/sim-failure") {
        return make_empty_toolpath_artifact_bytes();
    }

    build_artifact_bytes_from_cam_result_ref(result_ref)
}

fn read_cutting_simulation_toolpath_from_cam_result_ref(
    result_ref: &str,
) -> Result<ToolPath<f64>, BinaryFormatError> {
    let artifact_bytes = build_cutting_simulation_input_artifact_bytes(result_ref)?;

    let mut cursor = Cursor::new(artifact_bytes);
    let (_, toolpath) = read_toolpath_artifact_v1(&mut cursor)?;

    Ok(toolpath)
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

fn make_interference_artifact_bytes() -> Result<Vec<u8>, BinaryFormatError> {
    let interference = InterferencePayload {
        events: vec![InterferenceEvent {
            sample_index: 0,
            tool_id: "nc-post-src".to_string(),
            position: Point3D::new(0.0, 0.0, 0.0),
            normal: Point3D::new(0.0, 0.0, 1.0),
            penetration_depth: 0.1,
            kind: InterferenceKind::Tool,
        }],
    };

    let mut payload = Vec::new();
    write_interference_payload_v1(&mut payload, &interference)?;

    let header = ArtifactHeaderV1::new(ArtifactKind::Interference, payload.len() as u64);
    let mut bytes = Vec::new();
    header.write_to(&mut bytes)?;
    bytes.extend_from_slice(&payload);

    Ok(bytes)
}

impl JobExecutor for CamJobExecutorAdapter {
    fn execute(&self, job: &JobRecord) -> JobExecutionResult {
        match job.spec.job_type {
            JobType::CamProcess => self.run_cam_process(job),
            JobType::CuttingSimulation => self.run_cutting_simulation(job),
            JobType::NcPostFromCam => self.run_nc_post_from_cam(job),
        }
    }
}
