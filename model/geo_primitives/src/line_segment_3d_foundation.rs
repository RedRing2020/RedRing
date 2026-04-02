//! LineSegment3D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::LineSegment3D;
use geo_contracts::Scalar;
use geo_contracts::{MeasureFoundation, PrimitiveKind, PrimitiveMetadata};

impl<T: Scalar> PrimitiveMetadata for LineSegment3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}

impl<T: Scalar> MeasureFoundation<T> for LineSegment3D<T> {
    fn measure(&self) -> Option<T> {
        // 線分の長さを測度として返す
        Some(self.end_param - self.start_param)
    }
}
