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
- `view/app/src/selection_rect_renderer.rs`
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

- 第3弾として以下の命名修正を適用
  - API名:
    - `SelectionRectRenderer::update_rect` → `SelectionRectRenderer::update_selection_rect`

- 第4弾として以下の命名修正を適用
  - 設計ドキュメントの旧表記統一:
    - `SELECTION_RECT_STATE_DESIGN.md` の `ViewRECT/ViewRect` 表記を `SelectionRect` に統一
    - `RECT_WORLD_SELECTION_COORDINATE_DESIGN.md` の `ViewRECT` 表記を `SelectionRect` に統一
    - 旧ファイル参照 `view_rect.rs` を `selection_rect.rs` へ更新

- 第2弾検証結果
  - `cargo build` 成功
  - `cargo clippy -- -D warnings` 成功
  - `cargo fmt` 実行済み

## 9. 追加で検出した命名課題（2026-02-26）

- `Toolpath` / `ToolPath` の表記混在
  - 例: `ToolpathDebugData`（型名）が `ToolPath*` 系APIと混在
  - 対象の中心: `view/app/src/app_state/debug_scene/toolpath.rs`
- `load_debug_cutter_path_only` の語彙が `toolpath` 系の主語と不一致
  - 同モジュール内で `toolpath` が主要語彙のため、命名統一余地あり
- 設計ドキュメント名の旧語彙残存
  - 例: `SELECTION_RECT_STATE_DESIGN.md` / `RECT_WORLD_SELECTION_COORDINATE_DESIGN.md` へリネーム対応済み
- `debug_*` 接頭辞の広域使用
  - 機能上は妥当だが、将来の運用規約として `debug/sample/dev` の使い分け方針を固定する余地あり

## 10. 対応内容（追記方針）

- 対応A（優先・低リスク）
  - `Toolpath` → `ToolPath` へ統一（型名・関数名・コメント）
  - `load_debug_cutter_path_only` を `toolpath` 語彙と整合する名称へ改名
- 対応B（中リスク）
  - 旧設計ドキュメント名を現行語彙へリネーム
  - 既存参照リンク（Issue/設計メモ）を追従更新
- 対応C（方針策定）
  - `debug_*` 接頭辞の命名規約を定義し、適用対象を段階化
  - ただし広範囲影響があるため、別PRで独立管理

## 11. ブランチ戦略（Phase6-2 分割推奨）

- 推奨: **Phase6-2 を小粒で分割**
  - `feature/issue-258-phase6-2a-render-naming-20260226`（完了 / PR #277）
    - 対応A（`render_2d` / `render_3d` 命名対称化）
  - `feature/issue-258-phase6-2b-doc-filename-alignment-20260226`（完了 / PR #278）
    - 対応B（設計ドキュメント名のリネームとリンク追従）
  - `feature/issue-258-phase6-2c-toolpath-naming-20260226`（完了 / PR #279）
    - 対応A（`ToolPath` 統一 + `load_debug_cutter_path_only` 改名）
  - `feature/issue-258-phase6-2d-debug-prefix-policy-20260226`（実装完了 / PR未作成）
    - 対応C（命名規約定義 + 必要最小の適用）

- 分割理由
  - 影響範囲とレビュー観点が異なるため、差分を明確化できる
  - 挙動不変確認を段階化でき、リスク低減につながる

## 12. render/stage 追加調査結果（2026-02-26）

### 優先度: 高

- `render_2d` と `render_3d` の命名軸が非対称
  - 型名:
    - `Render2dResources`（2D側）
    - `Renderer3D`（3D側）
  - 生成関数:
    - `create_render_2d_resources`
    - `create_renderer_3d`
  - 描画関数:
    - `draw_render_2d`
    - `draw_renderer_3d`

### 優先度: 中

- 2D/3D の GPU ラベル文字列が非対称
  - 2D: `Render 2D`, `Render 2D Vertex Buffer`
  - 3D: `Renderer3D`, `Renderer3D Vertex Buffer`

### 優先度: 低（現状維持可）

- `toolpath`（モジュール名, snake_case）と `ToolPath`（型名, CamelCase）は Rust 規約上整合
  - 命名揺れではなく、規約に沿った表記差と判断

## 13. render/stage 対応案（Phase6-2A 候補）

- 目的: `render_2d` / `render_3d` の命名を対称化し、可読性を向上（挙動変更なし）
- 対応案（最小差分）
  - 3D側を 2D 側に合わせる方向
    - `Renderer3D` → `Render3dResources`（または `Render3DResources`）
    - `create_renderer_3d` → `create_render_3d_resources`
    - `draw_renderer_3d` → `draw_render_3d`
  - 文字列ラベルも `Render 3D` 系へ統一
- 注意点
  - 公開 API 名変更になるため、呼び出し元追従を同一コミットで完結
  - ロジック変更を混在させない

## 14. Phase6-2a 完了メモ（2026-02-26）

- 実施ブランチ: `feature/issue-258-phase6-2a-render-naming-20260226`
- 反映PR: #277（merged）
- 今回の適用範囲
  - `view/render/src/render_3d.rs` の命名を `render_2d` 系に合わせて対称化
  - `view/stage/src/shading.rs` の呼び出し名を追従
- 非対象（別サブフェーズ）
  - `Toolpath/ToolPath` 統一
  - `debug_*` 接頭辞方針の適用

## 15. Phase6-2b 完了メモ（2026-02-26）

- 目的
  - 旧語彙が残る設計ドキュメントのファイル名を現行語彙へ整合
- 対象ファイル（現状）
  - `dev/architecture/SELECTION_RECT_STATE_DESIGN.md`
  - `dev/architecture/RECT_WORLD_SELECTION_COORDINATE_DESIGN.md`
- リネーム候補
  - `VIEW_RECT_STATE_DESIGN.md` → `SELECTION_RECT_STATE_DESIGN.md`（適用済み）
  - `RECT_WORLD_VIEW_COORDINATE_DESIGN.md` → `RECT_WORLD_SELECTION_COORDINATE_DESIGN.md`（適用済み）
- 次修正時の実施手順
  - ファイル名リネーム
  - `**/*.md` の旧ファイル名参照を追従更新
  - `mdbook build` でリンク整合性を確認
  - 変更はドキュメントのみ（コード無変更）

## 16. Phase6-2c 完了メモ（2026-02-26）

- 目的
  - `toolpath` 周辺の命名軸を `ToolPath` 語彙へ統一し、読みやすさを向上（挙動変更なし）
- 主要対象
  - `view/app/src/app_state/debug_scene/toolpath.rs`
  - `view/app/src/app_state/input_actions.rs`
- 変更候補（最小差分）
  - 型名: `ToolpathDebugData` → `ToolPathDebugData`
  - 関数名:
    - `build_toolpath_debug_data` の戻り型追従
    - `apply_toolpath_debug_data` の引数型追従
    - `load_debug_cutter_path_only` → `load_debug_toolpath_only`
  - 呼び出し側: `input_actions.rs` のキー入力ハンドラ追従
- 非対象
  - `toolpath.rs`（モジュール名）など snake_case 命名の Rust 規約範囲は変更しない
  - `debug_*` 接頭辞方針の再編は 2d で扱う
- 実施結果
  - 型名: `ToolpathDebugData` → `ToolPathDebugData`
  - 関数名: `load_debug_cutter_path_only` → `load_debug_toolpath_only`
  - 追従: `input_actions.rs` の呼び出し更新
- 検証結果
  - `cargo build` 成功
  - `cargo clippy -- -D warnings` 成功
  - `cargo fmt` 実行済み

## 17. Phase6-2d 完了メモ（2026-02-26）

- 目的
  - `debug_*` 接頭辞の使い分け方針を明文化し、最小範囲で命名を適用
- 命名規約（本PRで定義）
  - `debug_*`: 状態遷移検証・診断・再生制御など、開発向けの動作切り替え
  - `sample_*`: サンプルデータの読込・表示（ユーザーが表示対象を切り替える用途）
  - `dev_*`: 一時検証コード（恒常APIには原則使わない）
- 最小適用範囲
  - `view/app/src/app_state/debug_scene/toolpath.rs`
    - `load_debug_toolpath` → `load_sample_toolpath`
    - `load_debug_toolpath_only` → `load_sample_toolpath_only`
  - `view/app/src/app_state/input_actions.rs`
    - `p` / `P` キーの呼び出しを新関数名へ追従
- 非対象
  - `debug_snapshot` 等の既存状態名は影響範囲が大きいため据え置き
  - 他 `load_debug_*` API 群は別PRで段階適用

  ## 17. Phase6-2d 完了メモ（2026-02-26）

  - 目的
    - `debug/sample/dev` 接頭辞の使い分けを明文化し、最小範囲で適用する
  - 命名ポリシー（2d確定）
    - `debug_*`: 状態再生・検証補助・診断操作など、デバッグ専用の運用操作
    - `sample_*`: サンプルデータを読み込んで表示する操作
    - `dev_*`: 開発中の一時実験用（恒久APIには原則残さない）
  - 最小適用（本PR範囲）
    - `load_debug_toolpath` → `load_sample_toolpath`
    - `load_debug_toolpath_only` → `load_sample_toolpath_only`
    - `input_actions.rs` の呼び出しを追従
  - 非対象
    - `debug_snapshot` など既存状態構造の命名変更は影響が広いため今回は対象外
    - `load_debug_line` / `load_debug_circle` など他カテゴリの一括改名は別途段階適用
