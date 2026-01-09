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
//!
//! ## 標準読み込み（頂点重複削除）
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
//!
//! ## 高速バルク読み込み（GPU転送用）
//! ```rust,no_run
//! use geo_io::StlTriangleBulk;
//! use std::path::Path;
//!
//! // 高速バルク読み込み（Binary STL f32専用）
//! let bulk = StlTriangleBulk::<f32>::from_binary_stl_fast(Path::new("model.stl"))?;
//!
//! // GPU転送用の連続配列を取得
//! let vertex_data: &[f32] = bulk.vertices_slice();
//!
//! // 必要に応じてTriangleMesh3Dに変換
//! let mesh = bulk.to_triangle_mesh()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## インデックス付きバルク読み込み（頂点重複削減 + GPU最適化）
//! ```rust,no_run
//! use geo_io::StlIndexedBulk;
//! use std::path::Path;
//!
//! // ASCII/Binary STL読み込み時に自動的に頂点重複を削減
//! let indexed = StlIndexedBulk::<f32>::from_binary_stl_fast(Path::new("model.stl"))?;
//! println!("メモリ削減率: {:.1}%", indexed.memory_reduction());
//!
//! // GPU転送用データアクセス
//! let vertices = indexed.vertices_slice();   // ユニーク頂点のみ
//! let indices = indexed.indices_slice();     // インデックスバッファ
//!
//! // TriangleMesh3Dへ変換（既に最適化済み）
//! let mesh = indexed.to_triangle_mesh()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod error;
pub mod stl;
pub mod stl_bulk;
pub mod svg;

// Re-exports
pub use error::{IoError, StlError};
pub use stl_bulk::{StlIndexedBulk, StlTriangleBulk};
