# Geometric Tolerance Usage Rules

## 目的

幾何演算におけるトレランスの責務を明確化し、正本と互換層の混在を段階的に解消するための運用ルールを定義する。

## 設計原則

- 正本は `geo_contracts::ToleranceSettings` とする。
- API入力トレランスは呼び出し元が `ToleranceSettings` から取得し、明示的に渡す。
- スケーリング責務は呼び出し元に置く。
- 互換層は暫定運用とし、新規実装での依存追加を禁止する。

## 現状の3層構造

### 層1: 正本（維持）

- `model/geo_contracts/src/tolerance.rs`
- 提供: `ToleranceSettings<T>`
- 役割: 幾何演算で使用する標準トレランスの単一の情報源

### 層2: 過渡期互換層（段階的廃止）

- `model/geo_contracts/src/tolerance_migration.rs`
- 提供: `DefaultTolerances`, `ScalarToleranceExt`
- 役割: 旧呼び出し経路の暫定サポート
- 方針: 新規利用禁止、`ToleranceSettings` へ順次移行後に削除

### 層3: geo_algorithms レガシー互換層（段階的廃止）

- `model/geo_algorithms/src/tolerance.rs`
- 提供: `ToleranceContext`
- 役割: 旧実装互換の最小定義
- 方針: `ToleranceSettings` ベースへ順次移行後に削除

## 廃止ロードマップ

### フェーズ1（Issue #361）

- 本ドキュメントを整備する。
- 互換層ファイルに廃止予定コメントと参照Issueを明記する。

### フェーズ2（別Issue）

- `geo_primitives` の `DefaultTolerances` 依存を `ToleranceSettings` へ移行する。
- 移行完了後に `model/geo_contracts/src/tolerance_migration.rs` を削除する。

### フェーズ3（別Issue）

- `geo_algorithms` 内で `ToleranceContext` 依存箇所を `ToleranceSettings` ベースへ移行する。
- 移行完了後に `model/geo_algorithms/src/tolerance.rs` を削除する。

### フェーズ3 実績（Issue #377）

- `interpolation.rs` / `numerical.rs` / `statistics.rs` / `sampling.rs` の
	`ToleranceContext` 依存を `geo_contracts::ToleranceSettings<f64>` ベースへ移行。
- `model/geo_algorithms/src/tolerance.rs` を削除。
- `geo_algorithms` 側の新規トレランス定義追加は行わず、
	呼び出し境界で `ToleranceSettings` を受け渡す方針に統一。

## 運用ルール

- 新規コードでは `ToleranceSettings` を使用し、互換層APIを増やさない。
- 互換層に変更を入れる場合は、削除に向けた移行目的を明記する。
- 既存コード移行時は、呼び出し境界でトレランス取得元を統一する。

## 関連Issue

- #361
- #318
- #320
- #360
