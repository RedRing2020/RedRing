//! EllipseArc2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::EllipseArc2D;
use geo_contracts::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for EllipseArc2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }

    fn measure(&self) -> Option<T> {
        // 楕円弧の長さを測度として返す
        Some(self.arc_length())
    }
}
