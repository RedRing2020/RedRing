use std::io::Cursor;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, BinaryFormatError, ContourLevelPath, CuttingDirection,
    PathSegment, SegmentType, Tool, ToolPath, read_toolpath_artifact_v1, write_toolpath_payload_v1,
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
                artifact_bytes: None,
                log_ref: Some(format!("log://cam/{}/invalid", job.id.0)),
                error: Some(format!(
                    "invalid input_ref for cam process: {}",
                    job.spec.input_ref
                )),
            };
        }

        let artifact_bytes = match build_cam_process_artifact_bytes(&job.spec.input_ref) {
            Ok(bytes) => bytes,
            Err(err) => {
                return JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: 20,
                    result_ref: None,
                    artifact_bytes: None,
                    log_ref: Some(format!("log://cam/{}/artifact-error", job.id.0)),
                    error: Some(format!("failed to build cam artifact bytes: {}", err)),
                };
            }
        };

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 200,
            result_ref: Some(format!("result://cam/{}/ok", job.id.0)),
            artifact_bytes: Some(artifact_bytes),
            log_ref: Some(format!("log://cam/{}/ok", job.id.0)),
            error: None,
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
                log_ref: Some(format!("log://sim/{}/invalid", job.id.0)),
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
                log_ref: Some(format!("log://sim/{}/invalid", job.id.0)),
                error: Some("cutting simulation requires parent_job_id".to_string()),
            };
        };

        if parent_job_id.0 != cam_job_id_from_ref {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(format!("log://sim/{}/invalid", job.id.0)),
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
                log_ref: Some(format!("log://sim/{}/invalid", job.id.0)),
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
                log_ref: Some(format!("log://sim/{}/missing-input", job.id.0)),
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
                artifact_bytes: None,
                log_ref: Some(format!("log://sim/{}/sim-failure", job.id.0)),
                error: Some(format!("failed to run cutting simulation: {}", err)),
            };
        }

        JobExecutionResult {
            status: JobStatus::Succeeded,
            elapsed_millis: 300,
            result_ref: Some(format!("result://sim/{}/ok", job.id.0)),
            artifact_bytes: None,
            log_ref: Some(format!("log://sim/{}/ok", job.id.0)),
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
                log_ref: Some(format!("log://nc-post/{}/invalid", job.id.0)),
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
                log_ref: Some(format!("log://nc-post/{}/invalid", job.id.0)),
                error: Some("nc post from cam requires parent_job_id".to_string()),
            };
        };

        if parent_job_id.0 != cam_job_id_from_ref {
            return JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 10,
                result_ref: None,
                artifact_bytes: None,
                log_ref: Some(format!("log://nc-post/{}/invalid", job.id.0)),
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
                log_ref: Some(format!("log://nc-post/{}/invalid", job.id.0)),
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
                log_ref: Some(format!("log://nc-post/{}/missing-input", job.id.0)),
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
            result_ref: Some(format!("result://nc-post/{}/ok", job.id.0)),
            artifact_bytes: None,
            log_ref: Some(format!("log://nc-post/{}/ok", job.id.0)),
            error: None,
        }
    }
}

fn build_cam_process_artifact_bytes(input_ref: &str) -> Result<Vec<u8>, BinaryFormatError> {
    let _ = input_ref;

    #[cfg(test)]
    {
        if input_ref.ends_with("/kind-mismatch") {
            return make_interference_artifact_bytes();
        }

        if input_ref.ends_with("/version-mismatch") {
            return make_toolpath_artifact_bytes(999);
        }

        if input_ref.ends_with("/artifact-read-failed") {
            return Ok(vec![0_u8, 1, 2, 3]);
        }
    }

    make_toolpath_artifact_bytes(cam_core::FORMAT_VERSION_MINOR_V1)
}

fn parse_cam_result_ref(input_ref: &str) -> Option<(u64, &str)> {
    let remainder = input_ref.strip_prefix("result://cam/")?;
    let (cam_job_id_str, suffix) = remainder.split_once('/')?;
    let cam_job_id = cam_job_id_str.parse::<u64>().ok()?;

    Some((cam_job_id, suffix))
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

#[cfg(test)]
#[allow(dead_code)]
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

#[cfg(test)]
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
