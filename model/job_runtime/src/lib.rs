//! 共通ジョブマネージャー基盤
//!
//! 実行制御（submit/status/cancel/retry）とイベント契約を提供

pub mod artifact_manifest;
pub mod events;
pub mod executor;
pub mod manager;
pub mod types;

pub use artifact_manifest::{
    ArtifactManifest, ArtifactManifestError, ArtifactType, ContractValidationDecision,
    FormatVersionCompatibility, decide_contract_validation_error,
    decide_contract_validation_result, evaluate_format_version_compatibility, validate_io_contract,
    validate_output_contract,
};
pub use events::JobEvent;
pub use executor::{JobExecutionResult, JobExecutor};
pub use manager::JobManager;
pub use types::{
    JobError, JobGroupSummary, JobId, JobOutputRecord, JobOutputValidity, JobRecord, JobRelation,
    JobSpec, JobStatus, JobType, RetryPolicy,
};

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
        if !dir.exists() {
            return;
        }

        for entry in fs::read_dir(dir).expect("source directory should be readable") {
            let entry = entry.expect("directory entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }

    #[test]
    fn test_job_runtime_source_does_not_reference_demo_symbols() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        collect_rs_files(&src_root, &mut files);

        let forbidden_symbols = [
            ["CamSimulation", "DemoScenario"].concat(),
            ["build_demo_", "artifacts_for_cam_simulation"].concat(),
            ["create_demo_", "snapshot_exports_for_scenario"].concat(),
            ["create_sample_", "snapshot_exports_for_demo"].concat(),
            ["cam_", "sim_demo"].concat(),
        ];

        for file in files {
            let content = fs::read_to_string(&file).expect("source file should be readable");
            for symbol in &forbidden_symbols {
                assert!(
                    !content.contains(symbol),
                    "job_runtime layer must not reference demo symbol '{}' in {}",
                    symbol,
                    file.display()
                );
            }
        }
    }
}
