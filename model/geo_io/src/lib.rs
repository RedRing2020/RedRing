//! geo_io - 幾何データのファイルI/Oクレート
//!
//! STL、OBJ、PLY等の3Dメッシュファイル形式の読み書きを提供します。
//!
//! # サポートフォーマット
//! - STL (ASCII/Binary)
//! - OBJ (予定)
//! - PLY (予定)
//!
//! # 使用例
//! ```rust,no_run
//! use geo_io::stl;
//! use geo_primitives::TriangleMesh3D;
//! use std::path::Path;
//!
//! // STLファイルの読み込み
//! let mesh: TriangleMesh3D<f64> = stl::load_stl(Path::new("model.stl"))?;
//!
//! // STLファイルの保存
//! stl::save_stl(&mesh, Path::new("output.stl"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod error;
pub mod stl;

// Re-exports
pub use error::{IoError, StlError};
