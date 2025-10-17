//! Core Foundation Legacy Bridge
//!
//! geometry/ の Core Foundation traits への bridge モジュール
//! 旧 abstract_types/geometry/core_foundation.rs のインポートを壊さない互換性レイヤー

// src直下のcore_foundationからの re-export で bridge を提供
pub use crate::core_foundation::*;
