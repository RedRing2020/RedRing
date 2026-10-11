//! NURBS重み格納方式

use analysis::Scalar;

/// NURBS重み格納方式
///
/// 重みを指定しない非有理NURBSは `Uniform` で表し、重みは常に 1 とする（値を持たない）。
/// 重みを指定した場合は、全重みが同じ値でも `Individual` として保持する。
#[derive(Debug, Clone, PartialEq, Default)]
pub enum WeightStorage<T: Scalar> {
    /// 非有理（全重み = 1）
    #[default]
    Uniform,
    /// 有理（制御点ごとの重み）
    Individual(Vec<T>),
}

impl<T: Scalar> WeightStorage<T> {}
