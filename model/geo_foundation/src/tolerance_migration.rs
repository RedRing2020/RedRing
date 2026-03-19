//! 段階的移行のための許容誤差デフォルト値提供（互換再エクスポート）
//!
//! 実体は `geo_contracts::tolerance_migration` に移設済み。

pub use geo_contracts::tolerance_migration::{DefaultTolerances, ScalarToleranceExt};

/// デフォルト許容誤差のアクセス用マクロ（互換維持）
#[macro_export]
macro_rules! default_distance_tolerance {
    ($t:ty) => {
        $crate::tolerance_migration::DefaultTolerances::distance::<$t>()
    };
}

/// デフォルト角度許容誤差のアクセス用マクロ（互換維持）
#[macro_export]
macro_rules! default_angle_tolerance {
    ($t:ty) => {
        $crate::tolerance_migration::DefaultTolerances::angle::<$t>()
    };
}
