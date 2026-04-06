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

CompositeCurve は、閉曲線であっても topology Loop の正本とはみなさず、Wire / Loop 構築前段の幾何補助表現として扱う。

## 標準的な命名と用途

本書では、B-Rep / CAD kernel 文脈で一般的な命名に合わせて、少なくとも次の用途で用語を固定する。

- `Vertex`: 位相上の点要素。共有接続点の正本として使う。
- `Edge`: 母曲線参照、parameter range、same_sense、start/end vertex を持つ位相要素。曲線そのものではなく、曲線の位相的利用を表す。
- `Wire`: 向きつけられた連続 Edge 列。Face 境界の候補となるが、それ自体はまだ Face 上の意味付けを持たない。
- `Loop`: 閉じた Wire が Face 境界として意味付けされた概念。outer / inner の区別は Face 側で与える。
- `CompositeCurve`: 複数曲線セグメントからなる幾何補助表現。閉曲線であっても topology の Edge / Wire / Loop の正本とはみなさない。

したがって、RedRing では「閉じた CompositeCurve をそのまま Loop と呼ぶ」ことはせず、CompositeCurve は Wire / Loop 構築前段の幾何入力または補助表現として扱う。

## 将来実装を見越した存在と設計確定項目

未実装であっても、topology entity layer の将来対象として少なくとも次の要素は存在を明示しておく。

- `Loop`: 閉じた Wire を Face 境界として意味付けるための概念
- `Face`: outer / inner loop、必要に応じて PCurve や surface 参照を伴う面位相要素
- `Shell`: 複数 Face の接続集合としての位相要素
- `Solid`: 閉じた Shell により境界づけられる立体位相要素
- `PCurve`: Face 上の境界表現を扱うための補助概念

これらは現時点では詳細未実装だが、本書のスコープ外ではない。用語の表記ゆれを防ぐため、少なくとも存在名は本書で固定する。

また、将来実装で設計確定が必要な項目として、少なくとも次を認識しておく。

- `Loop` を独立要素として持つか、閉じた `Wire` への意味付けとして扱うか
- `Face` における outer / inner loop の責務境界
- `Face` が 3D 母表現と `PCurve` をどう併用するか
- point-on-surface / curve-on-surface 判定における topology 専用 tolerance の責務分離
- `Shell` の閉性判定と法線整合の扱い
- `Solid` の境界閉包条件と non-manifold 拒否条件

したがって、本書は現時点で Edge / Wire 周辺を主に固定しているが、Face / Shell / Solid / PCurve を将来対象から除外しているわけではない。

## topology entity layer の基本方針

### 1. topology は shape 意味論を再定義しない

`geo_primitives` の shape は、それぞれの意味論を自分で持つ。

topology はそれを前提に、接続・向き・トリム・共有関係を管理する層として扱う。

したがって `LineSegment` の `support_line` / ideal endpoint / `length()` / `point_at_parameter()` の意味は、topology 側で独自に上書きしない。

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

## `#567` の設計結論

`#567` では、Arc / EllipseArc の endpoint semantics と topology の拘束端点責務を次のように固定する。

- primitive は exact geometry を表す
- Arc / EllipseArc の `start_point` / `end_point` は primitive の ideal endpoint として扱う
- topology 上の拘束端点は `Edge` の `start_vertex` / `end_vertex` 側で管理する
- `parameter_range` は evaluated endpoint を導く位相情報として扱う
- Edge の局所整合は `binding consistency`、`ideal endpoint consistency`、`evaluation endpoint consistency` を分離して定義する
- shared vertex を介した隣接 Edge 間の連続性整合は、Edge 単独の局所判定とは分けて `\delta_{shared}` 側で扱う

このとき、`#567` のスコープは Arc / EllipseArc と topology の責務境界を固定するところまでとし、LineSegment の endpoint semantics を bounded curve 共通ルールへ対称化する作業は follow-up の `#592` で扱う。

## LineSegment と topology の接続

`#592` の設計では、LineSegment も Arc / EllipseArc と同じ bounded curve 共通ルールで扱う。したがって topology では次を採用する。

- `CurveRef::Line::point_at_parameter` は support line 上の評価として扱う
- `CurveRef::Line::start_point` / `end_point` は primitive の ideal endpoint を返す
- `Edge` の `parameter_range` は support line / 母曲線上の有効区間を表す
- `start_vertex` / `end_vertex` は拘束端点との binding を表す

したがって、LineSegment の Edge でも少なくとも次の 3 種類の点を区別する。

- `constraint endpoint`: topology が binding の正本として扱う拘束端点
- `ideal endpoint`: primitive の endpoint capability が返す support line 上の端点
- `evaluation endpoint`: `parameter_range` の両端を母曲線評価した点

補足:

- `#592` 着手時点では実装に legacy な拘束点語彙が残る可能性があるが、これは移行対象であって最終設計ではない
- bounded curve 全体で primitive は ideal endpoint、拘束端点は topology 管理という原則を LineSegment にも揃える
- したがって LineSegment だけを topology 接続の例外として扱わない

## Arc / EllipseArc と topology の接続

`#558` と `#567` の前提として、Arc / EllipseArc では primitive endpoint と topology 上の拘束端点を区別して扱う。

固定方針:

- `CurveRef::Arc::start_point/end_point` は primitive の ideal endpoint を返す
- `CurveRef::EllipseArc::start_point/end_point` も primitive の ideal endpoint を返す
- `parameter_range` は evaluated endpoint 対を導くための位相情報として使う
- `start_vertex` / `end_vertex` は拘束端点との binding を表す

したがって、Arc / EllipseArc の Edge では少なくとも次の 3 種類の点を区別する。

- `constraint endpoint`: topology が binding の正本として扱う拘束端点
- `ideal endpoint`: primitive の endpoint capability が返す母曲線上の端点
- `evaluation endpoint`: `parameter_range` の両端を母曲線評価した点

設計上の既定値:

- primitive の `start_point/end_point` は ideal endpoint のまま維持する
- topology が拘束端点を必要とする場合は、Edge 側の責務として保持する
- primitive の endpoint semantics は topology 要件だけを理由に変更しない

この方針により、Arc / EllipseArc でも `binding consistency`、`ideal endpoint consistency`、`evaluation endpoint consistency` を topology 語彙で分けて定義できる。

## vertex binding invariant の設計方針

現時点では、次を固定する。

- vertex binding は topology の責務である
- binding の正判定は拘束点主導で行う
- 母曲線評価端点と vertex の完全一致を前提にしない
- ideal endpoint は binding の正本ではなく、補助検証として扱う
- binding の許容条件には topology 専用 tolerance を用いる

したがって、`Edge::is_vertex_binding_consistent` 相当の判定は少なくとも次の三段を分離できる形が望ましい。

- binding consistency: 拘束点と vertex の整合を判定する
- ideal endpoint consistency: primitive endpoint capability が返す ideal endpoint と拘束点のずれを補助検証する
- evaluation endpoint consistency: `parameter_range` の両端を評価した endpoint と拘束点のずれを補助検証する

このとき、1 本の Edge の局所整合に使う局所予算（局所許容誤差） `\delta_{edge}` は、少なくとも次の部分予算へ分解できる形が望ましい。

- `\delta_{ideal}`: ideal endpoint と拘束点の整合に使う予算
- `\delta_{eval}`: evaluated endpoint と拘束点の整合に使う予算
- `\delta_{bind}`: 拘束点と vertex の整合に使う予算

基本条件は次とする。

$$
\delta_{ideal} + \delta_{eval} + \delta_{bind} \le \delta_{edge}
$$

すなわち、ideal endpoint consistency、evaluation endpoint consistency、binding consistency は独立した局所判定として扱うが、その許容差の総和は単一 Edge の局所予算 `\delta_{edge}` の範囲で閉じなければならない。

現時点では、shape 種別、拘束の意味、評価カーネルの性質に応じて `\delta_{ideal}`、`\delta_{eval}`、`\delta_{bind}` の配分余地を残す。

ただし、shape 固有の根拠や別途の配分規約が存在しない場合の fallback として、対称配分を採用してよい。

$$
\delta_{ideal} = \delta_{eval} = \delta_{bind} = \delta_{edge} / 3
$$

この対称配分は、単一 Edge の局所整合に限定した既定値である。shared vertex を介した連続性整合にそのまま流用してはならず、`\delta_{shared}` は別予算として扱う。

すなわち、fallback として対称配分を持つ場合でも、「各段に同じ値を重複適用する」のではなく、`\delta_{edge}` の内訳として明示的に定義する。

fallback を上書きできる判断基準は、少なくとも次とする。

- 拘束点が単なる端点ではなく、編集拘束やスナップ結果として強い位相的意味を持つか
- ideal endpoint の評価誤差が、拘束点側より大きくなりやすい primitive を使うか
- 数値反復、近似、投影により evaluation endpoint consistency 側へ追加の誤差源が入るか
- 下流の continuity / point-on-surface / pitch 判定が、ideal 側誤差、evaluation 側誤差、binding 側誤差のどれにより敏感か

上記のいずれかに明確な偏りがある場合は、`\delta_{ideal}`、`\delta_{eval}`、`\delta_{bind}` を非対称に配分してよい。ただし、その場合でも配分根拠を shape 仕様または判定仕様で明示し、`\delta_{ideal} + \delta_{eval} + \delta_{bind} \le \delta_{edge}` を維持する。

未確定項目:

- `Edge::is_vertex_binding_consistent` 相当 API を `Edge` メソッドとして残すか、validator 系へ分離するか
- ideal endpoint consistency と evaluation endpoint consistency を常時必須にするか、診断系チェックとして分離するか
- tolerance を結果へどこまで記録するか
- 上書き判断基準を shape 別ガイドとして独立管理するか

### Edge の正規判定フロー

`Edge` の binding invariant は、少なくとも次の順で判定する。

1. `curve`、`parameter_range`、`same_sense`、`start_vertex`、`end_vertex` を入力として受け取る。
2. `start_vertex` / `end_vertex` から、Edge の拘束端点対 `(constraint_start_point, constraint_end_point)` を取得する。
3. `same_sense` を適用し、Edge の向きに沿った拘束端点対 `(oriented_constraint_start, oriented_constraint_end)` を決定する。
4. `start_vertex` / `end_vertex` と拘束端点対を `\delta_{bind}` で比較し、binding consistency を判定する。
5. `CurveRef::start_point()` / `end_point()` から primitive の ideal endpoint 対を取得する。
6. `same_sense` を適用し、Edge の向きに沿った ideal endpoint 対 `(oriented_ideal_start, oriented_ideal_end)` を決定する。
7. ideal endpoint 対と拘束端点対を `\delta_{ideal}` で比較し、ideal endpoint consistency を判定する。
8. `parameter_range` から evaluated endpoint 対を評価する。
9. `same_sense` を適用し、Edge の向きに沿った evaluated endpoint 対 `(oriented_evaluated_start, oriented_evaluated_end)` を決定する。
10. evaluated endpoint 対と拘束端点対を `\delta_{eval}` で比較し、evaluation endpoint consistency を判定する。
11. full edge invariant は、binding consistency、ideal endpoint consistency、evaluation endpoint consistency の全てが成立したときにのみ通過とする。

重要:

- `\delta_{shared}` は Edge 単独の判定では使用しない。
- `\delta_{shared}` は Wire など、shared vertex を介した隣接 Edge 間の連続性判定でのみ使用する。
- したがって Edge 判定だけでは、隣接 Edge 間の連続性保証までは与えない。

擬似フロー:

```text
input: edge, δ_bind, δ_ideal, δ_eval

constraint_pair = oriented_constraint_points(edge.curve, edge.same_sense)
binding_ok = compare_vertices_to_constraints(edge.vertices, constraint_pair, δ_bind)

ideal_pair = oriented_ideal_points(edge.curve, edge.same_sense)
ideal_ok = compare_ideal_to_constraints(ideal_pair, constraint_pair, δ_ideal)

evaluated_pair = oriented_evaluated_points(edge.curve, edge.parameter_range, edge.same_sense)
evaluation_ok = compare_evaluated_to_constraints(evaluated_pair, constraint_pair, δ_eval)

edge_ok = binding_ok && ideal_ok && evaluation_ok
```

このフローにより、Edge は「拘束端点と vertex の整合」「primitive ideal endpoint と拘束端点の整合」「parameter_range 由来の evaluated endpoint と拘束端点の整合」を明示的に分けて検証する。一方で、shared vertex を介した連続性は次段の Wire 判定へ委譲する。

## topology 専用 tolerance の方針

幾何判定全体のトレランス正本は別途 `ToleranceSettings` 系で管理するとしても、topology には少なくとも次の用途がある。

- vertex coincidence
- edge continuity
- binding consistency
- closed loop / closed shell 判定

このため、topology は幾何一般の tolerance をそのまま流用するだけではなく、位相判定に必要な tolerance の責務境界を持つ。

現時点の設計方針は次の通りとする。

- topology 専用 tolerance の責務は `geo_topology` の判定側に置く
- 初期段階では判定 API へ明示入力し、暗黙既定値に依存しない
- 将来的な値の正本は `ToleranceSettings` 系の共通 context と接続してよい
- ただし topology 側では、どの判定にどの tolerance を使うかという責務境界を保持する

### 誤差伝搬を抑えるための局所予算と共有予算の分離

vertex 一致判定では、単一の許容差をそのまま全ての段へ使い回すのではなく、少なくとも次の 2 種類の予算へ分けて扱う。

- Edge 単独の局所整合を扱う局所予算（局所許容誤差）
- shared vertex を介して隣接 Edge へ伝搬する連続性整合を扱う共有予算（共有許容誤差）

例えば、次を区別する。

- $\delta_{edge}$: 1 本の Edge の中で、ideal endpoint / 拘束点 / vertex の整合に使う局所予算（局所許容誤差）
- $\delta_{shared}$: shared vertex を介して隣接 Edge 同士の接続整合に使う共有予算（共有許容誤差）

さらに $\delta_{edge}$ の内訳として、少なくとも次を区別する。

- $\delta_{ideal}$: ideal endpoint と拘束点の整合に使う部分予算
- $\delta_{eval}$: evaluated endpoint と拘束点の整合に使う部分予算
- $\delta_{bind}$: 拘束点と vertex の整合に使う部分予算

基本関係は次とする。

$$
\delta_{ideal} + \delta_{eval} + \delta_{bind} \le \delta_{edge}
$$

このとき、binding consistency、ideal endpoint consistency、evaluation endpoint consistency の配分は、少なくとも $\delta_{edge}$ の範囲で閉じるように設計する。

shape 固有の配分根拠がない場合の fallback は、次とする。

$$
\delta_{ideal} = \delta_{eval} = \delta_{bind} = \delta_{edge} / 3
$$

ただし、この対称配分は単一 Edge 内の局所整合にのみ適用し、shared vertex を介した連続性整合へそのまま拡張しない。

重要なのは、`LineSegment` の 1 本の Edge 内で許した局所誤差と、shared vertex を介した隣接 Edge 間の誤差伝搬を同じ 1 つの値で管理しないことである。

これにより、ある Edge で許容した ideal endpoint と拘束点のずれが、shared vertex を介して隣接 Edge のずれと累積し、設計意図を超えた全体誤差へ膨らむことを避けやすくなる。

この考え方は `LineSegment` の binding だけに限定しない。Face 上の point-on-surface 判定、curve-on-surface 判定、ピッチ判定のような「母表現と拘束表現のずれ」を扱う場面でも、局所整合の予算と複数要素をまたぐ共有整合の予算を分離するトレラントモデリングの基本思想として再利用する。

ただし、個別の tolerance 名、値の最終配置、API 形状は別設計で具体化する。

### ニット前段で固定する設計判断

shared edge の一本化そのものを確定する前段として、少なくとも次を固定する。

- RedRing の通常 topology では non-manifold を受け付けない
- ニット判定の既定対象は 2 本の境界 Edge による bilateral knit とする
- アプリケーション層は shared edge 用の総許容差 `\delta` のみを与える
- カーネル内部は bilateral knit の既定規則として各側比較に `\delta / 2` を適用する
- この `\delta / 2` 配分規則は kernel invariant としてカプセル化し、外部 API へ露出しない

このとき、アプリケーション側は「共有境界全体としてどこまで許容するか」という総予算 `\delta` だけを指定し、A 側と B 側への配分責務はカーネルが負う。

bilateral knit の既定規則では、各側の比較条件を次で表す。

$$
d(A, E_{ref}) \le \delta / 2
$$

$$
d(B, E_{ref}) \le \delta / 2
$$

ここで `E_{ref}` は、共有境界候補を評価するための kernel 内部基準表現を表す。一本化後の正準 shared edge を直ちに意味するとは限らない。

この規則を採ることで、同一距離尺度の下では導出的に次を保証できる。

$$
d(A, B) \le \delta
$$

重要なのは、外部から受け取った同じ `\delta` を A 側比較と B 側比較へそのまま重複適用しないことである。各側に同じ `\delta` を直適用すると、導出的上界は `2\delta` となり、外部契約として期待した総許容差とずれる。

### ニット候補判定の安定化方針

non-manifold を拒否する前提であっても、spike 形状や鋭角合流では、境界線距離だけに依存した候補選択は不安定になりうる。

したがって、ニット候補判定は単一の最近傍距離だけで決めず、少なくとも次を満たすことを要求する。

- 距離条件: 候補境界間の乖離が総許容差 `\delta` の範囲内にあること
- 端点整合条件: 両端の対応が取れ、端点近傍だけの擬似一致でないこと
- 方向整合条件: 接線方向や走査向きが矛盾せず、対応追跡が単調に取れること
- 区間整合条件: 局所点一致ではなく、共有境界候補として扱うだけの連続区間が存在すること
- 曖昧性拒否条件: 同程度に成立する複数候補がある場合は、無理に採用せず reject すること

したがって、ニット処理の前段は少なくとも次の順で構成するのが望ましい。

1. 候補探索
2. 候補検証
3. 曖昧性判定
4. 共有境界の採否決定

この段階では、shared edge を最終的に 1 本化するか、どの幾何生成規則で正準化するかは未確定とする。ただし、少なくとも候補対応の安定化は上記の規則で先に固定する。

### shared edge 一本化の採用条件

shared edge を最終的に 1 本化するかどうかは、「G1 連続であるか」だけでは決めない。一本化は、shared boundary としての対応が一意かつ安定に追跡できる場合に限って採用する。

したがって、G1 は一本化を後押しする補助条件として扱うが、単独で十分条件とはみなさない。

一本化を採用してよい既定条件は、少なくとも次とする。

- bilateral knit として 2 本の境界 Edge の対応が一意に定まること
- 候補境界間の距離条件が総許容差 `\delta` の範囲内で成立すること
- 両端の対応が安定に定まり、端点近傍だけの擬似一致でないこと
- 対応区間の追跡が単調に行え、途中で相手候補が入れ替わらないこと
- 方向整合が取れ、接線方向の不連続が shared boundary として許容できる範囲に収まること
- 同程度に成立する競合候補が存在しないこと

上記を満たす場合、カーネルは 2 本の入力 Edge を 1 本の shared edge へ正準化してよい。

一方で、次のいずれかに該当する場合は、一本化を採用してはならない。

- spike 形状や鋭角合流により、距離最小候補が微小擾乱で入れ替わる
- 一部の局所区間だけが近接し、共有境界として扱う連続区間が安定に取れない
- 接線方向は近いが、端点対応または区間対応が一意に定まらない
- 複数候補が同程度に成立し、どれを shared edge 正本とすべきか決定できない

この場合、カーネルは「なるべく 1 本化する」よりも「誤った 1 本化を行わない」ことを優先し、一本化を reject する。

したがって、一本化の基本方針は次とする。

1. 候補対応が一意かつ安定に追跡できる場合のみ 1 本化する
2. G1 は一本化可否の補助条件として使う
3. 曖昧性が残る場合は 1 本化せず reject する

未確定項目:

- 正準 shared edge の具体的な幾何生成規則を、中間表現・片側優先・最適化近似のどれで定めるか
- 一本化後の parameter range をどの表現で正本化するか
- shared vertex 側の同時正準化を必須にするか

### 正準 shared edge の既定生成規則

一本化を採用する場合でも、正準 shared edge の生成規則は「2 本の入力を機械的に中間化すること」を既定としない。既定規則は、幾何平均よりも対応の安定性、端点拘束の保存、parameter 対応の単調性を優先する。

したがって、bilateral knit における正準 shared edge の生成は、少なくとも次の優先順で扱う。

1. 端点対応が安定に定まることを先に確認し、必要なら shared vertex を shared edge と同時に正準化する
2. 2 本の入力 Edge から、同一の母表現または安定な基準表現を選べる場合は、その表現を shared edge の正本候補として優先する
3. 一方の入力 Edge を基準として採用しても他方が `\delta / 2` 以内へ安定に再拘束できる場合は、決定的規則に基づく片側優先を許容する
4. 上記で安定な基準表現を得られない場合に限り、最適化近似または中間表現を fallback として検討してよい
5. fallback を用いても距離条件・端点整合・方向整合・区間整合・曖昧性拒否条件を満たせない場合は、一本化を reject する

この優先順において、既定の思想は「まず安定な参照を選ぶ」「平均化は最後の手段とする」である。

#### 1. 母表現優先

2 本の入力 Edge が、同一の解析曲線、同一 support line、または同一 NURBS 系表現として無理なく正準化できる場合は、その母表現を shared edge の正本候補として優先する。

この場合の shared edge は、入力 Edge の中点的平均ではなく、「両入力が整合すべき共通基準表現」として扱う。

#### 2. 片側優先

両入力を完全に対称な共通表現へ落とすより、一方の入力 Edge を基準として採用した方が安定である場合は、片側優先を許容してよい。

ただし、片側優先は任意に決めてはならず、少なくとも次のいずれかのような決定的規則に従う必要がある。

- 解析的により強い表現を優先する
- 数値評価の安定性が高い表現を優先する
- 既存 topology 所有者として継続利用すべき正本が定まっている

この場合でも、非採用側は採用側 shared edge へ `\delta / 2` 以内で再拘束できなければならない。

#### 3. 中間表現・最適化近似は fallback

中間表現や最適化近似は、入力 2 本の対称性を保ちやすい一方で、元の解析構造や parameter 意味を弱める可能性がある。

したがって、これらは既定の第一選択ではなく、母表現優先または片側優先で安定な shared edge を定められない場合の fallback とする。

fallback を採用する場合でも、少なくとも次を維持しなければならない。

- 両端の shared vertex との整合
- 各入力 Edge からの `\delta / 2` 以内の再拘束可能性
- 対応区間追跡の単調性
- 後段の topology 判定で利用できる一意な parameter 意味

このため、「G1 で見た目が滑らかだから平均化する」という理由だけで中間表現を既定採用してはならない。

#### 4. 現時点の既定方針

現時点では、正準 shared edge の既定方針を次とする。

1. まず母表現優先で共通基準を探す
2. それが無理なら決定的規則に基づく片側優先を検討する
3. それでも安定に定まらない場合に限り、中間表現または最適化近似を fallback 候補とする
4. fallback でも条件を満たせない場合は一本化を reject する

したがって、正準 shared edge の生成は「対称平均を既定とする設計」ではなく、「安定な正本を段階的に選び、それが無理なら reject する設計」とする。

## `#557` 時点の暫定許容範囲

`#557` では topology の現行構造を直ちに変更しないことを許容したが、`#592` ではその legacy 前提を最終設計として残さない。

許容する範囲:

- `CurveRef`
- `Edge` の基本保持フィールド
- `Wire` / `CompositeCurve` の基本構造そのもの

未確定として残す範囲:

- `Edge` / `Wire` / validator の具体 API 形状
- LineSegment の endpoint を ideal endpoint として解釈した上での topology 接続語彙
- tolerance 名と設定オブジェクトの最終配置
- Face / Shell / PCurve へ展開したときの個別判定フロー

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
