# Curve Tessellation Boundary Design

## 目的

Issue #642 では、curve tessellation に関する責務境界を次の観点で固定する。

- `geo_nurbs` に残すべき NURBS 固有の adaptive parameter sampling の境界
- `geo_algorithms` に置くべき curve discretization / facade / orchestration の境界
- `geo_contracts` に現時点では追加しないが、将来必要になり得る contract の発火条件
- `curve discretization` / `param-grid tessellation` / `surface meshing` を混同しない taxonomy

本書は「どの crate に今すぐ移すか」を決める文書ではなく、「どの責務を今どこに固定し、どの条件が満たされたら再検討するか」を決める文書である。

## 前提

2026-04-08 時点で、既に次は成立している。

- `geo_nurbs::adaptive_tessellation` は parameter 列 / parameter grid 生成を担う
- `geo_nurbs::curve_3d_extensions` と `surface_3d_extensions` は、NURBS 評価に直接結び付いた chord error ベースの adaptive sampling を実装している
- `geo_algorithms::curve_discretization` は現在 `circular_arc_to_polyline` を提供し、shape family ごとの options と離散化入口を置く層として使われ始めている
- `GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md` では、`adaptive parameter sampling` / `curve discretization` / `param-grid tessellation` / `surface meshing` の taxonomy 自体は既に定義済みである

したがって #642 の主論点は、taxonomy を再定義することではなく、その taxonomy に沿って crate 境界と contract 追加条件を具体化することにある。

## 既存実装の責務整理

### `geo_nurbs` にあるもの

`model/geo_nurbs/src/adaptive_tessellation.rs` とその extension 実装は、次の特徴を持つ。

- 入力が `NurbsCurve3D<T>` / `NurbsSurface3D<T>` に閉じている
- 誤差評価が `evaluate_at` に直結している
- 出力が polyline や triangle mesh ではなく parameter 列 / parameter grid である
- NURBS の parameter domain と knot / weight / evaluation 実装を前提にしている

このため、現状の adaptive tessellation は「汎用 curve discretization」ではなく、「NURBS evaluation に密着した adaptive parameter sampling」とみなすのが正しい。

### `geo_algorithms` にあるもの

`model/geo_algorithms/src/curve_discretization` は、次の特徴を持つ。

- shape family ごとの discretization entry を持つ
- 出力が線分列や点列など、上位利用へ直接渡せる geometry に近い
- options 型を family ごとに閉じて持つ
- topology や UI 設定の正本を持たず、離散化アルゴリズム入口として振る舞う

この層は現時点では「curve discretization の本体集約層」というより、「shape family ごとの discretization entry を上位ユースケースへ提供する facade / orchestration 層」とみなすのが妥当である。

## 代替案

### 案A: 現行 adaptive parameter sampling を `geo_nurbs` に残し、`geo_algorithms` は facade に留める

内容:

- `geo_nurbs` に NURBS 固有の parameter sampling を残す
- `geo_algorithms` は polyline 化や shape family ごとの discretization 入口を提供する
- `geo_contracts` には新しい tessellation contract を追加しない

利点:

- 現実装と整合する
- NURBS 固有の chord error 評価と parameter domain 処理を無理に抽象化しない
- contract を早まって固定しないため、将来の shape family 拡張に対して柔軟

欠点:

- `geo_algorithms` は facade / orchestration 色が残る
- NURBS 以外の parametric curve が増えたときに再整理が必要になる

### 案B: adaptive parameter sampling を `geo_algorithms` へ即時移管する

内容:

- NURBS の adaptive sampling 本体を `geo_algorithms` に移す
- `geo_nurbs` は evaluation 提供だけに寄せる

利点:

- 「離散化は algorithms」という見た目上の一貫性が高い

欠点:

- 現状では NURBS 固有評価に強く依存しており、単なる移管だと依存方向だけが上がる
- `geo_algorithms` が facade ではなく NURBS 実装詳細の実行層になりやすい
- 汎用 contract が未整備のまま実装だけが先に上位化する

### 案C: 先に `geo_contracts` に汎用 tessellation contract を追加してから整理する

内容:

- curve discretization や param-grid sampling の contract を先に `geo_contracts` に導入する
- その contract を前提に各 crate 配置を見直す

利点:

- 将来の統一 API 面を早期に定義できる

欠点:

- 現時点では shape family 間の共通最小公倍数がまだ不安定
- output 形式と誤差尺度の語彙が固定しきれていない段階で contract を追加すると、曖昧な抽象化を正本化しやすい

## 採用方針

案Aを採用する。

理由:

- 現在の adaptive sampling は NURBS 固有評価に密着しており、`geo_nurbs` に残す方が責務が明快
- `geo_algorithms` 側は、当面は shape family ごとの discretization 入口を束ねる facade / orchestration に留める方が安全
- `geo_contracts` に今すぐ tessellation contract を追加するには、共通の output と誤差尺度がまだ不足している

## 責務境界

### `geo_nurbs` に残す

- `AdaptiveTessellationSettings<T>`
- `AdaptiveParamList<T>` / `AdaptiveParamGrid<T>`
- `NurbsCurveAdaptiveTessellation<T>` / `NurbsSurfaceAdaptiveTessellation<T>`
- chord error を `evaluate_at` ベースで判定する parameter sampling 実装
- NURBS 曲線 / 曲面の parameter domain に閉じた sampling helper

これらは polyline や mesh を返す責務ではなく、NURBS evaluation の補助として parameter 列を生成する責務であるため、`geo_nurbs` に残す。

### `geo_algorithms` に置く

- shape family ごとの polyline / point 列離散化入口
- family ごとの options 型
- 上位ユースケース向けの discretization facade
- 必要時の topology adapter 入口

ただし、現時点で `geo_algorithms` は「汎用 tessellation 実装の一元所有者」ではなく、「shape family ごとの discretization entry を上位へ見せる facade / orchestration 層」と位置付ける。

### `geo_contracts` に現時点では置かない

- 汎用 `CurveDiscretization` trait
- 汎用 `ParamGridTessellation` trait
- 汎用 `SurfaceMeshing` trait

理由は、まだ shape family 間で共通化すべき output と誤差語彙が十分に安定していないためである。

## taxonomy の固定

### 1. adaptive parameter sampling

- parameter domain を適応分割して parameter 列または parameter grid を返す責務
- 現時点では NURBS evaluation に密着した補助として `geo_nurbs` に残す
- polyline / triangle mesh そのものは返さない

### 2. curve discretization

- analytic curve / parametric curve を polyline や点列へ落とす責務
- family ごとの options と入口は `geo_algorithms` に置く
- topology の正本責務とは分離する

### 3. param-grid tessellation

- 表示や GPU 評価向けに parameter grid を生成する責務
- 高品質 mesh 生成と同一視しない
- 現行 NURBS adaptive sampling はこちらに近いが、NURBS 固有実装として `geo_nurbs` に残す

### 4. surface meshing

- surface を triangle 群や高品質要素へ落とす責務
- 表示用 parameter grid とは別問題として扱う
- 現時点の #642 スコープ外

## 将来 contract を追加してよい条件

`geo_contracts` に tessellation 系 contract を追加してよいのは、少なくとも次の条件が揃ったときに限る。

- NURBS 以外にも同じ責務を持つ shape family が 2 系統以上存在する
- output 形式が shape family 横断で同じ語彙に乗る
- 誤差尺度が family 横断で同じ意味を持つ
- options 型の最小共通集合が説明可能である
- facade ではなく contract を正本化する利点が、実装詳細の露出コストを上回る

現時点ではこれらが未充足なので、contract 追加は defer する。

## 後続実装の分割単位

本整理を前提に、将来の実装や追加設計を切り出す場合は、少なくとも次の単位に分ける。

### 1. `geo_algorithms` 側の curve discretization family 拡張

対象:

- `circular_arc` 以外の curve family 追加
- family ごとの options 型追加
- topology adapter 入口の追加

この単位では、`geo_algorithms` を facade / orchestration として拡張するだけに留め、`geo_nurbs` 側の adaptive sampling 本体は動かさない。

### 2. `geo_nurbs` 側の adaptive parameter sampling 改善

対象:

- chord error 評価ロジックの改善
- parameter grid 生成条件の改善
- NURBS curve / surface の sampling 設定整理

この単位は NURBS 固有評価に閉じた改善として扱い、汎用 curve discretization と同じ issue に混在させない。

### 3. `geo_contracts` への tessellation 系 contract 追加検討

対象:

- `CurveParameterSampling` 相当の抽象化検討
- `CurveDiscretization` 相当の抽象化検討
- `SurfaceParamGridSampling` 相当の抽象化検討

この単位は、shape family 横断で output と誤差尺度が揃った後に限って着手する。先に実装を進めるための便宜だけで contract を追加しない。

### 4. surface meshing の別論点化

対象:

- parameter grid ではなく triangle mesh / 高品質要素生成
- CAM / CAE 向け品質要件

これは curve discretization や param-grid tessellation と同一 issue に戻さず、別系列として扱う。

## 追加候補となる contract の方向性

現時点では追加しないが、将来的な候補名としては次の方向が考えられる。

- `CurveParameterSampling`
- `CurveDiscretization`
- `SurfaceParamGridSampling`

ただし、これらは今の時点で trait定義として確定しない。名称よりも、どの output と誤差尺度を共有するのかが先に固まる必要がある。

## 今回の結論

- 現行 adaptive tessellation は「NURBS 固有の adaptive parameter sampling」として `geo_nurbs` に残す
- `geo_algorithms` は当面、curve discretization の facade / orchestration に留める
- `geo_contracts` には今すぐ tessellation 系 contract を追加しない
- `curve discretization` / `param-grid tessellation` / `surface meshing` は別責務として扱い続ける
- contract 追加は、shape family 横断で output と誤差尺度が揃った段階で再検討する