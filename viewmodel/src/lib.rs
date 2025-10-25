//! ViewModelクレート
//!
//! MVVMアーキテクチャにおけるViewModel層を提供します。
//! Modelクレート（geo_*）からのデータを受け取り、
//! View層（render）で使用可能な形式に変換します。

pub mod mesh_converter;

/// テスト用の関数（削除予定）
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
