//! cam_sim の数値計算の定数

/// トリデクセルの 3 方向の格子の線の本数の合計の上限（超える精度の入力は実行しない）
pub(crate) const MAX_DEXEL_LINES: usize = 4_000_000;

/// 既定のスナップショットの間隔の、素材の格子の間隔に対する倍率（工具の半径のほうが短ければ工具の半径）
pub(crate) const SNAPSHOT_GRID_SPACING_MULTIPLE: f64 = 4.0;

/// 表示用の面を作り直す単位の、ブロックの各軸のセルの数
pub(crate) const SURFACE_BLOCK_CELLS: usize = 32;
