//! LineSegment2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::LineSegment2D;
use geo_contracts::Scalar;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

impl<T: Scalar> PrimitiveMetadata for LineSegment2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}
