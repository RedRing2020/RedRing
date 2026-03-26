# Issue #435 ValidationError i18n 設計

作成日: 2026-03-26  
対象Issue: [#435](https://github.com/RedRing2020/RedRing/issues/435)

## 1. 目的

ValidationError の表現を以下の2層に分離する。

- 内部識別子: 安定キー（ロケール非依存）
- 表示文言: カタログ解決（ロケール依存）

これにより、`cam_core` はドメイン責務に専念し、表示責務は `viewmodel` 側に集約する。
また、ローカライズ共通基盤は `foundation/i18n_foundation` に切り出して RedRing 全体で再利用する。

## 2. 現状整理

- `cam_core::validation::ValidationError` は `Display` で英語文言を直接生成している
- `viewmodel` には `message_catalog` / `job_message_catalog` が既に存在し、キー解決基盤は利用可能
- Job系と ValidationError 系で同一パターンのローカライズ要求が再発している
- ValidationError 向けの安定キー定義と引数マッピングは未導入

## 3. 設計方針（確定）

### 3.1 安定キー形式

- 形式: `validation.<domain>.<reason>`
- 例:
  - `validation.contour.not_closed`
  - `validation.contour.insufficient_points`
  - `validation.machine.feed_rate_limit_exceeded`

命名ルール:
- 単語は snake_case
- 意味変更を伴う rename は禁止
- 互換維持時は旧キーを deprecated alias として一時維持

### 3.2 Display と i18n 解決責務

- `cam_core`:
  - `ValidationError` 自体と検証ロジックを保持
  - 人間向け多言語文言は持たない
- `viewmodel`:
  - `ValidationError -> UiMessage` 変換を担当
  - `message_catalog` で最終表示文言へ解決

補足:
- `Display` はデバッグ用途の固定英語（既存互換）として当面維持
- UI/ログ用途では `UiMessage` 経路を優先使用

### 3.3 メッセージカタログ配置

- 共通抽象は新規クレート `foundation/i18n_foundation` に配置する
  - `UiLocale`
  - `UiMessage`
  - `UiMessageArg`
  - `MessageCatalog` trait
  - `TableMessageCatalog`
  - `resolve_message`
- ValidationError 用カタログ実体は `viewmodel/converter` 配下に置く
- Job系の既存 message catalog も段階的に `i18n_foundation` 利用へ寄せる
- JA/EN の静的テーブルを `TableMessageCatalog` で解決

### 3.4 `i18n_foundation` の責務境界

- 文字列キー解決の共通抽象を提供する
- ロケールと引数付きメッセージの共通型を提供する
- 静的テーブルベースの最小実装を提供する

非スコープ:
- JSON/YAML ローダー
- 外部ファイル監視
- 複数形や高度な ICU 相当機能
- ドメイン固有のキー命名規則そのもの

### 3.5 フォールバック規約

解決順序:
1. 指定ロケールのキー
2. 既定ロケール（`en`）のキー
3. キー文字列そのもの

`i18n_foundation::resolve_message` 呼び出し層で統一適用する。

## 4. データ境界

ValidationError のロケール非依存表現:

- `key: String`
- `args: Vec<(name, value)>`

代表的な引数:
- `distance`, `tolerance`
- `point_count`
- `segment_index`, `axis_name`
- `value_mm`, `max_feed_rate` など

数値の丸め・単位表記は表示層（テンプレート）で扱う。

## 5. テスト戦略

### 5.1 cam_core

- ValidationError 生成ロジックの既存テストを維持
- キー解決依存を追加しない（ドメイン独立を保持）

### 5.2 viewmodel

- `ValidationError -> UiMessage` 変換テスト
- キー存在テスト（JA/EN 両方）
- フォールバックテスト（指定 -> 既定 -> キー）
- 既存メッセージ解決との互換テスト

### 5.3 i18n_foundation

- `TableMessageCatalog` の共通テスト
- フォールバック規約テスト
- 既存 `viewmodel` 実装からの移設互換テスト

## 6. フェーズ分割（実装Issue化）

### Phase 1

- ValidationError に安定キー導出APIを導入
- 既存 `Display` は維持（互換重視）

### Phase 2

- `foundation/i18n_foundation` クレートを追加
- 既存 `message_catalog` 共通抽象を `i18n_foundation` へ移設
- `viewmodel` に ValidationError 変換層を追加
- ValidationError 用 message catalog（JA/EN）を追加
- フォールバック規約を実装

2026-03-26 追記（#442 残タスク）:
- `validation_message_mapper` で `ValidationError -> UiMessage` を一元化する
- `validation_message_catalog` で JA/EN テーブルを提供する
- フォールバックは `指定ロケール -> En -> key文字列` をテストで担保する

### Phase 3

- UI/ログの呼び出し経路を新経路へ段階移行
- 旧経路の利用箇所を縮退

## 7. PR/Issue 運用ルール

- 親Issue #435 を起点に、実装PRは必ず `Refs #435` を付与
- 実装Phaseが完了したPRのみ `Closes <Phase Issue>` を使用
- 設計のみPRでは `Closes #435` を使わない
- 互換影響がある変更は PR 本文の先頭で明示する

## 8. 非スコープ

- 本設計Issueではコード実装を行わない
- ValidationError 以外の全エラー型の同時移行は行わない

## 9. 完了判定

以下を満たした時点で #435 を完了可能とする。

- 方針が本ドキュメントで確定している
- Phase 1/2/3 の実装Issueが定義済み
- PR/Issue 運用ルール（Refs/Closes）が明文化済み
