# Issue #350 実施準備チェックリスト

対象Issue: [#350 [Phase C][#347] 2D collision/intersection trait実装をgeo_algorithmsへ移管](https://github.com/RedRing2020/RedRing/issues/350)

関連設計論点: `dev/architecture/issues/2026-03-geometry-refactor-05-collision-intersection-responsibility.md`

## 1. 目的

- 2D 形状間の collision/intersection trait実装責務を `geo_algorithms` に集約する
- `geo_primitives` は形状コア実装に責務を絞り、複数形状演算の実装を段階的に削減する
- `geo_algorithms` 実装ファイルでは `use crate::...` の再エクスポート経由を維持する

## 2. 現状調査（2026-03-21）

- `model/geo_primitives/src/*_collision.rs`: 28ファイル
- `model/geo_primitives/src/*_intersection.rs`: 28ファイル
- `model/geo_algorithms/src/collision/primitive_2d.rs`: 既存あり
- `model/geo_algorithms/src/intersection/primitive_2d.rs`: 既存あり
- 親Issue #347 の分割Issueは #350（2D）, #348（3D）, #349（混在）, #351（削除・回帰）

### 2.2 進捗更新（2026-03-22）

- 実装ブランチ `issue-350-next` を作成し初回スライスを投入済み
- `geo_algorithms` 2D に対称 entry point を追加し、対称ラッパー一致性テストを追加
- `InfiniteLine2D` / `Ellipse2D` / `EllipseArc2D` について、2D entry point を追加し対称性を補完
- `geo_algorithms/src/lib.rs` の再エクスポートに `InfiniteLine2D` / `EllipseArc2D` を追加
- 検証結果: `cargo clippy -p geo_algorithms -- -D warnings` / `cargo fmt --all` / `cargo test -p geo_algorithms` 通過

### 2.1 実装制約の確認

- `BasicCollision` / `BasicIntersection` / `MultipleIntersection` は `geo_contracts` 側trait定義
- 2D形状型は `geo_primitives` 側型定義
- Rust の orphan rules により、`geo_algorithms` でこれらの trait を対象型へ直接実装することはできない
- さらに依存方向は `geo_algorithms -> geo_primitives` であり、`geo_primitives -> geo_algorithms` は導入できない
- したがって #350 の現実的な第一段階は、`geo_algorithms` に free-function / pair-base ロジックを集約し、`geo_primitives` 側trait実装の縮退候補を棚卸しすることになる

## 3. #350 の対象範囲

### 対象

- 2D形状ペアの collision/intersection 実装
- 代表候補:
  - `Arc2D`
  - `Circle2D`
  - `Ellipse2D`
  - `InfiniteLine2D`
  - `LineSegment2D`
  - `Ray2D`
  - `Triangle2D`

### 非対象

- 3D形状ペア移管（#348）
- NURBS/Primitive 混在移管（#349）
- 旧実装の大規模削除と回帰テストの総仕上げ（#351）

## 4. 実施方針

- 先に `geo_algorithms` 側の 2D free-function / pair-base 実装を充足させる
- trait実装そのものの物理移管ではなく、どのロジックを `geo_algorithms` 正本へ寄せられるかを優先して整理する
- `geo_primitives` 側trait実装は、削除可能になるまでの暫定ラッパーまたは旧経路として段階的に縮退させる
- 1PR で全削除までは狙わず、#350 では「ロジック集約先の明確化」と「削減対象の確定」を優先する

## 5. 推奨着手順

### Phase A: 棚卸し

- [x] `primitive_2d.rs` で既に扱っている形状ペアと未移管ペアを一覧化
- [x] `geo_primitives/src/*_collision.rs` / `*_intersection.rs` 側で 2D trait実装の所在を確認
- [x] orphan rules と依存方向の制約に照らして「直接移管不可」な点を明文化
- [x] テスト所在を確認し、移設が必要か参照維持で足りるか判断（`geo_algorithms` 側単体テストを拡充し、`geo_primitives` 側既存テストは #351 で段階縮退）

### Phase B: geo_algorithms 側実装補完

- [x] `model/geo_algorithms/src/collision/primitive_2d.rs` を補完（対称entry + InfiniteLine2D/Ellipse2D/EllipseArc2D）
- [x] `model/geo_algorithms/src/intersection/primitive_2d.rs` を補完（対称entry + InfiniteLine2D/Ellipse2D/EllipseArc2D）
- [ ] `pair_base.rs` に寄せられる共通ロジックを抽出
- [x] 必要な型再エクスポートがあれば `model/geo_algorithms/src/lib.rs` を更新
- [x] `geo_algorithms` 実装ファイル内の import は `use crate::...` に統一

### Phase C: 呼び出し整合

- [x] `geo_primitives` 側に残すべき trait実装ラッパーと shape-local helper を切り分け
- [x] `geo_algorithms` を正本ロジックとみなせる形状ペアを確定
- [x] #351 に回す削除候補を明文化

### Phase D: 検証

- [x] `cargo fmt --all -- --check`（`cargo fmt --all` 実行で整形済み）
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy -p geo_algorithms -- -D warnings`
- [x] `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1 -ExitOnError`

## 6. リスクと対策

- リスク: `geo_primitives` と `geo_algorithms` に同一 trait実装が併存し、衝突する
  - 対策: trait実装の直接移管は行わず、まず free-function / pair-base の正本化で整理する

- リスク: Issue 文言どおりに「trait実装を geo_algorithms へ移管」と解釈すると、orphan rules と依存方向に反する
  - 対策: #350 では「ロジック集約先の移管」と「trait実装縮退候補の整理」に読み替え、必要なら親Issue #347 の受け入れ条件を補足する

- リスク: `geo_algorithms` 実装側で `use geo_primitives::...` が再発する
  - 対策: `lib.rs` 再エクスポートを先に整え、実装ファイルは `use crate::...` のみ許可する

- リスク: 2D と 3D を同時に触って差分が肥大化する
  - 対策: #350 は 2D ペアに限定し、3D/混在は別Issueへ分離維持する

## 7. 完了条件

- [x] 2D collision/intersection trait実装の主要責務が `geo_algorithms` に寄る（対象7形状の代表ペアで free-function / pair-base 正本化を優先）
- [x] `geo_algorithms` の 2D 実装で必要な型・import 経路が安定する
- [x] #351 に送る削除対象が明確化される
- [x] `fmt/check/test/clippy` と依存チェックスクリプトが通る

## 8. 準備完了時点の判断

- 次の実装着手 Issue は #350 を優先する
- ブランチは `issue-350-execution-prep` を準備済み
- 実装開始ブランチは、この準備PRマージ後に `issue-350-collision-2d-execution` を推奨する

## 9. 初回棚卸しメモ

### 9.1 `geo_algorithms` 側で既にある 2D collision 関数

- `circle2d_point2d_collides`
- `circle2d_circle2d_collides`
- `line_segment2d_point2d_distance`
- `line_segment2d_circle2d_collides`
- `arc2d_point2d_collides`
- `arc2d_circle2d_collides`
- `ray2d_point2d_collides`
- `ray2d_circle2d_collides`
- `ray2d_line_segment2d_collides`
- `triangle2d_circle2d_collides`
- `triangle2d_triangle2d_collides`
- `arc2d_line_segment2d_collides`（2026-03-22 追加）
- `circle2d_ray2d_collides`（2026-03-22 追加）
- `line_segment2d_ray2d_collides`（2026-03-22 追加）
- `circle2d_triangle2d_collides`（2026-03-22 追加）
- `line_segment2d_triangle2d_collides`（2026-03-22 追加）
- `infinite_line2d_point2d_collides`（2026-03-22 追加）
- `infinite_line2d_circle2d_collides` / `circle2d_infinite_line2d_collides`（2026-03-22 追加）
- `infinite_line2d_line_segment2d_collides` / `line_segment2d_infinite_line2d_collides`（2026-03-22 追加）
- `infinite_line2d_ray2d_collides` / `ray2d_infinite_line2d_collides`（2026-03-22 追加）
- `ellipse2d_point2d_collides`（2026-03-22 追加）
- `ellipse2d_circle2d_collides` / `circle2d_ellipse2d_collides`（2026-03-22 追加）
- `ellipse_arc2d_point2d_collides`（2026-03-22 追加）
- `ellipse_arc2d_circle2d_collides` / `circle2d_ellipse_arc2d_collides`（2026-03-22 追加）

### 9.2 `geo_algorithms` 側で既にある 2D intersection 関数

- `circle2d_point2d_intersection`
- `circle2d_circle2d_intersections_algo`
- `circle2d_line_segment2d_intersections_algo`
- `arc2d_circle2d_intersections_algo`
- `line_segment2d_circle2d_intersections_algo`
- `line_segment2d_arc2d_intersections_algo`
- `line_segment2d_line_segment2d_intersection_algo`
- `ray2d_line_segment2d_intersection`
- `ray2d_circle2d_intersections`
- `triangle2d_line_segment2d_intersections`
- `ray2d_ellipse2d_intersection`
- `arc2d_point2d_intersection`
- `line_segment2d_ray2d_intersection`（2026-03-22 追加）
- `circle2d_ray2d_intersections`（2026-03-22 追加）
- `infinite_line2d_point2d_intersection`（2026-03-22 追加）
- `infinite_line2d_circle2d_intersection` / `circle2d_infinite_line2d_intersection`（2026-03-22 追加）
- `infinite_line2d_circle2d_intersections` / `circle2d_infinite_line2d_intersections`（2026-03-22 追加）
- `infinite_line2d_line_segment2d_intersection` / `line_segment2d_infinite_line2d_intersection`（2026-03-22 追加）
- `infinite_line2d_ray2d_intersection` / `ray2d_infinite_line2d_intersection`（2026-03-22 追加）
- `ellipse2d_point2d_intersection`（2026-03-22 追加）
- `ellipse2d_circle2d_intersection` / `circle2d_ellipse2d_intersection`（2026-03-22 追加）
- `ellipse2d_circle2d_intersections` / `circle2d_ellipse2d_intersections`（2026-03-22 追加）
- `ellipse_arc2d_point2d_intersection`（2026-03-22 追加）

### 9.3 `geo_primitives` 側に依然として残っている代表的な 2D trait実装

- `Circle2D`: Point / Circle / LineSegment
- `Arc2D`: Point / Circle
- `LineSegment2D`: Point / Circle / Arc / LineSegment
- `Ray2D`: Point / Circle / Arc / LineSegment / Triangle / Ellipse / Ray
- `InfiniteLine2D`: Point / Circle / Arc / LineSegment / Triangle / Ellipse / Ray / InfiniteLine
- `Triangle2D`: Point / Circle / Arc / LineSegment / Triangle
- `Ellipse2D`, `EllipseArc2D`: 2D複数形状ペア

この差から、#350 の最初の実装差分は `Circle2D` / `Arc2D` / `LineSegment2D` / `Ray2D` / `Triangle2D` 周辺を優先すると小さく始めやすい。

## 10. 実装スライス記録（2026-03-22）

- コミット: `c5cf280`
- 変更ファイル:
  - `model/geo_algorithms/src/collision/primitive_2d.rs`
  - `model/geo_algorithms/src/intersection/primitive_2d.rs`
- 概要:
  - 2D collision/intersection の対称 entry point 追加
  - 対称ラッパー一致性テスト追加
- 次アクション:
  - `geo_primitives` 2D の削減候補を #351 へ一次連携
  - 代表ペア以外（InfiniteLine2D / Ellipse2D / EllipseArc2D）の補完方針を確定

## 11. 実装スライス記録（2026-03-22 追補）

- 変更ファイル:
  - `model/geo_algorithms/src/collision/primitive_2d.rs`
  - `model/geo_algorithms/src/intersection/primitive_2d.rs`
  - `model/geo_algorithms/src/lib.rs`
- 概要:
  - `InfiniteLine2D` / `Ellipse2D` / `EllipseArc2D` の 2D collision/intersection entry point を追加
  - `EllipseArc2D` については `geo_primitives` 側の未公開 collision/intersection モジュールに依存せず、
    Core API（`contains_point`, `ellipse()`, `point_in_angle_range`, 端点判定）で判定を構成
  - 対称ラッパーとエントリポイントのテストを追加
- 検証:
  - `cargo clippy -p geo_algorithms -- -D warnings`: pass
  - `cargo fmt --all`: pass
  - `cargo test -p geo_algorithms`: pass
- 次アクション:
  - #351 へ 2D削減候補の詳細マッピング（関数対応表）を更新

## 12. #351 連携用 2D 削減候補対応表（一次）

### 12.1 残置ポリシー

- 残置対象: shape-local な補助計算や Core API 経由の判定（幾何プリミティブ固有実装）
- 縮退対象: 多形状ペアの trait実装本体（`BasicCollision` / `BasicIntersection` / `MultipleIntersection`）
- 実施単位: #351 で「wrapper化（`geo_algorithms` 呼び出し）」→ 回帰確認 → 段階削減

### 12.2 マッピング（代表ペア）

| geo_primitives 側（候補） | geo_algorithms 正本エントリ | #351 での扱い |
|---|---|---|
| `circle_2d_collision.rs` (`Circle2D-Point/Circle/LineSegment`) | `circle2d_point2d_collides`, `circle2d_circle2d_collides`, `line_segment2d_circle2d_collides` | wrapper化候補 |
| `circle_2d_intersection.rs` (`Circle2D-Point/Circle/LineSegment`) | `circle2d_point2d_intersection`, `circle2d_circle2d_intersections_algo`, `circle2d_line_segment2d_intersections_algo` | wrapper化候補 |
| `arc_2d_collision.rs` (`Arc2D-Point/Circle`) | `arc2d_point2d_collides`, `arc2d_circle2d_collides`, `circle2d_arc2d_collides` | wrapper化候補 |
| `arc_2d_intersection.rs` (`Arc2D-Point/Circle`) | `arc2d_point2d_intersection`, `arc2d_circle2d_intersections_algo`, `circle2d_arc2d_intersections_algo` | wrapper化候補 |
| `line_segment_2d_collision.rs` (`LineSegment2D-Circle/Arc/LineSegment`) | `line_segment2d_circle2d_collides`, `line_segment2d_arc2d_collides`, `arc2d_line_segment2d_collides` | wrapper化候補 |
| `line_segment_2d_intersection.rs` (`LineSegment2D-Circle/Arc/LineSegment`) | `line_segment2d_circle2d_intersections_algo`, `line_segment2d_arc2d_intersections_algo`, `line_segment2d_line_segment2d_intersection_algo` | wrapper化候補 |
| `ray_2d_collision.rs` (`Ray2D-Point/Circle/LineSegment`) | `ray2d_point2d_collides`, `ray2d_circle2d_collides`, `ray2d_line_segment2d_collides` | wrapper化候補 |
| `ray_2d_intersection.rs` (`Ray2D-Circle/LineSegment`) | `ray2d_circle2d_intersections`, `ray2d_line_segment2d_intersection` | wrapper化候補 |
| `triangle_2d_collision.rs` (`Triangle2D-Circle/LineSegment/Triangle`) | `triangle2d_circle2d_collides`, `triangle2d_line_segment2d_collides`, `triangle2d_triangle2d_collides` | wrapper化候補 |
| `triangle_2d_intersection.rs` (`Triangle2D-LineSegment`) | `triangle2d_line_segment2d_intersections` | wrapper化候補 |
| `infinite_line_2d_collision.rs` (`InfiniteLine2D-Point/Circle/LineSegment/Ray`) | `infinite_line2d_point2d_collides`, `infinite_line2d_circle2d_collides`, `infinite_line2d_line_segment2d_collides`, `infinite_line2d_ray2d_collides` | wrapper化候補 |
| `infinite_line_2d_intersection.rs` (`InfiniteLine2D-Point/Circle/LineSegment/Ray`) | `infinite_line2d_point2d_intersection`, `infinite_line2d_circle2d_intersection`, `infinite_line2d_circle2d_intersections`, `infinite_line2d_line_segment2d_intersection`, `infinite_line2d_ray2d_intersection` | wrapper化候補 |
| `ellipse_2d_collision.rs` (`Ellipse2D-Point/Circle`) | `ellipse2d_point2d_collides`, `ellipse2d_circle2d_collides`, `circle2d_ellipse2d_collides` | wrapper化候補 |
| `ellipse_2d_intersection.rs` (`Ellipse2D-Point/Circle`) | `ellipse2d_point2d_intersection`, `ellipse2d_circle2d_intersection`, `ellipse2d_circle2d_intersections` | wrapper化候補 |
| `ellipse_arc_2d_collision.rs` (`EllipseArc2D-Point/Circle`) | `ellipse_arc2d_point2d_collides`, `ellipse_arc2d_circle2d_collides`, `circle2d_ellipse_arc2d_collides` | wrapper化候補（未公開モジュール依存なしの現行実装に合わせる） |
| `ellipse_arc_2d_intersection.rs` (`EllipseArc2D-Point`) | `ellipse_arc2d_point2d_intersection` | wrapper化候補（Point判定のみ先行） |

### 12.3 #351 引き渡しメモ

- `geo_primitives/src/ellipse_arc_2d_collision.rs` / `geo_primitives/src/ellipse_arc_2d_intersection.rs` は `lib.rs` 未公開設定との整合を確認してから削減。
- `Arc2D` / `Triangle2D` / `Ellipse2D` の複数交点系で `pair_base` 抽出余地あり（#351 実施時に再確認）。

## 13. 実装スライス記録（2026-03-22 追補2）

- 変更ファイル:
  - `model/geo_algorithms/src/collision/pair_base.rs`
  - `model/geo_algorithms/src/collision/primitive_2d.rs`
- 概要:
  - 2D の `Ray2D-LineSegment2D` / `InfiniteLine2D-LineSegment2D` / `InfiniteLine2D-Ray2D` について、
    collision 判定を intersection 正本（`intersection::primitive_2d`）へ委譲する pair-base 関数を追加。
  - `primitive_2d` 側の該当 collision entry point を pair-base 呼び出しへ切り替え、
    endpoint 偏重の判定経路を削減。
  - 中点交差や ray-origin 以外での交差を検出する回帰テストを追加。
- 検証:
  - `cargo clippy -- -D warnings`: pass
  - `cargo fmt --all`: pass
  - `cargo check --workspace`: pass
  - `cargo test --workspace`: pass
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies_simple.ps1`: pass
- #350 観点の更新:
  - `pair_base.rs` への 2D 共通ロジック抽出を一部実施（線分・直線・半直線系）。
  - 残る抽出余地は曲線ペア系（Arc/Ellipse/EllipseArc）中心で、#351 側で段階整理可能。