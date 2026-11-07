//! Data Converter クレート
//!
//! 形状・メッシュ・工具経路などのデータを、描画用の形式に変換します。
//!
//! ## 主要機能
//! - 製品形状の表示用のメッシュ（Application の製品形状 → GPU 頂点データ）
//! - デバッグ用の形状（線分・円・円弧・三角形のエンティティ → GPU 頂点データ）
//! - CAM工具経路の可視化データ変換（ToolPath → GPU頂点データ）
//! - 切削シミュレーションの再生（時点の素材の面・工具の形 → GPU 頂点データ）
//! - 逆オフセット形状の表示（表示用の三角形 → GPU 頂点データ）

pub mod cam_sim_visualization_converter;
pub mod cutting_animation;
pub mod cutting_simulation_session;
pub mod entity_converter;
pub mod failure_message_catalog;
pub mod failure_message_mapper;
pub mod feature_command_converter;
pub mod inverse_offset_shape_converter;
pub mod mesh_converter;
pub mod message_catalog;
pub mod product_shape_converter;
pub mod shape_converter;
pub mod snapshot_converter;
pub mod space_model_document;
pub mod toolpath_converter;

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
