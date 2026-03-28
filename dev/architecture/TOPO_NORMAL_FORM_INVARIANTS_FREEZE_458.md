# topo正規形不変条件 凍結ドキュメント（Issue #458）

作成日: 2026-03-28
対象Issue: #458
親Epic: #406
関連: #338, #405, #457, #459

## 目的

Issue #406 の完了条件「正規形不変条件の合意」を、#338 再開前提として参照可能な形で明文化・凍結する。

## ドキュメント状態

- 状態: レビュー提出版（凍結候補 v0.1）
- 本版の目的: #338 再開前に「正規形の判定軸」を固定する

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

## 用語定義（凍結）

- 母曲線: エッジが参照する幾何曲線そのもの
- parameter range: 母曲線上の有効区間 `[t_start, t_end]`
- same_sense: 母曲線の自然方向とトポロジー走査方向の一致フラグ
- 正規形: 交差結果を安定して比較・連結できる内部表現

## 不変条件セット（凍結候補 v0.1）

### NFI-01: 母曲線とパラメータ区間の整合

- 規約: 全エッジは母曲線と parameter range を必ず保持する。
- DoD:
  - エッジに `curve` と `parameter_range` が常に存在する。
  - parameter range の端点包含（閉区間/半開区間）が仕様で一意に定義される。
  - `t_start` と `t_end` の意味（母曲線の自然方向基準）が仕様で固定される。

### NFI-02: 走査向きの表現責務

- 規約: 向き反転は母曲線反転ではなく `same_sense` で表現する。
- DoD:
  - 向き変更時に母曲線自体を破壊的変更しない。
  - 隣接要素との接続判定は `same_sense` を含めて評価される。
  - 形状比較時に「幾何本体」と「走査向き」を分離して判定できる。

### NFI-03: 交差結果との位相整合

- 規約: `IntersectionTopology` と `IntersectionGeometry` の整合を保持する。
- DoD:
  - `Disjoint` は `None` と対応。
  - `Touching` は 0次元結果（点）と対応。
  - `Crossing` は 0次元（点交差）または1次元以上（線/曲線交差）を許容する。
  - `Coincident` は一致を表す表現と対応。
  - 判定優先順位に従って最終Topologyが一意に決まる。

### NFI-04: トレランス責務

- 規約: 判定トレランスは呼び出し境界で入力され、結果に記録される。
- DoD:
  - トレランス取得元が仕様で一意に定義される。
  - 正規形変換で暗黙EPSILONを導入しない。
  - 結果型の `tolerance_used` に実使用値が保存される。

### NFI-05: 正規化の決定性

- 規約: 同一入力に対して正規化結果が決定的であること。
- DoD:
  - セグメント列の順序規約（起点選択・向き）を仕様で固定する。
  - 比較前処理（重複点除去・連結判定）の順序が定義される。
  - 実行環境差に依存しない比較キーを持つ。

## 境界ケース規約表（凍結候補 v0.1）

| ケース | Topology | Geometry | 最低保証 | 備考 |
|---|---|---|---|---|
| 非交差 | Disjoint | None | 結果空であること | 早期終了可 |
| 点接触 | Touching | Point / Points | 0次元で表現可能 | 接線判定を併記 |
| 点交差 | Crossing | Point / Points | 交点集合が得られること | 線×線/線×面で発生 |
| 線/曲線交差 | Crossing | Segment / InfiniteLine / CompositeCurve | 1次元交差を表現可能 | 面×面で重要 |
| 一致 | Coincident | Coincident または一致表現 | 一致状態が識別可能 | 面同士で重要 |

## 組み合わせ別返却規約（#405 整合）

| 組み合わせ | 非交差 | 点接触/点交差 | 線/曲線交差 | 一致 |
|---|---|---|---|---|
| 線×線 | None+Disjoint | Point+Crossing | InfiniteLine+Coincident | - |
| 線×面 | None+Disjoint | Point+Crossing | Segment+Coincident | - |
| 面×面 | None+Disjoint | Point+Touching | InfiniteLine/CompositeCurve+Crossing | Coincident |

## 判定優先順位（固定）

1. Coincident
2. Crossing
3. Touching
4. Disjoint

## 適用範囲

- 本規約は `geo_topology` と `geo_algorithms` 間の結果受け渡しに適用する。
- `geo_primitives` の既存 `Option<Point3D<T>>` API は段階移行中の互換層として扱う。
- 完全移行完了までは #459 の互換アダプタ方針を併用する。

## 受け入れ判定（レビューDoD）

次の3点を満たした時点で #458 完了と判定する。

1. 不変条件 NFI-01〜NFI-05 がレビューで承認される。
2. 境界ケース規約表と組み合わせ別返却規約に矛盾がない。
3. #406 に本ドキュメントの合意済みリンクを追記する。

## レビュー観点

- #338 再開前提として不足はないか
- #457 / #459 の既存実装と矛盾しないか
- 用語定義が trait / 型責務と整合しているか

## 合意ログ

- 合意日: TBD
- 合意者: TBD
- レビューIssue/PR: TBD
- 参照版: `TOPO_NORMAL_FORM_INVARIANTS_FREEZE_458.md` v0.1

## 次アクション（実装前）

1. 本ドキュメントを #458 のレビュー対象として確定
2. #406 に合意済みリンクを追記
3. #459 の互換アダプタ方針文書へ本規約を参照させる
