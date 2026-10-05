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

impl<T: Scalar> WeightStorage<T> {
    /// 指定されたインデックスの重みを取得
    pub fn get_weight(&self, index: usize) -> T {
        match self {
            WeightStorage::Uniform => T::ONE,
            WeightStorage::Individual(weights) => weights.get(index).copied().unwrap_or(T::ONE),
        }
    }

    /// 重みを指定しない非有理（`Uniform`）として保持しているかどうか
    #[must_use]
    pub fn is_uniform(&self) -> bool {
        matches!(self, WeightStorage::Uniform)
    }

    /// 非有理（全重みが1.0）かどうか判定
    #[must_use]
    pub fn is_non_rational(&self) -> bool {
        match self {
            WeightStorage::Uniform => true,
            WeightStorage::Individual(weights) => weights.iter().all(|&w| w == T::ONE),
        }
    }

    /// Uniform重みからIndividualに変換（指定したサイズで）
    #[must_use]
    pub fn to_individual(&self, num_points: usize) -> WeightStorage<T> {
        match self {
            WeightStorage::Uniform => WeightStorage::Individual(vec![T::ONE; num_points]),
            WeightStorage::Individual(_) => self.clone(),
        }
    }
}
