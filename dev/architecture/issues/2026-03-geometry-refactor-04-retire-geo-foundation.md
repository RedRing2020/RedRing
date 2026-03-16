## 概要

`geo_foundation` を廃止し、**形状定義専用クレート**を新設する。

- 形状定義（trait/契約）: 新クレート（`geo_contracts`）
- 形状実装: `geo_primitives` / `geo_nurbs`

## 背景

- 形状定義と実装を分離して責務を明確化したい
- `geo_primitives` / `geo_nurbs` は実装クレートとして整理したい
- 旧 `geo_foundation` の責務集中を解消したい

## タスク

- [ ] 新クレート `geo_contracts` を作成し、形状trait/契約を移設
- [ ] `geo_primitives` / `geo_nurbs` で各traitを実装
- [ ] 参照側 import を `geo_contracts` 基準へ置換
- [ ] `geo_foundation` の互換層を段階削除
- [ ] 最終的に `geo_foundation` クレートを削除

## 受け入れ条件

- [ ] 形状定義の正規参照先が `geo_contracts` へ一本化されている
- [ ] `geo_primitives` / `geo_nurbs` は実装責務に限定される
- [ ] `geo_foundation` なしでワークスペースがビルド可能
- [ ] `cargo check --workspace` が通る
- [ ] `cargo test --workspace` が通る
- [ ] 依存チェックスクリプトが通る

## 備考

- 破壊的変更許容を前提に進める
