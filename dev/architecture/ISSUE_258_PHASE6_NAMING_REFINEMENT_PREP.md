# Issue #258 Phase6: 不正確な命名修正 準備

- 対象Issue: #258 (Phase6)
- 作成日: 2026-02-26
- スコープ: 命名修正のみ（挙動変更なし）

## 1. 背景

- これまでの段階的リファクタで責務境界は整理されたが、旧来命名が残っている箇所がある。
- 命名と実責務の差があると、保守時の誤読や修正漏れを誘発しやすい。

## 2. 目的

- 実責務に沿った命名へ統一し、可読性と保守性を向上する。
- 既存挙動を維持し、命名変更の影響範囲を明確化する。

## 3. 実施方針

- リネーム専用PRとして扱い、ロジック変更は入れない。
- 影響が大きい名前から優先的に段階適用する。
- 置換後は `cargo build` / `cargo clippy -- -D warnings` / `cargo fmt` を必須実施する。

## 4. 候補命名（初期案）

- `ViewRect`（実態: 矩形選択）
  - 候補: `SelectionRect`
- `active_view_rect`
  - 候補: `active_selection_rect`
- `*_renderer` 系で実態が「生成/設定ヘルパ」のみのもの
  - 候補: `*_factory` / `*_builder` / `*_loader` への再命名を個別判定

## 5. 調査対象（優先）

- `view/app/src/app_state/mouse_actions.rs`
- `view/app/src/app_state/stage_orchestration.rs`
- `view/app/src/app_renderer.rs`
- `view/app/src/view_rect_overlay.rs`
- `view/render/src/*`

## 6. DoD

- 命名修正のみで構成され、挙動変更が含まれない
- 旧名称参照が残っていない（定義・呼び出し・コメントを確認）
- `cargo build` 成功
- `cargo clippy -- -D warnings` 成功
- `cargo fmt` 実行済み

## 7. リスクと対策

- リスク: 置換漏れによるビルドエラー、意図しない広域変更
- 対策:
  - 1つの命名グループごとに小PR化
  - `grep` で旧名の残存確認を必須化
  - 変更前後で API シグネチャ差分を確認

## 8. 進捗（2026-02-26）

- 第1弾として以下の命名修正を適用
  - 型名: `ViewRect` → `SelectionRect`
  - AppState フィールド: `active_view_rect` / `last_view_rect` → `active_selection_rect` / `last_selection_rect`
- 追従更新済み
  - `app_state/mouse_actions.rs`
  - `app_state/stage_orchestration.rs`
  - `app_renderer.rs`
  - `view_rect_renderer.rs`
  - `app_state/snapshot_playback.rs`
- 検証結果
  - `cargo build` 成功
  - `cargo clippy -- -D warnings` 成功
  - `cargo fmt` 実行済み

- 第2弾として以下の命名修正を適用
  - ファイル名:
    - `view_rect.rs` → `selection_rect.rs`
    - `view_rect_renderer.rs` → `selection_rect_renderer.rs`
  - 型名/API名:
    - `ViewRectRenderer` → `SelectionRectRenderer`
    - `update_view_rect_overlay` → `update_selection_rect_overlay`
  - 内部状態名:
    - `view_rect_drag_origin` → `selection_rect_drag_origin`

- 第2弾検証結果
  - `cargo build` 成功
  - `cargo clippy -- -D warnings` 成功
  - `cargo fmt` 実行済み
