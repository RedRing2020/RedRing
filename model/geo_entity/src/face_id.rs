use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// 面の同一性ID
///
/// - 全体で一意で、面が属するエンティティが替わっても同じ面を指す
/// - ユーザー指定を想定しない内部識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaceId(Uuid);

impl FaceId {
    /// ランダムなFaceIdを生成
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for FaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
