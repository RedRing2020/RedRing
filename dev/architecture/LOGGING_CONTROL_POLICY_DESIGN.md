# Logging Control Policy Design (Issue #230)

## 目的

描画ループや高頻度処理で発生するログ出力を抑制し、実行時負荷を軽減する。
同時に、必要時のみ詳細ログを取得できる運用ルールを整備する。

## 背景

現状は `tracing` + `tracing-subscriber` を利用し、`RUST_LOG` でフィルタ制御している。
ただし、`info` レベルに高頻度ログが混在しており、デフォルト設定で過剰出力が発生する。

`view/app/src/logging.rs` のデフォルトフィルタ:

```text
warn,
wgpu=warn,wgpu_hal=warn,wgpu_core=warn,naga=warn,
redring=info,stage=info,viewmodel_graphics=info,viewmodel_converter=info,render=info,cam_core=info
```

このため、上記モジュール配下の `info` は通常運用でも表示される。

## スコープ

- ログレベル設計の明確化（trace/debug/info/warn/error）
- 高頻度ログの `trace` 降格
- 必要箇所への軽量レート制限導入
- `RUST_LOG` 運用ルールの文書化

## 非スコープ

- 新規ログ制御クレート作成（承認が必要なため本Issueでは実施しない）
- ログ基盤の全面刷新（`tracing` から別基盤への移行）

## 将来拡張（承認後スコープ）

- Foundation配下にログ制御クレートを追加する設計検討
- 各実装クレートへ共通組み込みし、実装者が毎回同様コードを書くことを避ける
- 命名は `geo_*` を使わず、ドメイン非依存の名称を採用する

---

## 現状分析（高頻度・負荷懸念箇所）

### 1) 描画/ステージ系（フレーム反復の可能性）

- `view/render/src/line.rs`
- `view/stage/src/mesh_stage.rs`
- `view/stage/src/nurbs_curve_stage.rs`
- `view/stage/src/nurbs_surface_stage.rs`
- `view/stage/src/octree_stage.rs`
- `view/stage/src/toolpath_stage.rs`

### 2) 変換・評価系（大量データログの可能性）

- `viewmodel/converter/src/nurbs_view.rs`
- `viewmodel/converter/src/octree_converter.rs`
- `viewmodel/converter/src/toolpath_converter.rs`

### 3) 入力・状態遷移系（イベント粒度）

- `view/app/src/app_state.rs`
- `viewmodel/graphics/src/camera.rs`

---

## ログレベル方針（統一ルール）

### error

- 処理継続不能、または結果欠落が確定する失敗
- 例: 読み込み失敗、GPUリソース構築失敗

### warn

- 異常系だがフォールバック可能
- 例: 入力不整合、値クランプ、安全側補正、描画スキップ

### info

- 低頻度の重要イベント（ユーザー操作・状態遷移）
- 例: ファイル読込開始/完了、モード切替、ステージ切替
- 原則: 1イベント1行、ループ内出力禁止

### debug

- 開発時に有用な詳細情報
- 例: リソース件数、1回の変換要約
- 原則: 連続フレームで繰り返すものは避ける

### trace

- フレーム毎/頂点毎/ループ内の詳細ログ
- デフォルト運用では非表示

---

## 実装方針

### 方針A（必須）: フレーム毎ログの `trace` 降格

- 描画ループ由来の `info/debug` を `trace` に変更
- 高頻度関数では `info` を使わない

### 方針B（推奨）: レート制限ヘルパ（最小実装）

小規模ユーティリティを `view/app/src/logging.rs`（または `viewmodel_graphics` 内）に追加。

候補API:

```rust
pub fn should_log_every_n_frames(frame: u64, interval: u64) -> bool
pub fn should_log_after(last: &mut Instant, interval: Duration) -> bool
```

用途:
- フレーム関連の進捗ログを Nフレーム毎に間引く
- 状態監視ログを時間間隔で制御

### 方針C（必須）: デフォルトフィルタの引き締め

`view/app/src/logging.rs` の `DEFAULT_LOG_FILTER` を再調整。

提案:

```text
warn,
wgpu=warn,wgpu_hal=warn,wgpu_core=warn,naga=warn,
redring=info,
stage=warn,viewmodel_graphics=warn,viewmodel_converter=warn,render=warn,cam_core=info
```

考え方:
- ユーザー操作起点は `redring=info` で維持
- フレーム密度が高い層は `warn` をデフォルト

### 方針D（設計検討）: Foundation配下ログ制御クレート

承認が得られる場合、Foundation配下に共通ログ制御クレートを導入する。
目的は「ログ実装漏れ防止」と「実装の均一化」。

#### 想定クレート名

- `foundation/logging_foundation`（仮）

> `geo_logging` は Geometry ドメインに見えるため不採用。
> Foundation層の汎用クレートとして、ドメイン非依存の命名を採用する。

#### 依存方針

- `logging_foundation` は `tracing` / `tracing-subscriber` / `std` のみに依存
- 各実装クレート（`render`, `stage`, `viewmodel_*`, `geo_*`, `cam_*`）は
   直接 `tracing_subscriber` を持たず、`logging_foundation` のAPIを利用

#### 提供API（最小）

```rust
pub struct LoggingPolicy {
      pub default_filter: &'static str,
      pub frame_interval: u64,
      pub min_interval_ms: u64,
}

pub fn init_app_logging(policy: LoggingPolicy) -> Result<(), LoggingInitError>;
pub fn should_log_every_n_frames(frame: u64, interval: u64) -> bool;
pub fn should_log_after(last: &mut Instant, interval: Duration) -> bool;
```

#### 実装漏れを防ぐ仕組み（重要）

共通マクロを提供し、呼び出し側の定型コードを最小化する。

```rust
log_frame_trace!(frame, 60, "mesh vertices={}", vertices.len());
log_rate_limited_info!(state.logger_key("camera"), 500, "camera={:?}", camera);
```

期待効果:
- 各クレートで「間引き条件」を毎回手書きしない
- レート制限方式を一元化
- ログ粒度のルール逸脱を減らす

#### 設定の集約

- `RUST_LOG` は従来通り尊重
- 未設定時は `logging_foundation` 側の既定フィルタを適用
- 将来的に `REDRING_LOG_FRAME_INTERVAL` などの環境変数で間隔調整可能にする

#### 導入ステップ（承認後）

1. `foundation/logging_foundation` クレート追加（基盤API + マクロ + 初期化）
2. `view/app/src/logging.rs` を `logging_foundation::init_app_logging` 呼び出しへ縮約
3. `render/stage/viewmodel_*` の高頻度ログをマクロへ置換
4. 既存 `tracing_subscriber` 直接利用箇所を段階的に撤去

---

## 変更対象（初回実装セット）

### 必須セット（Phase 1）

1. `view/app/src/logging.rs`
   - `DEFAULT_LOG_FILTER` 更新
   - 運用コメント追加

2. `view/render/src/line.rs`
   - 反復的 `warn/info` を `trace`/`debug` へ見直し

3. `view/stage/src/*.rs`（5ファイル）
   - 描画ループ系ログの降格

4. `viewmodel/converter/src/nurbs_view.rs`
   - 制御点ダンプ等の大量ログを `trace` 化

### 拡張セット（Phase 2）

5. `view/app/src/app_state.rs`
   - デバッグ表示関連を `debug/trace` 中心へ
   - ユーザー操作イベントのみ `info` 残し

6. `viewmodel/graphics/src/camera.rs`
   - 連続入力由来ログに間引き導入

---

## 実装順序

1. デフォルトフィルタ更新（最小影響）
2. render/stage のフレーム系ログ降格
3. converter の大量ログ降格
4. app_state/camera のイベント分類
5. 必要箇所にレート制限ヘルパ導入

---

## 検証計画

### 機能検証

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- 手動確認:
  - デフォルト起動時にフレーム毎ログが出ない
  - `RUST_LOG=trace` で詳細ログが出る

### 負荷確認（簡易）

- 同一シーンでログON/OFF時のフレーム安定性を比較
- ログファイルサイズ増加速度を比較（一定時間）

---

## 運用ルール（開発者向け）

- ループ内ログは原則 `trace`
- 1操作1回の通知は `info`
- 失敗時のみ `warn/error`
- 大量データ（頂点列・行列フルダンプ）は `trace` に限定
- 新規ログ追加時は「頻度」と「既定表示可否」を必ず確認

---

## 完了条件（Issue #230 対応）

- [ ] フレーム毎ログがデフォルトで出力されない
- [ ] `RUST_LOG` で必要時のみ詳細ログ取得可能
- [ ] 高頻度ログにレート制限（または `trace` 化）が適用されている
- [ ] 本方針に基づく運用ルールが文書化されている

---

## 補足

新規クレート作成案は将来拡張候補とし、今回の実装は既存構成内で完結する。
（承認プロセスとスコープ最小化を優先）
