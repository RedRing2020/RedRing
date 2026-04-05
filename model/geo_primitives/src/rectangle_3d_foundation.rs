//! Rect3D の Foundation トレイト実装

use crate::Rect3D;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata, Scalar};

impl<T: Scalar> PrimitiveMetadata for Rect3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Rectangle
    }
}
