//! LineSegment2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::LineSegment2D;
use geo_contracts::Scalar;
use geo_contracts::{ExtensionFoundation, PrimitiveKind};

impl<T: Scalar> ExtensionFoundation<T> for LineSegment2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }

    fn measure(&self) -> Option<T> {
        // 線分の長さを測度として返す
        Some((self.end_param - self.start_param).abs())
    }
}
