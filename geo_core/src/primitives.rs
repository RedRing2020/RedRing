//! # geo_core primitives モジュール - 廃止予定
//!
//! **⚠️ このモジュールは廃止予定です。`geo_primitives` クレートをご利用ください。**
//!
//! ## 移行ガイド
//!
//! | 旧 (geo_core) | 新 (geo_primitives) |
//! |---------------|---------------------|
//! | `geo_core::Point2D` | `geo_primitives::Point2D` |
//! | `geo_core::Point3D` | `geo_primitives::Point3D` |
//! | `geo_core::LineSegment2D` | `geo_primitives::LineSegment2D` |
//! | `geo_core::Arc2D` | `geo_primitives::Arc2D` |
//! | `geo_core::Polygon2D` | `geo_primitives::Polygon2D` |
//!
//! ## 移行例
//! ```rust
//! // 旧:
//! // use geo_core::{Point2D, Point3D};
//!
//! // 新:
//! use geo_primitives::{Point2D, Point3D};
//! ```
//!
//! geo_coreは数値計算・許容誤差・ロバスト判定に特化し、
//! 幾何プリミティブはgeo_primitivesに統合されました。

#[deprecated(note = "Use geo_primitives::Point2D instead")]
pub use crate::primitives2d::Point2D;

#[deprecated(note = "Use geo_primitives::Point3D instead")]
pub use crate::point3d::Point3D;

#[deprecated(note = "Use geo_primitives instead")]
pub use crate::primitives2d::{
    LineSegment2D, Arc2D, Polygon2D,
    ParametricCurve2D,
};
