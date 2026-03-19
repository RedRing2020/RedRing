//! Ray2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::Ray2D;
use geo_contracts::Scalar;
use geo_contracts::{ExtensionFoundation, PrimitiveKind};

impl<T: Scalar> ExtensionFoundation<T> for Ray2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ray
    }

    fn measure(&self) -> Option<T> {
        None // 半無限直線の長さは定義されない
    }
}
