# Geometry Layer Refactor Issue Draft

## 0. 方針サマリ

今回の再編は次の4点を最優先で進める。

1. `geo_core` の依存逆転解消（`geo_core -> geo_foundation` を撤去）
2. Transform の2層化（共通核を `geo_core`、形状適用を `geo_primitives` / `geo_nurbs`）
3. `geo_commons` の分解移管（`analysis` と `geo_algorithms` へ）
4. `geo_foundation` の段階的廃止完了

---

## 1. 設計判断

### 1.1 Transformの配置

- 最適解は「`geo_primitives` / `geo_nurbs` に形状Transformを置く」＋「共通行列・座標変換を `geo_core` に置く」の2層構成
- `geo_algorithms` はクロス形状演算（intersect/collision）へ集中し、形状固有Transformは持たない

### 1.2 geo_commonsの扱い

- `geo_commons` は独立クレートとしては廃止可能
- 純粋数値近似・数学関数は `foundation/analysis` へ移管
- 形状横断の演算ロジックは `geo_algorithms` へ移管

### 1.3 geo_coreの再定義

- `geo_core` は最下層として再定義する
- 許容依存は `foundation/analysis` のみ
- `geo_core -> geo_foundation` の依存は解消する

---

## 2. 起票対象Issue（4件）

### Issue 1

- タイトル案: `geo_core依存逆転解消: geo_core -> geo_foundation を撤去`
- 目的: `geo_core` を最下層クレートとして成立させる
- 主タスク:
  - `geo_core` の `geo_foundation` 依存除去
  - 必要trait/型の再配置（`geo_core` or 呼び出し側）
  - 依存チェックスクリプト更新
- DoD:
  - `geo_core/Cargo.toml` から `geo_foundation` が消える
  - ワークスペースのビルド/テスト/依存チェック通過

### Issue 2

- タイトル案: `Transform再編: geo_core共通核 + geo_primitives/geo_nurbs形状実装の2層化`
- 目的: Transform責務の重複と分散を解消する
- 主タスク:
  - 共通変換ユーティリティを `geo_core` に統一
  - 形状固有Transform APIを `geo_primitives`/`geo_nurbs` に統一
  - 既存呼び出し側の import / API 更新
- DoD:
  - Transform共通核が `geo_core` に集約
  - 形状固有Transformが `geo_primitives`/`geo_nurbs` で提供

### Issue 3

- タイトル案: `geo_commons廃止: analysis/geo_algorithms への分解移管`
- 目的: 中間層を削減し責務を明確化する
- 主タスク:
  - `geo_commons` の公開API棚卸し
  - 数値近似を `analysis` へ移管
  - 形状横断計算を `geo_algorithms` へ移管
  - `geo_foundation` 経由の再エクスポート整理
- DoD:
  - `geo_commons` 依存がワークスペースから消える
  - 移管後APIで既存利用箇所がビルド通過

### Issue 4

- タイトル案: `geo_foundation廃止完了: geo_contracts導入と参照置換`
- 目的: Foundation集中を解消し、形状定義と実装の責務を分離する
- 主タスク:
  - `geo_contracts` を新設し、形状trait/契約を移設
  - `geo_primitives` / `geo_nurbs` は実装責務に限定
  - `geo_foundation` 参照箇所を段階置換し互換レイヤ削除
- DoD:
  - 形状定義の正規参照先が `geo_contracts` に統一
  - `geo_foundation` が不要クレートとして削除可能
  - 依存ルール違反なく全体テスト通過

---

## 3. 共通完了条件

- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies.ps1 -ExitOnError`

---

## 4. 合意が必要な前提

- 破壊的変更を許容する（互換レイヤを最小化）
- 依存境界の健全性を、短期互換性より優先する

---

## 5. 起票結果（2026-03-16）

- #317 `geo_core依存逆転解消: geo_core -> geo_foundation を撤去`
- #318 `geo_foundation廃止完了: geo_contracts導入と参照置換`
- #319 `Transform再編: geo_core共通核 + geo_primitives/geo_nurbs形状実装の2層化`
- #320 `geo_commons廃止: analysis/geo_algorithms への分解移管`

