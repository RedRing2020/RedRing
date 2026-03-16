//! InfiniteLine3D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::InfiniteLine3D;
use geo_contracts::Scalar;
use geo_foundation::{ExtensionFoundation, PrimitiveKind};

impl<T: Scalar> ExtensionFoundation<T> for InfiniteLine3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::InfiniteLine
    }

    fn measure(&self) -> Option<T> {
        None // 無限直線の長さは定義されない
    }
}
