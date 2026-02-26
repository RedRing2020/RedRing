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

## 7. 事前確認結果（2026-02-26）

- downcast 利用の主要箇所
  - `view/app/src/app_state/stage_orchestration.rs`（`OctreeStage`）
  - `view/app/src/app_state/input_actions.rs`（`OctreeStage`）
  - `view/app/src/app_state/snapshot_playback.rs`（`MeshStage` / `OctreeStage`）
  - `view/app/src/app_state/display_controls.rs`（`MeshStage` / `NurbsSurfaceStage`）
  - `view/app/src/app_state/settings_accessors/snapshot_settings.rs`（`MeshStage`）
- 生成経路の二重化
  - `view/app/src/stage_factory.rs` に Draft/Outline/Shading の生成関数
  - `view/app/src/app_renderer.rs` の `AppRendererFactory` でも同系統の生成責務
  - `stage_orchestration.rs` は `stage_factory` を利用、`app_state.rs` 初期化は `AppRendererFactory` を利用

## 8. Phase11着手チェックリスト

- [ ] Phase11では downcast 全廃を狙わず、主要ユースケースの集約を優先
- [ ] `RenderStage` に capability メソッドを追加する場合はデフォルト実装を持たせ API互換を維持
- [ ] 生成経路は最終的に単一路線へ整理（どちらを正とするか先に固定）
- [ ] 既存挙動（キー操作・snapshot再生・表示切替）を変更しない
- [ ] 検証順序を固定（`cargo build -p redring` → `cargo clippy -p redring -- -D warnings`）

## 9. 推奨実装順序（Phase11本体）

1. `render_stage.rs` に capability 入口を追加（デフォルト no-op / false）
2. `app_state` 側の downcast を capability 呼び出しへ段階置換
3. `stage_factory` と `AppRendererFactory` の役割を統一（重複除去）
4. 不要となった downcast 呼び出しや生成経路を整理
5. `cargo build -p redring` / `cargo clippy -p redring -- -D warnings` で確認

## 10. 非対象（Phase11で扱わないもの）

- Stage内部の描画ロジック最適化
- ログレベル調整（Phase12で扱う）
- 命名整理（Phase6で扱う）
