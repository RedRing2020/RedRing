# Issue #300 実施準備チェックリスト

対象Issue: [#300: [Batch Platform] カッターパス/干渉結果バイナリのレイアウト仕様策定](https://github.com/RedRing2020/RedRing/issues/300)

## 1. 目的

- `toolpath` / `interference` のバイナリレイアウト v1 を固定する
- `Job Manager` が中身を解釈しない責務境界を保ちながら、Model/Worker 間の I/O 契約を安定化する
- 後続Issue（#260, #257）が参照可能な基準仕様を先に確立する

## 2. 前提と責務境界

- ジョブ契約: `JobType + InputRef -> ResultRef`
- `job_runtime` / Job Manager はバイナリ本体を解釈しない
- Reader/Writer 実装責務は Model 側（初期対象: `cam_core`）

関連ドキュメント:
- `dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md`

## 3. v1 共通ヘッダ仕様

全アーティファクト共通で先頭に固定長ヘッダを持つ。

- `magic: [u8; 4]`
  - `toolpath`: `RRTP`
  - `interference`: `RRIN`
- `version_major: u16` = 1
- `version_minor: u16` = 0
- `units: u8`
  - 1 = Millimeter
- `coordinate_frame: u8`
  - 1 = WorldRightHandedZUp
- `reserved: [u8; 4]` (将来拡張)
- `payload_len: u64` (ヘッダ以降のバイト長)

エンディアン: little-endian 固定

## 4. toolpath v1 ペイロード仕様

### 4.1 ファイル単位メタ

- `tool_id_len: u16`
- `tool_id_bytes: [u8; tool_id_len]` (UTF-8)
- `cutting_direction: u8`
  - 1 = Down
  - 2 = Up
- `approach_count: u32`
- `contour_level_count: u32`
- `retract_count: u32`

### 4.2 セグメント共通

1セグメントは次を保持:

- `segment_type: u8`
  - 1 Cutting, 2 Rapid, 3 Approach, 4 Retract, 5 PassRetract
- `feed_rate: f64`
  - Rapid は 0.0
- `start: [f64; 3]`
- `geometry_type: u8`
  - 1 Line
  - 2 Arc
- `line_end: [f64; 3]` (Line時のみ)
- `arc_end: [f64; 3]` (Arc時のみ)
- `arc_center: [f64; 3]` (Arc時のみ)
- `arc_direction: u8` (Arc時のみ)
  - 1 Clockwise
  - 2 CounterClockwise

### 4.3 contour level

- `level_index: u32`
- `z_level: f64`
- `segment_count: u32`
- `segments[]`

## 5. interference v1 ペイロード仕様（最小）

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

- 同一 major (`1.x`) かつ既知 minor:
  - accept
- 同一 major だが未知 minor:
  - 既知範囲の後方互換が保証される場合のみ accept
  - それ以外は reject
- major 不一致:
  - reject（別コンバータがある場合のみ convert）

## 7. 実装タスク（初回）

- [ ] `cam_core` に共通ヘッダ Reader/Writer 実装を追加
- [ ] `toolpath` 用の最小 Writer 実装（1経路）を追加
- [ ] `toolpath` 用の最小 Reader 実装（round-trip）を追加
- [ ] `interference` はヘッダ + 空イベント列の最小 Reader/Writer を追加
- [ ] 単体テストで round-trip を検証

## 8. 完了条件

- [ ] v1 ヘッダ仕様が文書化されている
- [ ] `toolpath` / `interference` の最小バイナリを生成・読込できる
- [ ] unknown version の reject 動作がテスト化される
- [ ] `cargo fmt` / `cargo test -p cam_core` が通る

## 9. 着手時点メモ（2026-03-24）

- `cam_core::toolpath` は `PathSegment` / `PathGeometry` を既に保持しており、v1 シリアライズ対象に直接マッピング可能
- Issue #338 系の intersection 戻り値再編とは独立に進行可能
