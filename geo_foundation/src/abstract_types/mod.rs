//! 移行互換モジュール - 新実装アーキテクチャへの互換性ブリッジ
//!
//! 新しい構造への完全移行完了:
//! - `traits/` - Foundation操作トレイト群
//! - `abstracts/` - 最小責務抽象化
//! - virtual `geometry/` - backward compatibility

// 下位互換性のための再エクスポート
pub mod abstracts {
    pub use crate::abstracts::*;
}

// Virtual foundation module for compatibility
pub mod foundation {
    // 高次操作トレイト - traits/ から再エクスポート
    pub use crate::traits::{
        AdvancedCollision, AdvancedTransform, BasicCollision, BasicIntersection, BasicTransform,
        CollisionHelpers, MultipleIntersection, PointDistance, PointDistanceHelpers,
        SelfIntersection, TransformHelpers,
    };

    // Core Foundation - src直下から再エクスポート
    pub use crate::core_foundation::*;

    // Extension Foundation - src直下から再エクスポート
    pub use crate::extension_foundation::*;

    // 形状特化トレイト - abstracts/ から再エクスポート
    pub use crate::abstracts::circle_core::*;
}

// Virtual geometry module for backward compatibility
pub mod geometry {
    pub mod classification {
        pub use crate::classification::*;
    }
}

// Re-exports for backward compatibility
pub use crate::abstracts::arc_traits::Arc2D;
pub use foundation::{
    AdvancedCollision, AdvancedTransform, BasicCollision, BasicIntersection, BasicTransform,
    MultipleIntersection, PointDistance, SelfIntersection, TransformHelpers,
};
