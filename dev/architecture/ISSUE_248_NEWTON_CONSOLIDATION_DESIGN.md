# Issue #248: primitive_nurbs の Newton 法重複実装統合設計

- 日付: 2026-02-25
- 対象Issue: #248
- スコープ: `model/geo_algorithms/src/collision/primitive_nurbs.rs` と `foundation/analysis/src/linalg/solver/newton.rs`

## 背景

`primitive_nurbs` 側で最近接点探索のための Newton 反復をローカル実装しており、
`analysis` 側の Newton ソルバーと責務が重複している。

## 目的

- Newton 反復の責務（反復上限/収束判定/導関数ゼロ判定）を `analysis` に集約する
- `primitive_nurbs` 側は目的関数定義と拘束（u範囲）に専念する

## 方針

1. `analysis::linalg::solver::newton` に `newton_solve_bounded` を追加
   - 反復ごとに更新値を `[min, max]` にクランプ
   - 既存 `newton_solve` 同様に `DERIVATIVE_ZERO_THRESHOLD` を使用
2. `analysis::linalg::solver::newton` に `newton_solve_with_numeric_derivative_bounded` を追加
   - 導関数を前進差分で評価（ステップ幅引数）
   - 内部で `newton_solve_bounded` を利用し、反復ロジックを一元化
3. `primitive_nurbs` の `newton_refine_closest_point` は上記 API 呼び出しへ置換
   - 収束失敗時は初期値 `initial_u` をフォールバック

## 受け入れ条件との対応

- 重複ロジック解消: 反復ループを `analysis` に移管
- 収束条件/許容誤差/反復上限の責務: `analysis` API 引数 + 実装に集約
- 回帰確認: `collision::primitive_nurbs` と `cargo test --workspace` 実行

## 非対象

- #214（cam_sim Phase1a）への機能追加
- NURBS コリジョンアルゴリズム自体の高度化（目的関数定義の変更など）
