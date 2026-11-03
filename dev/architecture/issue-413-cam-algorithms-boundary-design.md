# CAM責務境界設計

**対象Issue**: #413  
**目的**: `cam_algorithms` 新設判断と CAM クレート責務境界の固定  
**更新日**: 2026年3月25日

---

## 1. 今回確定した判断

### 1.1 `cam_algorithms` は新設する

- 位置づけ: CAM 固有アルゴリズム層
- 主責務: 経路生成、順序最適化、干渉回避、機械制約検証
- 非責務: シミュレーション実行、UI 属性統合、中立データ保持

### 1.2 `cam_sim` は CAM 特化ユースケース実行層として扱う

- 物理配置は `model/` 配下
- ただし責務は純粋データ層ではなく、CAM ドメイン専用の application 層に近い
- 主責務: ToolPath 実行、時間進行、除去量更新、結果キャッシュ

### 1.3 `cam_core` は中立データ基盤に限定する

- ToolPath / Tool / artifact I/O / 最小機械制約を保持
- `cam_algorithms` や `cam_sim` への逆依存は禁止

---

## 2. 依存方向の確定点

### 許可

- `cam_algorithms -> cam_core`
- `cam_algorithms -> geo_algorithms`
- `cam_sim -> cam_core`
- `cam_sim -> cam_algorithms`（限定的）
- `cam_entity -> cam_core`

### 禁止

- `cam_core -> cam_algorithms`
- `cam_algorithms -> cam_sim`
- `cam_entity -> cam_algorithms`
- `geo_* -> cam_*`

### 限定許可の意味

- `cam_sim -> cam_algorithms` は、実行前後の問い合わせに限定する
- `cam_sim` から `cam_algorithms` へ渡すのは ToolPath や制約検証用入力などの中立データに留める
- シミュレーション内部状態を `cam_algorithms` に渡して密結合化しない

---

## 3. `cam_sim -> cam_algorithms` を許可する理由

代表例は次の3点。

1. ToolPath 実行前の順序最適化
2. 姿勢補間後の機械制約再検証
3. 再生前の干渉チェック委譲

この整理により、`cam_sim` は実行器、`cam_algorithms` は計算器として責務が分離される。

---

## 4. PoC の選定結果

### 選定対象

- PoC-1: ToolPath 順序最適化

### 選定理由

- `cam_core` の既存 ToolPath 表現をそのまま入力に使える
- `cam_algorithms` 単体責務を検証しやすい
- `cam_sim` 実装や UI 実装へ依存しない
- #426 の詳細拡張前でも先行できる

---

## 5. スクリプト反映で必要なこと

`cam_algorithms` 導入時は、依存許可の追記だけでは不十分で、以下を同一変更セットで実施する。

1. `Cargo.toml` の workspace members 更新
2. `scripts/_arch_rules_data.ps1` の layer mapping 更新
3. `scripts/_arch_rules_data.ps1` の layer group 更新
4. `scripts/_arch_rules_data.ps1` の allowed deps 更新
5. `scripts/_arch_rules_data.ps1` の forbidden deps 更新
6. クレート作成後に required model crates 更新
7. `ARCHITECTURE.md` と設計文書の同期更新

注意:

- 現時点では `model/cam_algorithms` が未作成のため、必須クレート一覧への追加は保留
- 先に追加すると依存チェックが常時失敗する

---

## 6. 今回更新した文書

- `dev/architecture/CAM_ALGORITHMS_DESIGN.md`
- `dev/architecture/ARCHITECTURE.md`
- `scripts/_arch_rules_data.ps1`

---

## 7. レビュー観点

レビューでは特に次を確認する。

1. `cam_sim` を application 層相当として扱う整理に違和感がないか
2. `cam_sim -> cam_algorithms` の限定許可が広すぎないか
3. `cam_entity` を計算非保持に固定して問題ないか
4. `cam_core` に将来ロジックが逆流しない定義になっているか
5. 後続 Issue 分割粒度が実装計画として十分か

---

## 8. 残課題

- `cam_algorithms` クレートの実体は未作成
- `scripts/_arch_rules_data.ps1` は設計メモ追記のみで、依存ルール自体は未有効化
- 後続実装 Issue の起票番号は未確定

---

## 9. 設計完了の判定

Issue #413 については、少なくとも以下は設計完了とみなせる。

- `cam_algorithms` 新設採否
- CAM クレート責務表
- 許可/禁止依存の固定
- PoC-1 の選定
- スクリプト反映時の作業範囲整理

未完了なのは、後続 Issue 起票と実クレート導入作業のみ。