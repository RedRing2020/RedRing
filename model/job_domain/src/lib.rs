//! job_domain - ジョブ投入/制約判定のドメイン抽象化
//!
//! `job_runtime` の実行基盤とは分離し、ユースケース/ポリシー層を定義する。

pub mod cam_policy;
pub mod job_view_bridge;
pub mod policy;
pub mod service;
pub mod types;

pub use cam_policy::{CamWorkflowPolicy, JOB_TYPE_CAM_PROCESS, JOB_TYPE_CUTTING_SIMULATION};
pub use job_view_bridge::{
    CamJobEvent, CamJobOutputRecord, CamJobOutputValidity, CamJobRecord, CamJobSpec, CamJobStatus,
    CamJobType,
};
pub use policy::{DomainRuleViolation, WorkflowPolicy};
pub use service::{DomainAction, JobDomainService};
pub use types::{JobNode, JobSubmissionRequest, WorkflowSnapshot};
