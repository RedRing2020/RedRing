use crate::classification::PrimitiveKind;

/// 幾何プリミティブ向けの metadata capability です。
///
/// この trait は、プリミティブ種別の識別を提供します。
pub trait PrimitiveMetadata {
    /// プリミティブ種別を返します。
    fn primitive_kind(&self) -> PrimitiveKind;
}
