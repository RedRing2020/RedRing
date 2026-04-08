# NURBS basis scalar sequence boundary design

## 目的

Issue #643 では、`geo_nurbs` の basis kernel について、`KnotVector<T>` 所有責務は `geo_nurbs` に残したまま、どこまでを scalar sequence ベースの純粋計算として切り出し可能かを整理する。

本書の目的は次の 3 点である。

- `KnotVector<T>` 自体を `geo_nurbs` の責務として維持する方針を明確化する
- basis kernel のうち、所有や妥当性検証ではなく scalar sequence 読み取りだけで成立する部分を分類する
- 将来 `analysis` 候補へ切り出す場合の最小単位と、`geo_nurbs` に残すべき部分の境界を説明可能にする

## 前提

### 1. `KnotVector<T>` の所有責務は `geo_nurbs` に残す

2026-04-08 時点では、次を `geo_nurbs` 側の責務として扱う。

- NURBS curve / surface が保持する knot vector の所有
- knot vector 生成 API
- knot vector 妥当性検証
- knot vector を含む shape constructor / operation API の公開面

これは `KnotVector<T>` が単なる `Vec<T>` エイリアスであっても同じである。重要なのは型の複雑さではなく、NURBS 形状が knot vector をどのような意味で保持し、どの操作で更新するかという ownership と domain semantics が `geo_nurbs` に属している点である。

したがって、#643 は `KnotVector<T>` の移設 issue ではない。

### 2. basis kernel の検討対象は「所有」ではなく「計算入力の最小単位」

`model/geo_nurbs/src/basis.rs` の basis 関数群は、シグネチャ上は `&KnotVector<T>` を受けているが、実際に使っている操作の大半は次に限られる。

- `len()`
- index access
- 隣接 knot の差分計算
- 非減少列であることを前提にした span 周辺読み取り

つまり basis kernel 自体は、NURBS shape の ownership を必要としておらず、実質的には index 付き scalar sequence を読む純粋計算に近い。

## 現状整理

### `geo_nurbs` に残す責務

次は scalar sequence 抽象に切り出す対象ではなく、`geo_nurbs` に残す。

- `KnotVector<T>` 型 alias とそれを用いた公開 API
- `validate_knot_vector`
- `uniform_knot_vector` / `clamped_knot_vector` / `open_knot_vector`
- curve / surface が保持する knot vector フィールド
- knot insertion / splitting / degree elevation のような knot vector を生成・更新する操作

理由は、これらが単なる sequence 読み取りではなく、NURBS domain semantics と shape ownership を伴うためである。

### scalar sequence ベースで説明できる責務

次は `KnotVector<T>` 所有を前提にしなくても説明できる。

- span 検索
- parameter domain の読み取り
- Cox-de Boor basis evaluation
- basis derivative evaluation
- rational basis の正規化前段

特に `find_knot_span` は既に `analysis::find_span_in_non_decreasing_sequence` を利用しており、非減少 scalar sequence に対する純粋 helper へ一段切り出し済みである。

このため、#643 で扱う主論点は「basis.rs の関数シグネチャと内部ロジックを、どの粒度まで scalar sequence 側へ寄せられるか」である。

## 境界判断

### A. `analysis` 候補

将来的に `analysis` 候補として扱ってよいのは、次の条件を全て満たす処理に限定する。

- 入力が `&[T]` と整数 index / degree だけで表現できる
- NURBS shape 型を参照しない
- knot vector の生成・検証・更新を行わない
- 戻り値が basis 値や span index のような純粋数値結果で閉じる

この条件に照らすと、候補は次のように整理できる。

- 非減少列 span 検索
- slice ベースの basis function 計算
- slice ベースの basis derivative 計算

### B. `geo_nurbs` 残置対象

次は `analysis` 候補にしない。

- `KnotVector<T>` を生成する helper
- multiplicity validation のような NURBS knot rule を伴う処理
- curve / surface evaluate の公開 API
- control point / weight / knot vector をまとめて扱う shape operation
- knot insertion / splitting / degree elevation のように新しい knot vector を返す操作

これらは scalar sequence を読むだけでは閉じず、NURBS shape の整合制約とセットで意味を持つためである。

## basis.rs に対する具体整理

2026-04-08 時点の `basis.rs` では、次の関数が `&KnotVector<T>` を受けている。

- `basis_function`
- `basis_functions`
- `basis_derivatives`
- `rational_basis_functions`
- `rational_basis_derivatives`

設計上の整理結果は次のとおりとする。

### 1. `basis_function` / `basis_functions` / `basis_derivatives`

これらは slice ベース候補である。

必要な入力は次に還元できる。

- `span` または `i`
- `degree`
- parameter value
- non-decreasing knot sequence

したがって、将来の切り出し対象としては `&KnotVector<T>` ではなく `&[T]` を正とする。

### 2. `rational_basis_functions` / `rational_basis_derivatives`

これらも knot 入力自体は slice 化可能だが、weights と control index 対応を伴うため、basis 純核より一段上の helper として扱う。

結論として、まず切り分けるべきは非有理 basis 側であり、有理 basis 側はその後段で扱うのが妥当である。

## curve / surface 実装への含意

`curve_2d.rs`、`curve_3d.rs`、`surface_3d.rs` には、basis 計算ロジックの重複が残っている。

これ自体は #643 の即時実装対象ではないが、設計判断としては次を採用する。

- curve / surface が `KnotVector<T>` を保持することは維持する
- 内部の basis evaluation は、将来 slice ベース helper に委譲できる形へ寄せる
- shape 側メソッドは ownership と orchestration を担い、basis kernel そのものは重複保持しない方向を正とする

つまり、移すべきなのは `KnotVector<T>` ではなく basis evaluation kernel である。

## 段階方針

### Phase 1

設計上の分類を固定する。

- `KnotVector<T>` 所有責務は `geo_nurbs`
- span / basis evaluation 純核は scalar sequence 候補
- rational basis は非有理 basis より一段上の helper として扱う

### Phase 2

非有理 basis 関数を `&[T]` ベースで受けられるように整理する。

候補:

- `basis_function`
- `basis_functions`
- `basis_derivatives`

### Phase 3

curve / surface 実装の重複 basis 計算を、slice ベース helper への委譲へ寄せる。

### Phase 4

必要なら rational basis 側の扱いを再評価する。

## 今回の結論

- `KnotVector<T>` 自体は `geo_nurbs` の責務として維持する
- basis kernel のうち、非減少 scalar sequence 読み取りで閉じる部分は slice ベース候補として分類する
- `analysis` 候補は ownership を持たない純粋数値計算に限定する
- NURBS shape API、knot 生成・検証・更新、shape operation は `geo_nurbs` に残す
- 次の実装候補は、`basis.rs` の非有理 basis 関数群を `&[T]` 前提で再整理することである