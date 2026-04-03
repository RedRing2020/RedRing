# Artifact Binary I/O Contract Design

**作成日**: 2026年4月3日  
**ステータス**: v0.1 仕様正本  
**関連Issue**: [#300](https://github.com/RedRing2020/RedRing/issues/300), [#411](https://github.com/RedRing2020/RedRing/issues/411), [#412](https://github.com/RedRing2020/RedRing/issues/412)

---

## 目的

- `toolpath` / `interference` artifact のバイナリ I/O 契約を長期参照可能な正本として固定する
- `Job Manager` がバイナリ本体を解釈しない責務境界を保ちながら、Model/Worker 間の I/O 契約を安定化する
- 後続の reader/writer 実装、命名運用、互換性判断の基準を一意化する

関連ドキュメント:

- [BATCH_COMPUTE_PLATFORM_DESIGN.md](./BATCH_COMPUTE_PLATFORM_DESIGN.md)
- [ARCHITECTURE.md](./ARCHITECTURE.md)

---

## 責務境界

- ジョブ契約は `JobType + InputRef -> ResultRef` を維持する
- `job_runtime` / Job Manager はバイナリ本体を解釈しない
- reader/writer 実装責務は Model 側、初期対象は `cam_core` とする
- 利用層は `read_artifact_v1` を標準入口として使用し、個別 payload の直接復元を乱立させない

---

## 共通ヘッダ仕様

全 artifact は先頭に固定長ヘッダを持つ。

- `magic: [u8; 4]`
  - `toolpath`: `RRTP`
  - `interference`: `RRIN`
- `version_major: u16` = 0
- `version_minor: u16` = 1
- `units: u8`
  - 1 = Millimeter
- `coordinate_frame: u8`
  - 1 = WorldRightHandedZUp
- `reserved: [u8; 4]`
- `payload_len: u64`

エンディアンは little-endian 固定とする。

---

## ToolPath Payload v0.1

### ファイル単位メタ

- `tool_id_len: u16`
- `tool_id_bytes: [u8; tool_id_len]` (UTF-8)
- `cutting_direction: u8`
  - 0 = Down
  - 1 = Up
- `approach_count: u32`
- `contour_level_count: u32`
- `retract_count: u32`

### セグメント共通

1セグメントは次を保持する。

- `segment_type: u8`
  - 0 Cutting
  - 1 Rapid
  - 2 Approach
  - 3 Retract
  - 4 PassRetract
- `feed_rate: f64`
  - Rapid は `0.0`
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

### Contour Level

- `level_index: u32`
- `z_level: f64`
- `segment_count: u32`
- `segments[]`

---

## Interference Payload v0.1

最小契約として接触イベント列を保持する。

- `event_count: u32`
- `events[]`
  - `sample_index: u32`
  - `tool_id_len: u16`
  - `tool_id_bytes: [u8; tool_id_len]`
  - `position: [f64; 3]`
  - `normal: [f64; 3]`
  - `penetration_depth: f64`
  - `kind: u8`
    - 1 Tool
    - 2 Holder
    - 3 Shank

`kind = 3` は shank 属性追加後の正式値として扱う。

---

## 互換性ポリシー

- `major == 0 && minor == 1`:
  - accept
- `major == 0 && minor != 1`:
  - reject
- `major != 0`:
  - convert 実装がある場合のみ convert、未実装時は reject

v0 系はプレ版とし、安定化までは strict reject を基本とする。

### バージョン同期ルール

`ToolPath` 構造体変更により wire layout が変わる場合、以下で同期する。

- 互換な変更:
  - `minor` を +1
- 非互換変更:
  - `major` を +1 し、`minor` を 0 に戻す

実装では `cam_core::toolpath` 側の schema version と artifact header version の同値性をテストで検証する。

---

## 回帰テスト運用

Read/Write の片側だけが変わって契約が壊れることを防ぐため、以下を必須ゲートとする。

- `toolpath_wire_write_regression`
- `toolpath_wire_read_regression`
- `interference_wire_write_regression`
- `interference_wire_read_regression`

運用ルール:

- ゴールデンバイト列を `artifact_binary.rs` に保持する
- CI の wire format 検証ステップを必須にする
- ゴールデン定数を更新する場合は `version_minor` も更新する

---

## 公開 API と命名運用

- `read_artifact_v1` は標準入口とする
- `write_toolpath_payload_v1` / `write_interference_payload_v1` は v0.1 契約の writer として扱う
- `*_v1` の `v1` は wire format v1.0 ではなく API 世代識別子を表す
- wire format 判定は常に `version_major` / `version_minor` の実値で行う

標準注記テンプレート:

```text
注記: `*_v1` は API 世代識別子を表す。wire format の実バージョンは
`version_major` / `version_minor` の実値で判定する（現行は v0.1）。
```

---

## 後続参照ルール

- `#411` は読込導線の標準入口として本書を参照する
- `#412` は API 名称と wire format 表記の関係整理で本書を参照する
- `#257` / `#260` 相当の後続実装は、本書の責務境界・ヘッダ仕様・互換性ポリシーを前提にする