//! geo_foundation - 幾何計算の基盤クレート
//!
//! 抽象型と橋渡し機能を提供し、他のgeoクレート間の
//! 共通基盤として機能する

// 抽象型モジュール
pub mod abstract_types;

// 共通機能モジュール
pub mod common;

// 主要な抽象型を再エクスポート
pub use abstract_types::{Scalar, ToleranceContext, TolerantEq};

// 共通機能を再エクスポート
pub use common::{constants, error, traits};

// テストモジュール
#[cfg(test)]
mod unit_tests;

/// 便利な再エクスポート
pub mod prelude {
    pub use crate::abstract_types::{Scalar, ToleranceContext, TolerantEq};
    pub use crate::common::constants::*;
    pub use crate::common::error::GeometryError;
    pub use crate::common::traits::*;
}
