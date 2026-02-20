use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// エンティティの同一性ID
///
/// - ユーザー指定を想定しない内部識別子
/// - フィーチャー再実行での再現性向けに決定的生成を提供
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    /// ランダムなEntityIdを生成
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }

    /// フィーチャー出力情報から決定的に生成
    pub fn from_feature_output(feature_id: &str, output_index: u32, local_key: &str) -> Self {
        let seed = format!("redring.entity:{feature_id}:{output_index}:{local_key}");
        Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, seed.as_bytes()))
    }

    /// 任意シードから決定的に生成
    pub fn from_seed(seed: &str) -> Self {
        Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, seed.as_bytes()))
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
