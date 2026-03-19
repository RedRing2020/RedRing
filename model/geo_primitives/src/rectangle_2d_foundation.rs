//! Rect2D の Foundation トレイト実装

use crate::Rect2D;
use geo_contracts::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for Rect2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Rectangle
    }

    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}
