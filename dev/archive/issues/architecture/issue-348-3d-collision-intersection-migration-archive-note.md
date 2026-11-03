# Issue #348 実施準備チェックリスト

対象Issue: [#348 [Phase C][#347] 3D collision/intersection trait実装をgeo_algorithmsへ移管](https://github.com/RedRing2020/RedRing/issues/348)

関連Issue:
- #356 collision/intersection責務整理（Foundationパターン対象外の再定義）
- #357 intersection返り値意味論（空集合と重複の区別）

## 1. 目的

- 3D形状ペアの collision/intersection ロジックを `geo_algorithms` 側の正本に寄せる
- `geo_primitives` は形状コア実装へ責務を絞り、3Dの重複経路を段階削減する
- `geo_algorithms` 実装ファイルでは `use crate::...` の再エクスポート経由を維持する

## 2. 現状調査（2026-03-21）

- `model/geo_primitives/src/*_3d_collision.rs`: 20ファイル
- `model/geo_primitives/src/*_3d_intersection.rs`: 20ファイル
- `model/geo_algorithms/src/collision/primitive_3d.rs`: 既存あり（free-function受け皿）
- `model/geo_algorithms/src/intersection/primitive_3d.rs`: 既存あり（free-function受け皿）

### 2.1 実装制約の確認

- `BasicCollision` / `BasicIntersection` / `MultipleIntersection` は `geo_contracts` 側trait定義
- 3D形状型は `geo_primitives` 側型定義
- Rust orphan rules により、`geo_algorithms` で対象traitを対象型へ直接実装することはできない
- 依存方向は `geo_algorithms -> geo_primitives` であり、`geo_primitives -> geo_algorithms` は導入しない
- したがって #348 は「trait実装の物理移管」ではなく、`geo_algorithms` free-function / pair-base を正本として補完する段階実施が現実的

## 3. #348 の対象範囲

### 対象

- 3D形状ペアの collision/intersection 実装補完と正本化
- 代表候補:
  - `Arc3D`, `Circle3D`, `Ellipse3D`, `EllipseArc3D`
  - `Plane3D`, `LineSegment3D`, `Ray3D`, `InfiniteLine3D`
  - `Spherical*`, `Conical*`, `Cylindrical*`, `Ellipsoidal*`, `Torus*`
  - `Triangle3D`, `TriangleMesh3D`

### 非対象

- NURBS/Primitive 混在（#349）
- 旧実装の大規模削除と全回帰整備（#351）
- 返り値モデル再設計（#357）

## 4. 実施方針

- 先に `geo_algorithms` 側 3D entry point の欠落ペアを補完する
- `pair_base.rs` に寄せられる共通ロジックは抽出し、重複を減らす
- `geo_primitives` 側trait実装は即時削除せず、#351 で削減対象を明示して段階実施する
- 1PRでの全面移管は狙わず、小さな追加と検証を積み上げる

## 5. 推奨着手順

### Phase A: 棚卸し

- [ ] `primitive_3d.rs` で既存形状ペアと未整備ペアを一覧化
- [ ] `geo_primitives/src/*_3d_collision.rs` / `*_3d_intersection.rs` 側の3D trait実装を棚卸し
- [ ] orphan rules / 依存方向の制約を #348 の作業ログに明記

### Phase B: geo_algorithms 側実装補完

- [ ] `model/geo_algorithms/src/collision/primitive_3d.rs` の未整備 entry point を追加
- [ ] `model/geo_algorithms/src/intersection/primitive_3d.rs` の未整備 entry point を追加
- [ ] `pair_base.rs` へ寄せられる共通ロジックを抽出
- [ ] 実装ファイル内 import を `use crate::...` へ統一

### Phase C: 呼び出し整合

- [ ] `geo_primitives` 側に残す互換ラッパーと削除候補を切り分け
- [ ] #351 に回す削除対象を明文化

### Phase D: 検証

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy -p geo_algorithms -- -D warnings`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1 -ExitOnError`

## 6. 最初の実装単位（小さく始める）

- collision: 線形要素（`Plane3D`, `Ray3D`, `LineSegment3D`, `InfiniteLine3D`）の対称entry point不足を先に補完
- intersection: 既存 `pair_base` 委譲関数の対称ラッパーを先に揃える
- テスト: point系に加え、線形要素同士の回帰ケースを `primitive_3d.rs` に追加

## 7. 完了条件

- [ ] 3D主要ペアのロジック入口が `geo_algorithms` で把握可能になっている
- [ ] `geo_algorithms` の import/依存経路が安定している
- [ ] #351 に引き渡す削除候補が明確化されている
- [ ] `fmt/check/test/clippy` と依存チェックスクリプトが通る

## 8. 準備完了時点の判断

- 次の実装着手Issueは #348 を優先
- 実装ブランチは `issue-348-collision-3d-execution` を推奨
- 初回PRは「3D線形要素の対称entry point補完 + テスト追加」を最小スコープとする
