/// 2Dベクトル専用のトレイト拡張

use super::vector::Vector;

/// 2Dベクトル専用のトレイト拡張
pub trait Vector2DExt: Vector<2> {
    /// 90度回転（反時計回り）
    fn perpendicular(&self) -> Self;

    /// 2D外積（スカラー値）
    fn cross_2d(&self, other: &Self) -> f64;
}