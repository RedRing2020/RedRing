//! Rect3D の Foundation トレイト実装

use crate::Rect3D;
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for Rect3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Rectangle
    }

    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}
