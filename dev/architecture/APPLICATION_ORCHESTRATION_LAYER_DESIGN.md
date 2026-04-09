# Application Orchestration Layer Design

**作成日**: 2026年3月29日
**関連Issue**: #335
**ステータス**: 設計検討中

---

## 1. 背景

- 現在、ユースケース実行責務が `job_domain` / `cam_sim` / `viewmodel` / `view` に分散している
- ViewModel 側に処理実行導線が残ると、表示変換責務と業務フロー責務が混在する
- 将来の ECS / 非ECS 差し替えを ViewModel 側で吸収すると、導線が不安定になりやすい

本設計では、ViewModel と domain/algorithm/job 系クレートの間に **Application Layer** を導入し、
その主要責務を **orchestration** として定義する。

---

## 2. 命名方針

### 2.1 層名

- 論理層名は **Application Layer** とする
- `usecase` や `workflow` は説明語としては使用可能だが、層の総称には使用しない

### 2.2 クレート/モジュール名

- Application Layer の実体は `*_orchestration` を基本命名とする
- 例:
  - `job_orchestration`
  - `cam_orchestration`
  - `feature_orchestration`
  - `geometry_orchestration`

### 2.3 命名理由

- `usecase` は個別機能を連想しやすく、複数処理の調停責務を表しにくい
- `workflow` は順序制御には適するが、port/adapter の選択や境界DTOの集約まで含めるにはやや狭い
- `orchestration` は「どこに委譲するか」「どういう順序で進めるか」「どの境界で返すか」をまとめて表現しやすい

---

## 3. 役割定義

Application Layer は、通知や入力を受けた後の処理の進め方を決定し、下位機能を調停する層とする。

### 3.1 責務

1. **入口APIの統一**
   - ViewModel や将来の外部入口から見える呼び出し窓口を安定化する
   - 下位クレートの内部型をそのまま公開しない

2. **委譲先の選択**
   - request 内容に応じて、どの domain / algorithm / runtime / adapter を利用するかを決定する
   - ECS / 非ECS の差分吸収点をここに置く

3. **実行順序の調停**
   - 検証、変換、domain 実行、job 投入、結果整形などの順序を管理する

4. **境界DTO変換**
   - 上位層向け request/result DTO と下位層向け DTO の橋渡しを行う

5. **エラー境界の正規化**
   - 下位層の詳細エラーを、上位層が扱いやすい契約へ寄せる

6. **実行文脈の管理**
   - ユースケース単位で必要な context、依存、設定、関連リソースを束ねる

7. **port契約の維持**
   - 下位実装を差し替えても上位APIを崩さないようにする

8. **実行方式境界の明示**
   - 同期実行、非同期実行、job 投入のどれを選ぶかを決める
   - ただし task/thread/queue/executor などの実装詳細は adapter / runtime に委譲する

### 3.2 非責務

- 幾何計算アルゴリズムそのものは持たない
- CAM 計算ロジックそのものは持たない
- `job_runtime` の低レベル状態遷移や内部実行制御は持たない
- UI 表示用の最終変換や文言解決は持たない
- 永続化の実装詳細は持たない

---

## 4. 依存境界

```text
View / ViewModel
      ↓
Application Layer (`*_orchestration`)
      ↓
Domain / Algorithm / Runtime / Adapter
  - geo_*
  - cam_*
  - job_*
```

### 4.1 上位境界

- ViewModel は orchestration の request/result DTO を扱う
- ViewModel は domain 固有の内部制約や adapter 切り替え方針を持たない

### 4.2 下位境界

- `job_domain` は投入制約やポリシー判定を担当する
- `job_runtime` は実行制御、状態遷移、イベント契約を担当する
- `cam_sim` は計算接続アダプタ責務へ寄せる
- `geo_algorithms` / `cam_algorithms` は純粋計算責務に留める

---

## 5. port / adapter 方針

### 5.1 port の責務

- orchestration から見た依存先を trait で抽象化する
- ECS / 非ECS / batch / local 実装差分を port の背後に隠す

### 5.2 adapter の責務

- 下位クレートや実行基盤の具体実装へ接続する
- 非同期実装、イベント配送、永続化アクセスなどの詳細を持つ

### 5.3 初期方針

- 最初から全ユースケースを port 化しない
- PoC 対象の導線で必要な最小 port から始める
- adapter 差し替えが必要になる箇所から先に切り出す

---

## 6. 既存クレートとの整理方針

### 6.1 現在の位置づけ

- `job_domain`: ドメインポリシー、投入制約、WorkflowSnapshot などの domain 寄り責務
- `job_runtime`: 実行基盤
- `cam_sim`: CAM 特化の実行ファサードと接続アダプタが混在

---

## 7. group relation 系 use case の扱い

group は display trait ではなく entity relation の一種として扱う。
そのため Application Layer では、geometry 更新系 use case と分離した relation-oriented use case として入口を定義する。
relation モデル全体としては、group / layer を親子階層ではなく並列軸で保持する。

### 7.1 group orchestration の責務

1. group 作成
2. entity の group 所属追加/解除
3. group 可視状態の更新
4. group 一覧、entity ごとの所属 group 一覧、group 配下 entity 一覧の query
5. group 可視状態変更時の影響 entity 集合の集約

### 7.2 推奨 port 境界

初期段階では以下の 2 port に分ける。

1. `GroupMutationPort`
   - group 作成
   - group 可視状態更新
2. `EntityGroupRelationPort`
   - entity の group 所属追加/解除
   - entity→group / group→entity 参照

この分離により、group 自体の state と membership relation を別実装へ逃がせる。

### 7.3 request / result DTO 例

```rust
pub struct GroupDto {
   pub group_id: GroupId,
   pub name: String,
   pub visible: bool,
   pub member_count: usize,
}

pub struct CreateGroupRequest {
   pub name: String,
   pub visible: bool,
}

pub struct CreateGroupResult {
   pub group: GroupDto,
}

pub struct AddEntityToGroupRequest {
   pub entity_id: EntityId,
   pub group_id: GroupId,
}

pub struct AddEntityToGroupResult {
   pub entity_id: EntityId,
   pub group_id: GroupId,
   pub memberships: Vec<GroupMembershipDto>,
}

pub struct SetGroupVisibilityRequest {
   pub group_id: GroupId,
   pub visible: bool,
}

pub struct SetGroupVisibilityResult {
   pub group: GroupDto,
   pub affected_entity_ids: Vec<EntityId>,
}

pub struct ListGroupsForEntityQuery {
   pub entity_id: EntityId,
}

pub struct ListGroupsForEntityResult {
   pub entity_id: EntityId,
   pub groups: Vec<GroupDto>,
}
```

### 7.4 設計上の注意

- group の `visible` は「強い ON/OFF」を成立させるため、Application query で取得可能な state とする
- entity が複数 group に属する前提のため、`AddEntityToGroupResult` は単一 membership 成否だけでなく更新後 membership 集合を返してよい
- `SetGroupVisibilityResult.affected_entity_ids` を返すことで、View 側は全件再構築ではなく影響範囲ベースの再評価へ拡張しやすくなる
- group は render batch key ではないため、Application DTO も shader/material 単位の情報は持たない
- Application は group / layer / entity の `visible` state と deny 優先の `visible_policy` を正本として返し、最終的な `effective_visible` 解決は ViewModel 側で行う
- 初期段階では `entity_visible` / `groups_all_visible` / `layer_visible_or_true` のような bool 群で十分だが、それらが並列軸か上位/下位段かは `visible_policy` で説明できるようにする
- `visible_policy` の正本は固定 `order` 値ではなく、parallel / sequential の構造 node を基本とし、leaf が `visible`、`source`、必要に応じて `source_id` を持つ形を優先する
- `source` は entity / group / layer / instance_placement / instance_root / instance_cluster などの判定由来を表し、`source_id` は型付き enum として、単一 ID だけでなく `group_id + entity_id`、`group_ids + entity_ids`、`instance_id + placement_element_key` のような複合文脈も保持できるようにする
- 要素種類については、点・線・面などごとに空間を分けることだけを合意事項とし、`Element` を独立 source にするかは下位データ設計と実装選択に委ねる
- instance 配置は「実体は 1 つでも placement は複数持てる」前提とし、visible state は entity 共有ではなく placement ごとに持てるようにしておく
- entity が複数 group に属する場合、Application は group 単位の visible 判定だけでなく「どの entity に効いた group 判定か」を source_id payload から辿れるようにしておく
- なお Group source の payload 形は未確定であり、単一 `group_id + entity_id` で十分か、`group_ids + entity_ids` をまとめた GroupContext が必要かは実装直前に再確認する
- placement 系の識別子体系は visibility 専用に再定義せず、外部の配置機構が管理する語彙を受け取って参照する
- 点・線・面などの要素種類で表示制御したくなった場合も、visibility 側ではまず要素種類ごとの空間を切り替える前提で扱い、実装が ECS かどうかは後付けのシステム用語として分離する
- ViewModel へ平坦な評価順が必要な場合は、構造 node から都度導出した evaluation order を渡してよいが、その順序数値を契約として固定しない
- `viewmodel/converter`: DTO 変換に加えて debug 実行導線が一部残存

## 8. layer relation 系 use case の扱い

layer も group と同様に entity relation の一種として扱うが、役割は図面・表示管理の基準単位とする。
そのため Application Layer では、group と並ぶ relation-oriented use case として layer の入口も定義する。

### 8.1 layer orchestration の責務

1. layer 作成
2. entity の layer 所属追加/解除
3. layer 可視状態の更新
4. layer 一覧、entity ごとの所属 layer、layer 配下 entity 一覧の query
5. layer 可視状態変更時の影響 entity 集合の集約

### 8.2 推奨 port 境界

初期段階では以下の 2 port に分ける。

1. `LayerMutationPort`
   - layer 作成
   - layer 可視状態更新
2. `EntityLayerRelationPort`
   - entity の layer 所属追加/解除
   - entity→layer / layer→entity 参照

これにより、layer 自体の state と membership を分離できる。

### 8.3 request / result DTO 例

```rust
pub struct LayerDto {
   pub layer_id: LayerId,
   pub name: String,
   pub visible: bool,
   pub member_count: usize,
}

pub struct CreateLayerRequest {
   pub name: String,
   pub visible: bool,
}

pub struct CreateLayerResult {
   pub layer: LayerDto,
}

pub struct AddEntityToLayerRequest {
   pub entity_id: EntityId,
   pub layer_id: LayerId,
}

pub struct AddEntityToLayerResult {
   pub entity_id: EntityId,
   pub layer_id: LayerId,
   pub membership: LayerMembershipDto,
}

pub struct SetLayerVisibilityRequest {
   pub layer_id: LayerId,
   pub visible: bool,
}

pub struct SetLayerVisibilityResult {
   pub layer: LayerDto,
   pub affected_entity_ids: Vec<EntityId>,
}
```

### 8.4 設計上の注意

- layer の `visible` は group と同様、Application query で取得可能な state とする
- layer 所属追加では既存 layer 所属がある場合を Application 境界で正規化エラーへ変換する
- layer は単一所属前提のため、更新後所属集合ではなく単一 `membership` または `Option<LayerDto>` を返す契約を優先する
- layer は group より表示管理寄りだが、初期 DTO では既定色・既定線種の継承責務まで持ち込まない
- group / layer の両方が非表示フィルタとして働くため、影響 entity 集合は両 relation から計算できる形を保つ
- 最終表示可否そのものは ViewModel が `visible_policy` に従って `effective_visible` を解決し、Application はそのための正本 state を返す役割に留める
- `visible_policy` の実装形は初手では軽量でよく、`VisibilityPolicyNode::Sequential` / `Parallel` と leaf の `visible` / `source` / `source_id` を組み合わせた加工しやすい構造を優先する
- これにより multi-group 所属時でも、どの group が非表示理由になったかを ViewModel と差分更新処理で共通に扱える
- 将来 object instance の配置を許容する場合は、instance placement / instance root / instance cluster の visible state と `cluster -> instance`、`instance -> entity` index を既存 policy tree の上位 node として追加できることを前提にする
- この上位 node が false のときは配下 instance 群の group / layer 判定を省略でき、同一 entity を参照する別 placement は独立に表示継続できるため、表示判定の効率化と個別制御を両立できる
- さらに instance 内で entity 単位の表示制御操作が行われた場合は、その操作を `instance_id + placement_element_key` に正規化し、同一 instance 内の同一要素へは同じ表示挙動を適用する一方、別 instance には波及させない
- ここで `placement_element_key` は外部の配置機構が供給する「instance 内で同一要素を指す識別子」を仮置きした語であり、visibility 側が独自に生成規則を持つべきではない

### 8.5 UI ハイブリッド運用との関係

- Application の request / result DTO は、UI が layer 中心ツリーを構成できるだけの情報を返してよい
- ただし DTO 契約でも `Layer > Group` の親子制約は持ち込まない
- UI 側の階層表示は query 結果の再構成として実現し、正本 relation は並列軸のまま維持する

### 6.2 移行方針

- `job_domain` は domain policy に集中させる
- `cam_sim` は計算接続アダプタ責務へ縮退させる
- ViewModel / View に残る debug 実行導線は orchestration 側へ移す

---

## 7. 最小PoC候補

Issue #335 の最小 PoC は、既存の debug snapshot / toolpath 可視化導線を Application Layer へ移す案を第一候補とする。

### 理由

- ViewModel / View に残っている実行責務を直接減らせる
- `job_domain` / `job_runtime` の既存責務と衝突しにくい
- orchestration 導入の効果を小さい変更で確認しやすい

### 初期候補

- `cam_orchestration`
  - debug snapshot series の生成要求を受ける
  - simulation 実行経路を選択する
  - ViewModel 向け境界DTOへ返す

---

## 8. 段階導入案

1. **Phase 0: 設計固定**
   - 本文書で責務、命名、境界、PoC対象を固定する

2. **Phase 1: 骨組み導入**
   - Application Layer 用の最小クレート/モジュールと依存ルールを追加する

3. **Phase 2: PoC移設**
   - debug snapshot / toolpath 導線のうち 1 系統を orchestration へ移す

4. **Phase 3: 展開計画**
   - job / feature / geometry 系へ横展開する

### 8.1 適用ポリシー（合意事項）

- 初期導入は **PoC から段階適用** を基本方針とする
- 初手で `Application/*_orchestration` の複数クレート分割は行わない
- まずは単一の Application 層受け皿を置き、内部を module 分割で開始する
- PoC は `cam_orchestration` 相当の最小導線を優先する

### 8.2 module から crate への昇格基準

以下のいずれかを満たす場合に、`*_orchestration` module の独立クレート化を検討する。

1. 依存先が明確に分岐し、同居すると依存ルールが複雑化する
2. 公開API契約（request/result/error/port）が安定し、他機能と独立に版管理したい
3. テスト戦略や変更頻度が明確に分かれ、分離で保守性が上がる
4. architecture check の運用上、独立レイヤー管理した方が違反検出が明確になる
5. PoC後の実運用で責務境界が固定され、再統合より分離維持の方が低コストと判断できる

---

## 9. 未決事項

- 初期配置を `model/` 直下の単一受け皿クレートにするか、既存クレート内 module から開始するか
   - 方針: PoC では単一受け皿 + module 分割を優先し、複数クレート化は 8.2 の基準で判断する
- `job_orchestration` の初回導入タイミング（PoC直後か、Phase 3 展開時か）
- architecture check における Layer Group 名を `Application` として追加する時期

---

## 10. 関連文書

- `dev/architecture/ARCHITECTURE.md`
- `dev/architecture/VIEWMODEL_MODEL_ROUTE_REDEFINITION_DESIGN.md`
- `dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md`
- `dev/architecture/VIEWMODEL_ARCHITECTURE_DESIGN.md`

---

## 11. cam_core 型露出方針（2026-03-29 追記）

Application Layer の PoC 以降では、CAM 計算・表示連携・制約適用の実要件を踏まえ、
`cam_core` の ToolPath / MachineConstraint / ToolSet 系の詳細型を orchestration 境界で扱うことを許容する。

### 11.1 方針

1. ToolPath / ToolSet / 機械特性の正本は `cam_core` とする
2. Application Layer は干渉チェック、姿勢制御、制約適用に必要な項目へ直接アクセスしてよい
3. ViewModel は最終表示形式の生成に集中し、表示向け投影を担当する
4. `cam_core` への依存は Application Layer で許可し、再エクスポートに依存しない

### 11.2 境界の使い分け

1. Command 系（編集/実行）
   - 詳細型を受け取る境界を許容する
2. Query 系（一覧/概要表示）
   - 軽量 DTO を返す境界を優先する

### 11.3 非責務の維持

上記を許容しても、Application Layer は次を持たない。

- CAM 計算アルゴリズム本体
- `job_runtime` の低レベル状態遷移実装
- UI 文言解決や最終表示表現

---

## 12. Phase 2 PoC 実施メモ（2026-03-29）

debug snapshot 導線のうち、cam_sim export DTO の正規化を Application Layer へ移設した。

### 12.1 実施内容

1. `model/application/src/cam_orchestration.rs` に境界DTOを追加
   - `CamSnapshotFrame`
   - `CamSnapshotSeriesRequest`
   - `CamSnapshotSeriesResult`
2. orchestration 実装を追加
   - `CamSnapshotSeriesOrchestrator`
   - `create_snapshot_series_from_exports`
3. `viewmodel/converter/src/snapshot_converter.rs` から
   `cam_sim` export DTO の直接整形ロジックを外し、Application 経由へ変更

### 12.2 意図

- ViewModel 側に残っていた実行責務の一部を削減し、境界変換を Application 層へ寄せる
- `cam_sim` の出力形式に追従する責務を converter 単体で持たないようにする

### 12.3 継続項目

- toolpath + simulation 実行そのものの導線移設は次段階で実施する
- Query 系の軽量DTO境界を別途定義する