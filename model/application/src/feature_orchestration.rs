//! Feature orchestration boundaries.

/// Marker request boundary for future feature orchestration use cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureOrchestrationRequest {
    pub feature_name: String,
}
