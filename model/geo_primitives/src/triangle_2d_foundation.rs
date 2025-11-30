//! Triangle2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::Triangle2D;
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for Triangle2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }

    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}
