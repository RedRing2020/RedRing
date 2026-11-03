# geo_commons 旧撤去案アーカイブメモ

## 概要

この文書は旧方針メモであり、**現在の正本方針では `geo_commons` は廃止対象ではない**。

現行方針では、`geo_commons` は `analysis` のみに依存する再利用可能な幾何数値カーネル層として意図的に維持する。したがって本メモの「撤去計画」は実施対象ではなく、旧検討経緯としてのみ参照する。

## 現在の結論

- `geo_commons` は削除対象ではなく、残存する価値がある
- 価値の中心は、shape 非依存の純粋数値 kernel を `analysis` 直上で再利用できる点にある
- `geo_contracts` の trait定義、`geo_primitives` / `geo_nurbs` の impl entry point、`geo_algorithms` の高レベル戦略と責務分離できること自体が `geo_commons` の設計上の価値である
- ellipse 周長近似、離心率、焦点距離、焦点座標、shape 型を要求しない距離 helper のような共通数値処理は、今後も `geo_commons` に残す前提で扱う
- 本ファイルに残る撤去チェックリストは履歴情報であり、現行タスクや完了条件として読まない

## 背景

- 起票当時は `geo_commons` を `analysis` / `geo_algorithms` へ分解移管する案を検討していた
- その後、`geo_commons` は shape 非依存の幾何数値カーネル置き場として維持する方針へ変更された
- 現在の配置ルールは [dev/architecture/ARCHITECTURE.md](../ARCHITECTURE.md) と `#547` の整理方針を正とする

## 現在地

- 現在は `geo_commons` を削除しない
- ellipse 周長近似、焦点、離心率、ellipse 距離計算のような shape 非依存カーネルは `geo_commons` に残す
- `geo_contracts` は trait定義の正本、`geo_primitives` は impl entry point、`geo_algorithms` は cross-shape / heavy strategy を担う
- したがって本メモにある「物理削除と痕跡整理」は現行タスクではなく、旧案の記録として扱う
- `geo_commons` の価値は「残しても害がない」ではなく、「低依存の共通数値 kernel 層として残す意味がある」点にある

## 旧タスク（履歴情報。現行では非採用）

### Phase A: API棚卸し

- [x] `geo_commons` 公開関数24件を棚卸し
- [x] `analysis` / `geo_algorithms` への分類方針を確定
- [x] 依存元クレート（当時 `geo_foundation` のみ）を確認

### Phase B: 実装移管

- [x] `metrics::area_volume::*` を `geo_algorithms` へ移管
- [x] `approximations::*` を `geo_algorithms` へ移管
- [x] `geo_foundation` 側の参照を `geo_algorithms` 基準へ切替
- [x] `cargo check -p geo_algorithms` 成功
- [x] `cargo test -p geo_algorithms` 全パス

### Phase C: analysis責務整理

- [x] `analysis` には形状を含む関数を残さないことを確認
- [x] 必要なら `geo_algorithms` 側で数値基盤ヘルパーを整理
- [ ] `cargo check -p analysis -p geo_algorithms`

### Phase D: geo_commons撤去

- [x] `geo_foundation` の `geo_commons` 依存を削除
- [ ] `model/geo_commons` クレートを削除
- [x] ワークスペース参照/依存ルールを更新
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies.ps1 -ExitOnError`

## 旧受け入れ条件（履歴情報。現行では非採用）

- [x] ワークスペースに `geo_commons` 依存が存在しない
- [x] `analysis` は形状を含まない純粋数値計算のみを保持
- [x] 面積/体積/近似/距離を含む形状計算は `geo_*` 側に集約されている
- [ ] 移管後APIで既存利用箇所がビルド通過
- [ ] `cargo fmt --all -- --check` が通る
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` が通る
- [ ] `cargo check --workspace` が通る
- [ ] `cargo test --workspace` が通る

## 現行運用メモ

- `geo_commons` に残すかどうかは、「shape 型や orchestration を必要とせず、純粋関数として再利用できるか」で判定する
- `geo_commons` から外す候補は、cross-shape capability、solver orchestration、高レベル strategy、trait定義そのもの
- ellipse calculation 系の詳細な配置ルールは [dev/architecture/GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md](../GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md) の `#547` 節を正本とする

## 備考

- 分割移管の途中で中間PRを許容する、という当時の前提は記録として残す
- 現在の正本は [dev/architecture/ARCHITECTURE.md](../ARCHITECTURE.md) と `#547` の責務整理を参照する
- `geo_commons` の位置づけ更新経緯は [dev/archive/issues/architecture/issue-320-contracts-commons-normalization-archive-note.md](../../archive/issues/architecture/issue-320-contracts-commons-normalization-archive-note.md) を参照
