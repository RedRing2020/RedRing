# Geometry Shape Semantics Design

## 目的

本書は、RedRing における **shape の意味論と意図** の正本を定義する。

特に、以下のような問いに対する答えを固定する。

- ある shape は exact primitive か、tolerant な shape か
- どの値が正本で、どの値が派生か
- `start` / `end` / `length` / `point_at_parameter` などの API が何を返すか

本書は trait の責務分担や topology 正規形そのものを定義する文書ではない。

## 正本の役割分担

- shape 意味論の正本: 本書
- trait 境界と capability 分担の正本: `GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md`
- topology entity layer 設計の正本: `TOPOLOGY_ENTITY_LAYER_DESIGN.md`
- topology 実装計画の補助説明: `PHASE4_TOPOLOGY_ENTITY_DESIGN.md`
- #458 時点の topology 正規形凍結スナップショット: `TOPO_NORMAL_FORM_INVARIANTS_FREEZE_458.md`

補足:

- 本書は「shape が何を意味するか」を定義する
- trait 構成やモジュール配置は、本書で固定した意味論を前提に別文書で扱う

## スコープ

現時点では、`#557` で論点化された `LineSegment2D/3D` の意味論を最初の固定対象として定義する。

ただし本論点は `LineSegment` 固有ではなく、shape 横断で再発しうる意味論項目を先に `LineSegment` で明文化する位置付けとする。

Arc / EllipseArc / Circle / Triangle などを含む shape 横断の `Measure / parameter / endpoint` capability 論点は `#558` で別途整理する。

## `#558` の現状棚卸し

`#558` 着手時点の `geo_contracts` では、Arc / EllipseArc / Circle / Triangle / LineSegment の各 trait が、shape ごとに異なる粒度で `Measure / endpoint / evaluation / containment / distance` を抱えている。

少なくとも、次の非対称が存在する。

| Shape | 現状の主な意味要素 | 現状の混在状況 |
| --- | --- | --- |
| `LineSegment` | support line、理想端点、長さ、parameter 評価 | endpoint は `Properties`、拘束端点は topology 側責務として分離対象 |
| `Arc` | 中心、半径、角度区間、始終点、parameter 評価 | `Measure` に始終点、midpoint、angle 評価、distance、containment が同居 |
| `EllipseArc` | 中心、主軸情報、角度区間、始終点、parameter 評価 | `Measure` に始終点、midpoint、angle 評価、containment、bounding box が同居 |
| `Circle` | 中心、半径、周回 parameter 評価、閉曲線性 | `Measure` に circumference、area、containment、distance、projection、parameter 評価が同居 |
| `Triangle` | 3頂点、面積、`perimeter`、面内判定 | `Measure` に edge length、perimeter、containment、distance、向き/planarity 判定が同居 |

この棚卸しから、現状の `Measure` は単なる測度ではなく、shape ごとに次の異種 capability を束ねていることが分かる。

- primary quantity: 単独線 shape の `length` / 閉曲線 shape の `circumference` / 面の `area`
- boundary quantity: 複数辺境界 shape の `perimeter` / edge length
- 境界参照: `start_point` / `end_point` / 頂点参照 / edge length
- 評価: `point_at_parameter` / `point_at_angle`
- 関係判定: `contains_point`
- 距離・射影: `distance_to_point` / `closest_point_to`
- 形状固有派生: `direction_vector` / `bounding_box` / `is_clockwise` / `is_planar`

重要なのは、これは trait 分割の問題である前に、shape ごとに「どの capability が本質的に存在するか」が揃っていない問題だという点である。

## `#558` で固定したい shape 横断分類

本書では trait 名や配置を先に固定せず、まず shape 意味論として次の分類を採用する。

### 1. 境界付き曲線 shape

明確な始点・終点を持ち、評価 parameter の両端が境界点と対応する shape である。

対象:

- `LineSegment`
- `Arc`
- `EllipseArc`

意味論上の共通点:

- 始点・終点が意味を持つ
- `point_at_parameter(0/1)` に境界端の意味を与えやすい
- 長さ系の測度を持つ

### 2. 閉曲線 shape

周回 parameter を持つが、標準の始点・終点を本質意味としては持たない shape である。

対象:

- `Circle`

意味論上の共通点:

- `point_at_parameter` は定義できる
- ただし `start` / `end` は shape の本質語彙ではない
- primary quantity としては `circumference` を持ち、派生 quantity として `area` も持ちうる

### 3. 面 shape

境界頂点や辺は持つが、単一の曲線 parameter で代表しない shape である。

対象:

- `Triangle`

意味論上の共通点:

- pre-topology の単一 face primitive として扱う
- オイラー操作を前提にしない mesh 表現とは分けて扱う
- primary quantity として `area` を持ち、boundary quantity として `perimeter` や edge length を持ちうる
- 頂点参照はできる
- 各辺長は boundary quantity、`perimeter` は boundary 全体の派生量として扱う
- `curve parameter` や `start/end point` を標準 capability とみなすべきではない
- parameter evaluation を導入する場合でも curve family ではなく surface family として扱う
- trimmed range や periodic parameter は標準 capability とみなさない

## `#558` における意味論上の一次結論

### `point_at_parameter` は全 shape 共通 capability ではない

`point_at_parameter` は少なくとも「単一の連続 support 表現を持つ shape」にだけ自然に定義される。

したがって、LineSegment / Arc / EllipseArc / Circle には自然だが、Triangle へそのまま横展開する前提は置かない。

### `start` / `end` は全 curve に共通ではない

`start` / `end` は、境界付き曲線 shape に対してだけ本質語彙として扱う。

Circle のような閉曲線では、parameter の基準位置を便宜的に定められても、それを shape 意味論上の正本 endpoint とみなしてはならない。

### `measure` は単一語で統一しない

shape 横断で `measure` だけを共通語彙にすると、長さ、面積、`perimeter`、拘束点間距離が同じ名前に吸い込まれ、意味が弱くなる。

したがって本書では、shape 意味論上の primary quantity 語彙は次を優先する。

- 線状 shape: `length`
- 閉曲線: `circumference`
- 面 shape: `area`
- 多義的な互換 API: `measure`

### 境界参照と評価は分けて扱う

`start/end`、頂点参照、edge 長などの「境界の語彙」と、`point_at_parameter` や `point_at_angle` のような「評価の語彙」は同じではない。

今後の capability 分離では、少なくともこの 2 系統を別物として扱う。

## 今回の棚卸しで見えた #558 の設計対象

次段では、上記の shape 横断分類を前提に、少なくとも次を trait 境界文書側で整理する必要がある。

- 境界付き曲線 shape にだけ与える endpoint capability
- curve family にだけ与える parameter evaluation capability
- 閉曲線に対する periodic parameter の扱い
- 面 shape に対する vertex / boundary access の語彙
- `length` / `circumference` / `area` と `measure` の関係
- containment / distance / projection を shape definition からどこまで外すか

## 横展開前提の論点

`#557` で固定する内容は、将来的に他 shape へ横展開が必要な論点として扱う。

- exact primitive と tolerant shape の区別
- どの値を正本とし、どの値を派生として扱うか
- `start` / `end` 系 API が拘束点を返すのか、ideal endpoint を返すのか
- `length` / `measure` / parameter range が何を表すのか
- `point_at_parameter` が support curve 上の評価か、拘束点補間か

したがって `LineSegment` は局所例外ではなく、shape semantics の横断整理を先行して具体化する最初のケースとして扱う。

## 将来 shape 追加時の意味論チェックリスト草案

`#558` の整理は、既存 shape の後追い整理だけでなく、将来追加する CAD shape の意味論判定基準として使う。

新しい shape を導入するときは、少なくとも次を先に確認する。

1. その shape は curve / surface / solid のどれか
2. その shape は境界付きか、閉じているか、trimmed か
3. support shape と有効領域を分離して扱う必要があるか
4. 正本 definition parameter は何か
5. primary quantity 語彙は `length` / `circumference` / `area` / `volume` のどれか
6. `point_at_parameter` や `uv_to_point` のような evaluation を本質的に持つか
7. `start/end` や vertex / edge のような boundary access を本質的に持つか
8. containment / distance / projection を shape definition から分離すべきか
9. orientation / normal / trim-range のような shape 固有 capability が必要か

補足:

- trimmed shape では、support shape 上の evaluation と trimmed domain 上の有効判定を混在させない
- periodic shape では、parameter 基準位置を持てても、直ちに endpoint を持つとはみなさない
- face element として使う shape では、boundary access、boundary quantity、orientation を convenience ではなく主要 capability 候補として扱う

このチェックリストは、sweep surface、rotation surface、fillet surface、trimmed surface のような CAD 便宜 shape を追加するときの一次判定基準として利用する。

## LineSegment の意味論

### 基本方針

`#592` の設計では、RedRing における `LineSegment2D/3D` を **bounded curve 共通ルールに従う exact な有限線分 primitive** として扱う。

すなわち、primitive 側では次を固定する。

- support line と trim 区間が shape 定義を与える
- `start` / `end` は support line 上の ideal endpoint を返す
- topology 上の拘束端点は primitive ではなく `Edge` の `start_vertex` / `end_vertex` 側で管理する

これにより、bounded curve 全体で「primitive は ideal endpoint、拘束端点は topology 管理」という責務分離を揃える。

### 正本と派生

`LineSegment` では、概念的に次の値を区別する。

- 正本: `support_line`
- 正本: trim 区間
- 正本 endpoint capability: `start` / `end`（ideal endpoint）
- 派生: `midpoint`
- 派生: `length`
- 派生: support line 上の評価点
- topology 管理: constraint endpoint

重要:

- 具体的な struct フィールド構成は実装都合で調整してよい
- ただし public API と意味論は、拘束端点を primitive 正本として扱わない
- 実装移行中に補助フィールドが残っていても、最終的な公開意味論は ideal endpoint 基準で固定する

### API 意味の固定

`LineSegment2D/3D` の主要 API は次の意味で扱う。

| API | 意味 |
| --- | --- |
| 始点を返す API（現行: `start()` / `start_point()`） | support line と trim 区間から定まる ideal start endpoint を返す |
| 終点を返す API（現行: `end()` / `end_point()`） | support line と trim 区間から定まる ideal end endpoint を返す |
| `length()` | 有限線分 primitive としての長さを返す |
| `point_at_parameter(t)` | support line 上の評価点を返す |
| `measure()` | 主語彙にしない。互換の委譲としてのみ扱う |

補足:

- `start/end` と `point_at_parameter(0/1)` は ideal endpoint / evaluation endpoint の関係として整合させる
- 拘束端点とのずれは primitive API ではなく topology の binding で扱う

### 補助 API の扱い

以下の補助 API は、意味の明確化に有効であれば導入を許容する。

- `support_line()`
- `ideal_start()` / `ideal_end()`
- `ideal_length()`
- support line 評価と topology 側拘束参照を分ける追加 API

補助 API を導入する場合は、ideal 系と topology 依存語彙を名称で明確に分離する。

## geo_primitives と geo_topology の境界

`geo_primitives::LineSegment2D/3D` は shape 意味論を持つ。

一方、`geo_topology` は以下を追加で管理する。

- constraint endpoint と vertex binding
- same_sense
- parameter range
- edge / wire / face 文脈での接続関係

したがって、topology は `LineSegment` の endpoint semantics を上書きしない。
topology 側の Edge 正規形は、primitive の ideal endpoint を前提に、拘束端点・接続・向き・トリムを管理する層として扱う。

## `#592` 着手時点の実装ギャップ

`#592` 着手時点では、設計目標と実装が完全には一致していない可能性がある。

特に確認対象は次の通りである。

- `CurveRef::Line::point_at_parameter` が support line 上の評価を返すこと
- `CurveRef::Line::start_point` / `end_point` が legacy な拘束点語彙を残していないか
- `Edge` / `Wire` / `CompositeCurve` が LineSegment の endpoint を拘束端点として暗黙利用していないか

このギャップは「現状実装の確認対象」であり、最終設計では次を満たす必要がある。

- primitive endpoint は ideal endpoint として解釈される
- 拘束端点とのずれは topology の binding rule で吸収する
- 位相専用 tolerance の正本化と運用方針は topology 側で維持する

したがって `#592` では、legacy な拘束点語彙を残したまま既成事実化せず、設計と実装の差分を埋める。

## 非目標

本書は以下を直接の対象としない。

- `Measure` の shape 横断 taxonomy の最終確定
- 各 shape の数値アルゴリズムや個別補助 API の完全確定
- topology 正規形の完全仕様
- tolerance 値そのものの単一正本化

## #557 での利用

`#592` では、本書を前提に以下を整理する。

- `LineSegment` の API を bounded curve 共通ルールに沿って明文化する
- `length` と `measure` の役割分担を整理する
- primitive と topology の endpoint 責務を混同しないよう contracts / primitives / topology 周辺を調整する

## 今後の拡張

本書は将来的に、`#557` で固定した論点を横展開し、次の shape semantics を追加できる形で維持する。

- Arc
- EllipseArc
- Circle
- Triangle
- NURBS curve / surface

ただしその追加は、shape 横断 capability 整理を扱う `#558` の方針確定後に行う。

## `#558` の設計結論

`#558` では、shape 横断の capability 命名を次の意味論で固定する。

### 1. Arc / EllipseArc の endpoint は ideal endpoint として扱う

- `Arc` / `EllipseArc` の `start_point` / `end_point` は、母曲線上の ideal endpoint を返す語彙として扱う
- `midpoint` / `mid_point` は endpoint capability の正本語彙としては採用しない
- 利用者が中間点を必要とする場合は、parameter 評価か、将来の弧長比評価のような明示 API を使う
- topology 上の拘束端点とは同一視せず、拘束端点の語彙は topology 層で別管理する

### 2. Circle / Ellipse は endpoint capability を持たない

- `Circle` / `Ellipse` は閉曲線として `point_at_parameter` を持てる
- ただし parameter 原点を与える `ref_direction` や `rotation` は endpoint の根拠にしない
- 閉曲線では `start` / `end` を正本語彙として導入しない

### 3. Triangle は curve endpoint ではなく boundary access を持つ

- `Triangle` の `vertex_a/b/c` は面 shape の boundary access であり、curve endpoint capability とは別に扱う
- `edge_*_length` や `perimeter` も vertex/boundary 由来の派生量として扱う
- `Triangle` には curve parameter evaluation capability を持ち込まない

### 4. primary quantity vocabulary は shape family ごとに固定する

- `LineSegment` / `Arc` / `EllipseArc` のような単独線 shape は `length` を正本語彙とする
- `Circle` / `Ellipse` は閉曲線 shape として `circumference` を正本語彙とし、`area` は interior を伴う派生量として扱う
- `Triangle` など複数辺からなる境界 shape は `perimeter` を使い、`area` などの面積系語彙とは分離する
- `measure` は新規の正本語彙にせず、互換 API としてのみ残してよい

### 5. evaluation と boundary access は別 capability とする

- `point_at_parameter` / `point_at_angle` は evaluation capability に置く
- `start_point` / `end_point` は境界付き曲線に限って endpoint capability に置く
- `point_at_parameter(0.5)` のような parameter midpoint は evaluation の一例であり、endpoint 語彙へ昇格させない
- 将来的に support curve evaluation と拘束点補間を併存させる場合は、同じ `point_at_parameter` 名に押し込めず名称で分ける

補足:

- 中点が重要なユースケースでも、`midpoint` のような convenience 名ではなく、何の中点かを API 名で明示する
- 特に楕円弧や NURBS では、parameter 中点と弧長中点が一致しないため、曖昧な `midpoint` は導入しない

## `#558` 実装単位 A の詳細設計

実装単位 A では、`Arc` / `EllipseArc` に対して「primitive の endpoint semantics を維持したまま、topology の拘束端点要求と衝突しないこと」を設計目標とする。

### primitive 側で固定すること

- `Arc` / `EllipseArc` の `start_point` / `end_point` は ideal endpoint を返す
- `Arc` / `EllipseArc` の長さ語彙は `length` を使い、`perimeter` は導入しない
- `point_at_parameter` は curve evaluation であり、拘束点補間 API ではない
- `point_at_angle` は angle evaluation であり、endpoint capability の一部ではない
- `point_at_parameter` の `t` は Arc / EllipseArc のトリム区間を `0..1` へ正規化した parameter として扱う
- `point_at_angle` の `angle` は primitive 局所角度系の評価入力として扱い、world 空間の極角や拘束角度と混同しない
- どちらの evaluation も返すのは ideal evaluation point であり、拘束端点や拘束点補間結果ではない

補足:

- Arc で円弧中点を取りたい場合でも、`midpoint` は置かず `point_at_parameter(0.5)` または `point_at_angle((start+end)/2)` を呼び出し側で明示する
- EllipseArc で弧長中点が必要な場合は、将来 `point_at_normalized_arc_length(0.5)` 相当の明示 API を検討する
- parameter midpoint 専用 API を追加する場合も、`midpoint` ではなく parameter を名称に含める
- Arc / EllipseArc ともに `point_at_angle` は total な support evaluation として扱い、角度範囲検証は trim-range 側で担う
- トリム区間内かどうかを見たい場合は `contains_angle` 相当の trim-range 語彙を使う

### topology 側へ委譲すること

- trim や binding に必要な拘束端点は topology 層で保持する
- primitive の `start_point` / `end_point` を拘束端点へ再定義しない
- `parameter_range` は母曲線上の evaluated endpoint を導くための位相情報として扱う

### 実装単位 A の命名ルール

- primitive / contract では `start_point` / `end_point` を ideal endpoint 語彙として使う
- topology では `constraint_start_point` / `constraint_end_point`、`ideal_start_point` / `ideal_end_point`、`evaluated_start_point` / `evaluated_end_point` のように役割を名前へ出す
- 同じ `start/end` で ideal と constraint を兼用しない

### 実装単位 A の非目標

- Arc / EllipseArc primitive の endpoint semantics を拘束端点へ変更すること
- `point_at_parameter(0/1)` と `start_point/end_point` の常時一致を不変条件にすること
- Topology の trim / binding 完全仕様をこの段階で完了すること

`#567` では、この実装単位 A の意味論を topology 側の拘束端点語彙と接続して固定する。すなわち、Arc / EllipseArc primitive の endpoint semantics は ideal endpoint のまま維持し、拘束端点の正本は topology 側へ置く。

補足:

- `#567` は Arc / EllipseArc と topology の責務境界を固定する設計 Issue として扱う
- LineSegment を bounded curve 共通ルールへ対称化する作業は `#592` の follow-up として分離する
- したがって `#567` では Arc / EllipseArc の primitive semantics を動かさず、topology 側の語彙整理で閉じる
