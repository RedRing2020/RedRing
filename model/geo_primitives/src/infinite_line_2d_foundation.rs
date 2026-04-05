//! InfiniteLine2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::InfiniteLine2D;
use geo_contracts::Scalar;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

impl<T: Scalar> PrimitiveMetadata for InfiniteLine2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::InfiniteLine
    }
}
