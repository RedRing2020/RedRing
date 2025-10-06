/// 3Dベクトル専用のトレイト拡張

use super::vector::Vector;

/// 3Dベクトル専用のトレイト拡張
pub trait Vector3DExt: Vector<3> {
    /// 外積
    fn cross(&self, other: &Self) -> Self;

    /// 2Dへの投影
    fn to_2d_xy(&self) -> crate::geometry2d::Vector2D;
    fn to_2d_xz(&self) -> crate::geometry2d::Vector2D;
    fn to_2d_yz(&self) -> crate::geometry2d::Vector2D;
}