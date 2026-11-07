use std::collections::HashSet;
use std::fmt;

use cam_core::StockMargins;
use geo_entity::FaceId;
use uuid::Uuid;

use crate::{MachiningOperation, MachiningOperationId};

/// 加工手順の ID（全体で一意で、中身を変えても同じ手順を指す）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MachiningSequenceId(Uuid);

impl MachiningSequenceId {
    /// ランダムな ID を作る
    fn new_random() -> Self {
        Self(Uuid::new_v4())
    }

    /// UUID から作る（保存した手順の ID に戻すときに使う）
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MachiningSequenceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 加工手順の素材（最初の加工工程の素材になる）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MachiningStock {
    /// 加工の対象を囲む箱からの余裕
    pub margins: StockMargins,
    /// 切削シミュレーションの素材の形状の許容誤差
    pub precision: f64,
}

/// 加工手順
///
/// 加工工程は 1 つの加工手順に属し、手順が加工の順に持つ。加工の対象・素材を変えると、すべての加工工程の
/// 改訂の番号が増える（前の結果は、変える前の加工の対象・素材で計算したものになるため）。ID が全体で一意で
/// あるように、複写（`Clone`）はしない。
#[derive(Debug, PartialEq)]
pub struct MachiningSequence {
    id: MachiningSequenceId,
    name: String,
    target_faces: Vec<FaceId>,
    stock: MachiningStock,
    operations: Vec<MachiningOperation>,
}

impl MachiningSequence {
    /// 新しい ID で、加工工程のない手順を作る（加工の対象の同じ面は 1 回だけ持つ）
    pub fn new(name: String, target_faces: Vec<FaceId>, stock: MachiningStock) -> Self {
        Self {
            id: MachiningSequenceId::new_random(),
            name,
            target_faces: unique_faces(target_faces),
            stock,
            operations: Vec::new(),
        }
    }

    /// 保存した ID・名前・加工の対象・素材・加工工程から作る（加工の対象の同じ面は 1 回だけ持つ）
    pub fn restore(
        id: MachiningSequenceId,
        name: String,
        target_faces: Vec<FaceId>,
        stock: MachiningStock,
        operations: Vec<MachiningOperation>,
    ) -> Self {
        Self {
            id,
            name,
            target_faces: unique_faces(target_faces),
            stock,
            operations,
        }
    }

    pub fn id(&self) -> MachiningSequenceId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// 加工の対象（製品形状の面の ID の集合。同じ面は 1 回だけ、加えた順に持つ）
    pub fn target_faces(&self) -> &[FaceId] {
        &self.target_faces
    }

    pub fn stock(&self) -> &MachiningStock {
        &self.stock
    }

    /// 加工の順の加工工程
    pub fn operations(&self) -> &[MachiningOperation] {
        &self.operations
    }

    /// ID が `id` の加工工程
    pub fn operation_mut(&mut self, id: MachiningOperationId) -> Option<&mut MachiningOperation> {
        self.operations
            .iter_mut()
            .find(|operation| operation.id() == id)
    }

    /// 加工工程を最後に加える
    pub fn push_operation(&mut self, operation: MachiningOperation) {
        self.operations.push(operation);
    }

    /// 加工の対象を `target_faces` に替える。面の集合として今と違えば、すべての加工工程の改訂の番号を増やす
    /// （順と重なりは比べない）
    pub fn set_target_faces(&mut self, target_faces: Vec<FaceId>) {
        let target_faces = unique_faces(target_faces);
        let current: HashSet<FaceId> = self.target_faces.iter().copied().collect();
        let changed = target_faces.len() != current.len()
            || !target_faces.iter().all(|face| current.contains(face));
        self.target_faces = target_faces;
        if changed {
            self.revise_operations();
        }
    }

    /// 素材を `stock` に替える。今と違えば、すべての加工工程の改訂の番号を増やす
    pub fn set_stock(&mut self, stock: MachiningStock) {
        if stock != self.stock {
            self.stock = stock;
            self.revise_operations();
        }
    }

    fn revise_operations(&mut self) {
        for operation in &mut self.operations {
            operation.revise();
        }
    }
}

/// 同じ面を除いた、最初に現れた順の面
fn unique_faces(faces: Vec<FaceId>) -> Vec<FaceId> {
    let mut seen = HashSet::new();
    faces
        .into_iter()
        .filter(|face| seen.insert(*face))
        .collect()
}
