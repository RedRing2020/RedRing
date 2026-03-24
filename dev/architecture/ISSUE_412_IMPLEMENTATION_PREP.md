# Issue #412 実施準備チェックリスト

対象Issue: [#412: [Design] refactor: artifact binary API 名称と wire format バージョン表記を整理](https://github.com/RedRing2020/RedRing/issues/412)

## 1. 目的

- artifact binary API 名称と wire format バージョン表記の責務を分離して整理する
- `V1` が「API 初版」か「wire format 版」かを明文化する
- 改名の要否を判断し、後続実装で混乱しない基準を確定する

## 2. 現状整理（2026-03-25）

### 2.1 現在の定数・型・関数

`model/cam_core/src/artifact_binary.rs` では以下が併存している。

- `FORMAT_VERSION_MAJOR_V1 = 0`
- `FORMAT_VERSION_MINOR_V1 = 1`
- `KNOWN_MINOR_VERSIONS_V0 = &[1]`
- `ArtifactHeaderV1`
- `read_artifact_v1`
- `read_toolpath_artifact_v1`
- `read_interference_artifact_v1`

### 2.2 混乱ポイント

- API 名称は `V1` だが、wire format の実値は `v0.1`
- 初見で「v1.0 を読む API」に見える
- `ArtifactHeaderV1` が `version_major == 1` を想定しているように見える

### 2.3 API 名称と wire format 表記の対応表（確定）

| 区分 | 現行シンボル | 実際の意味 | wire format 実値 | 根拠 |
| --- | --- | --- | --- | --- |
| 定数 | `FORMAT_VERSION_MAJOR_V1` | 現行実装が受理する major の基準値 | `0` | `artifact_binary.rs` 定義 |
| 定数 | `FORMAT_VERSION_MINOR_V1` | 現行実装が受理する minor の基準値 | `1` | `artifact_binary.rs` 定義 |
| 定数 | `KNOWN_MINOR_VERSIONS_V0` | major=0 系で受理する minor 一覧 | `[1]` | `artifact_binary.rs` 定義 |
| 型 | `ArtifactHeaderV1` | 現行 API 世代のヘッダ型 | `version_major=0`, `version_minor=1` を内包 | `ArtifactHeaderV1::new` の初期化値 |
| 関数 | `read_artifact_v1` | 現行 API 世代の統合 reader | 入力の `v0.1` を strict accept | `ensure_acceptable_version` 呼び出し |
| 関数 | `read_toolpath_artifact_v1` | ToolPath 向け adapter reader | 内部で `read_artifact_v1` を使用 | `artifact_binary.rs` 実装 |
| 関数 | `read_interference_artifact_v1` | Interference 向け adapter reader | 内部で `read_artifact_v1` を使用 | `artifact_binary.rs` 実装 |
| 関数 | `write_toolpath_payload_v1` | ToolPath payload writer | payload 自体に version は持たない（ヘッダ別管理） | `artifact_binary.rs` 実装 |
| 関数 | `write_interference_payload_v1` | Interference payload writer | payload 自体に version は持たない（ヘッダ別管理） | `artifact_binary.rs` 実装 |

補足:

- 本対応表の確定により、`*_v1` 命名は「wire format v1.0」ではなく「API 世代識別子」として扱う暫定運用を採用する
- wire format のバージョン判定は引き続き `version_major/minor` の実値で行う

## 3. 命名整理の選択肢

### 選択肢A: 現行名称維持 + 意味を明文化

例:
- `ArtifactHeaderV1` は「API 初版」を意味する
- wire format は `version_major/minor` で管理する

利点:
- 公開 API 互換を維持できる
- 影響範囲が最小

懸念:
- 新規参加者に説明コストが残る

### 選択肢B: wire format 連動名称へ改名

例:
- `read_artifact_v0_1`
- `ArtifactHeaderV0_1`
- `write_toolpath_payload_v0_1`

利点:
- 名称と wire format の対応が直感的

懸念:
- 公開 API 変更が発生
- 移行期間の互換運用が必要

### 選択肢C: 二層命名（互換 alias 併存）

例:
- 実体は wire format 連動名称へ寄せる
- 既存 `*_v1` は互換 alias として段階廃止

利点:
- 可読性と移行安全性の両立

懸念:
- 一時的に API 面が重複し複雑化する

## 4. 最終採択方針（2026-03-25）

#412 では **選択肢A（現行名称維持 + 意味を明文化）を採択** する。

- `*_v1` は「wire format v1.0」ではなく「API 世代識別子」として扱う
- wire format 判定は `version_major` / `version_minor` の実値で行う
- 実コード改名は行わない
- 将来改名はトリガー発火時のみ再評価する

### 4.1 改名する/しないの判定基準（確定）

改名しない（現時点の採択）:

- 公開 API 互換を優先する場合
- 現行利用者に対する移行コストを避ける場合
- v0.1 単一運用で、命名注記により誤解を十分抑制できる場合

改名を再検討する:

- `version_minor` の増加（例: v0.2）で API と wire format の混同が顕著化した場合
- 主要利用層（`cam_sim` / 将来 `cam_algorithms`）で命名誤解による障害が発生した場合
- 破壊的変更を許容できるリリース計画が確保された場合

### 4.2 改名しない場合の注記テンプレート（確定）

以下を設計書・PR本文で共通利用する。

```text
注記: `*_v1` は API 世代識別子を表す。wire format の実バージョンは
`version_major` / `version_minor` の実値で判定する（現行は v0.1）。
```

### 4.3 将来改名する場合の移行手順（草案）

1. wire format 連動名称（例: `read_artifact_v0_1`）を追加する
2. 既存 `*_v1` を互換 alias として併存させる
3. `#[deprecated]` と移行期間を明示する
4. 利用側更新（`cam_sim` / 将来 `cam_algorithms`）を完了する
5. 次のメジャー互換境界で `*_v1` alias を削除する

## 5. 影響範囲の棚卸し対象

- `model/cam_core/src/artifact_binary.rs`
- `model/cam_core/src/lib.rs`
- `dev/architecture/ISSUE_300_IMPLEMENTATION_PREP.md`
- `dev/architecture/ISSUE_411_IMPLEMENTATION_PREP.md`
- 利用側（将来）:
  - `cam_sim`
  - `cam_algorithms`（新設検討中）

### 5.1 既存テスト・公開 API への影響整理

- 公開 API: 変更なし（rename なし）
- 既存テスト: 変更なし（命名運用を文書化するのみ）
- 互換性: 現行利用コードはそのまま継続利用可能

## 6. #412 で決めるべきこと

- [x] `V1` の正式意味（API 初版 or wire format 版）
- [x] 改名する/しないの判定基準
- [x] 改名しない場合の注記テンプレート
- [x] 改名する場合の移行手順（互換期間、deprecation 方針）
- [x] `dev/architecture` での正規参照ドキュメント

正規参照ドキュメント:

- 本書（`dev/architecture/ISSUE_412_IMPLEMENTATION_PREP.md`）
- 運用ルール固定 Issue（#415）

## 7. #412 の完了条件

- [x] API 名称と wire format 表記の対応表が確定する
- [x] 命名方針（維持 or 改名）が採択される
- [x] 採択方針に応じた後続 Issue 分割が可能な状態になる（#415 起票済み）

## 8. out-of-scope

- 新規 wire format バージョン追加
- #411 の読込導線実装の再作業
- 5軸 ToolPath 表現や NC 逆変換

## 9. 次アクション（#412 完了後）

1. #415 で運用ルールの文書適用範囲を固定する
2. v0.2 以降の計画時に、改名トリガーの発火有無を再評価する
