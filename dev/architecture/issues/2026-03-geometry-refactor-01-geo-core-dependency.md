## 概要

`geo_core` を最下層クレートとして再定義するため、`geo_core -> geo_foundation` 依存を解消する。

## 背景

- 現状は `geo_core` が `geo_foundation` に依存しており、レイヤーが逆転している
- 今後の `geo_foundation` 廃止計画を成立させるために先行解消が必要

## 現在地

- `model/geo_core/Cargo.toml` から `geo_foundation` 依存は削除済み
- `geo_core` の依存先は実質 `analysis` と数値ユーティリティのみに整理済み
- Transform / AABB / Vector trait の `geo_core` 側再配置は完了済み
- 後続 Issue の前提としては完了しており、未反映なのは workspace 全体 `cargo test` の実績記録のみ

## タスク

- [x] `model/geo_core/Cargo.toml` から `geo_foundation` 依存を削除
- [x] `geo_core` が必要としている trait/型を再配置（`geo_core` or 呼び出し側）
- [x] 依存チェックスクリプトの許可ルールを最終形へ更新
- [x] 影響クレート（`geo_primitives`, `geo_nurbs`, `geo_algorithms`）の import を修正

段階実施:

- [x] Phase A: 型依存（`Scalar`, `Angle`, `TolerantEq`）を `analysis` 直参照へ置換
- [x] Phase B: Transform/AABB/Vector trait を `geo_core` 側へ再配置
- [x] Phase C: `geo_foundation` 依存削除 + 依存チェックスクリプト更新

詳細チェックリスト:

- `dev/archive/issues/architecture/issue-317-geo-core-dependency-inversion-archive-note.md`

## 受け入れ条件

- [x] `geo_core` は `analysis` のみに依存する
- [x] `cargo check --workspace` が通る
- [ ] `cargo test --workspace` が通る
- [x] 依存チェックスクリプトが通る

## 備考

- 破壊的変更を許容し、互換レイヤは最小限とする
- 現在は「完了済みだが最終検証記録だけ未更新」の状態として扱う
