# Issue #258 Phase5: Manager分割方針とloader配置方針

- 日付: 2026-02-26
- 対象Issue: #258 (Phase5準備)
- 目的: 実装前に、責務分割の判断基準を明文化する（方針のみ）

## 1. Manager分割方針（AppState肥大化対策）

### 基本原則
- `AppState` は**オーケストレーション専用**に寄せる。
- Managerは「状態 + その状態に対する遷移/更新」を1責務で持つ。
- 1 Manager 1責務を守り、相互依存を最小化する。
- 既存挙動を変えない（API互換優先・段階的移行）。

### Manager追加の判断基準
以下を満たす場合に、新規Manager切り出しを検討する。

1. データ構造と更新ロジックがまとまって移動できる
2. 呼び出し元が複数あり、責務境界が曖昧になっている
3. テスト観点（状態遷移）が単体で切り出せる
4. 既存のManagerへ追加すると責務が混在する

### 逆に分割しない条件
- 単なる薄い委譲しか増えず、実質的な責務分離にならない
- 将来拡張よりも理解コストが増える
- 状態所有者が一意で、他モジュールから再利用されない

### 想定候補（例）
- `SelectionManager`: 選択対象/選択遷移
- `VisibilityManager`: 可視/不可視トグル群
- `OverlayStateManager`: 矩形選択・snapshot overlay 表示状態

## 2. loader配置方針（stl_loader.rs, svg_loader.rs について）

### 現時点の方針
- 当面は `view/app/src` 直下のフラット配置を許容する。
- ただし、呼び出し側は `AppAssetLoaderFacade` 経由に統一する。

### フラット維持の理由
- 既存コードへの影響を最小化できる
- API互換を保ったまま段階的移行しやすい
- Phase5時点のスコープでは配置変更より責務境界明確化を優先

### サブディレクトリ化へ移行するトリガー
以下のいずれかを満たしたら、`loaders/` への集約を検討する。

1. loaderが3〜4個以上に増え、責務別に整理した方が可読性が高い
2. 共通前処理/後処理（パス解決、エラー整形、座標変換）が増える
3. テスト対象を loader単位で独立させる必要が出る

### 将来像（必要時のみ）
- `view/app/src/loaders/mod.rs`
- `view/app/src/loaders/stl_loader.rs`
- `view/app/src/loaders/svg_loader.rs`
- `view/app/src/app_asset_loader.rs` は Facade として残し、内部参照先のみ差し替える

## 3. 実施ルール

- 小PR分割で段階適用する
- 各段階で `cargo fmt` / `cargo build -p redring` / `cargo clippy -p redring -- -D warnings` を通す
- 命名変更（Phase6）は責務分割PRと混ぜない
