//! RedRing 幾何計算中核 (Geometry Computational Core)
//!
//! このクレートは **許容誤差 (tolerance)** と **ロバスト幾何判定** を中心とした
//! 最小構成の計算インフラを提供します。形状プリミティブ（線分 / 平面 / 円など）の
//! 正準定義は `geo_primitives` クレートへ段階的に移管され、`geo_core` は
//! 「数値安定性・比較ロジック・低レベル補助」のみに収束していきます。
//!
//! ## 役割境界 (Target Architecture)
//! ```text
//! +----------------------+  上位アルゴリズム / 解析 (model, analysis)
//! |   model / analysis   |
//! +-----------+----------+
//!             |
//!             v
//! +----------------------+  f64 ベース幾何プリミティブ (点 / ベクトル / 方向 / 面 / 曲線 ...)
//! |    geo_primitives    |
//! +-----------+----------+
//!             |
//!             v
//! +----------------------+  許容誤差 / ロバスト判定 / 基本数値ユーティリティ
//! |       geo_core        |
//! +----------------------+
//! ```
//!
//! ## 今後の縮小計画 (Roadmap Snapshot)
//! - [Phase] f64 正準プリミティブ: `geo_primitives::f64geom::*` に集約 (進行中)
//! - [Phase] 旧 3D プリミティブ (`legacy-primitives3d` feature) の段階的削除
//! - [Planned] Point/Vector の最終配置評価: `geo_core` に残すか `geo_primitives` へ移すかを
//!   trait ベース抽象 (`Vec3Like`, `Point3Like`) の試作結果で判断
//! - [Planned] ロバスト演算 (orientation / intersection 判定) をジェネリック化し
//!   f64 プリミティブ型と直接連携可能に (循環依存は避け、実装は下流で `impl`)
//!
//! ## 使用指針
//! - 幾何“型”が欲しい場合: `geo_primitives` を参照してください
//! - 数値誤差許容・比較・ロバスト検出: 本クレート (`ToleranceContext`, `TolerantEq`, `Orientation` 等)
//! - 移行期間中のレガシー 3D 型利用: `--features legacy-primitives3d` を付与 (近い将来削除予定)
//!
//! ## Deprecation Policy (概要)
//! 1. f64 正準型公開 → 旧型は feature gate + deprecated
//! 2. alias 段階 (旧名 = type alias 新実装) で警告誘導
//! 3. gate 削除 & 旧名 removal (BREAKING)
//!
//! 詳細な移行履歴はワークスペースルートの `MIGRATION_VECTOR_F64.md` を参照してください。
//!
//! ---
//! (c) RedRing Project

pub mod scalar;
pub mod vector;
pub mod tolerance;
pub mod robust;
pub mod primitives;
pub mod primitives2d;
// 3D primitives (deprecated) are now optional: enable with feature "legacy-primitives3d"
#[cfg(feature = "legacy-primitives3d")]
pub mod primitives3d;
pub mod point3d; // always-on Point3D

// テストモジュール
#[cfg(test)]
mod unit_tests;

// 主要な型の再エクスポート
pub use tolerance::{ToleranceContext, TolerantEq, TolerantOrd};
pub use scalar::Scalar;
pub use vector::{Vector, Vector2D, Vector3D};
// Direction3D is deprecated in core; only export when legacy 3D primitives are enabled
#[cfg(feature = "legacy-primitives3d")]
pub use vector::Direction3D;
pub use primitives::{
    Point2D, LineSegment2D,
    Arc2D, Polygon2D,
    ParametricCurve2D,
};
pub use point3d::Point3D;

// Re-export deprecated 3D items only when legacy feature is on
#[cfg(feature = "legacy-primitives3d")]
pub use primitives3d::{
    LineSegment3D, Plane, Sphere, ParametricCurve3D, ParametricSurface
};
pub use robust::{Orientation, RobustSolver};

/// 標準的な許容誤差コンテキスト
pub const DEFAULT_TOLERANCE: ToleranceContext = ToleranceContext {
    linear: 1e-6,      // 1マイクロメートル
    angular: 1e-8,     // 約0.0057度
    parametric: 1e-10, // パラメータ空間
    curvature: 1e-3,   // 曲率許容誤差
    area: 1e-12,       // 面積許容誤差
    volume: 1e-18,     // 体積許容誤差
};

/// プリファクトリ：よく使用される値の作成
pub mod prelude {
    pub use crate::{
        Point2D, Point3D, Vector2D, Vector3D,
        ToleranceContext, TolerantEq, TolerantOrd,
        DEFAULT_TOLERANCE,
    };
    #[cfg(feature = "legacy-primitives3d")]
    pub use crate::Direction3D;
    #[cfg(feature = "legacy-primitives3d")]
    pub use crate::{LineSegment3D, Plane, Sphere, ParametricCurve3D, ParametricSurface};
}


