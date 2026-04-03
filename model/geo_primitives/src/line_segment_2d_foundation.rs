//! LineSegment2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::LineSegment2D;
use geo_contracts::Scalar;
use geo_contracts::{MeasureFoundation, PrimitiveKind, PrimitiveMetadata};

impl<T: Scalar> PrimitiveMetadata for LineSegment2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}

impl<T: Scalar> MeasureFoundation<T> for LineSegment2D<T> {
    fn measure(&self) -> Option<T> {
        // 線分の長さを測度として返す
        Some(self.length())
    }
}
