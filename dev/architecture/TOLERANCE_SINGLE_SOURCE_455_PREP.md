# Issue #455 着手準備: Tolerance 単一正本化

最終更新: 2026-03-29
関連Issue: #455, #361, #377

## 1. 目的

Issue #455 の着手前に、現状のトレランス責務を再確認し、実装時の判断基準と作業順序を固定する。
本ドキュメントは「設計合意と分割計画の確定」までを対象とし、全コード一括置換は対象外とする。

## 2. 現状確認（2026-03-29）

### 2.1 正本として利用可能な定義

- `model/geo_contracts/src/tolerance.rs`
  - `ToleranceSettings<T>`
  - `GeometryContext<T>`
  - `default_distance_tolerance` / `default_angle_tolerance`
  - 型依存閾値 (`default_parallel_cross_error_tolerance`, `default_orthogonality_dot_error_tolerance`)

### 2.2 既存方針の文書

- `dev/architecture/GEOMETRIC_TOLERANCE_USAGE_RULES.md`
  - 正本は `geo_contracts::ToleranceSettings` と定義済み
  - ただし本文に `tolerance_migration.rs` / `geo_algorithms/src/tolerance.rs` の記述が残っており、現実装との差分確認が必要

### 2.3 実装側の利用状況（抜粋）

- `model/geo_algorithms/src/octree/tolerance.rs`
  - `ToleranceSettings::relaxed()` 起点で `OctreeTolerance` を派生
- `foundation/analysis/src/consts.rs`
  - 数値安定用定数に加え、幾何判定で誤用され得る定数群が存在

## 3. #455 で解消すべきギャップ

1. 「単一正本」の定義はあるが、適用境界（どこで取得しどこへ渡すか）の記述が不足
2. 判定種別ごとの選択基準（距離/角度/内積・外積）が規約として十分に機械化されていない
3. `analysis::consts` のうち「数値計算定数」と「幾何判定閾値」の責務境界が読み手に伝わりづらい
4. 移行計画が Issue 分割まで落ちていない

## 4. 設計オプション

### Option A（推奨）

- 幾何判定の正本を `geo_contracts::ToleranceSettings` に統一
- 呼び出し境界で `ToleranceSettings` を受け渡し、実装内の新規閾値定義を抑止
- `analysis::consts` は数値解法・数学定数の責務へ限定し、幾何判定閾値は参照しない

利点:
- #455 の目的（単一正本化）と直接整合
- #484 など intersection 意味論適用時の閾値揺れを抑制

注意点:
- 既存関数シグネチャの変更範囲を段階化しないと差分が大きくなる

### Option B

- `analysis::consts` にも幾何判定閾値を残し、用途別に使い分ける

利点:
- 既存コード変更量を抑えやすい

注意点:
- 正本が再び二重化し、#455 の受け入れ条件を満たしにくい

## 5. 推奨方針（着手時の採用案）

Option A を採用する。

固定ルール:
1. 幾何判定の既定値は `ToleranceSettings` 起点
2. 呼び出し境界で受け取り、下位処理へ明示伝播
3. `T::EPSILON` は局所的な数値安定化用途に限定（ドメイン判定の既定値にしない）
4. 新規に互換層を増やさない

## 6. 実施順（#455 の準備対象）

### Step 1: 規約文書更新（Doc-first）

更新対象:
- `dev/architecture/GEOMETRIC_TOLERANCE_USAGE_RULES.md`

反映内容:
- 現在存在しない層の記述を整理
- 判定種別ごとの選択ルールを表形式で明記
- 呼び出し境界の責務（取得/受渡し/消費）を明文化

### Step 2: 影響範囲の棚卸し

対象:
- `model/geo_algorithms`
- `model/geo_primitives`
- `model/geo_nurbs`

成果物:
- 「既定閾値参照」「ハードコード」「`T::EPSILON` 利用」を分類した一覧

### Step 3: 分割Issue化

最小分割案:
1. 文書正規化（ルール固定）
2. `geo_algorithms` 高頻度経路移行
3. 残りクレート移行
4. 仕上げ（残存参照排除 + 回帰テスト）

## 7. 受け入れ条件トレース（#455 対応）

- [ ] 単一正本の定義合意
  - 判定: `GEOMETRIC_TOLERANCE_USAGE_RULES.md` で正本が一意に定義されている
- [ ] 判定種別ごとの選択ルール文書化
  - 判定: 距離/角度/内積/外積の選択表が存在する
- [ ] 移行フェーズIssue分割
  - 判定: 実行順を持つ分割Issueが作成されている

## 8. 着手時チェックコマンド

```powershell
git status -sb
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
.\scripts\check_architecture_dependencies_simple.ps1
```

## 9. 実装を始める前の確認事項

- #455 は設計Issueなので、まず文書更新の差分を先に提示する
- 一括置換は行わず、分割Issue単位で小さく進める
- #484（intersection 返り値意味論適用）は #455 の方針確定後に着手する
