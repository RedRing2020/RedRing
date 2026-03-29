//! Feature向け orchestration 境界。

/// 将来の feature orchestration ユースケース向けマーカー入力境界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureOrchestrationRequest {
    pub feature_name: String,
}
