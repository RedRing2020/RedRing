use geo_contracts::Scalar;

/// topology 判定で外部から与える公開設定。
///
/// B-rep の表の全体の許容誤差（頂点・稜線の許容誤差の下限）を与える。頂点はループの中で番号で共有するため、
/// 頂点どうしの離れを測る距離は持たない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologyToleranceSettings<T: Scalar> {
    /// B-rep の表の全体の許容誤差（頂点・稜線の許容誤差の下限。確かめは、頂点・稜線ごとの許容誤差で行う）。
    pub distance_tolerance: T,
}

impl<T: Scalar> TopologyToleranceSettings<T> {
    /// 代表距離トレランスから公開設定を生成する。
    pub fn new(distance_tolerance: T) -> Self {
        Self { distance_tolerance }
    }
}
