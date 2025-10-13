//! 円弧（Arc）基本トレイト - Foundation統一システム参照
//!
//! 円弧の基本トレイト定義は Foundation統一システムに移行済み
//! このファイルは Foundation システムへの参照ブリッジとして機能

// Foundation統一システムから再エクスポート
pub use crate::abstract_types::foundation::{
    Arc3DCore,
    // Arc Foundation
    ArcCore,
    ArcMetrics,
    EllipseArc3DCore,
    // EllipseArc Foundation
    EllipseArcCore,
    EllipseArcMetrics,
    UnifiedArcFoundation,
    UnifiedEllipseArcFoundation,
};

// Legacy support - 旧式import用の再エクスポート
pub use ArcCore as ArcCoreLegacy;
pub use EllipseArcCore as EllipseArcCoreLegacy;

// Foundation統一システムに移行完了
// 旧式の実装は foundation/arc_extensions.rs に統合済み
