# SelectionRect State 設計

## 目的

`SelectionRect` を画面座標（ピクセル）で扱う入力中間状態として `view/app` に実装する。

## 責務

- ドラッグ開始点と現在カーソル位置から矩形を生成
- 正規化済み矩形（左上 + 正の幅/高さ）として保持
- 幾何演算（ワールド包含判定等）は持たない

## 非責務

- ワールド座標への変換
- 選択判定アルゴリズム
- 描画パイプライン統合

## 実装方針

- `view/app/src/selection_rect.rs` に `SelectionRect` を追加
  - `from_points(start, end)` で正規化
  - `contains(point)` を補助APIとして提供
- `AppState` に以下の state を追加
  - `active_selection_rect: Option<SelectionRect>`
  - `last_selection_rect: Option<SelectionRect>`
  - `cursor_position: Option<(f32, f32)>`
  - `selection_rect_drag_origin: Option<(f32, f32)>`
- `app.rs` の `WindowEvent::CursorMoved` で `AppState::handle_cursor_moved` を呼ぶ
- 左ドラッグ（Ctrl 非押下）で `SelectionRect` 更新
