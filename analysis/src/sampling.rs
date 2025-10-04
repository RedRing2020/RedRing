//! サンプリング/交差候補機能は geo_algorithms::sampling に移動。
//! 移行期間中の再エクスポート層。

pub use geo_algorithms::{
    SamplingResult,
    QualityMetrics,
    IntersectionCandidate,
    AdaptiveSampler,
    PoissonDiskSampler,
};

