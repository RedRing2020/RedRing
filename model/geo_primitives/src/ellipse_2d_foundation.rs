//! Ellipse2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::Ellipse2D;
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for Ellipse2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ellipse
    }

    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}
