## 概要

`geo_foundation` を廃止し、責務を `geo_contracts` / `geo_core` / `analysis` へ再配置する。

- 形状定義（trait/契約）: `geo_contracts`
- 共通変換核・共通エラー: `geo_core`
- 数値抽象・定数: `analysis`
- 形状実装: `geo_primitives` / `geo_nurbs`

## 背景

- 形状定義と実装を分離して責務を明確化したい
- `geo_primitives` / `geo_nurbs` は実装クレートとして整理したい
- 旧 `geo_foundation` の責務集中を解消したい

## 現状整理

- `geo_contracts` クレート自体は作成済みだが、現状は `Angle` / `Scalar` 再エクスポート中心で、形状契約の本体移設は未完
- `geo_primitives` / `geo_nurbs` の一部ファイルでは `geo_contracts::{Angle, Scalar}` 利用が始まっているが、公開 re-export と形状契約の正規参照先はまだ混在している
- `geo_core -> geo_foundation` 依存解消は #317 で実施済み
- Transform 共通責務の `geo_core` 側整理は #319 の前提として概ね揃っている
- 依然として以下 6 クレートが `geo_foundation` に直接依存している
	- `geo_primitives`
	- `geo_nurbs`
	- `geo_algorithms`
	- `geo_io`
	- `geo_entity`
	- `cam_core`

## 実施方針

- `geo_contracts` には形状定義専用の責務だけを移す
- `Scalar` / `Angle` / `TolerantEq` など数値抽象は `analysis` を正とする
- Transform trait / error など共通変換責務は `geo_core` を正とする
- 置換順は、契約移設 -> 実装クレート切替 -> 利用側切替 -> 互換層縮退 -> 最終削除 とする
- 境界判断は `dev/archive/issues/issue-318-retire-geo-foundation-archive-note.md` の固定境界表（3.0節）を正とする

## タスク

### Phase 0: 棚卸しと責務分類

- [ ] `geo_foundation` 公開 API を棚卸しし、移設先を `geo_contracts` / `geo_core` / `analysis` に分類
- [ ] `geo_foundation` を参照している各クレートの import パターンを分類
	- 形状契約参照
	- tolerance / constants 参照
	- transform / extension / collision 参照
- [ ] `geo_foundation` に残すべきでない項目一覧を文書化

### Phase 1: geo_contracts API 拡充

- [x] 新クレート `geo_contracts` を作成
- [ ] `geo_foundation::geometry::core::*_traits` にある形状契約を `geo_contracts` 側へ移設または再編
- [ ] `geo_contracts` の公開 API を、利用側が `geo_foundation` を経由しなくてよい形に整理
- [ ] `PrimitiveKind` / 分類系の所属先を確定し、必要なら `geo_contracts` へ移す
- [ ] `cargo check -p geo_contracts`

### Phase 2: 実装クレート切替

- [ ] `geo_primitives` の形状契約 import を `geo_contracts` 基準へ切替
- [ ] `geo_primitives` の `Cargo.toml` から `geo_foundation` 依存を外せる状態にする
- [ ] `cargo check -p geo_primitives`
- [ ] `cargo test -p geo_primitives`
- [ ] `geo_nurbs` の形状契約 import を `geo_contracts` 基準へ切替
- [ ] `geo_nurbs` の `Cargo.toml` から `geo_foundation` 依存を外せる状態にする
- [ ] `cargo check -p geo_nurbs`
- [ ] `cargo test -p geo_nurbs`

### Phase 3: 利用側クレート切替

- [ ] `geo_algorithms` の `geo_foundation` 参照を用途別に置換
	- 形状契約 -> `geo_contracts`
	- 数値抽象 / tolerance -> `analysis` または適切な所有クレート
	- transform / 共通責務 -> `geo_core`
- [ ] `geo_io` の三角形契約 / `Scalar` 参照を置換
- [ ] `geo_entity` の entity 系契約の所属先を整理して置換
- [ ] `cam_core` の `geo_foundation` 依存を除去
- [ ] 各クレートの `Cargo.toml` から `geo_foundation` を除去

### Phase 4: 互換層縮退

- [ ] `geo_foundation` を一時的な薄い再エクスポート層に縮退
- [ ] workspace 内で旧 import ルートが残っていないことを検索で確認
- [ ] README / examples / コメント内の旧 import も更新

### Phase 5: 最終撤去

- [ ] workspace 全体で `geo_foundation` 参照がゼロであることを確認
- [ ] workspace members から `model/geo_foundation` を削除
- [ ] `model/geo_foundation` クレート本体を削除
- [ ] 関連ドキュメントと依存図を更新

## PR分割案

- PR1: Phase 0-1（棚卸し + `geo_contracts` API 拡充）
- PR2: Phase 2（`geo_primitives` / `geo_nurbs` 切替）
- PR3: Phase 3（利用側クレート切替）
- PR4: Phase 4-5（互換層撤去 + `geo_foundation` 削除 + 全体検証）

## 受け入れ条件

- [ ] 形状定義の正規参照先が `geo_contracts` へ一本化されている
- [ ] `geo_primitives` / `geo_nurbs` は実装責務に限定される
- [ ] 数値抽象は `analysis`、Transform 共通責務は `geo_core` に整理されている
- [ ] workspace 内の `geo_foundation` 直接参照がゼロになっている
- [ ] `geo_foundation` なしでワークスペースがビルド可能
- [ ] `cargo check --workspace` が通る
- [ ] `cargo test --workspace` が通る
- [ ] 依存チェックスクリプトが通る

## 備考

- 破壊的変更許容を前提に進める
- 最初にやるべきなのは `geo_contracts` API 拡充であり、ここを固めずに import 置換を始めない
- 実施準備の詳細は `dev/archive/issues/issue-318-retire-geo-foundation-archive-note.md` を参照
- 実行時は同ファイルの `Phase詳細` と `6.1/6.2` チェックリストをPRテンプレとして利用する
