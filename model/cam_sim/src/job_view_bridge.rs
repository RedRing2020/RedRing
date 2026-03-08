use std::time::SystemTime;

use job_runtime::{JobEvent, JobOutputValidity, JobRecord, JobStatus, JobType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CamJobType {
    CamProcess,
    CuttingSimulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamJobStatus {
    Queued,
    Running,
    NeedsRecompute,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamJobOutputValidity {
    Active,
    Superseded,
    InvalidatedByDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CamJobSpec {
    pub job_type: CamJobType,
    pub input_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CamJobOutputRecord {
    pub result_ref: String,
    pub log_ref: Option<String>,
    pub produced_at: SystemTime,
    pub validity: CamJobOutputValidity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CamJobRecord {
    pub id: u64,
    pub spec: CamJobSpec,
    pub parent_job_id: Option<u64>,
    pub group_id: Option<String>,
    pub status: CamJobStatus,
    pub attempts: u32,
    pub output_history: Vec<CamJobOutputRecord>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CamJobEvent {
    StatusChanged {
        job_id: u64,
        to: CamJobStatus,
    },
    ProgressUpdated {
        job_id: u64,
        progress: f32,
        message: Option<String>,
    },
    ArtifactReady {
        job_id: u64,
        result_ref: String,
    },
    Completed {
        job_id: u64,
        status: CamJobStatus,
        result_ref: Option<String>,
        log_ref: Option<String>,
    },
    OutputValidityChanged {
        job_id: u64,
        result_ref: String,
        validity: CamJobOutputValidity,
    },
    MarkedNeedsRecompute {
        job_id: u64,
        by_job_id: u64,
    },
    GroupProgressUpdated {
        group_id: String,
        status: CamJobStatus,
        progress: f32,
        total: usize,
    },
}

impl From<JobType> for CamJobType {
    fn from(value: JobType) -> Self {
        match value {
            JobType::CamProcess => Self::CamProcess,
            JobType::CuttingSimulation => Self::CuttingSimulation,
        }
    }
}

impl From<JobStatus> for CamJobStatus {
    fn from(value: JobStatus) -> Self {
        match value {
            JobStatus::Queued => Self::Queued,
            JobStatus::Running => Self::Running,
            JobStatus::NeedsRecompute => Self::NeedsRecompute,
            JobStatus::Succeeded => Self::Succeeded,
            JobStatus::Failed => Self::Failed,
            JobStatus::Canceled => Self::Canceled,
        }
    }
}

impl From<JobOutputValidity> for CamJobOutputValidity {
    fn from(value: JobOutputValidity) -> Self {
        match value {
            JobOutputValidity::Active => Self::Active,
            JobOutputValidity::Superseded => Self::Superseded,
            JobOutputValidity::InvalidatedByDependency => Self::InvalidatedByDependency,
        }
    }
}

impl From<&JobRecord> for CamJobRecord {
    fn from(value: &JobRecord) -> Self {
        Self {
            id: value.id.0,
            spec: CamJobSpec {
                job_type: value.spec.job_type.clone().into(),
                input_ref: value.spec.input_ref.clone(),
            },
            parent_job_id: value.parent_job_id.map(|id| id.0),
            group_id: value.group_id.clone(),
            status: value.status.into(),
            attempts: value.attempts,
            output_history: value
                .output_history
                .iter()
                .map(|o| CamJobOutputRecord {
                    result_ref: o.result_ref.clone(),
                    log_ref: o.log_ref.clone(),
                    produced_at: o.produced_at,
                    validity: o.validity.into(),
                })
                .collect(),
            last_error: value.last_error.clone(),
        }
    }
}

impl From<&JobEvent> for CamJobEvent {
    fn from(value: &JobEvent) -> Self {
        match value {
            JobEvent::StatusChanged { job_id, to, .. } => Self::StatusChanged {
                job_id: job_id.0,
                to: (*to).into(),
            },
            JobEvent::ProgressUpdated {
                job_id,
                progress,
                message,
            } => Self::ProgressUpdated {
                job_id: job_id.0,
                progress: *progress,
                message: message.clone(),
            },
            JobEvent::ArtifactReady { job_id, result_ref } => Self::ArtifactReady {
                job_id: job_id.0,
                result_ref: result_ref.clone(),
            },
            JobEvent::Completed {
                job_id,
                status,
                result_ref,
                log_ref,
            } => Self::Completed {
                job_id: job_id.0,
                status: (*status).into(),
                result_ref: result_ref.clone(),
                log_ref: log_ref.clone(),
            },
            JobEvent::OutputValidityChanged {
                job_id,
                result_ref,
                validity,
            } => Self::OutputValidityChanged {
                job_id: job_id.0,
                result_ref: result_ref.clone(),
                validity: (*validity).into(),
            },
            JobEvent::MarkedNeedsRecompute { job_id, by_job_id } => Self::MarkedNeedsRecompute {
                job_id: job_id.0,
                by_job_id: by_job_id.0,
            },
            JobEvent::GroupProgressUpdated {
                group_id,
                status,
                progress,
                total,
            } => Self::GroupProgressUpdated {
                group_id: group_id.clone(),
                status: (*status).into(),
                progress: *progress,
                total: *total,
            },
        }
    }
}
