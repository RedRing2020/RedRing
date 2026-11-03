#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSubmissionRequest {
    pub job_type: String,
    pub input_ref: String,
    pub parent_job_id: Option<u64>,
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobNode {
    pub id: u64,
    pub job_type: String,
    pub parent_job_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowSnapshot {
    pub jobs: Vec<JobNode>,
}

impl WorkflowSnapshot {
    pub fn children_of(&self, parent_id: u64) -> impl Iterator<Item = &JobNode> {
        self.jobs
            .iter()
            .filter(move |j| j.parent_job_id == Some(parent_id))
    }

    pub fn find(&self, id: u64) -> Option<&JobNode> {
        self.jobs.iter().find(|j| j.id == id)
    }
}
