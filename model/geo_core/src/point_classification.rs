//! 点の位置の分類

/// 領域を囲む形状に対する点の位置の分類
///
/// 境界までの距離が許容誤差以内なら `OnBoundary` とし、それ以外の点を境界の内側（`Inside`）と
/// 外側（`Outside`）に分ける。閉曲線では、曲線が平面上に囲む領域に対して分類する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointClassification {
    /// 境界の内側（境界までの距離が許容誤差を超える）
    Inside,
    /// 境界上（境界までの距離が許容誤差以内）
    OnBoundary,
    /// 境界の外側（境界までの距離が許容誤差を超える）
    Outside,
}
