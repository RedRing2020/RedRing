//! CAM の工程のエンティティ（設計: CAM_PROCESS_ENTITY_DESIGN.md）
//!
//! 加工工程は、1 つの加工領域を 1 つの工具・1 つの加工の種類で加工する、工程の最小の単位とする。
//! 加工手順は、加工の対象（製品形状の面の ID の集合）と素材を持ち、加工工程を加工の順に持つ。

mod machining_operation;
mod machining_sequence;

pub use machining_operation::{
    MachiningOperation, MachiningOperationContent, MachiningOperationId,
};
pub use machining_sequence::{MachiningSequence, MachiningSequenceId, MachiningStock};

#[cfg(test)]
mod tests;
