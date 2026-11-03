# ISSUE #397 実装準備

対象Issue: [#397 [Phase C][#347] 2D Foundation Integration 残骸整理と旧参照コメント清掃](https://github.com/RedRing2020/RedRing/issues/397)

## 背景

Phase C の主要移管は develop に反映済みだが、`geo_primitives` 側に旧 Foundation Integration 由来の 2D API と、削除済み旧ファイルを示すコメントが残っている。

本Issueでは、`collision/intersection` の正本が `geo_algorithms` にある現状態と整合しない残骸を削除・更新する。

## 対象ファイル

- `model/geo_primitives/src/circle_2d_extensions.rs`
- `model/geo_primitives/src/ellipse_2d_extensions.rs`
- `model/geo_primitives/src/circle_2d_tests.rs`
- `model/geo_primitives/src/ellipse_2d_tests.rs`
- `model/geo_primitives/src/ellipsoidal_surface_3d_extensions.rs`
- `model/geo_algorithms/src/intersection/mod.rs`

## 実施方針

1. `circle_2d_extensions.rs` から `foundation_resolve_collision` / `foundation_weighted_center` を削除する
2. `ellipse_2d_extensions.rs` から `foundation_resolve_collision` / `foundation_weighted_center` を削除する
3. 未接続テストソースに残る上記旧API参照と、存在しない `foundation_intersection` 参照を整理する
4. `ellipsoidal_surface_3d_extensions.rs` の削除済みファイル参照コメントを、現行責務に沿う説明へ更新する
5. `geo_algorithms/src/intersection/mod.rs` の段階移行コメントを現状に合わせて更新する
6. 検索、clippy、fmt、test で確認する

## 変更しないもの

- `foundation_scale_from_point` など Transform 系の補助 API
- 3D/NURBS collision/intersection の実装ロジック
- 新しい collision/intersection API の追加

## 完了条件

- 旧 Foundation Integration 由来の collision/intersection 補助 API が削除される
- 削除済み旧ファイル名を指すコメントが残らない
- `cargo clippy -- -D warnings` / `cargo fmt --all` / `cargo test --workspace` が通る
