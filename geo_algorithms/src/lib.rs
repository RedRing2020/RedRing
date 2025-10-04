//! geo_algorithms: 幾何／数値アルゴリズム層
//!
//! 目的:
//! - geo_core の軽量プリミティブを操作する再利用可能な解法群
//! - サンプリング / フィッティング / 解法 / 交差検出 などを集約
//! - 上位クレートからは feature ゲーティングで段階利用可能に
//!
//! 方針:
//! - 状態を持つアルゴリズムは明示的な構造体 (e.g., AdaptiveSampler)
//! - 設定は *Config 構造体 / Builder パターン (将来拡張)
//! - geo_core への依存のみ（逆依存禁止）

pub mod sampling;
pub mod solver;
pub mod fitting;
pub mod intersection;

pub use sampling::*;
pub use solver::*;
pub use fitting::*;
pub use intersection::*;
