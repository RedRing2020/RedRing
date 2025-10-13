//! Core Foundation Legacy Bridge
//!
//! foundation/ の Core Foundation traits への bridge モジュール
//! 旧 geometry/core_foundation.rs のインポートを壊さない互換性レイヤー

// foundation/ からの re-export で bridge を提供
pub use crate::abstract_types::foundation::core_foundation::*;
