//! 計算の区切りごとの記録（止まった計算を、続きから計算するため）
//!
//! 計算は、区切りを終えるたびに、その区切りで増えた成果と、続けるための状態をバイト列で出す。何を区切りにし、状態と
//! 成果に何を入れるかは、計算だけが決める。記録を書き残す側は、バイト列の中身を見ずに、区切りの順に書き足して読み
//! 戻す。

/// 前の実行の、区切り 1 つの記録
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedRecord {
    /// 続けるための状態
    pub state: Vec<u8>,
    /// その区切りで増えた成果
    pub result: Vec<u8>,
}

/// 区切りの記録の読み書き
pub trait Checkpoints {
    /// 前の実行の記録を、区切りの順に 1 つずつ返す（もうなければ `None`）
    fn next_saved(&mut self) -> Option<SavedRecord>;

    /// 前の実行の記録を使えない（理由 `reason`。英語）。記録をすべて捨てる（計算は最初からやり直す）
    fn discard(&mut self, reason: &str);

    /// 区切りを終えた。続けるための状態 `state` と、その区切りで増えた成果 `result` を書き足す
    fn record(&mut self, state: &[u8], result: &[u8]);
}
