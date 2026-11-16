use geo_contracts::Scalar;

/// topology 判定で外部から与える公開設定。
///
/// 稜線の端の点と頂点の離れ、パラメータ曲線を曲面の上で評価した点と稜線の点の離れを、この距離で確かめる。
/// 頂点はループの中で番号で共有するため、頂点どうしの離れを測る距離は持たない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologyToleranceSettings<T: Scalar> {
    /// topology 判定全体の代表距離トレランス。
    pub distance_tolerance: T,
}

impl<T: Scalar> TopologyToleranceSettings<T> {
    /// 代表距離トレランスから公開設定を生成する。
    pub fn new(distance_tolerance: T) -> Self {
        Self { distance_tolerance }
    }
}
