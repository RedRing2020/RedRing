# Issue #350 実施準備チェックリスト

対象Issue: [#350 [Phase C][#347] 2D collision/intersection trait実装をgeo_algorithmsへ移管](https://github.com/RedRing2020/RedRing/issues/350)

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

- [ ] `primitive_2d.rs` で既に扱っている形状ペアと未移管ペアを一覧化
- [ ] `geo_primitives/src/*_collision.rs` / `*_intersection.rs` 側で 2D trait実装の所在を確認
- [ ] orphan rules と依存方向の制約に照らして「直接移管不可」な点を明文化
- [ ] テスト所在を確認し、移設が必要か参照維持で足りるか判断

### Phase B: geo_algorithms 側実装補完

- [ ] `model/geo_algorithms/src/collision/primitive_2d.rs` を補完
- [ ] `model/geo_algorithms/src/intersection/primitive_2d.rs` を補完
- [ ] `pair_base.rs` に寄せられる共通ロジックを抽出
- [ ] 必要な型再エクスポートがあれば `model/geo_algorithms/src/lib.rs` を更新
- [ ] `geo_algorithms` 実装ファイル内の import は `use crate::...` に統一

### Phase C: 呼び出し整合

- [ ] `geo_primitives` 側に残すべき trait実装ラッパーと shape-local helper を切り分け
- [ ] `geo_algorithms` を正本ロジックとみなせる形状ペアを確定
- [ ] #351 に回す削除候補を明文化

### Phase D: 検証

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy -p geo_algorithms -- -D warnings`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1 -ExitOnError`

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

- [ ] 2D collision/intersection trait実装の主要責務が `geo_algorithms` に寄る
- [ ] `geo_algorithms` の 2D 実装で必要な型・import 経路が安定する
- [ ] #351 に送る削除対象が明確化される
- [ ] `fmt/check/test/clippy` と依存チェックスクリプトが通る

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

### 9.3 `geo_primitives` 側に依然として残っている代表的な 2D trait実装

- `Circle2D`: Point / Circle / LineSegment
- `Arc2D`: Point / Circle
- `LineSegment2D`: Point / Circle / Arc / LineSegment
- `Ray2D`: Point / Circle / Arc / LineSegment / Triangle / Ellipse / Ray
- `InfiniteLine2D`: Point / Circle / Arc / LineSegment / Triangle / Ellipse / Ray / InfiniteLine
- `Triangle2D`: Point / Circle / Arc / LineSegment / Triangle
- `Ellipse2D`, `EllipseArc2D`: 2D複数形状ペア

この差から、#350 の最初の実装差分は `Circle2D` / `Arc2D` / `LineSegment2D` / `Ray2D` / `Triangle2D` 周辺を優先すると小さく始めやすい。