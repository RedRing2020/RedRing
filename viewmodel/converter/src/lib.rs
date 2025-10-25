//! Data Converter クレート（ViewModel層）
//!
//! MVVMアーキテクチャにおけるViewModel層のデータ変換機能を提供します。
//! Model層（geo_*）からのデータを受け取り、
//! View層（render）で使用可能な形式に変換することに特化しています。
//!
//! ## 主要機能
//! - メッシュデータ変換（Model → GPU形式）
//! - STL読み込み・変換統合
//! - 境界ボックス計算・変換

pub mod mesh_converter;
pub mod stl_loader;

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
