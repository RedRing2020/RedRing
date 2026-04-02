//! Rect2D の Foundation トレイト実装

use crate::Rect2D;
use geo_contracts::{MeasureFoundation, PrimitiveKind, PrimitiveMetadata, Scalar};

impl<T: Scalar> PrimitiveMetadata for Rect2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Rectangle
    }
}

impl<T: Scalar> MeasureFoundation<T> for Rect2D<T> {
    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}
