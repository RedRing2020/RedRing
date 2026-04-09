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
- そのため `EntityManager` の命名は shape 名ではなく、現在保持している描画表現を示す語彙を優先する
  - 現行PoCでは `LineList` 頂点列を保持するため `line_list_*` を基本とする

---

## 3. 基本判断

### 3.0 relation モデル方針

- group / layer は、正本モデルでは親子階層ではなく並列の relation 軸として扱う
- 一方で UI 表示では、layer 主体の階層ビューや group を折りたたんだ擬似階層ビューを提供してよい
- つまり採用方針は「正本モデルは並列軸、UI はハイブリッド運用」とする
- 理由は以下の通り
  1. group の複数所属を維持したい
  2. layer の強さに group を従属させると、横断的な作業単位としての group が弱くなる
  3. View / ViewModel では階層 UI を後から構成できるが、正本モデルを階層固定すると横断 relation へ戻しにくい
- このため `Layer > Group` の親子制約は初期設計へ入れない

#### 補記: 並列軸を採用する理由

- 過去の CAD では `Layer > Group` 型の階層で group 機能が layer に強く従属し、横断的な束ねや再利用が制限されやすかった
- 本設計では、その制約を正本モデルへ持ち込まないことを優先する
- 特に以下を守るため、group を layer の子にしない
  1. 複数所属 group の維持
  2. 作業単位・レビュー単位・機能単位など、layer をまたぐ横断 grouping
  3. UI 表示方式と正本 relation 構造の分離
- 一方で layer 中心 UI 自体は有用なため、操作感は階層的に見せつつ、内部 model は並列 relation のまま保つ
- つまり本判断は「階層 UI を否定する」のではなく、「階層 UI を正本構造へ昇格させない」ためのものである

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

### 3.5 描画導線は 2D / 3D を分離して扱う

- ViewModel から View へ返す描画用 DTO / キャッシュは、shape 名ではなく描画カテゴリで分離する
- 少なくとも以下の 2 系統を先に固定する
  1. 2D 定義平面ベースの線・曲線表示
  2. 3D 空間ベースの wireframe / mesh 表示
- 目的は shape 種別の増加ではなく、描画条件・キャッシュ条件・更新条件の分離にある
- したがって `Circle2D` と `LineSegment2D` は同じ 2D 表示カテゴリに入る可能性があり、`LineSegment3D` と `Arc3D` は同じ 3D wireframe 表示カテゴリに入る可能性がある

### 3.6 2D 線種表示は視点条件付きで扱う

- 2D 線種（点線、破線、一点鎖線、二点鎖線など）は、定義平面上の線長と位相を保って見える場合にのみ正しく表示できる
- そのため初期方針として、2D 線種は「ビュー始点が対象の定義平面に対して直交」と判定できる場合にのみ表示対象とする
- 直交条件を満たさない場合は以下のいずれかに正規化する
  1. 非表示
  2. 実線へフォールバック
- どちらを採用するかは use case ごとに決められるが、少なくとも ViewModel / View の契約として「2D 線種は常時表示される」とはみなさない
- この制御は表示最適化でもあり、2D 線種専用のパターン生成や GPU 転送を不要な視点で抑制する目的を持つ

### 3.7 表示属性 contract は common / stroke / planar-2d に分割する

- `geo_contracts` の entity trait は、全 entity に共通な属性と、線表現にのみ必要な属性を分ける
- 現在の `EntityDisplayProperties` は `visible` と `color` のみを持つが、ここへ線種や定義平面を直接追加しない
- 理由は、mesh や solid 表示まで同じ trait に引きずられ、不要な責務が common trait に混ざるためである
- 方針は以下とする
  1. `EntityDisplayProperties`
    - 共通属性のみ
    - `visible`, `color`
  2. `StrokeDisplayProperties`
    - 線表現に必要な属性
    - `line_style`, `line_width`
  3. `PlanarStroke2DProperties<T>`
    - 2D 線種表示条件に必要な属性
    - `definition_plane_origin`, `definition_plane_normal`, `definition_plane_u_axis`
- 2D の点線・破線・一点鎖線・二点鎖線は `PlanarStroke2DProperties<T>` を満たす entity / DTO にのみ要求する
- 3D wireframe は `StrokeDisplayProperties` までは要求してよいが、`PlanarStroke2DProperties<T>` は要求しない

### 3.8 group は表示属性ではなく関係語彙として扱う

- group は shape 種別や表示属性と同列に持つのではなく、entity の所属関係として扱う
- group 所属は多対多を前提とし、1 entity が複数 group に属することを許可する
- したがって group は `EntityDisplayProperties` / `StrokeDisplayProperties` / `PlanarStroke2DProperties<T>` には入れない
- group は render batch key の主軸にしない
- group の主目的は以下とする
  1. UI ツリーや一覧での論理分類
  2. group 単位の表示 ON/OFF
  3. group 単位の一括選択・一括属性変更
  4. query 系ユースケースでのまとまり取得

### 3.9 group 単位表示 ON/OFF は deny 優先で解釈する

- group 表示制御は entity 単体の `visible` より上位のフィルタとして扱う
- entity が複数 group に属する場合、1つでも非表示 group に所属していればその entity は非表示とする
- つまり有効表示判定は以下の論理積で定義する

```text
effective_visible = entity.visible AND groups_all_visible
groups_all_visible = 所属 group が空なら true、1つ以上あるなら全 group.visible が true
```

- この規則により、複数所属を許しつつ group 単位 ON/OFF を強い制御として扱える
- 一方で描画最適化の観点では、group ごとに GPU バッファを持つことは初期方針としない
- group 変更は render batch の再分類ではなく、表示対象フィルタの再評価として扱う

### 3.10 layer も同時に relation 語彙として固定する

- layer も group と同様に display trait へ混在させず、entity relation として扱う
- ただし layer は group と役割が異なる
  1. group: 複数所属を前提にした論理的束ね単位
  2. layer: 図面・表示管理の基準単位
- layer は将来のロック、印刷対象、編集対象制限、既定色・既定線種の継承元になりやすいため、group より表示管理寄りの語彙として扱う
- 一方で初期段階では layer も render batch key の主軸にはしない
- layer は単一所属を前提とし、1 entity は最大 1 layer にのみ属する
- したがって現設計では layer を group のような複数所属 relation としては扱わない
- layer は group の親ではなく、別軸の表示管理単位として扱う

### 3.11 layer 可視制御も deny 優先で扱う

- layer も group と同様に entity 単体の `visible` より上位のフィルタとする
- entity は最大 1 layer 所属とし、その layer が非表示なら entity は非表示とする
- 有効表示判定は以下の論理積に拡張する

```text
effective_visible = entity.visible AND groups_all_visible AND layers_all_visible
groups_all_visible = 所属 group が空なら true、1つ以上あるなら全 group.visible が true
layers_all_visible = 所属 layer が空なら true、所属 layer があればその layer.visible
```

- この規則により、group と layer のどちらも強い ON/OFF 制御として扱える
- 表示の意味論としては layer の方が図面管理寄り、group の方が横断的な論理束ね寄りである
- ただし初期描画実装では、どちらも cache key ではなく可視フィルタとして合成する

### 3.12 visible_policy の表現方針

- 初期段階の DTO / query 応答では、`entity_visible`、`groups_all_visible`、`layer_visible_or_true` のような bool 群で十分とする
- ただし bool 群だけでは「どの条件が並列軸で、どの条件が上位/下位の優先関係か」が将来拡張しにくいため、Application/Model 側には `visible_policy` を別語彙として保持する
- 将来的に object instance の配置と instance 構造の塊単位での表示制御を導入する前提では、instance placement / instance root / instance cluster を `visible_policy` の最上位段として扱える形にしておく
- ここで instance placement は「実体 entity は 1 つでも、その参照配置 instance は複数持てる」ものとし、表示切替は placement ごとに独立して制御できる前提とする
- `visible_policy` は固定数値の `order` を仕様値として持つ前提にせず、parallel / sequential の構造と node の親子関係から評価順を表現できればよい
  1. 同順位の並列フィルタであること
  2. 上位/下位の評価段であること
  3. deny 優先で合成すること
-  4. どの軸の判定結果かを追跡できること
- 例えば `VisibilityPolicyNode::Sequential(vec![ ... ])` の配下に `VisibilityPolicyNode::Parallel(vec![ ... ])` を持つような構造で、上位/下位段と並列条件を同時に表現できる
- source 候補は少なくとも `Entity`、`Group`、`Layer`、`InstancePlacement`、`InstanceRoot`、`InstanceCluster`、`ViewCondition` を持てるようにしておき、要素種類の扱いは「要素種類ごとに空間を分ける」を合意事項として固定する
- instance root / instance cluster を group / layer より上位の node に置ければ、該当塊全体が非表示な時点でその配下 placement / entity の下位判定を短絡できる
- `source` は判定由来を表し、`source_id` は単一の汎用 ID ではなく型付き enum を優先し、group_id / layer_id / instance_id / cluster_id に加えて複合文脈を保持できるようにする
- 検討案としては `VisibilitySourceId::Entity { entity_id }`、`VisibilitySourceId::GroupContext { group_ids, entity_ids }` のような形がありうる
- `Element` については、点・線・面などの要素種類ごとに空間を分けることだけをこの段階の合意事項とする
- その内部表現を Entity + Geometry / Topology の type 判別で持つか、将来別の実装機構へ載せ替えるかは後続の実装選択に委ねる
- このため現段階では `Element` を独立 source に固定せず、要素種類ごとの空間分離を前提に Entity 側または下位実装側で吸収する
- 特に Group は単一 group_id だけでなく group_id 群と entity_id 群を束ねて持てるようにしておくと、複数 group 所属時の説明と差分再評価を崩しにくい
- instance 内 entity 単位の表示操作は `VisibilitySourceId::InstanceElement { instance_id, element_key }` のような識別子で表し、「同一 instance 内の同一要素」には同じ表示挙動を適用し、別 instance の同名要素とは分離して扱う
- これにより「表示ON/OFF」と「評価優先順位」を同じデータ構造から取り出せるため、enum に数値を埋め込んでビットフラグ風に扱うより加工しやすい
- さらに `effective_visible = false` になった理由を ViewModel 側で説明可能になり、`affected_instance_ids` / `affected_entity_ids` の計算時にもどの source を起点に差分再評価すべきかを揃えやすい
- 将来の上位層追加時も、既存 node の上に新しい親 node を挿入して整合性を保てばよく、固定 `order = 100` のような予約値運用を避けられる
- ただし初期実装では、外部 DTO に複雑な policy 配列を露出するより、bool 群 + `visible_policy` 正本語彙の併用を優先する
- ViewModel / DTO 境界で平坦化が必要な場合に限り、構造から導出した evaluation order を一時的に使ってよいが、その数値は永続的な契約値にしない

例:

```rust
pub enum VisibilitySourceId {
  Entity { entity_id: EntityId },
  Group { group_id: GroupId, entity_id: EntityId },
  GroupContext { group_ids: Vec<GroupId>, entity_ids: Vec<EntityId> },
  Layer { layer_id: LayerId, entity_id: EntityId },
  InstancePlacement { instance_id: InstanceId },
  InstanceRoot { root_id: InstanceRootId },
  InstanceCluster { cluster_id: InstanceClusterId },
  InstanceElement { instance_id: InstanceId, element_key: PlacementElementKey },
  ViewCondition { view_id: ViewId, condition_key: ViewConditionKey },
}
```

- 単純な単一 ID より variant ごとの payload を持てるため、group 複数所属や instance 内 element 操作のような複合ケースを `source_id` の時点で正規化できる
- ただし上記 enum 形はまだ Fix しておらず、現時点では「Group に group_id 群 / entity_id 群を持たせる案」を中心に検討し、`Element` は要素種類ごとの空間分離を前提に下位実装へ委ねる
- `PlacementElementKey` は現時点の仮名であり、最終的に `entity_local_key` へ固定することはまだ決めない
- `PlacementElementKey` は visibility 側が定義する ID ではなく、将来必要な配置系・参照系から外部供給される「instance 内で同一要素を指すキー」として扱う
- visibility policy はその外部供給キーを参照して表示判定に使うが、配置識別子の体系そのものをここで定義しない

---

## 4. 責務境界

### 4.1 ViewModel

- UI 入力を request DTO に正規化する
- result / error を表示向け DTO に変換する
- 2D / 3D 描画カテゴリの分類を行う
- 2D 線種の表示可否判定に必要な View 条件を受け取り、表示条件を満たさない 2D 線種を描画要求へ昇格させない
- `EntityDisplayProperties` / `StrokeDisplayProperties` / `PlanarStroke2DProperties<T>` のどこまでを使うかで DTO の描画カテゴリを決める
- Application/Model が保持する `visible_policy` と View から渡る表示条件を用いて `effective_visible` を解決し、描画要求へ反映する
- bool 群だけで解決できる初期ケースでも、`visible_policy` が示す parallel / upper / lower の規約に加え、`source` / `source_id` による原因軸の識別を保ったまま評価順を固定する
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
- group 作成、entity の group 所属追加/解除、group 可視状態更新、group 単位 query の入口を持つ
- layer 作成、entity の layer 所属追加/解除、layer 可視状態更新、layer 単位 query の入口を持つ
- entity / group / layer の `visible` state と deny 優先の `visible_policy` を正本語彙として保持し、ViewModel が `effective_visible` を計算できる材料を返す
- 将来の instance 配置導入を見据え、Application は instance placement / instance root / instance cluster の `visible` state と relation index も同じ正本系列へ拡張できるようにしておく
- 初期 DTO では bool 群を返してよいが、それがどの policy 軸に対応するかは `visible_policy` 側で一意に説明できるようにする
- `visible_policy` の各 leaf node は少なくとも `source` を持ち、必要に応じて型付き enum の `source_id` で group 文脈・layer 文脈・instance 文脈を識別できるようにする
- 可視状態更新では `affected_instance_ids` と必要に応じて `affected_entity_ids` を返せる構造を優先し、最上位 instance cluster が非表示になった場合はその配下 placement だけを再評価対象へ絞り込めるようにする
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
- ただし placement 系で必要になる複合識別子は別の語彙として扱い、entity 単体 ID と混同しない
- View ローカルの選択状態管理とは分離する
- group / layer などの所属関係は entity relation として保持し、display trait へ混在させない

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

### 5.4 表示カテゴリ別の View ローカル導線

```text
Application Result DTO
  -> ViewModel display classification
  -> 2D display cache or 3D display cache
  -> View stage rebuild
```

- 2D cache は「定義平面」「ビュー条件」「線種パターン」をキーに持つ
- 3D cache は「wireframe / mesh」「表示属性」「dirty 状態」をキーに持つ
- これにより、shape 名を増やしても cache 単位を増やしすぎない
- group / layer は cache key には入れず、可視フィルタと UI 操作の単位として別管理する

### 5.5 UI はハイブリッド表示を許可する

- 正本モデルが並列軸であっても、UI では layer 主体のツリー表示を構成してよい
- 例えば以下の表示は許可する
  1. layer 一覧の下に、その layer に関連する group を補助表示する
  2. group 一覧の下に、その group に属する entity を表示する
  3. entity 詳細で所属 layer / 所属 group を別セクションで見せる
- ただしこれは UI 表示モデルであり、Model の relation 構造そのものを `Layer > Group` 階層へ固定することは意味しない
- これにより、CAD 的な layer 中心 UI を維持しつつ、group の横断的利用も阻害しない

### 5.6 group command / query 導線

```text
ViewModel Event
  -> Application Group Request DTO
  -> entity relation update or group query
  -> group/result DTO
  -> ViewModel
```

- command 例:
  1. group 作成
  2. entity を group へ追加
  3. entity を group から除外
  4. group 可視状態を更新
- query 例:
  1. group 一覧取得
  2. entity の所属 group 一覧取得
  3. group 配下 entity 一覧取得
- これらは geometry 更新とは別の entity relation 系 use case として扱う

### 5.7 group request / result DTO の最小契約

group 機能は entity relation 系 use case として `feature_orchestration` 直下、または同等の group-oriented orchestration で受ける。
初期段階では以下の request / result DTO を最小契約とする。

```rust
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

pub struct RemoveEntityFromGroupRequest {
  pub entity_id: EntityId,
  pub group_id: GroupId,
}

pub struct RemoveEntityFromGroupResult {
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

pub struct ListGroupsQuery;

pub struct ListGroupsResult {
  pub groups: Vec<GroupDto>,
}

pub struct ListGroupsForEntityQuery {
  pub entity_id: EntityId,
}

pub struct ListGroupsForEntityResult {
  pub entity_id: EntityId,
  pub groups: Vec<GroupDto>,
}

pub struct ListEntitiesForGroupQuery {
  pub group_id: GroupId,
}

pub struct ListEntitiesForGroupResult {
  pub group_id: GroupId,
  pub entity_ids: Vec<EntityId>,
}

pub struct GroupDto {
  pub group_id: GroupId,
  pub name: String,
  pub visible: bool,
  pub member_count: usize,
}

pub struct GroupMembershipDto {
  pub entity_id: EntityId,
  pub group_id: GroupId,
}
```

- `CreateGroupRequest.visible` は初期表示状態であり、group ON/OFF を View ローカル専用状態へ閉じ込めないために必要とする
- `SetGroupVisibilityResult.affected_entity_ids` は View 側が差分再評価を行うための最小情報として返す
- `ListEntitiesForGroupResult` は初期段階では `EntityId` のみ返せばよく、表示用 summary は別 query として分けてよい
- `GroupDto.member_count` は UI ツリーや group 一覧に必要な集約値であり、Application で整形して返す

### 5.8 layer request / result DTO の最小契約

layer 機能も entity relation 系 use case として group と同時に定義する。
初期段階では以下の request / result DTO を最小契約とする。

```rust
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

pub struct RemoveEntityFromLayerRequest {
  pub entity_id: EntityId,
  pub layer_id: LayerId,
}

pub struct RemoveEntityFromLayerResult {
  pub entity_id: EntityId,
  pub layer_id: LayerId,
  pub removed: bool,
}

pub struct SetLayerVisibilityRequest {
  pub layer_id: LayerId,
  pub visible: bool,
}

pub struct SetLayerVisibilityResult {
  pub layer: LayerDto,
  pub affected_entity_ids: Vec<EntityId>,
}

pub struct ListLayersQuery;

pub struct ListLayersResult {
  pub layers: Vec<LayerDto>,
}

pub struct GetLayerForEntityQuery {
  pub entity_id: EntityId,
}

pub struct GetLayerForEntityResult {
  pub entity_id: EntityId,
  pub layer: Option<LayerDto>,
}

pub struct ListEntitiesForLayerQuery {
  pub layer_id: LayerId,
}

pub struct ListEntitiesForLayerResult {
  pub layer_id: LayerId,
  pub entity_ids: Vec<EntityId>,
}

pub struct LayerDto {
  pub layer_id: LayerId,
  pub name: String,
  pub visible: bool,
  pub member_count: usize,
}

pub struct LayerMembershipDto {
  pub entity_id: EntityId,
  pub layer_id: LayerId,
}
```

- `LayerDto.visible` は layer 単位 ON/OFF の正本 state を表す
- `AddEntityToLayerRequest` は既存 layer 所属がある場合に Application 境界で正規化エラーを返す
- layer は単一所属前提のため、entity ごとの query も `GetLayerForEntity*` のように単数契約を優先する
- layer は将来、既定色・既定線種の継承元になりうるが、初期 DTO にはその責務を混ぜない

### 5.9 強い group / layer ON/OFF を支える state の位置づけ

- 現在の `geo_entity::GroupEntity` は `id` と `name` のみを持つ
- しかし「group 単位 ON/OFF を強い制御とする」要件を満たすには、`visible` を group state の正本側で保持する必要がある
- したがって将来実装では以下のどちらかを採用する
  1. `GroupEntity` 自体へ `visible: bool` を追加する
  2. `GroupDisplayState { group_id, visible }` を relation と並列の語彙として持つ
- 初期設計では DTO 契約を `visible` ありで固定し、Model 実装方式は後段で選択してよい
- ただし View ローカル状態だけで group 可視性を持つ方式は、Application query と整合しないため採用しない
- layer については現状の `LayerEntity` が `visible` を持つため、その方針を維持してよい
- ただし group と layer の実装方式が異なっても、Application DTO 契約ではどちらも `visible` を持つ形に揃える

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

### 6.4 entity / topology orchestration の初期配置

- `entity_orchestration` と `topology_orchestration` は無条件に増やすのではなく、責務境界が追跡できる最小単位で導入する
- 2026-04-09 時点では、topology は geometry からの導出 / 検証責務が独立しているため `topology_orchestration` を独立 module として配置している
- 一方で entity は専用 module をまだ追加せず、`feature_orchestration` 配下の store port として保持している
- したがって初期段階の判断基準は module 数の抑制そのものではなく、ViewModel→Application→Model 導線の責務分割をコード上で追跡できることに置く

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

### 7.1 表示属性 contract 境界

`geo_contracts` に露出する表示属性 contract は、次のように整理する。

```rust
pub trait EntityDisplayProperties {
  fn entity_visible(&self) -> bool;
  fn entity_color(&self) -> [f32; 4];
}

pub enum StrokePattern {
  Solid,
  Dashed,
  Dotted,
  DashDot,
  DashDotDot,
  Hidden,
}

pub trait StrokeDisplayProperties {
  fn entity_stroke_pattern(&self) -> StrokePattern;
  fn entity_stroke_width(&self) -> f32;
}

pub trait PlanarStroke2DProperties<T: Scalar> {
  fn definition_plane_origin(&self) -> (T, T, T);
  fn definition_plane_normal(&self) -> (T, T, T);
  fn definition_plane_u_axis(&self) -> (T, T, T);
}
```

- `StrokePattern` は `geo_entity` ではなく `geo_contracts` 側へ寄せる
- 理由は、ViewModel contract が `geo_entity` の具体型へ依存しないようにするためである
- `geo_entity::DisplayAttributes` の `LineStyle` は将来的に `StrokePattern` へ統合または alias 化する
- 「二点鎖線」はここで stable 語彙として先に追加する

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

### 2026-04-09 時点の #501 実装反映

- ViewModel 起点の最小 command 導線は `viewmodel/converter/src/feature_command_converter.rs` から `application` へ接続している
- 現在の非ECS入口は `FeatureCommandOrchestration` であり、`FeatureOrchestrator` が line / circle / arc の geometry → topology → entity を直列実行する
- triangle は face を導入せず、`GeometryOrchestrator` と `TopologyOrchestrator` を経由して closed wire として topology 責務だけを固定している
- 通常の平面トポロジーとして triangle を Wire / Loop で扱う場合は、各辺を `LineSegment` 相当の直線 edge として表現する方針を採る
- #256 で扱う三角形限定 topology は、工具逆オフセット法計算の前段で局所的な凸部縫合と探索効率向上を目的とする別系統の用途であり、ここでの通常 topology 表現とは分離して扱う
- geometry の差し替え点は `LineGeometryMutationPort` と `DebugShapeGeometryMutationPort` である
- topology の差し替え点は `LineTopologyMutationPort` と `DebugShapeTopologyMutationPort` である
- entity / topology 保存の差し替え点は `LineEntityStorePort` / `CircleEntityStorePort` / `ArcEntityStorePort` / `CurveTopologyStorePort` である
- 現在は in-memory 実装だが、ECS 置換時はこれら port の背後実装を ECS adapter へ差し替え、ViewModel 側の command 入口は維持する

### NURBS topology の段階導入方針

- NURBS topology は 1 段で trimmed face まで入れず、まず curve edge のみを対象にする
- 初手では `NurbsCurve3D` を Edge の母曲線候補として扱い、native parameter domain を `parameter_range` にそのまま写す
- `geo_topology` は既に `geo_primitives` に依存しているため、同列 shape family である `geo_nurbs` への依存も curve edge 対応の範囲で許可する
- ただし依存は `geo_topology -> geo_nurbs` の片方向に限定し、`geo_nurbs -> geo_topology` の逆依存は許可しない
- この段階では trim curve / Face / Loop / CoEdge / surface 上の 2D parameter curve は導入しない
- trimmed NURBS face は別 Issue として扱い、face 語彙・loop 語彙・surface trim 境界の設計を先に固定してから実装する
- したがって、NURBS 対応の初期スコープは `CurveRef` / `CurveSegment3D` / topology derive port の NURBS curve 対応までとする
- `geo_nurbs` 側の split / evaluation は edge 段階でも利用可能だが、trimmed face の責務とは混同しない

### #501 受け入れ条件との対応

- `ViewModel起点の1ユースケースがModel更新まで通る`
  - line / circle / arc は `feature_command_converter` → `FeatureOrchestrator` → in-memory entity/topology store まで到達している
- `Entity/Topology責務境界がコード上で追跡可能`
  - geometry は `geometry_orchestration.rs`、topology は `topology_orchestration.rs`、entity 保存は `feature_orchestration.rs` に分離されている
- `ECS置換時に差し替えるポイントがドキュメント化済み`
  - 本節の port 一覧を現行コード名ベースの差し替え点として扱う

---

## 9. 判断メモ

- `view/app` の `EntityManager` は View ローカル状態であり、Model 側 entity 正本の代替にしない
- ただし View ローカル状態は 2D / 3D の描画カテゴリと表示条件を反映した cache 単位を持つ
- 2D 線種は「定義平面に対する直交ビュー」でのみ表示保証し、それ以外では非表示または実線フォールバックを許容する
- 表示属性は common trait に詰め込まず、common / stroke / planar-2d に分割して露出する
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
