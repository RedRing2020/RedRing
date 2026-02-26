# Issue #258 Phase12: Stage品質改善（ログ/テスト/運用性）

- 対象Issue: #258 (Phase12準備)
- 作成日: 2026-02-26
- スコープ: 挙動不変の品質改善

## 1. 背景

- `stage` 内で描画パスやカメラ更新時の `info` ログが多く、運用時ノイズになりやすい。
- テストはプレースホルダが残り、回帰検知力が十分でない。
- 深度サイズ運用や resize 対応の責務が明確でない。

## 2. 目的

- ログ方針の統一（必要十分な情報量へ調整）
- 最小テストを追加し、構造変更時の安全性を向上
- resize運用時の責務を明確化

## 3. 実施方針

- ログ:
  - 描画ループの `info` は原則抑制し、`debug/trace` + 間引きへ統一
- テスト:
  - 可能な純ロジック（深さ遷移など）を単体テスト化
  - GPU必須部分は統合テスト方針へ明記
- 運用:
  - 深度リソースの resize ルールを実装/ドキュメント化

## 4. 対象ファイル（予定）

- `view/stage/src/octree_stage.rs`
- `view/stage/src/toolpath_stage.rs`
- `view/stage/src/nurbs_curve_stage.rs`
- `view/stage/src/nurbs_surface_stage.rs`
- `dev/architecture/*`（必要に応じ運用ガイド更新）

## 5. DoD

- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- ログ出力ポリシーが stage 内で整合
- 既存挙動を維持しつつ、追加テストが実行可能

## 6. リスクと対策

- リスク: ログ削減で調査情報不足
- 対策: 環境変数で間引き間隔を可変にし、必要時のみ詳細化
