use std::io::Cursor;

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ArtifactPayload, ContourLevelPath, CuttingDirection,
    InterferenceEvent, InterferenceKind, InterferencePayload, PathSegment, SegmentType, ToolPath,
    read_artifact_v1, write_interference_payload_v1, write_toolpath_payload_v1,
};
use geo_algorithms::Point3D;
use job_runtime::{JobExecutionResult, JobExecutor, JobRecord, JobStatus, JobType};

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
        if !job.spec.input_ref.starts_with("input://sim/") {
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

        let artifact_bytes = match build_mock_artifact_from_result_ref(&job.spec.input_ref) {
            Ok(bytes) => bytes,
            Err(message) => {
                return JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: 20,
                    result_ref: None,
                    log_ref: Some(format!("log://nc-post/{}/artifact-error", job.id.0)),
                    error: Some(message),
                };
            }
        };

        let mut cursor = Cursor::new(artifact_bytes);
        let (_, payload) = match read_artifact_v1(&mut cursor) {
            Ok(result) => result,
            Err(err) => {
                return JobExecutionResult {
                    status: JobStatus::Failed,
                    elapsed_millis: 20,
                    result_ref: None,
                    log_ref: Some(format!("log://nc-post/{}/artifact-read-failed", job.id.0)),
                    error: Some(format!("failed to read artifact binary: {}", err)),
                };
            }
        };

        match payload {
            ArtifactPayload::ToolPath(_) => JobExecutionResult {
                status: JobStatus::Succeeded,
                elapsed_millis: 240,
                result_ref: Some(format!("result://nc-post/{}/ok", job.id.0)),
                log_ref: Some(format!("log://nc-post/{}/ok", job.id.0)),
                error: None,
            },
            ArtifactPayload::Interference(_) => JobExecutionResult {
                status: JobStatus::Failed,
                elapsed_millis: 20,
                result_ref: None,
                log_ref: Some(format!("log://nc-post/{}/kind-mismatch", job.id.0)),
                error: Some(
                    "artifact kind mismatch: expected toolpath artifact for nc post from cam"
                        .to_string(),
                ),
            },
        }
    }
}

fn build_mock_artifact_from_result_ref(result_ref: &str) -> Result<Vec<u8>, String> {
    if result_ref.ends_with("/kind-mismatch") {
        return make_interference_artifact_bytes();
    }

    if result_ref.ends_with("/version-mismatch") {
        return make_toolpath_artifact_bytes(999);
    }

    make_toolpath_artifact_bytes(cam_core::FORMAT_VERSION_MINOR_V1)
}

fn make_toolpath_artifact_bytes(version_minor: u16) -> Result<Vec<u8>, String> {
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
    write_toolpath_payload_v1(&mut payload, &toolpath).map_err(|err| err.to_string())?;

    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64);
    header.version_minor = version_minor;

    let mut bytes = Vec::new();
    header.write_to(&mut bytes).map_err(|err| err.to_string())?;
    bytes.extend_from_slice(&payload);

    Ok(bytes)
}

fn make_interference_artifact_bytes() -> Result<Vec<u8>, String> {
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
    write_interference_payload_v1(&mut payload, &interference).map_err(|err| err.to_string())?;

    let header = ArtifactHeaderV1::new(ArtifactKind::Interference, payload.len() as u64);
    let mut bytes = Vec::new();
    header.write_to(&mut bytes).map_err(|err| err.to_string())?;
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
