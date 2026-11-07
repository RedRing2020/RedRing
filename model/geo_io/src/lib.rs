//! geo_io - 幾何データのファイルI/Oクレート
//!
//! STL、OBJ、PLY等の3Dメッシュファイル形式の読み書きと、形状（三角形メッシュ・NURBS 曲面）のバイト列の
//! 読み書き（`binary_geometry`）を提供します。
//!
//! # サポートフォーマット
//! - STL (ASCII/Binary)
//! - OBJ (予定)
//! - PLY (予定)

pub mod binary_geometry;
pub mod error;
pub mod fixtures;
pub mod stl;
pub mod stl_bulk;
pub mod svg;

// Re-exports
pub use error::{IoError, StlError};
pub use stl_bulk::{StlIndexedBulk, StlTriangleBulk};
