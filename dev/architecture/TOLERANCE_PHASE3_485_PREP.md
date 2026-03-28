# #485 着手準備メモ（Phase 3）

最終更新: 2026-03-29
対象Issue: #485
親Issue: #455

## 目的

`geo_primitives` / `geo_nurbs` に残るトレランス参照の未整合箇所を棚卸しし、
単一正本（`geo_contracts::ToleranceSettings`）へ段階移行する実行計画を固定する。

## 前提ルール

- 正本は `geo_contracts::ToleranceSettings`。
- 幾何意味判定で `T::EPSILON` を既定値として使わない。
- 呼び出し境界で取得した `tolerance` を下位へ明示伝播する。
- 数値安定化のゼロ判定は引数トレランスに依存させず、カーネル固定しきい値を使う。

## 棚卸し結果（要約）

### 1. geo_primitives: 高優先（幾何意味判定に直接関与）

- `model/geo_primitives/src/arc_2d.rs`
  - `is_full_circle` / 半円判定 / 点包含判定で `T::EPSILON` を直接使用。
- `model/geo_primitives/src/arc_2d_extensions.rs`
  - `is_degenerate` / 角度範囲判定で `default_distance_tolerance::<T>()` と `T::EPSILON` が混在。
- `model/geo_primitives/src/arc_3d.rs`
  - 円弧判定・角度判定・交差補助で `T::EPSILON` を直接使用。
- `model/geo_primitives/src/circle_2d.rs`, `model/geo_primitives/src/circle_3d.rs`
  - 退化判定、点一致判定、近傍判定に `T::EPSILON` を直接使用。
- `model/geo_primitives/src/infinite_line_2d.rs`, `model/geo_primitives/src/infinite_line_3d.rs`
  - contains/coplanar/parallel 系で `default_distance_tolerance::<T>()` 依存が残存。
- `model/geo_primitives/src/ray_2d.rs`, `model/geo_primitives/src/ray_3d.rs`
  - 交差・方向判定に `default_distance_tolerance::<T>()` 依存が残存。

### 2. geo_nurbs: 中優先（変換/評価の安定化との境界整理が必要）

- `model/geo_nurbs/src/curve_2d_transform.rs`
- `model/geo_nurbs/src/curve_3d_transform.rs`
- `model/geo_nurbs/src/surface_3d_transform.rs`
  - ゼロ除算回避や軸退化判定に `T::EPSILON` を使用。
- `model/geo_nurbs/src/curve_3d_extensions.rs`
  - `tolerance.max(T::EPSILON)` があり、幾何意味判定との境界整理が必要。
- `model/geo_nurbs/src/curve_3d_foundation.rs`
  - `1e-6` の生数値が残存。

### 3. テスト側の生数値（低優先だが追従必要）

- `model/geo_primitives/src/*_tests.rs` に `1e-6`, `1e-9` が散在。
- `model/geo_primitives/src/rectangle_3d.rs` のテストに `1e-9`。

## 提案する小PR分割

### PR-A（最優先）

対象:
- `arc_2d.rs`, `arc_2d_extensions.rs`, `arc_3d.rs`

作業:
- 幾何意味判定の `T::EPSILON` を設定トレランス（`ToleranceSettings`）へ置換。
- ゼロベクトル長/分母ゼロ近傍の判定は、用途別の固定しきい値へ置換。
- 既存 public API を壊さない範囲で、内部ヘルパーに `tolerance` を伝播。

完了条件:
- 退化/角度範囲/全周判定の既存テストが通過。

### PR-B

対象:
- `circle_2d.rs`, `circle_3d.rs`, `infinite_line_2d.rs`, `infinite_line_3d.rs`, `ray_2d.rs`, `ray_3d.rs`

作業:
- `default_distance_tolerance::<T>()` 依存を呼び出し境界へ寄せる。
- `T::EPSILON` は数値安定化用途へ限定し、可能な限り意味付き固定定数へ置換。

完了条件:
- line/circle/ray の包含・平行・交点系テストが通過。

### PR-C

対象:
- `geo_nurbs` 変換・拡張系（`*_transform.rs`, `curve_3d_extensions.rs`, `curve_3d_foundation.rs`）

作業:
- `T::EPSILON` の用途を「数値安定化」に明示し、幾何意味判定の閾値は呼び出し側由来へ整理。
- 生数値 `1e-6` を意味付き定数化。

完了条件:
- `geo_nurbs` の既存テスト通過、変換系の退化ケース回帰なし。

### PR-D（追従）

対象:
- `geo_primitives`/`geo_nurbs` テストの生数値整理

作業:
- 通常ケースは共有定数、境界ケースは意味付きローカル定数へ置換。

完了条件:
- 生数値直書きが原則排除され、例外は理由コメントあり。

## 実行コマンド（各PR共通）

- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

## リスクと対策

- リスク: `T::EPSILON` が数値安定化と幾何意味判定で混在している箇所の切り分け漏れ。
- 対策: PRごとに「幾何意味判定」「数値安定化」の2分類をレビュー項目として明示する。

- リスク: 数値安定化ガードをアプリケーション設定トレランスで駆動し、設定値変更で特異点判定が崩れる。
- 対策: 固定しきい値を `foundation/analysis/src/consts.rs` に追加し、用途別定数で置換する。

- リスク: APIシグネチャ変更が広範囲へ波及。
- 対策: まず内部ヘルパーから伝播し、公開API変更は最小化する。

## 次アクション（実装前）

1. #485 に本メモ要約と PR-A 着手範囲をコメント。
2. PR-A 用ブランチを作成。
3. `arc_2d*` / `arc_3d*` から着手。