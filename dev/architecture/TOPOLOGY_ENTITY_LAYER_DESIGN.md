# Topology Entity Layer Design

## 目的

本書は、RedRing における topology entity layer の設計正本を定義する。

特に、以下を固定する。

- `geo_topology` が何を正本として保持するか
- `curve + parameter range + same_sense + vertex binding` の責務境界
- shape 意味論と topology 不変条件の接続方法
- vertex binding と位相専用 tolerance をどこで扱うか

本書は、個別 shape の意味論や実装計画そのものを定義する文書ではない。

## 正本の役割分担

- shape 意味論の正本: `GEOMETRY_SHAPE_SEMANTICS_DESIGN.md`
- topology entity layer 設計の正本: 本書
- trait 境界と capability 分担の正本: `GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md`
- Phase 4 実装計画の補助説明: `PHASE4_TOPOLOGY_ENTITY_DESIGN.md`
- #458 時点の凍結スナップショット: `TOPO_NORMAL_FORM_INVARIANTS_FREEZE_458.md`

補足:

- `TOPO_NORMAL_FORM_INVARIANTS_FREEZE_458.md` は、#458 時点の簡易実装前提でまとめた凍結文書であり、現行 topology 設計の単独正本とはみなさない
- topology の現行設計判断は、本書を起点に管理する

## スコープ

本書は、少なくとも次を対象とする。

- `Vertex`
- `CurveRef`
- `Edge`
- `Wire`
- `CompositeCurve`
- vertex binding invariant
- topology 専用 tolerance の責務境界

Face / Shell / Solid / PCurve の詳細仕様は、必要に応じて本書から派生文書へ分離してよい。

## topology entity layer の基本方針

### 1. topology は shape 意味論を再定義しない

`geo_primitives` の shape は、それぞれの意味論を自分で持つ。

topology はそれを前提に、接続・向き・トリム・共有関係を管理する層として扱う。

したがって `LineSegment` の `support_line` / 拘束点 / `length()` / `point_at_parameter()` の意味は、topology 側で独自に上書きしない。

### 2. Edge の正本は母曲線参照と位相情報の組である

`Edge` は少なくとも次を保持する。

- `curve`
- `parameter_range`
- `same_sense`
- `start_vertex`
- `end_vertex`

このとき、母曲線の幾何と位相上の走査向きは分離して扱う。

### 3. 向き反転は `same_sense` で表現する

母曲線自体を破壊的に反転せず、位相上の走査方向は `same_sense` で管理する。

### 4. vertex binding は母曲線評価そのものと同一視しない

topology で保持する vertex は、shape の拘束点と接続して扱う。

したがって、vertex binding invariant は「母曲線評価端点と vertex が常に一致すること」ではなく、shape 意味論を前提にした binding ルールとして定義し直す。

## LineSegment と topology の接続

`#557` で固定した `LineSegment` semantics を前提に、topology では次を採用する。

- `CurveRef::Line::point_at_parameter` は support line 上の評価として扱う
- `CurveRef::Line::start_point` / `end_point` は拘束点参照として扱う
- `Edge` の `parameter_range` は support line / 母曲線上の有効区間を表す
- `start_vertex` / `end_vertex` は拘束点との binding を表す

このため、support line 上の ideal endpoint と vertex が一致しない場合を許容できる topology invariant が必要になる。

## vertex binding invariant の現行判断

現時点では、次を固定する。

- vertex binding は topology の責務である
- binding 判定は shape 意味論を前提に行う
- 母曲線評価端点と vertex の完全一致を前提にしない
- binding の許容条件には topology 専用 tolerance を用いる

未確定項目:

- binding 判定を拘束点ベースで固定するか、ideal endpoint との二重検証にするか
- `Edge::is_vertex_binding_consistent` 相当 API の最終仕様
- topology 専用 tolerance の正本配置

## topology 専用 tolerance の方針

幾何判定全体のトレランス正本は別途 `ToleranceSettings` 系で管理するとしても、topology には少なくとも次の用途がある。

- vertex coincidence
- edge continuity
- binding consistency
- closed loop / closed shell 判定

このため、topology は幾何一般の tolerance をそのまま流用するだけではなく、位相判定に必要な tolerance の責務境界を持つ。

ただし、値の最終配置と API は別 Issue で確定する。

## `#557` 時点の暫定許容範囲

`#557` の段階では、topology の現行構造を直ちに変更しないことを許容する。

許容する範囲:

- `CurveRef`
- `Edge` の基本保持フィールド
- `Wire` / `CompositeCurve` の拘束点ベース利用

未確定として残す範囲:

- vertex binding invariant
- topology 専用 tolerance
- ideal endpoint と拘束点のずれを含む binding ルール

## 非目標

本書は以下を直接対象としない。

- shape 個別意味論の定義
- `geo_primitives` の API 詳細
- full B-Rep 実装計画
- ロードマップや週次計画

## 利用ルール

- topology 設計判断を追加する場合は、まず本書の責務境界に反しないか確認する
- 実装計画や過去凍結内容は本書を上書きしない
- `PHASE4_TOPOLOGY_ENTITY_DESIGN.md` と `TOPO_NORMAL_FORM_INVARIANTS_FREEZE_458.md` は、必要なら本書への参照を追加して利用する
