//! Geometry向け orchestration 境界。

/// 将来の geometry orchestration ユースケース向けマーカー入力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryOrchestrationRequest {
    pub operation_name: String,
}
