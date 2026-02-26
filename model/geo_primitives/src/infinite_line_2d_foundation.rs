//! InfiniteLine2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::InfiniteLine2D;
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for InfiniteLine2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::InfiniteLine
    }

    fn measure(&self) -> Option<T> {
        None // 無限直線の長さは定義されない
    }
}
