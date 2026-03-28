# topo正規形不変条件 凍結ドキュメント（Issue #458）

作成日: 2026-03-28
対象Issue: #458
親Epic: #406
関連: #338, #405, #457, #459

## 目的

Issue #406 の完了条件「正規形不変条件の合意」を、#338 再開前提として参照可能な形で明文化・凍結する。

## 完了条件チェック

- [ ] 不変条件セットの定義（DoD付き）
- [ ] 境界ケース（一致/接触/交差/非交差）の規約表を記載
- [ ] #406 へ合意済みリンクを追記

## スコープ

- topo正規形（母曲線 + parameter range）の不変条件定義
- 交差結果との整合ルール（IntersectionTopology / IntersectionGeometry）
- 設計レビューで判断可能なDoDの明文化

## 非スコープ

- コード実装
- 既存APIの挙動変更
- full B-Rep 実装詳細

## 用語定義（凍結候補）

- 母曲線: エッジが参照する幾何曲線そのもの
- parameter range: 母曲線上の有効区間 `[t_start, t_end]`
- same_sense: 母曲線の自然方向とトポロジー走査方向の一致フラグ
- 正規形: 交差結果を安定して比較・連結できる内部表現

## 不変条件セット（草案）

### NFI-01: 母曲線とパラメータ区間の整合

- 規約: 全エッジは母曲線と parameter range を必ず保持する。
- DoD:
  - エッジに `curve` と `parameter_range` が常に存在する。
  - parameter range は開区間/閉区間の扱いが仕様で一意に定義される。

### NFI-02: 走査向きの表現責務

- 規約: 向き反転は母曲線反転ではなく `same_sense` で表現する。
- DoD:
  - 向き変更時に母曲線自体を破壊的変更しない。
  - 隣接要素との接続判定は `same_sense` を含めて評価される。

### NFI-03: 交差結果との位相整合

- 規約: `IntersectionTopology` と `IntersectionGeometry` の整合を保持する。
- DoD:
  - `Disjoint` は `None` と対応。
  - `Touching` は 0次元結果（点）と対応。
  - `Crossing` は 1次元以上の結果を許容。
  - `Coincident` は一致を表す表現と対応。

### NFI-04: トレランス責務

- 規約: 判定トレランスは呼び出し境界で入力され、結果に記録される。
- DoD:
  - トレランス取得元が仕様で一意に定義される。
  - 正規形変換で暗黙EPSILONを導入しない。

## 境界ケース規約表（草案）

| ケース | Topology | Geometry | 最低保証 | 備考 |
|---|---|---|---|---|
| 非交差 | Disjoint | None | 結果空であること | 早期終了可 |
| 点接触 | Touching | Point / Points | 0次元 | 接線判定を併記 |
| 交差 | Crossing | Point / Segment / CompositeCurve / InfiniteLine | 次元>=0 | 対象ペア依存 |
| 一致 | Coincident | Coincident または一致表現 | 一致状態が識別可能 | 面同士で重要 |

## 判定優先順位（固定）

1. Coincident
2. Crossing
3. Touching
4. Disjoint

## レビュー観点

- #338 再開前提として不足はないか
- #457 / #459 の既存実装と矛盾しないか
- 用語定義が trait / 型責務と整合しているか

## 合意ログ

- 合意日: TBD
- 合意者: TBD
- レビューIssue/PR: TBD

## 次アクション（実装前）

1. 本ドキュメントを #458 のレビュー対象として確定
2. #406 に合意済みリンクを追記
3. #459 の互換アダプタ方針文書へ本規約を参照させる
