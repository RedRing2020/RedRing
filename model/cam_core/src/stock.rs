//! 素材の値

/// 製品形状を囲む箱から素材の箱の各面までの余裕（既定はすべて 0。製品形状を囲む箱そのもの）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StockMargins {
    /// −X 側
    pub minus_x: f64,
    /// +X 側
    pub plus_x: f64,
    /// −Y 側
    pub minus_y: f64,
    /// +Y 側
    pub plus_y: f64,
    /// 底（−Z 側）
    pub bottom: f64,
    /// 上（+Z 側）
    pub top: f64,
}
