# Issue #300 実施準備チェックリスト

対象Issue: [#300: [Batch Platform] カッターパス/干渉結果バイナリのレイアウト仕様策定](https://github.com/RedRing2020/RedRing/issues/300)

## 1. 目的

- `toolpath` / `interference` のバイナリレイアウト v0.1（プレ版）を固定する
- `Job Manager` が中身を解釈しない責務境界を保ちながら、Model/Worker 間の I/O 契約を安定化する
- 後続Issue（#260, #257）が参照可能な基準仕様を先に確立する

## 2. 前提と責務境界

- ジョブ契約: `JobType + InputRef -> ResultRef`
- `job_runtime` / Job Manager はバイナリ本体を解釈しない
- Reader/Writer 実装責務は Model 側（初期対象: `cam_core`）

関連ドキュメント:
- `dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md`

## 3. v0.1 共通ヘッダ仕様

全アーティファクト共通で先頭に固定長ヘッダを持つ。

- `magic: [u8; 4]`
  - `toolpath`: `RRTP`
  - `interference`: `RRIN`
- `version_major: u16` = 0
- `version_minor: u16` = 1
- `units: u8`
  - 1 = Millimeter
- `coordinate_frame: u8`
  - 1 = WorldRightHandedZUp
- `reserved: [u8; 4]` (将来拡張)
- `payload_len: u64` (ヘッダ以降のバイト長)

エンディアン: little-endian 固定

## 4. toolpath v0.1 ペイロード仕様

### 4.1 ファイル単位メタ

- `tool_id_len: u16`
- `tool_id_bytes: [u8; tool_id_len]` (UTF-8)
- `cutting_direction: u8`
  - 0 = Down
  - 1 = Up
- `approach_count: u32`
- `contour_level_count: u32`
- `retract_count: u32`

### 4.2 セグメント共通

1セグメントは次を保持:

- `segment_type: u8`
  - 0 Cutting, 1 Rapid, 2 Approach, 3 Retract, 4 PassRetract
- `feed_rate: f64`
  - Rapid は 0.0
- `start: [f64; 3]`
- `geometry_type: u8`
  - 0 Line
  - 1 Arc
- `line_end: [f64; 3]` (Line時のみ)
- `arc_end: [f64; 3]` (Arc時のみ)
- `arc_center: [f64; 3]` (Arc時のみ)
- `arc_direction: u8` (Arc時のみ)
  - 0 Clockwise
  - 1 CounterClockwise

### 4.3 contour level

- `level_index: u32`
- `z_level: f64`
- `segment_count: u32`
- `segments[]`

## 5. interference v0.1 ペイロード仕様（最小）

切削/干渉結果の最小契約として、接触イベント列を持つ。

- `event_count: u32`
- `events[]`
  - `sample_index: u32`
  - `tool_id_len: u16`
  - `tool_id_bytes: [u8; tool_id_len]`
  - `position: [f64; 3]`
  - `normal: [f64; 3]`
  - `penetration_depth: f64`
  - `kind: u8` (1 Tool, 2 Holder, 3 Shank)

備考: #260 の shank 属性追加で `kind=3` を正式利用する。

## 6. 互換ポリシー（accept/reject/convert）

- 同一 major (`0.x`) かつ既知 minor:
  - accept
- 同一 major (`0.x`) だが未知 minor:
  - プレ版のため原則 reject（明示的に互換テスト済み minor のみ accept）
- major 不一致:
  - reject（別コンバータがある場合のみ convert）

### 6.0 実装固定ルール（Issue #300 時点）

- `major == 0 && minor == 1`:
  - accept
- `major == 0 && minor != 1`:
  - reject（v0系は strict reject）
- `major != 0`:
  - convert（専用コンバータ実装時） / 未実装時は reject

注記: convert は判定結果として返す契約を先に固定し、実変換処理は後続Issueで実装する。

### 6.1 構造体変更時のバージョン同期ルール

`ToolPath` 構造体変更により wire layout が変わる場合、以下の規則でバージョンを同期する。

- バイナリ互換な変更（末尾への任意フィールド追加、既存既定値で復元可能）:
  - `minor` を +1
- 非互換変更（既存フィールド削除/型変更/意味変更、必須フィールド追加）:
  - `major` を +1 し、`minor` を 0 に戻す

実装方式（可能）:

- `cam_core::toolpath` に `const TOOLPATH_SCHEMA_VERSION: (u16, u16)` を定義
- `artifact_binary` の `FORMAT_VERSION_MAJOR_V1` / `FORMAT_VERSION_MINOR_V1` と同値であることを `#[cfg(test)]` で検証
- CIで round-trip テストに加え、`unknown version` reject テストを必須化

注記: v0系（プレ版）の間は後方互換を保証しない前提で、安定化までは strict reject を基本とする。

### 6.2 wire layout 回帰テスト運用ルール（CI ゲート）

Read/Write の片方だけ変更されて読み書き不可になることを防ぐために以下を運用する。

- `GOLDEN_TOOLPATH_MINIMAL` / `GOLDEN_INTERFERENCE_MINIMAL` をゴールデン定数として `artifact_binary.rs` に保持する
- 4つの回帰テストが CI の必須ゲートになっている:
  - `toolpath_wire_write_regression`: Writer が変わると失敗
  - `toolpath_wire_read_regression`: Reader が変わると失敗
  - `interference_wire_write_regression`: interference Writer が変わると失敗
  - `interference_wire_read_regression`: interference Reader が変わると失敗
- CI ステップ `Verify artifact binary wire format (cam_core)` が `feature_ci.yml` / `develop_ci.yml` に追加済み
- ゴールデン定数を更新する場合は必ず `FORMAT_VERSION_MINOR_V1` を +1 する

## 7. 実装タスク（初回）

- [x] `cam_core` に共通ヘッダ Reader/Writer 実装を追加
- [x] `toolpath` 用の最小 Writer 実装（1経路）を追加
- [x] `toolpath` 用の最小 Reader 実装（round-trip）を追加
- [x] `interference` はヘッダ + 空イベント列の最小 Reader/Writer を追加
- [x] 単体テストで round-trip を検証
- [x] wire layout 回帰テスト（ゴールデンバイト）を追加し CI ゲートに組み込み
- [x] `read_artifact_v1` 統合エントリーポイントを実装し、`ensure_acceptable_version` を構造的に強制

## 8. 完了条件

- [x] v0.1 ヘッダ仕様が文書化されている
- [x] `toolpath` / `interference` の最小バイナリを生成・読込できる
- [x] unknown version の reject 動作がテスト化される
- [x] `ToolPath` 構造体の変更時にバージョン同期テストが失敗し、更新漏れを検知できる
- [x] wire layout 変更時に Reader/Writer 片方だけ変更した場合に CI が失敗する
- [x] `ensure_acceptable_version` が読み込みパスで構造的に強制される（`read_artifact_v1` 経由）
- [x] `cargo fmt` / `cargo test -p cam_core` が通る

## 9. 着手時点メモ（2026-03-24）

- `cam_core::toolpath` は `PathSegment` / `PathGeometry` を既に保持しており、v0.1 シリアライズ対象に直接マッピング可能
- Issue #338 系の intersection 戻り値再編とは独立に進行可能

## 9.1 後続 Issue への v0.1 I/O 契約ベースライン（Issue #300 完了時点）

本ドキュメントは Issue #257（`NcPostFromCam` 出力契約）および Issue #260（バッチ実行パイプライン）が参照すべき
v0.1 バイナリ I/O 契約のベースライン仕様書として機能する。

- Issue #257 が参照すべき契約: Section 2（Reader/Writer 責務境界）、Section 5（wire 値一覧）、本 Section
- Issue #260 が参照すべき契約: Section 3（ヘッダ仕様）、Section 6（互換性ポリシー）、本 Section
- これらの Issue が v0.1 形式を前提として実装を進める際は、`read_artifact_v1` / `write_toolpath_payload_v1` / `write_interference_payload_v1` を公開 API として使用すること
- 本ドキュメントの Section 6 互換性ポリシーを変更する場合は、#257/#260 のレビュー時に影響を明示すること

## 10. 実装埋没を防ぐ運用ルール（Issue #300）

Issue本文の可読性を維持しつつ、実施機能の埋没を防ぐために次を運用する。

- Issue本文は「目的 / 受け入れ条件 / 関連Issue」の要点のみを保持する
- 実装の詳細タスク、wire値、互換方針の更新履歴は本ドキュメントに集約する
- PR本文には「Issue #300 受け入れ条件との対応表」を必ず記載する
- マージ後は本ドキュメントのチェックリストを更新し、Issue本文には完了サマリのみ追記する

対応表の最小テンプレート（PR記載用）:

- AC1: `toolpath` / `interference` のv0.1仕様文書化 -> 本ドキュメント該当節
- AC2: Reader/Writer責務境界の明文化 -> Section 2
- AC3: 互換性ポリシー（accept/reject/convert） -> Section 6
- AC4: `NcPostFromCam` 参照可能な入出力契約 -> 関連Issue/後続タスクにリンク

## 11. 命名運用注記（#412 決定反映）

- `read_artifact_v1` / `ArtifactHeaderV1` の `V1` は API 世代識別子を表す
- wire format の判定は `version_major` / `version_minor` の実値で行う（現行 v0.1）
- 正規参照先: `dev/architecture/ARCHITECTURE.md` の「Artifact API 命名運用ルール（#415）」
- 標準注記テンプレート:

```text
注記: `*_v1` は API 世代識別子を表す。wire format の実バージョンは
`version_major` / `version_minor` の実値で判定する（現行は v0.1）。
```
