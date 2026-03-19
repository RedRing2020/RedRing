//! Plane3D Core Traits - 互換再公開レイヤー
//!
//! 実体は geo_contracts に移設済み。後方互換のため再エクスポート。

pub use geo_contracts::geometry::core::plane3d_traits::{
    Plane3DConstructor, Plane3DCore, Plane3DMeasure, Plane3DProperties,
};
