//! Ellipse2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::Ellipse2D;
use geo_contracts::{MeasureFoundation, PrimitiveKind, PrimitiveMetadata, Scalar};

impl<T: Scalar> PrimitiveMetadata for Ellipse2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ellipse
    }
}

impl<T: Scalar> MeasureFoundation<T> for Ellipse2D<T> {
    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}
