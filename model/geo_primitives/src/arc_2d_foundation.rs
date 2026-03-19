//! Arc2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::Arc2D;
use geo_contracts::Scalar;
use geo_contracts::{ExtensionFoundation, PrimitiveKind};

impl<T: Scalar> ExtensionFoundation<T> for Arc2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }

    fn measure(&self) -> Option<T> {
        // 円弧の長さを測度として返す
        Some(self.arc_length())
    }
}
