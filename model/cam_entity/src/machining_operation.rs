use std::fmt;
use std::num::NonZeroU32;

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

    /// UUID から作る（保存した工程の ID に戻すときに使う）
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MachiningOperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 加工工程の改訂の鍵（改訂の番号を増やすたびに作り直す、全体で一意な値）
///
/// 改訂の番号は、保存した工程を複製したときや、保存せずに捨てた変更でも、同じ工程の ID で同じ番号になりうる。
/// 改訂の鍵は、同じ鍵なら同じ中身から計算したことを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachiningRevisionKey(Uuid);

impl MachiningRevisionKey {
    /// ランダムな鍵を作る
    fn new_random() -> Self {
        Self(Uuid::new_v4())
    }

    /// UUID から作る（保存した改訂の鍵に戻すときに使う）
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MachiningRevisionKey {
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
    revision_key: MachiningRevisionKey,
}

impl MachiningOperation {
    /// 新しい ID と、最初の改訂の番号（1）・新しい改訂の鍵で作る
    pub fn new(content: MachiningOperationContent) -> Self {
        Self {
            id: MachiningOperationId::new_random(),
            content,
            revision: 1,
            revision_key: MachiningRevisionKey::new_random(),
        }
    }

    /// 保存した ID・中身・改訂の番号・改訂の鍵から作る
    pub fn restore(
        id: MachiningOperationId,
        content: MachiningOperationContent,
        revision: NonZeroU32,
        revision_key: MachiningRevisionKey,
    ) -> Self {
        Self {
            id,
            content,
            revision: revision.get(),
            revision_key,
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

    /// 改訂の鍵（改訂の番号を増やすたびに作り直す）
    pub fn revision_key(&self) -> MachiningRevisionKey {
        self.revision_key
    }

    /// 中身を `content` に替える。今の中身と違えば、改訂の番号を増やす
    pub fn set_content(&mut self, content: MachiningOperationContent) {
        if content != self.content {
            self.content = content;
            self.revise();
        }
    }

    /// 改訂の番号を増やし、改訂の鍵を作り直す（番号は上限で止める。見分けは改訂の鍵で行う）
    pub(crate) fn revise(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.revision_key = MachiningRevisionKey::new_random();
    }
}
