//! Abstract Types Module
//!
//! 数値解析の基盤となる抽象型とトレイトを提供

pub mod angle;
pub mod scalar;

// 主要な型とトレイトを再エクスポート
pub use angle::{Angle, AngleType};
pub use scalar::Scalar;

/// 許容誤差を考慮した等価比較トレイト
pub trait TolerantEq {
    /// 許容誤差を考慮した等価比較
    fn tolerant_eq(&self, other: &Self, tolerance: Self) -> bool;
}

// Scalarトレイトを実装する型に対する汎用実装
impl<T: Scalar> TolerantEq for T {
    fn tolerant_eq(&self, other: &Self, tolerance: Self) -> bool {
        (*self - *other).abs() < tolerance
    }
}
