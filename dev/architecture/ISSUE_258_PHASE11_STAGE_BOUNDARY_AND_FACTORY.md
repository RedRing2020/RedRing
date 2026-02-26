# Issue #258 Phase11: Stage境界整理と生成経路一本化

- 対象Issue: #258 (Phase11準備)
- 作成日: 2026-02-26
- スコープ: `app`↔`stage` 境界の明確化と factory 統一

## 1. 背景

現状は `app` 側で `RenderStage` を `downcast_mut` して具体型操作しており、
抽象境界が弱い。また stage 生成は `stage_factory` と `AppRendererFactory` に二重化している。

## 2. 目的

- `RenderStage` 抽象と具体型操作の境界を整理
- 生成責務を一箇所へ集約し、変更点を局所化

## 3. 実施方針

- 段階導入で境界を改善
  - 直ちに downcast を全廃せず、用途別 capability メソッドを追加
  - 利用箇所から順に置換
- stage生成は factory を一本化
  - `stage_factory` か `AppRendererFactory` のいずれかに統一
  - API互換を維持しながら移行

## 4. 対象ファイル（予定）

- `view/stage/src/render_stage.rs`
- `view/app/src/app_state/stage_orchestration.rs`
- `view/app/src/app_state/input_actions.rs`
- `view/app/src/app_state/snapshot_playback.rs`
- `view/app/src/stage_factory.rs`
- `view/app/src/app_renderer.rs`

## 5. DoD

- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- downcast 呼び出しが削減または集約され境界が明確
- stage生成の重複経路が解消

## 6. リスクと対策

- リスク: インターフェース変更による影響範囲拡大
- 対策: capability 追加→呼び出し置換→不要経路削除の順で小PR分割
