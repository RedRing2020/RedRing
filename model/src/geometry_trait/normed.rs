/// ベクトルのノルム（長さ）を抽象化するトレイト
pub trait Normed {
    fn norm(&self) -> f64;
}
