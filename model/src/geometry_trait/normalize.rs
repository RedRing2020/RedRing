/// ベクトルを正規化（単位ベクトル化）するトレイト
pub trait Normalize {
    /// 正規化されたベクトルを返す
    fn normalize(&self) -> Self;
}
