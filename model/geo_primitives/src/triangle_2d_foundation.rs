//! Triangle2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::Triangle2D;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata, Scalar};

impl<T: Scalar> PrimitiveMetadata for Triangle2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }
}
