use std::fmt;

use cam_core::{MachiningBoundary, MachiningStage, OperationSpec, ToleranceProfile, Tool};
use uuid::Uuid;

/// 加工工程の ID（全体で一意で、中身を変えても同じ工程を指す）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MachiningOperationId(Uuid);

impl MachiningOperationId {
    /// ランダムな ID を作る
    fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for MachiningOperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 加工工程の中身（変えると改訂の番号が増える）
#[derive(Debug, Clone, PartialEq)]
pub struct MachiningOperationContent {
    /// 利用者が付ける名前
    pub name: String,
    /// 加工の段階
    pub stage: MachiningStage,
    /// 加工の種類と、その条件（送りを含む）
    pub spec: OperationSpec,
    /// 工具
    pub tool: Tool<f64>,
    /// 加工領域（指定しなければ加工の対象の XY の範囲）
    pub boundary: Option<MachiningBoundary>,
    /// ツールパス生成のトレランスを決める精度プロファイル
    pub tolerance_profile: ToleranceProfile,
}

/// 加工工程
///
/// ID が全体で一意であるように、複写（`Clone`）はしない。工程を複写する場合は、中身から新しい工程を作る。
#[derive(Debug, PartialEq)]
pub struct MachiningOperation {
    id: MachiningOperationId,
    content: MachiningOperationContent,
    revision: u32,
}

impl MachiningOperation {
    /// 新しい ID と、最初の改訂の番号（1）で作る
    pub fn new(content: MachiningOperationContent) -> Self {
        Self {
            id: MachiningOperationId::new_random(),
            content,
            revision: 1,
        }
    }

    pub fn id(&self) -> MachiningOperationId {
        self.id
    }

    pub fn content(&self) -> &MachiningOperationContent {
        &self.content
    }

    /// 改訂の番号（中身を変えるたびと、属する加工手順の加工の対象・素材を変えるたびに増える）
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// 中身を `content` に替える。今の中身と違えば、改訂の番号を増やす
    pub fn set_content(&mut self, content: MachiningOperationContent) {
        if content != self.content {
            self.content = content;
            self.revise();
        }
    }

    /// 改訂の番号を増やす
    pub(crate) fn revise(&mut self) {
        self.revision += 1;
    }
}
