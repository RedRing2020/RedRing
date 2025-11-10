//! NURBS共通定義（重み格納方式とエラー型）

use analysis::Scalar;

/// NURBS操作エラー
#[derive(Debug, Clone, PartialEq)]
pub enum NurbsOperationError {
    /// 無効なパラメータ値
    InvalidParameter,
    /// 無効なインデックス
    InvalidIndex,
    /// 無効な次数
    InvalidDegree,
    /// 無効な重み値
    InvalidWeight,
    /// 無効な幾何形状
    InvalidGeometry,
    /// 操作がサポートされていない
    UnsupportedOperation,
}

/// NURBS重み格納方式
#[derive(Debug, Clone, PartialEq)]
pub enum WeightStorage<T: Scalar> {
    /// 全ての制御点で同じ重み値を使用（非有理NURBS用）
    Uniform(T),
    /// 各制御点で個別の重み値を持つ（有理NURBS用）
    Individual(Vec<T>),
}

impl<T: Scalar> WeightStorage<T> {
    /// 指定されたインデックスの重みを取得
    pub fn get_weight(&self, index: usize) -> T {
        match self {
            WeightStorage::Uniform(w) => *w,
            WeightStorage::Individual(weights) => weights.get(index).copied().unwrap_or(T::ONE),
        }
    }

    /// 重みの総数を取得
    pub fn len(&self) -> usize {
        match self {
            WeightStorage::Uniform(_) => 1,
            WeightStorage::Individual(weights) => weights.len(),
        }
    }

    /// 重みストレージが空かどうか
    pub fn is_empty(&self) -> bool {
        match self {
            WeightStorage::Uniform(_) => false,
            WeightStorage::Individual(weights) => weights.is_empty(),
        }
    }

    /// 全て同じ重み値かどうかを判定
    pub fn is_uniform(&self) -> bool {
        matches!(self, WeightStorage::Uniform(_))
    }

    /// 非有理（全重みが1.0）かどうか判定
    pub fn is_non_rational(&self) -> bool {
        match self {
            WeightStorage::Uniform(w) => *w == T::ONE,
            WeightStorage::Individual(weights) => weights.iter().all(|&w| w == T::ONE),
        }
    }

    /// Uniform重みからIndividualに変換（指定したサイズで）
    #[must_use]
    pub fn to_individual(&self, num_points: usize) -> WeightStorage<T> {
        match self {
            WeightStorage::Uniform(w) => WeightStorage::Individual(vec![*w; num_points]),
            WeightStorage::Individual(_) => self.clone(),
        }
    }
}

impl<T: Scalar> Default for WeightStorage<T> {
    fn default() -> Self {
        WeightStorage::Uniform(T::ONE)
    }
}
