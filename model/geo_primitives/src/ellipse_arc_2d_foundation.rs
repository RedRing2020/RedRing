//! EllipseArc2D Foundation 実装
//!
//! ExtensionFoundation トレイトの実装

use crate::EllipseArc2D;
use geo_contracts::{MeasureFoundation, PrimitiveKind, PrimitiveMetadata, Scalar};

impl<T: Scalar> PrimitiveMetadata for EllipseArc2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }
}

impl<T: Scalar> MeasureFoundation<T> for EllipseArc2D<T> {
    fn measure(&self) -> Option<T> {
        // 楕円弧の長さを測度として返す
        Some(self.arc_length())
    }
}
