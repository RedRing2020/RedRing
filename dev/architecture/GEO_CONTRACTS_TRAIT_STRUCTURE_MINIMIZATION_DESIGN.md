# geo_contracts trait Structure Minimization Design

## 目的

Issue #535 では、`geo_contracts` の trait構造を次の最小構造へ再編する。

- 形状定義コンストラクタ trait
- 形状参照 trait
- それ以外の capability trait

本書は、その前提として現行 trait を棚卸しし、一次分類と曖昧境界の個別判定を明文化する。

## 現行構造の棚卸し

### `geometry/core`

`geometry/core` には shape ごとの trait ファイルが並び、概ね次の組を持つ。

- `*Constructor`
- `*Properties`
- `*Measure`
- `*Core`

代表例:

- `point_traits.rs`: `Point2DConstructor` / `Point2DProperties` / `Point2DMeasure` / `Point2DCore`
- `vector_traits.rs`: `Vector2DConstructor` / `Vector2DProperties` / `Vector2DMeasure` / `Vector2DCore`
- `circle_traits.rs`: `Circle2DConstructor` / `Circle2DProperties` / `Circle2DMeasure` / `Circle2DCore`

ただし、現行 `core` には shape 定義以外も混在している。

代表例:

- `arc_traits.rs`: `Arc2DSampling`, `Arc2DContainment`
- `linesegment_traits.rs`: `LineSegment3DCollisionDetection`
- `aabb_traits.rs`: 参照系と関係判定系が単一 trait に混在

### `geometry/foundation`

`geometry/foundation` には次の 2 trait がある。

- `ExtensionFoundation<T>`
- `Bounded<T>`

`ExtensionFoundation<T>` は `primitive_kind()` と `measure()` を持つが、後者は各 shape の `*Measure` と責務が重なる。

### `geometry/operations`

`geometry/operations` には横断演算系 trait がある。

- `collision.rs`: `BasicCollision`, `AdvancedCollision`, `PointDistance`, `BBoxCollision`
- `distance.rs`: `CrossDistance`, `FallibleCrossDistance`, `DistanceConvergenceError`
- `intersection.rs`: `BasicIntersection`, `MultipleIntersection`, `SelfIntersection`
- `ellipse_calculation_traits.rs`: `EllipseCalculation`, `EllipseAdaptiveCalculation`, `EllipseAccuracyAnalysis`

## 一次分類

### A. shape definition core 残留候補

この層には、形状を定義し、形状の定義パラメータを参照する trait だけを残す。

- 全 `*Constructor`
- 全 `*Properties`
- `*Core` は残す場合でも `Constructor + Properties` の統合 alias に限定する

該当例:

- `Point2DConstructor`, `Point2DProperties`
- `Vector3DConstructor`, `Vector3DProperties`
- `Circle2DConstructor`, `Circle2DProperties`
- `Plane3DConstructor`, `Plane3DProperties`

### B. minimal extension 候補

この層には、単一 shape に対する追加 capability を置く。

- 全 `*Measure`
- `Arc2DSampling`
- `Arc2DContainment`
- `Bounded<T>`
- `primitive_kind()` のような shape metadata capability

### C. operations 残留候補

この層には、shape 間演算や計算戦略を置く。

- `BasicCollision`, `AdvancedCollision`, `PointDistance`, `BBoxCollision`
- `CrossDistance`, `FallibleCrossDistance`, `DistanceConvergenceError`
- `BasicIntersection`, `MultipleIntersection`, `SelfIntersection`
- `EllipseCalculation`, `EllipseAdaptiveCalculation`, `EllipseAccuracyAnalysis`
- `LineSegment3DCollisionDetection`

## 曖昧境界の個別判定

### 1. `*Core` は shape definition か

判定:

- Yes, ただし統合対象は `Constructor + Properties` に限定する

理由:

- `*Core` 自体は責務ではなく公開上の統合 alias だから
- 現行の `Constructor + Properties + Measure` は最小構造の境界を曖昧にするから

設計反映:

- `*Core` を残すなら `shape definition core alias` としてのみ扱う
- `*Measure` は `*Core` から外す

### 2. `*Properties` の中の派生 getter は参照か

判定:

- Yes

対象例:

- `dimension()`
- `diameter()`
- `angle_span()`
- `is_unit_circle()`
- `is_on_xy_plane()`
- `is_zero()`

理由:

- 単一 shape の保持パラメータから直接導ける lightweight metadata であり、別 capability 層へ出すと分解過剰になるから
- 他 shape や外部入力を取らず、探索・射影・サンプリング・solver を伴わないから

設計反映:

- `Properties` には lightweight metadata を残してよい
- ただし point evaluation, projection, containment, distance は残さない

### 3. `*Measure` は shape definition か

判定:

- No

理由:

- 長さ、面積、体積、距離、補間、射影、法線評価は shape の定義そのものではなく capability だから
- 現行ソースにも `TODO(#318): Measure contracts are temporarily colocated and will be split by responsibility.` が残っているから

設計反映:

- 全 `*Measure` を shape definition core から分離する

### 4. `contains_point` は `Properties` 側か `Measure` 側か

判定:

- shape definition には置かない
- 単一 shape capability として `Measure` 側または後継 capability trait 側へ置く

理由:

- 点 inclusion 判定は外部入力を取る関係判定であり、単なる参照ではないから
- `PointDistance` や collision 系との責務境界も明示しやすくなるから

設計反映:

- `contains_point` は definition 層から除外する

### 5. `point_at_parameter` / `sample_points` はどこに属するか

判定:

- shape definition には置かない
- 単一 shape の evaluation / sampling capability として extension 側へ置く

理由:

- これらは shape の参照ではなく、shape を使った計算・離散化だから

設計反映:

- `Arc2DSampling` は extension 側へ移す
- `point_at_parameter` を含む API 群も definition 層から除外する

### 6. `distance_to_point` / `distance_to_circle` / `distance_to_segment` はどこに属するか

判定:

- shape definition には置かない
- cross-shape を含むものは `operations`
- 単一点を相手にするものも definition 層には置かない

理由:

- 距離計算は shape 参照ではなく演算だから
- 現行でも同種 API は `geometry::operations::CrossDistance` へ移行中だから

設計反映:

- deprecated な same-shape distance API は `operations` へ寄せるか削除する
- `distance_to_point` も definition から外す

### 7. `ExtensionFoundation<T>` は現形のまま残すべきか

判定:

- No

理由:

- `primitive_kind()` は metadata capability だが、`measure()` は `*Measure` と責務重複するから
- 1 trait に metadata と generic measure summary を束ねると境界が再び曖昧になるから

設計反映:

- `ExtensionFoundation<T>` はそのまま残さない
- 必要なら `primitive_kind()` 専用の lightweight metadata trait に分離する
- `measure()` は dedicated capability 側へ寄せる

### 8. `Bounded<T>` は shape definition か

判定:

- No

理由:

- AABB は shape の定義パラメータではなく、shape から導出される spatial envelope だから
- shape definition を読むだけなら `aabb()` を必須にしなくてよいから

設計反映:

- `Bounded<T>` は extension 側の capability として扱う

### 9. `Aabb2DTrait` / `Aabb3DTrait` は 1 trait のままでよいか

判定:

- No

理由:

- `min()` / `max()` は shape 参照だが、`contains_point()` / `contains_bbox()` / `intersects()` は関係演算だから
- `width()` / `height()` / `depth()` / `center()` / `area()` / `volume()` / `is_valid()` は unary derived capability であり、definition と operations の中間にあるから

設計反映:

- AABB は少なくとも次の 2 層に分ける
- `Aabb*Properties`: `min`, `max`
- `Aabb*Derived`: `width`, `height`, `depth`, `center`, `area`, `volume`, `is_valid`
- `contains_point`, `contains_bbox`, `intersects` は extension または operations へ移す
- 既存 `Aabb2DTrait` / `Aabb3DTrait` の延命は前提にせず、新規 capability 群として再定義してよい
- AABB 単体で参照系 / derived / relation の意味を再設計し、旧 trait 命名との互換維持より責務境界の明確化を優先する

### 10. `EllipseCalculation*` は extension か operations か

判定:

- operations

理由:

- これは shape の定義でも一般的な単一 shape metadata でもなく、近似式・数値計算戦略の選択そのものだから
- capability としても algorithm 寄りであり、他の operations 群と同じ層に置く方が自然だから

設計反映:

- `EllipseCalculation`, `EllipseAdaptiveCalculation`, `EllipseAccuracyAnalysis` は operations 側に残す
- ただし `geo_contracts` に置くのは trait定義の正本までとし、adaptive 選択や比較分析の default 実装は持ち込まない
- strategy の実装本体は `geo_primitives` の impl entry point または将来的な `geo_algorithms` 側 helper へ置く
- 近似式・距離計算の数値カーネル自体は `geo_commons` に維持し、`geo_contracts` は数値閾値や比較ロジックを保持しない

### 11. `LineSegment3DCollisionDetection` は extension か operations か

判定:

- operations

理由:

- AABB との cross-shape 演算であり、単一 shape の自己記述ではないから

設計反映:

- `geometry/core` から外し、operations 側へ寄せる

### 12. #541 で優先して移す対象

Issue #541 では、まず既存の `geometry::operations` trait に自然に載るものから移す。

- deprecated な same-shape distance API
- `LineSegment3D` と AABB の距離
- `InfiniteLine3D` / `Plane3D` / `Ellipse3D` が持つ明示的な交点計算

設計反映:

- 距離は `CrossDistance`
- 単一点交差は `BasicIntersection`
- 複数交点は `MultipleIntersection`
- concrete 型の convenience method は必要最小限だけ残し、trait定義の正本は `operations` 側に寄せる

### 13. relation 系メソッドは独立に厳格判定する

判定:

- optional ではなく、責務分離の中核ルールとして扱う
- 相手 shape の存在を前提にし、幾何学的な関係を返す API は relation 系として個別判定する

relation 系の代表例:

- `is_parallel_to`
- `is_perpendicular_to`
- `is_same_line`
- `intersects`
- `is_skew_to`
- `is_same_direction`
- `is_opposite_direction`
- `angle_to`
- `angle_between`
- `closest_points`

理由:

- unary な measure / evaluation / metadata と、binary な relation が混在すると `core` と `minimal extension` の責務が再び曖昧になるから
- 相手 shape を取る relation API を曖昧に残すと、same-shape distance や intersection と同様に `operations` へ寄せるべき責務が再流入するから
- #541 の完了条件は、cross-shape の計算だけでなく relation API の所属も一貫して説明できる状態にすることだから

設計反映:

- relation 系メソッドは `measure` の残余として扱わない
- `other: &Self` や他 shape 引数を取り、関係判定・角度・最近点対を返す API は relation 系候補として必ず棚卸しする
- relation 系を `minimal extension` に残す場合でも、`operations` に寄せない理由を Issue / PR で明示する

## 採用する境界線

最終的な境界線は次のとおりとする。

### shape definition core

- `*Constructor`
- `*Properties`
- 必要なら `*Core = Constructor + Properties`

### minimal extension

- `*Measure`
- `Bounded`
- sampling / evaluation / containment のような単一 shape capability
- `primitive_kind()` のような lightweight metadata capability

補足:

- relation 系メソッドはこの層へ無批判に残さない
- 相手 shape を取る API は、まず `operations` 候補として判定する

### operations

- collision / distance / intersection
- strategy / approximation / solver oriented capability
- cross-shape specialized contracts
- relation API

## 次段で必要な作業

- 各 `*_traits.rs` を上記 3 層へ再配置した場合の配置案を作る
- `ExtensionFoundation` の後継を `primitive_kind` 専用に残すか廃止するか決める
- `Aabb*Trait` の分割粒度を確定する
- `*Core` alias を公開 API として残すか決める

## 実装タスク管理

段階的移行タスクと、各実装 Issue 本文に記載すべきチェック項目は、#535 の成果物として本設計書で管理する。

- 各形状または各責務分離タスクの実装 Issue は、本設計書のチェック項目を適用して起票する
- チェック項目は独立のテンプレート Issue ではなく、#535 の設計成果物として扱う

実装 Issue は shape 単位ではなく、責務分離単位で分ける。

- `definition core 縮退`
- `measure 系 extension 分離`
- `metadata capability 分離`
- `operations への移送`
- `AABB 特例整理`

## 実装 Issue 用チェック項目

各実装 Issue では、少なくとも以下を本文に含める。

### 1. 対象確定

- [ ] 対象 trait 名と対象ファイルが列挙されている
- [ ] 非対象 trait が明記されている
- [ ] 変更先の層が `shape definition core` / `minimal extension` / `operations` のどれか明記されている

### 2. 責務判定

- [ ] `Constructor` は definition core に残すか確認した
- [ ] `Properties` の各メソッドが参照系に残せるか確認した
- [ ] `Measure` 系 API を definition core に残していない
- [ ] `contains_point` / `distance_to_point` / `point_at_parameter` / sampling 系を個別判定した
- [ ] relation 系メソッド（平行・垂直・交差・同方向・角度・最近点対）を個別判定した

### 3. 配置変更

- [ ] 移動先モジュールが決まっている
- [ ] `mod.rs` の公開が更新されている
- [ ] `geo_contracts::lib.rs` の公開が必要なら更新されている
- [ ] `*Core` alias を残す場合は `Constructor + Properties` のみを束ねている

### 4. 互換性

- [ ] 旧経路は duplicate trait ではなく re-export または alias で維持している
- [ ] 利用側の import 経路が壊れていない
- [ ] 一時互換を残す場合、削除予定が Issue または文書に書かれている

### 5. 境界の明確化

- [ ] extension と operations の境界が今回の変更で曖昧になっていない
- [ ] metadata capability と measure capability を混在させていない
- [ ] cross-shape 演算を definition core に持ち込んでいない
- [ ] relation 系メソッドを "measure のついで" で残していない
- [ ] shape 参照と capability の責務を文章で説明できる

### 6. 検証

- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo fmt`
- [ ] `cargo test --workspace`
- [ ] 必要なら `./scripts/check_architecture_dependencies.ps1 -ExitOnError`

### 7. 完了判定

- [ ] 正本モジュールが 1 つに定まっている
- [ ] 旧構造へ逆戻りする duplicate 定義が入っていない
- [ ] 次段階で扱う未解決事項が列挙されている
- [ ] PR 本文で「残したもの」と「次 Issue へ送るもの」を分離して説明できる
