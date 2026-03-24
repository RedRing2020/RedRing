# Issue #411 実施準備チェックリスト

対象Issue: [#411: [Phase1] refactor: CAM結果/シミュレーションの artifact 読込導線を統一](https://github.com/RedRing2020/RedRing/issues/411)

## 1. 目的

- `toolpath` / `interference` artifact の読込入口を、Issue #300 で導入済みの `read_artifact_v1` に統一する
- 上位層でのバージョンチェック漏れを防止する
- `cam_sim` を中心に、artifact を読む利用層で v0.1 I/O 契約を揃える

## 2. 現時点の前提

- Issue #300 で `cam_core::artifact_binary` に以下が導入済み
  - `ArtifactHeaderV1`
  - `ArtifactPayload`
  - `read_artifact_v1`
  - `write_toolpath_payload_v1`
  - `write_interference_payload_v1`
- `read_artifact_v1` は #411 の成果物ではなく、#300 で追加済みの既存 API
- `read_artifact_v1` は `cam_core` 内の実装・テストでのみ利用されている
- 個別 Reader / Writer は公開 API のまま残っている
- v0.1 の互換性ポリシーは `major == 0 && minor == 1` を strict accept とする

### 2.1 命名上の注意

- `read_artifact_v1` / `ArtifactHeaderV1` の `V1` は API/型名上の初版識別子として残っている
- 一方で wire format の実バージョンは `major = 0`, `minor = 1` であり、仕様としては v0.1 を読む
- したがって #411 の成果物は「`read_artifact_v1` を新規実装すること」ではなく、「既存の `read_artifact_v1` を上位層の標準入口として使う導線を揃えること」
- 命名の完全整理（例: `read_artifact_v0_1` への改名）は別判断とし、本 Issue の初回スコープには含めない
- 命名整理の検討は Issue #412 で扱う

注記（#412 決定反映）:

- `*_v1` は API 世代識別子を表す
- wire format の判定は `version_major` / `version_minor` の実値で行う（現行 v0.1）

関連ドキュメント:
- `dev/architecture/ISSUE_300_IMPLEMENTATION_PREP.md`

関連Issue:
- #412 artifact binary API 名称と wire format バージョン表記を整理

## 3. 初回棚卸結果（2026-03-25）

### 3.1 `cam_sim`

現時点で `cam_sim` は artifact バイナリを直接読んでおらず、`ToolPath` を直接入力として受ける。

- `model/cam_sim/src/simulator.rs`
  - `CuttingSimulator::simulate(&ToolPath, &Tool)` を提供
- `model/cam_sim/src/simulator/segments.rs`
  - `ToolPath` から線分列を抽出
- `model/cam_sim/src/tests.rs`
  - `ToolPath::new(...)` で直接テストデータを組み立てている

### 3.2 `cam_algorithms`

- 現ワークスペースに `model/cam_algorithms/` クレートは存在しない
- そのため、Issue #411 の初回スコープでは「将来 `cam_algorithms` 相当の CAM 演算結果利用層が追加された際に、
  `read_artifact_v1` を標準入口にする方針」を文書化対象とする

### 3.3 関連候補層

- `cam_entity`
- `job_runtime`

これらは現時点で artifact payload を直接読んでいないが、`input_ref` / `result_ref` の境界を持つため、
後続フェーズでは導線整理の候補になり得る。

## 4. 責務境界

- `cam_core`
  - artifact の trait定義ではなく、Reader / Writer 実装と payload 型を提供する
- `cam_sim`
  - `ToolPath` / `InterferencePayload` を消費するが、v0.1 では artifact 読込 facade は未提供
- `job_runtime`
  - バイナリ本体を解釈しない
- 将来の CAM 演算結果利用層
  - `read_artifact_v1` を標準入口として利用する

## 5. 実装選択肢

### 選択肢A: 利用層ごとに `ArtifactPayload` を直接 `match` する

利点:
- 実装が単純
- `cam_core` 側 API を増やさずに済む

懸念:
- 各利用層で kind 不一致処理を重複しやすい

### 選択肢B: 利用目的ごとの薄い adapter / facade を追加する

例:
- `read_toolpath_artifact_v1(...) -> Result<(ArtifactHeaderV1, ToolPath<f64>), ...>`
- `read_interference_artifact_v1(...) -> Result<(ArtifactHeaderV1, InterferencePayload), ...>`

利点:
- 上位層の kind 判定重複を減らせる
- 利用意図が明確

懸念:
- `cam_core` 公開 API が増える

### 選択肢C: `cam_sim` 側に artifact 読込専用 facade を持つ

利点:
- `cam_sim` の利用意図に近い API になる

懸念:
- 読込責務の重心が `cam_core` から分散する
- 後続の CAM 演算結果利用層と API 方針が揃わない可能性がある

## 6. 推奨方針（初回案）

初回は **選択肢B** を推奨する。

- `read_artifact_v1` を唯一の低レベル入口とする
- 上位層で必要な kind 固定読込は薄い adapter を追加して吸収する
- `cam_sim` は adapter 経由で `ToolPath` を受け取る側に留め、バイナリ仕様自体の解釈を持ち込まない

この方針なら、`ArtifactPayload` の総称性を維持しつつ、上位層の誤用も減らせる。

### 6.1 初回実装の対象ファイル候補

- `model/cam_core/src/artifact_binary.rs`
  - kind 固定 adapter の追加候補
- `model/cam_core/src/lib.rs`
  - 追加 API の再エクスポート
- `model/cam_sim/src/lib.rs`
  - facade を cam_sim 側へ置く場合の公開境界候補
- `model/cam_sim/src/simulator.rs`
  - 読込済み `ToolPath` を消費する既存本体。初回はここを直接変更しない案を優先
- `model/cam_sim/src/tests.rs`
  - `ToolPath::new(...)` 直組みテストから artifact 読込経由テストを追加検討

### 6.2 adapter API 候補

初回案では `cam_core` に以下の kind 固定 adapter を追加する案を第一候補とする。

- `read_toolpath_artifact_v1<R: Read>(reader: &mut R) -> Result<(ArtifactHeaderV1, ToolPath<f64>), BinaryFormatError>`
- `read_interference_artifact_v1<R: Read>(reader: &mut R) -> Result<(ArtifactHeaderV1, InterferencePayload), BinaryFormatError>`

期待する責務:

- 低レベル読込は内部で `read_artifact_v1` を呼ぶ
- `ArtifactKind` 不一致時は kind mismatch として失敗する
- 上位層は `ArtifactPayload` を直接 `match` しなくてよい

注記:

- kind mismatch 用エラー型が未定義のため、必要なら `BinaryFormatError` へ追加する
- `read_artifact_v1` 自体は引き続き唯一の汎用入口として残す

### 6.3 段階導入順

1. `cam_core` に kind 固定 adapter と必要最小限のエラー契約を追加する
2. `cam_sim` で adapter を使う読込ユースケースをテストで先行導入する
3. `cam_sim` 本体に facade が必要か再評価する
4. 将来の CAM 演算結果利用層が追加された際は同じ adapter を再利用する

### 6.4 現時点の採用判断

- 初回実装では `cam_core` に kind 固定 adapter を追加する
- adapter の内部実装は必ず `read_artifact_v1` を経由する
- kind mismatch は `BinaryFormatError` で明示的に返す
- `cam_sim` 本体 API の変更は次段階に回し、初回は `cam_core` 側の読込面を先に固める

### 6.5 `cam_sim` 薄い facade 再評価（2026-03-25）

結論:

- 現段階では `cam_sim` に artifact 読込用の薄い facade は追加しない

判断理由:

- `cam_sim` 本体（`CuttingSimulator`）は `ToolPath` 消費責務に限定され、バイナリI/O境界を持たない
- `cam_sim` のジョブ連携は `input_ref` の妥当性確認に留まり、artifact 本体解釈は行っていない
- kind 固定読込は `cam_core` の adapter で既に表現でき、責務重複を避けられる

再評価トリガー:

- `cam_sim` が `input_ref` から実バイト列を取得して自前で復元する責務を持つ場合
- 同一復元手順が `cam_sim` 内で複数箇所に増え、呼び出し側の重複が顕在化した場合

## 7. 初回実装スコープ

- [x] `cam_sim` で artifact 読込 facade を必要とする具体的ユースケースを列挙する
- [x] `cam_core` に kind 固定 adapter を追加する前提で API 形状を確定する
- [x] kind 不一致時のエラー契約を定義する
- [x] `cam_sim` テストデータを「直接 `ToolPath::new`」から「artifact 読込経由」に置換すべき範囲を整理する（`simulate` 系テスト2件で先行導入）
- [x] `job_runtime` 境界で解釈しない原則を再確認する

## 8. out-of-scope

- NC 取り込みから `ToolPath` / `Path` への逆変換
- 5軸 ToolPath 表現や姿勢補間
- Gコード固有情報の完全復元
- 新バージョンの artifact 仕様追加

## 9. 完了条件

- [ ] `cam_sim` を中心とした利用層で、`read_artifact_v1` を標準入口とする設計が明文化される
- [ ] 個別 Reader を直接使ってよい境界 / 禁止する境界が整理される
- [ ] 後続の実装 Issue に移れる程度に候補ファイル・テスト観点が整理される

## 10. 実装前に詰める論点

- kind 固定 adapter を `cam_core` に置くか、利用層側に置くか
- `cam_sim` が `InterferencePayload` をどの粒度で受け取るべきか
- 将来の CAM 演算結果利用層が追加された際、同じ facade を共有できるか
