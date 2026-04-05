//! Rect2D の Foundation トレイト実装

use crate::Rect2D;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata, Scalar};

impl<T: Scalar> PrimitiveMetadata for Rect2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Rectangle
    }
}
