//! Abstract Types Module
//!
//! 数値解析の基盤となる抽象型とトレイトを提供

pub mod angle;
pub mod scalar;
pub mod tolerance;

// 主要な型とトレイトを再エクスポート
pub use angle::{Angle, AngleType};
pub use scalar::Scalar;
pub use tolerance::{ToleranceContext, TolerantEq, GEOMETRIC_TOLERANCE};
