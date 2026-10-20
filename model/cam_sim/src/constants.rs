//! cam_sim の数値計算の定数

/// 素材の Octree の分割の深さの上限
///
/// 現在の Octree（状態のみを持つボクセル）で、メモリと時間を実用の範囲に収めるための上限。
/// 精度から求めた深さがこれを超える入力は、Preflight 判定で実行不可とする。
pub(crate) const STOCK_OCTREE_MAX_DEPTH: usize = 10;
