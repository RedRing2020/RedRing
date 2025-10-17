//! Core Foundation Legacy Bridge
//!
//! geometry/ の Core Foundation traits への bridge モジュール
//! 旧 abstract_types/foundation/core_foundation.rs のインポートを壊さない互換性レイヤー

// geometry/ からの re-export で bridge を提供
pub use crate::geometry::core_foundation::*;
