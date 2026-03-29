//! Geometry orchestration boundaries.

/// Marker request boundary for future geometry orchestration use cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryOrchestrationRequest {
    pub operation_name: String,
}
