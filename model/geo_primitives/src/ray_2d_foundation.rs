//! Ray2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::Ray2D;
use geo_contracts::Scalar;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

impl<T: Scalar> PrimitiveMetadata for Ray2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ray
    }
}
