# Geometry 抽象レイヤ README (日本語)

本書は旧 `model::geometry` (geometry2d / geometry3d) 実装を削除した後、`model` クレートが担う **抽象化レイヤ** の目的と使い方を日本語で整理したものです。

## 現在の役割まとめ

`model` クレートは以下の “抽象インターフェース” のみを保持します。

- `geometry_trait/` : `Curve2D`, `Curve3D`, （将来: `Surface` など）
- `geometry_kind/` : 幾何種別列挙 (`CurveKind2D`, `CurveKind3D`, `SurfaceKind` など)
- `geometry_common/` : 共通補助（必要最小限）

実データ型 (Point, Vector, Line, Circle, Arc など) は **`geo_primitives`** に集約され、数値的基盤 (Scalar, ToleranceContext, ベクトル演算ロジック) は **`geo_core`** にあります。

```
redring ─┬─ viewmodel ── model(抽象) ──┐
         │                             │
         └──────── stage ── render      │
                                       ├─ geo_primitives (幾何プリミティブ)
                                       └─ geo_core (低レベル数値/トレイト)
```

## 旧 geometry 削除の背景

| 課題       | 詳細                                                | 対応                         |
| ---------- | --------------------------------------------------- | ---------------------------- |
| 重複定義   | Point/Vector/曲線が複数箇所で二重管理               | geo_primitives に集約        |
| 保守コスト | 旧実装 vs 新実装の平行メンテ                        | 旧ディレクトリ完全削除       |
| 依存混線   | model → (独自実装) + geo_core/geo_primitives の二重 | 抽象のみ残し参照方向を単純化 |

## `Curve2D` / `Curve3D` の新しい形

旧実装では `model::geometry::geometry2d::Point` のような具体型に直接依存していました。現在は **関連型 (associated types)** を用いて具象を外部で差し替える構造です。

```rust
use model::geometry_trait::{Curve2D, Curve3D};
use geo_foundation::{Point2D, Vector2D, Point3D, Vector3D};  // CI/CD compliance

struct MyCurve2D { /* ... */ }

impl Curve2D for MyCurve2D {
    type Point = Point2D;
    type Vector = Vector2D;

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn kind(&self) -> model::geometry_kind::CurveKind2D { model::geometry_kind::CurveKind2D::Line }
    fn evaluate(&self, t: f64) -> Self::Point { /* ... */ unimplemented!() }
    fn derivative(&self, t: f64) -> Self::Vector { /* ... */ unimplemented!() }
    fn length(&self) -> f64 { /* ... */ 0.0 }
    // parameter_hint / domain はデフォルト利用可
}
```

### 利点

- 具象型差し替え (将来: SIMD / f32 専用型 など) が簡単
- モック実装やテストダブルの注入が容易
- 移行 (migration) 時に抽象 API を固定し内部実装を並行開発可能

## 旧 → 新 マッピング早見表

| 旧 `model::geometry`                                             | 新しい参照先                           |
| ---------------------------------------------------------------- | -------------------------------------- |
| geometry2d::Point / Vector                                       | `geo_foundation::Point2D` / `Vector2D` |
| geometry2d::Line / Circle / Arc / Ellipse / Ray / InfiniteLine   | `geo_foundation` 2D 群                 |
| geometry3d::Point / Vector                                       | `geo_foundation::Point3D` / `Vector3D` |
| geometry3d::Line / Circle / Arc / Ellipse / Plane / InfiniteLine | `geo_foundation` 3D 群                 |
| NurbsCurve / NurbsSurface (stub)                                 | 今後 `geo_foundation` に正式移行予定   |
| SimpleAdaptedLine adapter                                        | 廃止 (履歴のみ)                        |

## 今後の拡張予定 (案)

1. NURBS 評価本実装 (2D/3D 曲線・サーフェス) を `geo_foundation` に統合 → 抽象 `Curve*` / `Surface*` 実装例追加
2. `Surface` トレイトの導入: `evaluate(u,v)`, `normal(u,v)`, `is_closed_{u,v}` など
3. Tolerance 戦略の抽象インターフェース化 (今は `geo_foundation::ToleranceContext` を直参照)
4. Parametric Domain 拡張: 曲線の `domain()` を閉区間以外 (周期関数/ラップ) に拡張するためのフラグ付加
5. エラーハンドリング: `evaluate` が失敗し得る場合に `Result` 版 API を並行提供

## 実装ポリシー

| 項目      | 指針                                                                                |
| --------- | ----------------------------------------------------------------------------------- |
| 精度      | `Scalar(f64)` 基本。要求が出たら f32 版を関連型差し替えで導入。                     |
| 正規性    | ベクトル正規化は `geo_core` のトレイトを利用しゼロ除算対策一元化。                  |
| Tolerance | 幾何的比較は必ず `ToleranceContext` を受け取る or 既定値を内部で取得。              |
| エラー    | パニックより `Option` / `Result` 優先。境界パラメータはクランプしない（責務分離）。 |
| 拡張性    | 新しい曲線型は `CurveKind2D/3D` に列挙子追加し影響を明示。                          |

## 典型的な移行手順 (既存コードを新層へ移す場合)

1. 旧 `model::geometry` 型 import を `geo_foundation::*` に切り替え
2. `impl Curve2D for X` へ関連型指定を追加
3. 固有の内部数値計算は `geo_core` のベクトル/スカラー演算に置換
4. テスト: 幾何単体テストは `geo_foundation` サイドへ移譲
5. ドキュメント: 移行履歴を `GEOMETRY_REMOVAL.md` に追記

## よくある質問 (FAQ)

**Q. 旧 geometry ディレクトリを復活させる必要は？**
A. 想定なし。PoC 的 adapter が必要なら独立クレート/モジュールで局所的に作成してください。

**Q. 抽象トレイトにデフォルトメソッドを足しても安全？**
A. 既存実装が（まだ）少ないため後方互換リスクは低い。必要なら `#[default_method_body_is_ok]` 的コメントで意図を明示。

**Q. 今後の型互換層（adapter）は？**
A. 外部インポートが増えるまでは不要。要件発生時に最小単位で再設計。

## 参考ファイル

- `GEOMETRY_REMOVAL.md`: 削除の英語履歴 (史料)
- `geometry_trait/curve2d.rs`, `curve3d.rs`: 関連型化後の最新インターフェース

---

本 README に対する改善や追記は Issue / PR で提案してください。
