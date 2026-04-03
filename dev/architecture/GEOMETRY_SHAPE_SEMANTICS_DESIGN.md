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

## 横展開前提の論点

`#557` で固定する内容は、将来的に他 shape へ横展開が必要な論点として扱う。

- exact primitive と tolerant shape の区別
- どの値を正本とし、どの値を派生として扱うか
- `start` / `end` 系 API が拘束点を返すのか、ideal endpoint を返すのか
- `length` / `measure` / parameter range が何を表すのか
- `point_at_parameter` が support curve 上の評価か、拘束点補間か

したがって `LineSegment` は局所例外ではなく、shape semantics の横断整理を先行して具体化する最初のケースとして扱う。

## LineSegment の意味論

### 基本方針

RedRing における `LineSegment2D/3D` は、単なる exact な有限線分ではなく、**support line を正本とし、拘束点としての始点・終点を併せ持つ tolerant な trimmed line** として扱う。

これにより、以下の意図を表現できる。

- 理想的には一直線上にある線分である
- ただし始点・終点は拘束や数値誤差の結果として、理想支持線からトレランス以内でずれる可能性がある
- topology 側では、その拘束点の一致/不一致が接続判定や編集挙動に影響する

### 正本と派生

`LineSegment` では、概念的に次の値を区別する。

- 正本: `support_line`
- 正本: `start` / `end`（拘束点）
- 派生: support line 上の理想トリム区間
- 派生: ideal な端点
- 派生: support line 上の評価点

重要:

- 具体的な struct フィールド構成は実装都合で調整してよい
- ただし API と意味論は、上記の区別を壊してはならない

### API 意味の固定

`LineSegment2D/3D` の主要 API は次の意味で扱う。

| API | 意味 |
| --- | --- |
| 拘束点としての始点を返す API（現行: `start()` / `start_point()`） | 拘束点としての始点を返す |
| 拘束点としての終点を返す API（現行: `end()` / `end_point()`） | 拘束点としての終点を返す |
| `length()` | 拘束点間距離を返す |
| `point_at_parameter(t)` | support line 上の評価点を返す |
| `measure()` | 主語彙にしない。互換の委譲としてのみ扱う |

補足:

- `start/end` と `point_at_parameter(0/1)` は一致を前提としない
- `length()` は support line 上の ideal length を意味しない

### 補助 API の扱い

以下の補助 API は、意味の明確化に有効であれば導入を許容する。

- `support_line()`
- `ideal_start()` / `ideal_end()`
- `ideal_length()`
- support line 評価と拘束点補間を分ける追加 API

補助 API を導入する場合は、拘束点系と ideal 系を名称で明確に分離する。

## geo_primitives と geo_topology の境界

`geo_primitives::LineSegment2D/3D` は shape 意味論を持つ。

一方、`geo_topology` は以下を追加で管理する。

- vertex binding
- same_sense
- parameter range
- edge / wire / face 文脈での接続関係

したがって、topology は `LineSegment` の意味論を再定義しない。
topology 側の Edge 正規形は、shape 意味論を前提に、接続・向き・トリムを管理する層として扱う。

## `#557` における topology 未修正の暫定許容範囲

`#557` では、LineSegment の shape semantics を先に固定する。

その際、topology 側は未修正のまま次の範囲までを暫定的に許容する。

- `CurveRef::Line::point_at_parameter` が support line 上の評価を返すこと
- `Edge` が `curve + parameter range + same_sense + vertex binding` を保持する現行構造を維持すること
- `Wire` / `CompositeCurve` が拘束点ベースの端点参照を使うこと

一方、次の項目は `#557` の完了条件には含めず、別途 topology 側で設計確定が必要とする。

- `Edge::is_vertex_binding_consistent` のように、母曲線評価端点と vertex の一致を前提にする不変条件の最終定義
- 拘束点と ideal endpoint のずれを topology がどう許容するかという binding ルール
- 位相専用 tolerance の正本化と運用方針

したがって `#557` の段階では、「topology の現行構造を直ちに変更しないこと」は許容されるが、「現行 topology 不変条件が最終設計である」とはみなさない。

## 非目標

本書は以下を直接の対象としない。

- `Measure` の shape 横断 taxonomy の最終確定
- Arc / EllipseArc / Circle / Triangle など各 shape の個別 semantics 確定
- topology 正規形の完全仕様
- tolerance 値そのものの単一正本化

## #557 での利用

`#557` では、本書を前提に以下を整理する。

- `LineSegment` の API を tolerant trimmed line semantics に沿って明文化する
- `length` と `measure` の役割分担を整理する
- 利用側が exact finite segment と誤解しないよう contracts / primitives / topology 周辺を調整する

## 今後の拡張

本書は将来的に、`#557` で固定した論点を横展開し、次の shape semantics を追加できる形で維持する。

- Arc
- EllipseArc
- Circle
- Triangle
- NURBS curve / surface

ただしその追加は、shape 横断 capability 整理を扱う `#558` の方針確定後に行う。
