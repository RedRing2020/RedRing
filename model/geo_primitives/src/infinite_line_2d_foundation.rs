//! InfiniteLine2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::InfiniteLine2D;
use geo_contracts::Scalar;
use geo_contracts::{MeasureFoundation, PrimitiveKind, PrimitiveMetadata};

impl<T: Scalar> PrimitiveMetadata for InfiniteLine2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::InfiniteLine
    }
}

impl<T: Scalar> MeasureFoundation<T> for InfiniteLine2D<T> {
    fn measure(&self) -> Option<T> {
        None // 無限直線の長さは定義されない
    }
}
