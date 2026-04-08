# ViewModel→Model導線再定義設計

**作成日**: 2026年4月9日
**関連Issue**: #500
**ステータス**: 設計整理中

---

## 1. 背景

- `model/application` を導入し、ViewModel と Model の間にユースケース責務を集約する方針は始まっている
- ただし現状は `cam_orchestration` の一部 PoC がある一方で、`feature_orchestration` / `geometry_orchestration` / `job_orchestration` は骨格段階に留まる
- `geo_entity` / `geo_topology` / `job_runtime` はそれぞれ独立した語彙を持ち始めたが、1つのユースケース内でどう接続するかは未固定
- そのため、ViewModel 起点の導線を実装しようとすると、geometry / topology / entity / job を同列に並べて扱うべきか、処理段階で分けるべきかが曖昧になる

本設計では、#501 以降の実装前提として、ViewModel→Application→Model の最小導線と責務境界を固定する。

---

## 2. 現状整理

### 2.1 既に存在する基盤

- `model/application`
  - `cam_orchestration` は PoC 実装あり
  - `feature_orchestration` / `geometry_orchestration` / `job_orchestration` は marker 段階
- `model/geo_entity`
  - `GeometricEntity<T, G>`、`EntityId`、表示属性、属性・関係管理を保持
- `model/geo_topology`
  - `Vertex` / `Edge` / `Wire` / `CompositeCurve3D` と検証語彙を保持
- `model/job_runtime`
  - `JobManager`、`JobExecutor`、`JobEvent`、`JobType + InputRef -> ResultRef` 契約を保持
- `model/cam_sim`
  - `CamJobExecutorAdapter` により `job_runtime` との接続スタブが存在

### 2.2 まだ未固定の点

- ViewModel 起点の command が `Application` を経由して Model 更新まで到達する標準導線がない
- Entity 更新は PoC では deterministic な ID 組み立てに留まり、Model 側ストレージ更新には至っていない
- Topology は語彙と検証機能を持つが、Application の use case から呼ばれる位置が未定
- Job は実行方式として独立語彙を持つが、geometry / topology / entity の更新とどう関係づけるかが未整理
- `view/app` の `EntityManager` は View ローカル状態であり、Model 正本ではない

---

## 3. 基本判断

### 3.1 Application は「Model の一部」ではなく「Model 直上のユースケース層」とする

- `Application Layer` は ViewModel から見た安定した入口を提供する
- geometry / topology / entity / job の内部詳細は Application の下に隠す
- ViewModel は request/result DTO を扱い、処理順序や実行方式選択を持たない

### 3.2 geometry / topology / entity / job は同列の実行レーンとして扱わない

4者は用途が異なる。

- geometry: 幾何生成・更新の正本
- topology: 幾何間の接続・整合性の正本
- entity: 識別子、表示属性、メタデータ、参照単位の正本
- job: 実行方式と進捗・成果物管理の正本

したがって、これらを「常に並列に処理する 4 系統」として設計しない。
設計上は以下のように分ける。

1. geometry / topology / entity は **Model 状態を構成する語彙**
2. job は **実行方式を切り替える語彙**
3. Application は use case ごとに、必要な語彙だけを順序付きで組み合わせる

### 3.3 初期導線は command 系で直列を基本にする

初期の非ECS導線では、形状作成・形状編集のような Model 更新を伴う command 系ユースケースに対して、以下の直列を基本とする。

1. 入力検証
2. geometry 更新
3. 必要であれば topology 更新 / 検証
4. entity 更新
5. result DTO 生成

query 系ユースケースや単純参照系ユースケースにはこの直列をそのまま適用しない。

job はこの直列の内部段階ではなく、
「同期実行するか」「job 化して後で結果を受けるか」を切り替える外側の実行方式として扱う。

### 3.4 command 系と query 系の区別

- command 系:
  - Model の正本状態を更新するユースケース
  - 例: 形状作成、形状編集、feature 実行、entity 更新、job submit
- query 系:
  - Model の正本状態を更新せず、既存状態や計算結果を参照して返すユースケース
  - 例: entity 一覧取得、topology 検証結果参照、job 状態照会、既存結果の表示用変換

本書で「初期導線」と呼ぶ直列は command 系に対する基本順序であり、query 系は read-only な参照導線として別に扱う。

---

## 4. 責務境界

### 4.1 ViewModel

- UI 入力を request DTO に正規化する
- result / error を表示向け DTO に変換する
- 非責務:
  - geometry 生成順序の決定
  - topology 構築判断
  - entity 更新規則の保持
  - job 実行方式の詳細分岐

### 4.2 Application Layer

- use case 単位の入口を定義する
- geometry / topology / entity / job の利用順序を決める
- sync / async / job 投入の切り替え点を持つ
- 下位エラーを上位エラーへ正規化する
- 非責務:
  - 幾何計算アルゴリズム本体
  - topology データ構造の詳細実装
  - entity 属性モデルの詳細実装
  - job 状態遷移の低レベル制御

### 4.3 geometry

- 形状の生成・更新・評価を担う
- 出力は形状本体または feature 出力語彙とする
- entity ID や job 状態を持ち込まない

### 4.4 topology

- geometry 間の接続、拘束端点、連続性、整合性検証を担う
- geometry 更新結果から必要に応じて導出・検証される
- command すべてで必須ではないが、位置づけは geometry の後段とする

### 4.5 entity

- geometry / topology を参照可能な識別単位を提供する
- 表示属性、メタデータ、関係管理を担う
- `feature_id + output_index + local_key` からの決定的 ID 生成は entity 側の語彙として扱う
- View ローカルの選択状態管理とは分離する

### 4.6 job

- 長時間実行の受付、状態遷移、再実行、成果物参照、進捗通知を担う
- geometry / topology / entity の意味論は持たない
- `Application` から見た job は optional な execution port であり、Model 更新語彙そのものではない

---

## 5. 推奨する導線モデル

### 5.1 同期 command 導線

```text
ViewModel Event
  -> Application Request DTO
  -> Feature UseCase Orchestration
  -> Geometry update
  -> Topology derive/validate (if needed)
  -> Entity assemble/store
  -> Result DTO
  -> ViewModel
```

この経路を、#501 の最小 command 系実装対象とする。

### 5.2 job 導線

```text
ViewModel Event
  -> Application Request DTO
  -> Job submission port
  -> job_runtime
  -> executor adapter
  -> JobEvent / result_ref
  -> Application query/result assembly
  -> ViewModel
```

job 導線は同期 command 導線の代替 execution mode であり、
geometry / topology / entity と同一階層の並列レーンとして定義しない。

### 5.3 Topology の扱い

- Topology は独立した UI command 群から直接更新を始めるのではなく、geometry 更新の後段で導出・検証される位置を基本とする
- ただし将来、明示的なトポロジー編集機能を追加する場合は、専用 use case を `Application` に追加してよい

---

## 6. Application module の再解釈

### 6.1 feature_orchestration

- 各 UI command に近い use case 入口を持つ
- 1つの feature 編集要求に対して geometry / topology / entity の順序を束ねる
- 初期の ViewModel→Model 導線の主入口はここに置く

### 6.2 geometry_orchestration

- 単なる `operation_name` の marker ではなく、geometry 更新の adapter / port 群へ発展させる
- ただし UI から直接叩く主入口ではなく、feature orchestration の下位協調点として使う

### 6.3 job_orchestration

- submit / status / cancel / retry の実行方式境界を担う
- geometry / topology / entity を直接保持しない
- 既存 `job_runtime` / `cam_sim::CamJobExecutorAdapter` との接続窓口として育てる

### 6.4 entity / topology 専用 orchestration は初手では追加しない

- 初期段階では `entity_orchestration` や `topology_orchestration` を独立 module として増やさない
- 理由は、use case 入口が増えるだけで責務が明確にならず、かえって導線が分散するため
- entity / topology は feature orchestration 配下の port として扱う方が自然である

---

## 7. 初期 port 境界

初期段階で固定する port は以下を推奨する。

1. `FeatureCommandPort`
   - ViewModel から受ける use case 入口
2. `GeometryMutationPort`
   - geometry 生成 / 更新
3. `TopologyDerivationPort`
   - geometry から topology を導出 / 検証
4. `EntityStorePort`
   - entity の生成 / 更新 / 参照
5. `JobDispatchPort`
   - submit / status / cancel / retry

このうち `JobDispatchPort` は optional とし、#501 の最小同期導線では必須にしない。

---

## 8. #501 へ向けた実装順序

### Phase 0: #500 で固定すること

- ViewModel→Application→Model の最小同期導線を文章で固定する
- geometry / topology / entity / job の責務を同列化しないと明記する
- Application module の役割を再定義する

### Phase 1: #501 で実装すること

- 非ECS の 1 use case を `feature_orchestration` 入口で通す
- geometry 更新結果を entity 更新までつなぐ
- topology は optional port として位置だけ固定し、対象 use case で必要なら導入する
- result / error を ViewModel に返す

### Phase 2: 後続で拡張すること

- job_orchestration と `job_runtime` の接続を `Application` 正本に寄せる
- Topology を必須にする use case を追加し、導出 / 検証 port を実装する
- ECS 差し替え時は `GeometryMutationPort` / `EntityStorePort` / `JobDispatchPort` の背後を置換する

---

## 9. 判断メモ

- `view/app` の `EntityManager` は View ローカル状態であり、Model 側 entity 正本の代替にしない
- `cam_orchestration` の `ToolEntityManagementOrchestrator` は現状では deterministic ID 組み立て PoC と位置づけ、Model ストレージ更新の完成形と見なさない
- `job_runtime` はすでに execution 語彙を持っているため、#500 では job 独自モデルを再発明せず接続点だけ定義する
- `geo_topology` は shape 意味論の代替ではなく、接続・整合性語彙として geometry 後段に置く

---

## 10. 関連文書

- `dev/architecture/APPLICATION_ORCHESTRATION_LAYER_DESIGN.md`
- `dev/architecture/ARCHITECTURE.md`
- `dev/architecture/VIEWMODEL_ARCHITECTURE_DESIGN.md`
- `dev/architecture/ENTITY_VIEW_INTEGRATION_DESIGN.md`
- `dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md`
