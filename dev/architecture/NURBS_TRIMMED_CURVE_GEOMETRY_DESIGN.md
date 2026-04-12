# NURBS Trimmed Curve Geometry Design

## 最終更新日: 2026年4月12日

## 背景

- #650 の検討中に、`geo_topology` 側だけで NURBS edge を先行導入すると、trimmed NURBS curve を geometry 層で自然に扱えない問題が先に顕在化した
- 現状の `NurbsCurve3D` は曲線本体であり、`LineSegment3D` のような bounded geometry としての trim 状態を geometry 単体では表現していない
- そのため、交差計算・距離計算・離散化など geometry 指定の処理系で、trimmed NURBS だけが topology 経由を要求しやすい
- `geo_topology::Edge` の `parameter_range` は位相利用の区間であり、geometry そのものの有限曲線表現の代替にはしない

## 問題設定

整理したい論点は次の 2 つである。

1. `NurbsCurve3D` の母曲線と、trim 区間つき有限曲線を geometry 層でどう分離するか
2. `geo_nurbs` / `geo_topology` / `geo_algorithms` の責務境界を崩さずに trimmed NURBS curve を導入できるか

現時点では、topology 先行ではなく geometry 表現を先に固定する方が収まりがよい。

## 現状整理

### 既存 bounded geometry

- `LineSegment3D` は support line と trim 区間を geometry 自身が保持する
- `Arc` / `EllipseArc` は有限曲線 primitive として self-contained に振る舞う
- したがって、line/arc 系は geometry 単体で有限曲線として扱える

### 既存 NURBS

- `NurbsCurve3D` は制御点・重み・ノット・次数を保持する曲線本体である
- `parameter_domain()` は曲線本体の native parameter domain を返す
- 現状の `NurbsCurve3D` には trim 区間付き有限曲線の責務を混ぜていない

### topology 側の役割

- `Edge` は母曲線参照、`parameter_range`、`same_sense`、vertex binding を持つ位相要素である
- ここでの `parameter_range` は topology の走査・整合・接続に必要な区間情報であり、geometry API の入力統一を担うものではない

## 判断

第一候補は、`geo_nurbs` に trimmed NURBS curve 専用の geometry 型を導入する方針とする。

### 採らない方針

#### A. `NurbsCurve3D` 自体に trim 概念を混ぜる

この案は採らない。

理由:

- `parameter_domain()` が母曲線 domain なのか trim 後 domain なのか曖昧になる
- 既存 trait定義の意味が崩れやすい
- full curve 前提の既存アルゴリズムに影響が波及する
- 「元曲線」と「有限曲線」を同一型で兼ねると責務が混ざる

#### B. topology-aware API を先に増やす

この案は第一候補にしない。

理由:

- geometry API と topology API が二系統化しやすい
- NURBS だけ topology 経由を要求しやすく、一貫性が悪い
- 問題の根本である「trimmed NURBS を geometry として持てない」を解決しない

## 第一候補

`geo_nurbs` に、元の `NurbsCurve3D` を保持する有限曲線型を追加する。

仮称:

- `TrimmedNurbsCurve3D<T>`

想定データ:

- `basis_curve: NurbsCurve3D<T>` または共有参照
- `trim_range: (T, T)`
- `geometry_direction` 相当の幾何向き情報

基本方針:

- split 済みの別 NURBS を正本にしない
- 正本は「母曲線 + trim 区間」とする
- 必要な場面でのみ exact split を補助処理として使う

## Entity 正本モデル

既存 CAD データの編集整合性は、geometry や topology を直接正本にしないことで担保する。

### 正本の置き場所

- 永続状態の正本は Entity に置く
- geometry と topology は Entity から生成して取得する
- 取得される geometry / topology はコピーとして独立してよい

### 編集の原則

- 既存 CAD データの編集は Entity を介した操作のみとする
- geometry 単独、topology 単独をそのまま永続状態へ戻す前提は置かない
- 編集コマンドの内部で、Entity を起点に geometry / topology の両方を再整合する

### この原則が意味すること

- geometry copy を形状処理で編集してよい
- topology copy を位相処理で編集してよい
- ただし最終的な永続反映は Entity 経由に限定する
- したがって geometry 側と topology 側が別々の向き情報を持っていても直ちに矛盾ではない

## この方針の利点

- trimmed NURBS を geometry 指定 API に自然に載せやすい
- 元の `NurbsCurve3D` を失わない
- `LineSegment3D` や `Arc` と同じく有限曲線として扱える
- `geo_topology` は geometry 後段の接続語彙として責務を維持できる
- 将来の trimmed face / p-curve 設計でも、bounded geometry と topology の境界を保ちやすい
- Entity 正本モデルにより、geometry/topology コピーの編集整合を application 側で制御できる

## 想定 API 原則

### 母曲線アクセス

- trimmed curve は母曲線への参照または所有を保持する
- 呼び出し側は必要に応じて母曲線へアクセスできる

### endpoint semantics

- `start_point` / `end_point` は trim 区間端を評価した有限曲線の端点を返す
- ここでは topology の拘束端点語彙とは混同しない

### geometry の向き

- trimmed NURBS geometry は、形状処理で必要な向きを持ってよい
- ここでの向きは finite geometry の始終・評価順序・編集順序のための向きである
- topology の `same_sense` や Loop / Face / Shell の向きとは別概念として扱う

### parameter semantics

検討案は 2 つある。

1. trimmed curve も母曲線 native domain を公開する
2. trimmed curve の局所 parameter domain を別途持ち、母曲線 domain への写像 API を明示する

現時点では 1 を優先候補とする。

理由:

- NURBS の native parameter を隠さない
- `geo_topology::Edge::parameter_range` との接続が自然
- split や knot insertion など既存 NURBS 操作との整合がよい

ただし、bounded geometry としての使い勝手をそろえるため、局所 `0..=1` parameter を受ける `evaluate_local()` は convenience API として初手から持ってよい。

固定方針:

- 主 API はあくまで native parameter を受ける `evaluate()` とする
- `evaluate_local()` は trim 区間を `0..=1` へ線形写像する convenience に留める
- `evaluate_local()` は local domain 自体を shape の正本 domain とみなさない
- topology の edge-local parameter と値域は同じ `0..=1` を採ってよいが、責務は別とする

理由:

- [GEOMETRY_SHAPE_SEMANTICS_DESIGN.md](./GEOMETRY_SHAPE_SEMANTICS_DESIGN.md) では bounded curve family の trim-local parameter を `0..=1` で扱う既存方針がある
- 一方で trimmed NURBS は mother curve native parameter を失いたくないため、local parameter を主語彙へ昇格させるべきではない
- `geo_topology::Edge::point_at()` も `0..=1` の local parameter を使うため、geometry 側に同じ convenience があると接続上は扱いやすい

## 向き概念の分離

現行議論では、geometry の向きと topology の向きが `same_sense` 一語へ混在しやすい。ここは明示的に分離する。

### geometry 側の向き

- trimmed curve 自体の始点→終点
- 局所 parameter 増加方向
- 編集処理や形状処理で使う向き

### topology 側の向き

- Edge の走査方向
- Loop / Face / Shell 等との接続・境界利用の向き
- topology 文脈での有向利用
- 本文書では概念名を `topology_orientation` と呼ぶ

### 関係値

- geometry 向きと topology 走査方向の一致/不一致を表す relation は別概念として扱う
- 現在の `same_sense` はこの relation に近く、geometry の純粋向きでも topology の高位向きでもない
- 本文書では概念名を `orientation_relation` と呼ぶ

### 命名判断

- finite geometry 側の向きは `geometry_direction` で固定する
- topology 側の向き概念は `topology_orientation` で固定する
- 両者の一致/不一致は `orientation_relation` で固定する
- `orientation_relation` は少なくとも `Aligned` / `Reversed` を表現できればよい
- 既存 `same_sense` は `Edge` 上で `orientation_relation` を表す暫定フィールド名として扱う
- #666 の完了条件では「概念名を固定する」ことを優先し、`same_sense` の実コード再命名は必須にしない

### 命名をこう分ける理由

- `geometry_direction` は finite geometry 単体で完結する unary な性質である
- `topology_orientation` は Edge / Loop / Face の接続文脈で意味を持つ topology 側の語彙である
- `orientation_relation` は geometry と topology の二者関係であり、単体 shape property ではない
- したがって `same_sense` を geometry property 名として流用せず、relation 名へ押し戻す

### 現時点の運用方針

- `geo_nurbs` の trimmed curve では `geometry_direction` を使う
- `geo_topology` の Edge では、現状の field 名として `same_sense` を残してよい
- ただし設計文書・Issue・今後の説明では、`same_sense` を「`orientation_relation` を保持する既存 bool field」として説明する
- 将来 `geo_topology` の命名整理を行う場合は、`same_sense` の再命名を独立タスクとして切り出す

## 最小 trait定義案

初期案では、`geo_contracts` に concrete な `geo_nurbs::NurbsCurve3D` や `Arc` の所有形態を持ち込まない。

そのため、trimmed NURBS curve 用 trait は「母曲線 capability を持つ associated type を参照する」形を第一候補とする。

### 方針

- contracts 層は concrete な `NurbsCurve3D` 実装型を知らない
- 初手の trimmed curve は `basis_curve` を値所有する方針を第一候補とする
- まずは finite geometry として必要な capability だけを最小露出する
- 既存 NURBS curve が `Constructor + Properties` を definition core として持つため、trimmed curve も同じ説明軸へ寄せる
- `orientation_relation` は unary な geometry property ではないため、初手の trimmed curve core trait には含めない
- relation を contracts へ出す場合でも、既存方針に合わせて core ではなく operations 側の relation trait として扱う
- 既存 contracts の `Direction2D` / `Direction3D` は正規化ベクトル方向を指すため、Forward/Reversed 用 enum は別名にして意味衝突を避ける

### ownership 方針

- 第一候補は `basis_curve: Self::BasisCurve` の値所有とする
- 初手では `Arc<NurbsCurve3D<T>>` のような共有参照前提を contracts に持ち込まない
- 共有所有は、必要になった時点で実装型内部最適化または別 constructor 方針として後から検討する
- associated type の一般化は `BasisCurve` の 1 個に留める

理由:

- 既存 `NurbsCurve3DConstructor` は shape 自身を値として生成する definition trait であり、trimmed curve だけ shared ownership 前提にすると family 内の一貫性が崩れる
- `geo_nurbs` 側の shape responsibility は ownership と domain semantics を含むため、初手で shape 自身が basis curve を所有する方が責務境界を説明しやすい
- contracts 層へ共有所有戦略を先に固定すると、`Arc` 依存や clone コスト前提を API へ持ち込みやすい
- trimmed curve の第一目的は finite geometry semantics の固定であり、共有最適化は後追いでよい
- 既存 [model/geo_contracts/src/geometry/foundation/bounds.rs](../../model/geo_contracts/src/geometry/foundation/bounds.rs#L18) の `Bounded::Aabb` も、返す concrete family だけを隠す最小 associated type に留めている

### associated type の一般化境界

初手で一般化する associated type は `BasisCurve` だけに固定する。

採用方針:

- `BasisCurve: NurbsCurve3DCore<T>` は残す
- `BasisCurveRef`
- `BasisCurveStorage`
- `BasisCurveOwned`
- `LocalParameter`
- `ConstructorError`

のような追加 associated type は初手では導入しない。

理由:

- いま必要なのは「母曲線 family の concrete 実装を contracts から隠す」ことだけであり、参照形態や storage policy まで抽象化する必要はない
- storage / ref / error まで associated type 化すると、#666 の設計対象が finite geometry semantics から generic plumbing へずれる
- `Result<Self, String>` の粗さは残るが、少なくとも現段階では error taxonomy より責務境界の固定が優先である
- local parameter は `0..=1` convenience として固定済みであり、associated type で抽象化する利得が薄い

### associated type の置き場所

`BasisCurve` は各 trait に重複定義せず、共通の basis trait に 1 回だけ置く第一候補とする。

候補:

```rust
pub trait TrimmedNurbsCurve3DBasis<T: Scalar> {
	type BasisCurve: NurbsCurve3DCore<T>;
}
```

この basis trait を、`Properties` / `Constructor` / 必要なら `Evaluation` が継承する。

理由:

- `Properties` と `Constructor` の両方で別々に `type BasisCurve` を宣言すると、trait ごとに異なる basis type を理論上は許してしまう
- 共通 trait へ寄せれば、「trimmed curve の母曲線 family は 1 つ」という前提を contracts 側で明示できる
- 一方で basis trait 自体は最小であり、storage policy や relation capability まで巻き込まない

最終方針:

- `Properties` は `TrimmedNurbsCurve3DBasis<T>` を継承する
- `Constructor` は `TrimmedNurbsCurve3DBasis<T>` を継承する
- `Evaluation` も `TrimmedNurbsCurve3DBasis<T>` を継承する
- `Endpoint` / `Derived` は basis trait を必須継承にしなくてよい

理由:

- `Evaluation` は generic な利用側で `Self::BasisCurve` を参照できた方が、native parameter semantics との接続を崩しにくい
- 一方 `Endpoint` / `Derived` は basis type を直接参照せずとも意味論を完結させやすく、不要な trait 依存を増やさない方が軽い

### `geo_nurbs` 実装側の固定方針

`geo_nurbs` の trimmed curve 実装では、`BasisCurve` を concrete に `NurbsCurve3D<T>` へ固定する。

想定:

```rust
impl<T: Scalar> TrimmedNurbsCurve3DBasis<T> for TrimmedNurbsCurve3D<T> {
	type BasisCurve = NurbsCurve3D<T>;
}
```

意味:

- contracts 側は `BasisCurve: NurbsCurve3DCore<T>` までしか知らない
- `geo_nurbs` 実装では、その concrete family が `NurbsCurve3D<T>` であることをここで確定する
- これにより contracts は抽象を保ちつつ、実装側は追加 adapter なしで既存 `NurbsCurve3D<T>` 能力を再利用できる

### geometry_direction の値型命名

- property 名は `geometry_direction` を維持する
- ただし値型名は `CurveDirection` ではなく `ParametricDirection` を第一候補とする

理由:

- 既存の `Direction2D` / `Direction3D` はベクトル方向型であり、`CurveDirection` だと同系列のベクトル型に見えやすい
- trimmed curve 側で必要なのは「単位ベクトル方向」ではなく「母曲線 native parameter に対して Forward / Reversed のどちらか」である
- `ParametricDirection` なら、値の意味が parameter 増加方向基準であることを直接表せる
- property 名としての `geometry_direction` は、編集順序・評価順序を含む geometry 側の語彙として残せる

### 候補 trait

```rust
pub enum ParametricDirection {
    Forward,
    Reversed,
}

pub trait TrimmedNurbsCurve3DBasis<T: Scalar> {
	type BasisCurve: NurbsCurve3DCore<T>;
}

pub trait TrimmedNurbsCurve3DProperties<T: Scalar>: TrimmedNurbsCurve3DBasis<T> {

	/// 母曲線へアクセスする
	fn basis_curve(&self) -> &Self::BasisCurve;

	/// 母曲線 native parameter 空間での trim 区間
	fn trim_range(&self) -> (T, T);

	/// finite geometry としての公開向き
	fn geometry_direction(&self) -> ParametricDirection;
}

pub trait TrimmedNurbsCurve3DEndpoint<T: Scalar> {
	/// trim 区間始端を評価した有限曲線の始点
	fn start_point(&self) -> (T, T, T);

	/// trim 区間終端を評価した有限曲線の終点
	fn end_point(&self) -> (T, T, T);
}

pub trait TrimmedNurbsCurve3DEvaluation<T: Scalar>: TrimmedNurbsCurve3DBasis<T> {
	/// 母曲線 native parameter を直接指定して評価する
	fn evaluate(&self, u: T) -> Option<(T, T, T)>;

	/// trim 区間を `0..=1` へ正規化した局所 parameter で評価する convenience API
	fn evaluate_local(&self, local_t: T) -> Option<(T, T, T)>;
}

pub trait TrimmedNurbsCurve3DDerived<T: Scalar> {
	/// trim 区間に限定した曲線長
	fn arc_length(&self, tolerance: T) -> T;
}

pub trait TrimmedNurbsCurve3DConstructor<T: Scalar>: TrimmedNurbsCurve3DBasis<T> + Sized {

	/// basis curve を値として受け取り finite geometry を構築する
	///
	/// `trim_range` は mother curve native parameter 空間で与え、
	/// 実装は暗黙の並べ替えや反転正規化を行わない。
	fn from_basis_curve(
		basis_curve: Self::BasisCurve,
		trim_range: (T, T),
	) -> Result<Self, String>;

	/// finite geometry の parametric 向きを補助的に上書きする
	fn with_geometry_direction(self, direction: ParametricDirection) -> Self;
}
```

### relation を contracts に出さない理由

- `geometry_direction()` は trimmed curve 単体で完結する unary property である
- 一方 `orientation_relation` は「ある finite geometry の向き」と「ある topology 利用の向き」の一致関係であり、単体 shape property ではない
- 既存の contracts 方針でも、相手を必要とする relation は core ではなく operations 側へ分離する
- 今回の最小スコープでは、trimmed curve を topology 非依存の geometry として成立させることが先であり、topology との relation 契約を同時に固定しない

### 将来 relation を追加する場合の置き場所

将来 `orientation_relation` を contracts 化する必要が出た場合でも、追加先は trimmed curve の properties/core ではない。

第一候補:

- `geo_contracts::geometry::operations` 相当の relation trait として追加する
- ただし相手型は `Edge` そのものではなく、topology 向きを問い合わせられる capability に切り出す

避けること:

- `TrimmedNurbsCurve3DProperties` に topology relation を直接混在させる
- `geometry_direction()` と `orientation_relation` を同じ trait責務に入れる
- `same_sense` 相当の bool を geometry 側 public API に早期固定する

### 露出する意味論

- `basis_curve()` は元の NURBS 曲線を失わないことを保証するための最小 access とする
- `trim_range()` は母曲線 native domain 基準で固定する
- `geometry_direction()` は finite geometry の向きを返す
- 返り値の `ParametricDirection` は、native parameter 増加方向に対して Forward / Reversed を表す
- これは topology の `same_sense` と同一責務にしない
- `evaluate()` は native parameter を隠さない
- `evaluate_local()` は trim 区間を `0..=1` へ写像する convenience として分離し、native semantics の主 API を曖昧にしない
- `start_point()` / `end_point()` は finite geometry の端点語彙として扱う

### constructor の失敗条件

`from_basis_curve()` は `trim_range` を黙って正規化しない。少なくとも次の場合は失敗を返す第一候補とする。

- `trim_range.0 >= trim_range.1`
- `trim_range` が `basis_curve.parameter_domain()` の外に出る
- 実装上、端点評価や有限曲線生成の前提を満たせない

明示的に採らないこと:

- `(t1, t0)` を自動で `(t0, t1)` に並べ替える
- reverse を表す目的で `trim_range` の順序反転を黙って受け入れる
- domain 外の値を clamp して成功扱いにする

理由:

- `geometry_direction` / `ParametricDirection` を導入した以上、向き表現は `trim_range` の順序反転ではなく明示的な direction で持つ方が責務が分かれる
- constructor が黙って reordering / clamp を行うと、native parameter semantics を破壊しやすい
- 既存 [model/geo_topology/src/topology_core.rs](../../model/geo_topology/src/topology_core.rs#L152) でも `parameter_range.0 >= parameter_range.1` は reject しており、trim 区間の正当性を明示的に扱う流れと整合する

### 初手で追加しないもの

- `parameter_at_length`
- split 済み別曲線への変換 API
- topology binding を前提にした constraint endpoint 語彙
- topology 走査方向との relation capability
- `orientation_relation` を返す contracts trait
- `Direction2D` / `Direction3D` への安易な置換

理由:

- `parameter_at_length` は full curve 側の capability をそのまま移す前に、trim 区間限定 semantics を先に固定すべきである
- split 生成は補助変換であり、finite geometry の正本 API とは切り分けたい
- constraint endpoint は geometry ではなく topology の責務である
- topology との relation は Entity/application 側の再整合責務と絡むため、geometry 単体 trait へ早期固定しない
- 既存 contracts 方針でも relation 系 API は core ではなく operations 側で個別に扱うため、初手の core trait 群へ混在させない
- `Direction2D` / `Direction3D` は正規化ベクトル型であり、Forward / Reversed 二値の表現には責務が合わない
- `BasisCurveRef` / `BasisCurveStorage` / `ConstructorError` などの追加 associated type は、現段階では抽象化コストが利得を上回る

### constructor 方針

- 初手の trimmed curve でも `TrimmedNurbsCurve3DConstructor` を definition trait として追加する
- 基本 constructor は `from_basis_curve(basis_curve, trim_range)` とし、`basis_curve` は値で受ける
- `geometry_direction` は主 constructor 引数へ混ぜず、`with_geometry_direction()` の builder 的補助で与える第一候補とする
- constructor は `trim_range` の妥当性を検証するが、順序入れ替えや domain clamp のような暗黙正規化は行わない

理由:

- 既存の `NurbsCurve3DConstructor` と同じく、shape 生成は definition trait として contracts に置く方が family の説明軸が揃う
- `geometry_direction` は trim 区間と違って shape 定義の最小成立条件ではなく、既定値を `Forward` にできる補助パラメータである
- 引数を最小限に保ち、追加オプションは `with_*` へ逃がす方が constructor の責務を絞りやすい

共有所有を採る最適化が後で必要になった場合は、次のいずれかを別タスクで再検討する。

- 実装型内部での shared storage 化
- shared 用の inherent constructor 追加
- contracts の constructor trait とは別の factory / adapter 導入

## 既存 trait との関係

- `TrimmedNurbsCurve3DConstructor` は trimmed finite geometry の definition trait として追加する
- `TrimmedNurbsCurve3DBasis` が `BasisCurve` associated type の唯一の定義箇所になる
- `geo_nurbs` 実装では `TrimmedNurbsCurve3DBasis::BasisCurve = NurbsCurve3D<T>` を採る
- `NurbsCurve3DProperties` は母曲線本体の定義パラメータを返す
- `TrimmedNurbsCurve3DProperties` は finite view としての trim 状態を返す
- full curve と trimmed curve は同名 trait の流用ではなく、別 capability 群として分離する
- ただし associated type `BasisCurve: NurbsCurve3DCore<T>` により、母曲線 capability との接続は明示する
- `orientation_relation` は trimmed curve 単体の capability ではなく、必要なら将来 operations/relation 側で別途扱う

## 実装開始時の最小スコープ案

最初の実装は次に限定する。

1. `geo_contracts` に trimmed NURBS curve 用 trait 群を追加する
2. `geo_nurbs` に仮称 `TrimmedNurbsCurve3D<T>` を追加する
3. `from_basis_curve` と `with_geometry_direction` を実装する
4. `start_point` / `end_point` / `evaluate` / `arc_length` を実装する
5. `evaluate_local(0..=1)` を convenience として実装する
6. `geo_algorithms` で trimmed NURBS curve を geometry 入力として受ける拡張点を整理する

この段階では、`geo_topology` 側の `CurveRef` 拡張はまだ着手しない。

## 責務境界

### `application` / Entity orchestration

- Entity を永続状態の正本として保持する
- geometry / topology を生成して返す
- geometry 編集と topology 編集の最終整合を担保する
- geometry 向きと topology 向きの対応づけを管理する

### `geo_nurbs`

- `NurbsCurve3D` 本体の保持
- trimmed NURBS curve の geometry 表現
- finite geometry としての向き情報
- trim 区間端の評価
- 長さ・評価・境界など有限曲線としての振る舞い

### `geo_topology`

- vertex binding
- Edge / Loop / Face / Shell 等の位相向き
- geometry 向きとの relation 値
- 接続・整合・走査の位相語彙

### `geo_algorithms`

- geometry 入力の交差計算・距離計算・離散化
- trimmed NURBS curve を geometry として受ける入口の追加
- topology を入力必須にしない処理系と、Entity/application 由来メタデータを使う処理系を分けて整理する

## #650 への影響

- #650 は `geo_topology` に NURBS edge を最小導入する Issue だが、その前提として trimmed NURBS curve の geometry 表現を先に整理する
- したがって #650 は本設計の整理完了後に再開する

## 再開条件

以下が定まったら #650 を再開してよい。

- trimmed NURBS curve の geometry 型の有無
- `NurbsCurve3D` と trimmed curve の責務分離
- parameter semantics の方針
- `geo_algorithms` 側で trimmed NURBS をどう受けるかの入口方針

## 設計フェーズ完了条件

本 Design issue は、次の論点が文章として固定できた時点で完了とする。

### 1. 型責務の確定

- `NurbsCurve3D` を母曲線本体として維持するか
- `TrimmedNurbsCurve3D` を finite geometry として導入するか
- 両者の責務境界をどう分けるか

### 2. 向き概念の確定

- geometry 向きの名称と意味
- topology 向きの名称と意味
- geometry 向きと topology 向きの relation 値の名称と意味
- 既存 `same_sense` を暫定互換名として維持するか、将来的に再命名対象とするか

### 3. Entity 正本モデルの確定

- Entity を永続状態の唯一の正本とするか
- geometry / topology を Entity から生成したコピーとして扱うか
- geometry 編集後、topology 編集後の再整合を application 側でどう担保するか

### 4. contracts 追加範囲の確定

- `geo_contracts` に追加する trimmed NURBS curve 用 trait 群
- 初手で追加しない trait / capability
- constructor を definition trait として置く方針
- associated type は `TrimmedNurbsCurve3DBasis::BasisCurve` の 1 個に留める方針
- `Evaluation` は `TrimmedNurbsCurve3DBasis` を継承し、実装側では `BasisCurve = NurbsCurve3D<T>` へ固定する方針
- `orientation_relation` を core に入れず、必要時のみ operations 側候補として扱う方針
- `geometry_direction` の値型は `CurveDirection` ではなく `ParametricDirection` を第一候補とする方針

### 5. 実装開始スコープの確定

- 最初の実装対象クレート
- 最初に追加する型・trait・最小 API
- 今回の設計段階で触らないクレートと機能

### 6. #650 再開条件の確定

- #650 を再開するために必要な前提の一覧
- #650 の最小実装スコープ
- #666 完了後に #650 へ戻す順序

### 7. 波及範囲の確定

- `geo_nurbs` への追加点
- `geo_topology` への影響範囲
- `geo_algorithms` の入口変更有無
- `application` / Entity orchestration で必要な対応

## 残課題の決定（2026-04-12）

#666 の設計残件は次の内容で固定し、本 Issue の未決論点を解消したものとして扱う。

### 1. ownership 戦略（固定）

- 初手の `TrimmedNurbsCurve3D` は `basis_curve` を値所有で固定する
- shared ownership（例: `Arc`）は contracts の前提にしない
- shared 化が必要な場合は、後続 Issue で実装型内部最適化または別 constructor 方針として扱う

### 2. associated type の固定範囲（固定）

- associated type は `TrimmedNurbsCurve3DBasis::BasisCurve` の 1 個に固定する
- `BasisCurveRef` / `BasisCurveStorage` / `ConstructorError` 等の追加 associated type は導入しない
- `Evaluation` は `TrimmedNurbsCurve3DBasis` を継承し、`geo_nurbs` 実装側で `BasisCurve = NurbsCurve3D<T>` とする

### 3. local parameter API の境界（固定）

- 主 API は native parameter を受ける `evaluate()` とする
- `evaluate_local()` は `0..=1` 線形写像 convenience に限定する
- `evaluate_local()` は shape の正本 domain を定義しない
- `0..=1` の範囲外入力は暗黙 clamp しない
- periodic/seam を跨ぐ trim の正規形は #666 の対象外とし、face/surface 側の設計 Issue（#668）で扱う

### 4. constructor の最終配置（固定）

- 主 constructor は `from_basis_curve(basis_curve, trim_range)` とする
- `geometry_direction` は `with_geometry_direction()` による builder 的補助で指定する
- direction を主 constructor 必須引数へ昇格させない
- constructor は `trim_range` 妥当性を検証し、reordering / clamp の暗黙正規化は行わない

### 5. `geo_algorithms` 入口粒度（固定）

- #666 では「trimmed NURBS curve を geometry 入力として受けられる入口を定義する」方針までを固定する
- 初手対象は geometry 単独で完結する入口（交差・距離・離散化）とし、topology 入力必須化は行わない
- Entity/application 由来メタデータを要求する処理系は別入口として分離する方針を維持する
- face/surface/PCurve 文脈の入口設計は #668 側で扱う

### 6. #650 再開時の最小実装順（固定）

- #666 完了後の推奨順は `geo_contracts` → `geo_nurbs` → `geo_topology` → `geo_algorithms` とする
- #650 は `geo_topology` の NURBS curve edge 最小対応（`CurveRef` / `CurveSegment3D` / endpoint/evaluation / parameter semantics）に限定する
- derive port の拡張は #650 で最小限に留め、face/loop/coedge/halfedge と surface trim は除外する
- #650 の完了条件は「curve edge まで」で固定し、trimmed face/surface は #668 へ委譲する

## 完了時の成果物

設計フェーズ完了時には、少なくとも次を満たす。

- 本文書に責務境界、向き概念、Entity 正本モデルが記載されている
- #666 issue 本文が最新設計と整合している
- #650 の blocker と再開条件が文書と issue の両方で一致している
- 実装開始時に最初に触るファイル群またはクレート群が列挙されている

## 非目標

- trimmed NURBS face の導入
- `Face` / `Loop` / `CoEdge` / `HalfEdge` の導入
- surface 上の trim curve / p-curve の仕様固定
- split 結果の別 NURBS を正本にする設計への移行

## 関連

- #666 Design issue
- #650 blocker
- #668 follow-up design issue（trimmed face/surface 側の整合設計）
- `dev/architecture/VIEWMODEL_MODEL_ROUTE_REDEFINITION_DESIGN.md`
- `dev/architecture/TOPOLOGY_ENTITY_LAYER_DESIGN.md`
- `dev/architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md`