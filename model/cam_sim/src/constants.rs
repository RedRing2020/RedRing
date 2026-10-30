//! cam_sim の数値計算の定数

/// 表示用のトリデクセルで、素材の最大の辺を分ける格子線の本数
pub(crate) const TRI_DEXEL_DISPLAY_LINES: usize = 512;

/// 既定のスナップショットの間隔の、素材の格子の間隔に対する倍率（工具の半径のほうが短ければ工具の半径）
pub(crate) const SNAPSHOT_GRID_SPACING_MULTIPLE: f64 = 4.0;

/// 表示用の面を作り直す単位の、ブロックの各軸のセルの数
pub(crate) const SURFACE_BLOCK_CELLS: usize = 32;
