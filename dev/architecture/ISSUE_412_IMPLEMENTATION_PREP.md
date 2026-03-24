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

## 4. 初回推奨方針（案）

初回は A で明文化を先行し、B/C は将来の破壊的変更許容タイミングで再評価する。

- #412 では命名責務の定義を固定する
- #413 以降の CAM アーキテクチャ整理時に rename コストを再見積もりする
- 現時点の実装変更は最小化し、まずドキュメント規約を統一する

## 5. 影響範囲の棚卸し対象

- `model/cam_core/src/artifact_binary.rs`
- `model/cam_core/src/lib.rs`
- `dev/architecture/ISSUE_300_IMPLEMENTATION_PREP.md`
- `dev/architecture/ISSUE_411_IMPLEMENTATION_PREP.md`
- 利用側（将来）:
  - `cam_sim`
  - `cam_algorithms`（新設検討中）

## 6. #412 で決めるべきこと

- [ ] `V1` の正式意味（API 初版 or wire format 版）
- [ ] 改名する/しないの判定基準
- [ ] 改名しない場合の注記テンプレート
- [ ] 改名する場合の移行手順（互換期間、deprecation 方針）
- [ ] `dev/architecture` での正規参照ドキュメント

## 7. #412 の完了条件

- [ ] API 名称と wire format 表記の対応表が確定する
- [ ] 命名方針（維持 or 改名）が採択される
- [ ] 採択方針に応じた後続 Issue 分割が可能な状態になる

## 8. out-of-scope

- 新規 wire format バージョン追加
- #411 の読込導線実装の再作業
- 5軸 ToolPath 表現や NC 逆変換

## 9. 次アクション（着手時）

1. `artifact_binary.rs` の API 名称と version 定数の対応表を作成
2. 選択肢 A/B/C の比較表を `dev/architecture` に追記
3. 採択方針に応じて、必要なら Phase 実装 Issue を追加起票
