# Analysis Newton Solver Genericization Design

**作成日**: 2026年4月7日  
**対象Issue**: #603  
**関連**: #596, #600, #619

## 1. 背景

`foundation/analysis` には `newton_solve` / `newton_solve_bounded` / `newton_solve_with_numeric_derivative_bounded` / `newton_solve_2d` が存在するが、現状は `f64` 固定 API である。

一方、`geo_nurbs` およびその周辺では `T: Scalar` ベースの評価 API が整っており、最近接 parameter 探索や projection 系ロジックを `analysis` の Newton solver へ直接接続しづらい。

現時点で確認できる接続点は次の通り。

- `analysis` 側 solver 本体: `foundation/analysis/src/linalg/solver/newton.rs`
- `analysis` 側公開面: `foundation/analysis/src/lib.rs`, `foundation/analysis/src/linalg/solver/mod.rs`
- production 利用箇所: `model/geo_algorithms/src/collision/nurbs_3d.rs`
- `geo_nurbs` 側の Newton 関連定数: `model/geo_nurbs/src/lib.rs`

## 2. 現状制約

### 2.1 API が `f64` 固定

現在の Newton solver 群は、入力、境界、許容誤差、数値微分ステップのすべてが `f64` 固定である。

このため、`T: Scalar` ベースの実装は次のいずれかを強いられる。

- solver 呼び出し前後で `T <-> f64` 変換を都度行う
- `geo_nurbs` / `geo_algorithms` 側に局所的な Newton 実装を持つ
- `analysis` を使わずに別経路の最適化を実装する

### 2.2 downstream 互換性をすぐには捨てにくい

`analysis` は独立クレートであり、既存の doctest / test / export 面も `f64` 前提で書かれている。

そのため、いきなり既存 API を generic 版へ置換すると、設計論点と互換破壊を同時に処理する必要がある。

### 2.3 1変数 solver と 2変数 solver の論点が異なる

1変数 Newton solver 群は scalar root solve として比較的素直に generic 化できるが、`newton_solve_2d` は次の論点を別途持つ。

- Jacobian の表現方法
- residual / step norm の評価方法
- 2変数に限定した API のままにするか
- 将来的な多変数一般化の可否

したがって、`newton_solve_2d` を第1段から同時に扱うと設計論点が肥大化する。

## 3. 比較案

### 案1: analysis の Newton solver を一気に generic 化して既存 API を置き換える

概要:

- `newton_solve` 系を直接 `T: Scalar` ベースへ置換する
- `f64` 固定 API は原則廃止する

利点:

- 最終形としては最も単純
- `analysis` の数値基盤を早く一本化できる

欠点:

- 既存 export 面と doctest への影響が大きい
- 1変数 solver と 2変数 solver の論点が混ざる
- Design issue の次段としては変更量が重い

評価:

- 長期形としては妥当だが、第1段の移行案としては重い

### 案2: analysis に generic core を追加し、既存 `f64` API は互換ラッパとして残す

概要:

- 内部に generic Newton core を新設する
- 既存の `newton_solve` 系は `f64` 専用の薄い互換ラッパとして残す
- downstream は段階的に generic API へ移行する

利点:

- `analysis` を共通数値基盤として育てられる
- 既存利用者を直ちに壊さない
- `geo_nurbs` / `geo_algorithms` の需要に段階的に接続できる
- 参照が少ない箇所から並行移行しやすい

欠点:

- 一時的に API が二層になる
- ラッパと core の責務境界を文書化する必要がある

評価:

- 互換性、実装コスト、将来性のバランスが最も良い

### 案3: analysis は `f64` 固定のまま据え置き、`geo_nurbs` 側に generic Newton を局所実装する

概要:

- `analysis` は変更せず、必要な closest-parameter / projection ロジックだけ `geo_nurbs` または `geo_algorithms` に局所実装する

利点:

- 目先の実装速度は速い
- `analysis` の公開 API を触らずに済む

欠点:

- #603 の目的に反する
- 数値反復の責務が各層に重複する
- 将来の closest point / inverse parameter solve の横展開がしにくい

評価:

- 短期回避策としてはありうるが、採用案にはしない

## 4. 推奨案

本 Issue では **案2** を採用する。

設計方針:

- `analysis` に generic な 1変数 Newton core を追加する
- 既存の `f64` API は互換ラッパとして残す
- 第1段では 1変数 solver 群だけを対象とする
- `newton_solve_2d` の generic 化は follow-up として分離する

この方針を採る理由は次の通り。

- `geo_nurbs` / `geo_algorithms` の `T: Scalar` ベース需要に直接応えられる
- 既存の `analysis` 利用面を即時破壊しない
- 参照が少ない API から並行移行できる
- 2変数 Newton の別論点を第1段へ混入させずに済む

## 5. 第1段の対象範囲

第1段で generic core 化の対象とするのは、次の 1変数 solver 群である。

- `newton_solve`
- `newton_solve_bounded`
- `newton_solve_with_numeric_derivative_bounded`
- `newton_inverse`

ただし、`newton_inverse` は `newton_solve` の薄い派生 API として残し、まずは root solve core の generic 化を優先する。

第1段の非対象:

- `newton_solve_2d`
- 多変数一般 Newton solver
- Jacobian 抽象化
- 高次元 residual / norm abstraction

## 6. 並行移行の優先順位

参照数と廃止コストの観点では、次の順で並行移行するのが妥当である。

### 優先1: `newton_solve_with_numeric_derivative_bounded`

理由:

- production 参照が実質 1 箇所だけである
- 利用箇所が `model/geo_algorithms/src/collision/nurbs_3d.rs` の NURBS 最近接 parameter 精密化であり、#603 の背景に最も近い
- generic core へ移行した際の波及が小さい

扱い:

- 最初の並行移行対象とする
- generic 版を追加後、`geo_algorithms` 側の呼び出しを優先的に切り替える

### 優先2: `newton_solve_bounded`

理由:

- 現状では外部直接利用が薄く、実質的に core API に近い
- bounded root solve は parameter domain を持つ shape evaluation と相性がよい

扱い:

- generic core の基盤 API として先に整える

### 優先3: `newton_solve` / `newton_inverse`

理由:

- test / doc での比率が高く、production 波及は小さい
- generic core を導入した後に、互換ラッパとして残しやすい

扱い:

- outward-facing API は維持しつつ内部実装を generic core に寄せる

### 後回し: `newton_solve_2d`

理由:

- 参照自体は少ないが、設計論点が 1変数 solver と別物である
- 今回は「参照が少ないから先に移す」のではなく、「別論点なので混ぜない」を優先する

扱い:

- 第1段からは外し、follow-up Issue 候補とする

## 7. 段階的移行方針

### Phase 1: generic core 導入

- `analysis` に generic 1変数 Newton core を追加する
- 既存 `f64` API はその core を呼ぶラッパにする
- `newton_solve_with_numeric_derivative_bounded` を generic core へ接続する

### Phase 2: 参照の薄い利用箇所から切り替え

- `model/geo_algorithms/src/collision/nurbs_3d.rs` の最近接 parameter 精密化を generic 版へ移行する
- `geo_nurbs` 側の最近接 parameter / projection 需要に対し、局所実装を増やす前に analysis 側 core を利用する経路を優先する

### Phase 3: `f64` ラッパの整理

- 既存 `f64` API の利用状況を監査する
- downstream 参照が十分減った段階で、deprecated 化の要否を判断する
- 即時廃止は行わず、互換負債と保守コストの均衡で決める

### Phase 3 監査結果（2026年4月7日時点）

- ワークスペース内の production 呼び出しは generic API へ移行済み
- 旧 `f64` ラッパーの実利用は `foundation/analysis` 内の互換テストに限定される
- doctest / モジュール例は generic API 優先へ切り替え済み
- したがって現時点の `f64` ラッパーは「downstream 即時互換維持」のために残すが、workspace 内の正本利用経路ではない
- 次段では deprecated 化そのものより、外部利用者向けの移行告知と `newton_solve_2d` の follow-up 分離判断を優先する

### 互換ラッパー運用方針（2026年4月8日時点）

- `newton_solve` / `newton_solve_bounded` / `newton_solve_with_numeric_derivative_bounded` / `newton_inverse` は、generic API 導入後も downstream 互換維持のため公開を継続する
- ただし、workspace 内ではこれらを正本 API と見なさず、新規の production 呼び出しを増やさない
- `foundation/analysis` 内では互換性を検証する最小限の wrapper テストだけを維持する
- doctest、モジュール使用例、設計文書、今後の実装は generic API を基準に記述する
- deprecated 化は即時には行わず、外部利用状況、保守コスト、移行告知手段が揃った段階で別判断とする
- したがって当面の整理方針は「公開は維持」「内部の新規利用は抑止」「説明責務は generic API 側へ寄せる」の3点で固定する

### `newton_solve_2d` の扱い判断（2026年4月8日）

- workspace 内監査では `newton_solve_2d` の利用は doctest と `foundation/analysis` 内テストに限定され、production 呼び出しは確認されなかった
- 一方で 2変数 solver は Jacobian 表現、step norm、将来的な多変数一般化など 1変数 generic core とは別の設計論点を持つ
- したがって `newton_solve_2d` は #603 の継続実装へ混在させず、follow-up Issue として切り出す判断を採る
- follow-up では「2変数専用 API を generic 化する」のか「多変数一般 solver へ拡張する」のかを先に設計確定する

### #619 に向けた設計判断（2026年4月8日）

- `foundation/analysis` にはすでに `Matrix2x2<T>` と `Vector2<T>` があり、`newton_solve_2d` を `T: Scalar` 化するための基礎材料は揃っている
- そのため、「必要になったら対応する」という扱いは、analysis クレートの機能網羅の観点では弱い
- 特に 1変数 solver 群だけが generic 化され、2変数 solver だけが `f64` 固定で残る状態は、数値解析基盤としての API 一貫性を損ないやすい
- よって #619 では `newton_solve_2d` の generic 化を単なる将来候補ではなく、計画された follow-up として扱う

### #619 の比較案

#### 案1: 2変数専用 API を維持したまま generic 化する

概要:

- `newton_solve_2d` の責務を 2変数連立 solver に限定したまま `T: Scalar` 化する
- 戻り値や system 関数の構造は大きく変えず、`(T, T)` と `[[T; 2]; 2]` を基本に保つ

利点:

- 既存の `Matrix2x2<T>` / `Vector2<T>` 基盤にそのまま乗せやすい
- 設計論点を 2変数 solver に閉じ込められる
- API 変更範囲が最小で、#619 の設計スコープを保ちやすい

欠点:

- 多変数一般 solver への道筋は別途設計が必要
- 将来の一般化で API 再編が発生する可能性がある

評価:

- #619 の第一段として最も妥当

#### 案2: 2変数専用 API は維持しつつ、内部を generic core 化する

概要:

- outward-facing には `newton_solve_2d` を残しつつ、内部だけ generic な 2変数 core を導入する
- 必要なら `f64` 互換ラッパーと generic 本体の二層構造を採る

利点:

- 将来の API 移行を段階的に行いやすい
- 1変数 solver 群の移行パターンとある程度揃えられる

欠点:

- 現状の利用量に対して構造がやや重い
- 2変数 solver 単体のために二層化すると、設計の説明コストが先に立つ

評価:

- 実装は可能だが、#619 の第一歩としては過剰になりやすい

#### 案3: 多変数一般 Newton solver を新規追加し、2変数 API はその特殊化として扱う

概要:

- `newton_solve_2d` を直接 generic 化するのではなく、ベクトル residual と Jacobian を扱う多変数 solver を別 API として追加する
- 将来的に `newton_solve_2d` はその convenience wrapper に寄せる

利点:

- analysis クレートの数値解析基盤としては最も理想形に近い
- 2変数だけでなく多変数への拡張余地を最初から確保できる
- 機能網羅の観点では最も筋が良い

欠点:

- Jacobian 抽象化、線形ソルバー接続、residual / step norm など論点が一気に増える
- #619 の設計スコープを超えて、多変数 Newton 基盤設計へ膨らみやすい

評価:

- 長期の理想形だが、#619 の初手としては重い

### #619 の推奨方針

- まずは案1を採用し、`newton_solve_2d` を 2変数専用 API のまま generic 化する
- ただし案3は「必要になったら検討する」ではなく、analysis の機能網羅を揃えるための次段計画として明示的に保持する
- つまり #619 では「案1を実装方針とし、案3を将来理想形として設計上予約する」という二段構えを採る

この方針を採る理由は次の通り。

- 既存の generic 行列・ベクトル基盤を活用でき、短期の設計負荷を抑えられる
- 2変数 solver だけが `f64` 固定で残る不整合を解消できる
- 一方で多変数一般 solver まで一気に広げると、#619 の設計 issue としては重すぎる
- 先に案1で機能網羅の穴を塞ぎ、その後に案3を別 API として追加する方が、段階設計として整合的である

### #619 の段階方針

#### Phase A: 2変数専用 generic solver の設計

- `newton_solve_2d` を `T: Scalar` 化する API 形状を確定する
- Jacobian と residual の型表現を、既存の `Matrix2x2<T>` / `Vector2<T>` と整合させる
- 収束判定と特異行列判定の trait 要件を整理する

#### Phase B: 2変数専用 generic solver の実装

- `f64` 固定実装を generic 実装へ置き換えるか、必要最小限の互換ラッパを残す
- doctest / test を generic API 中心へ切り替える

#### Phase C: 多変数一般 solver の設計着手

- `newton_solve_2d` を超えた多変数 Newton API の入口を別 issue で定義する
- `newton_solve_2d` は将来的にその convenience wrapper として再位置付けできるようにする

### #619 に対する提案

提案:

- #619 では案1を正式採用し、`newton_solve_2d` を 2変数専用 API のまま `T: Scalar` 対応へ拡張する
- その際、戻り値と system 関数の責務は維持しつつ、内部表現は `Matrix2x2<T>` と `Vector2<T>` を利用する方向で揃える
- 併せて、案3を別 issue の既定路線として予約し、多変数一般 Newton solver の入口を後続設計へ接続する

この提案を採る理由:

- analysis クレート内に generic 2D 行列・ベクトル基盤がすでにあり、機能網羅の穴を今の段階で埋める条件が揃っている
- 1変数 solver 群だけ generic、2変数 solver だけ `f64` 固定という不整合を解消できる
- 一方で案3を同時実施すると、#619 は 2変数 solver generic 化ではなく多変数 Newton 基盤再設計へ膨張する
- したがって「まず穴を塞ぐ」「次に理想形を別 API で追加する」という順序が、設計負荷と機能網羅の両面で最も妥当である

非採用とする判断:

- 案2は、現状の利用量に対して二層化の説明コストが先に立つため、第一段の提案としては採らない
- 案3は長期理想形として維持するが、#619 の主目的を超えるため同時採用はしない

次アクション:

- #619 の設計承認後、専用設計文書を分離するか、あるいは本書に Phase A の API 詳細を追記する
- 実装前に、`newton_solve_2d` の generic API 形状、収束判定、特異行列判定、doctest/test 移行方針を確定する

### #619 詳細設計

#### 公開 API 方針

- `newton_solve_2d` は関数名を維持したまま generic 化する
- `f64` 専用の別名ラッパーは初手では追加しない
- 既存の `f64` 呼び出しは型推論によりそのままコンパイルできる形を維持する

想定シグネチャ:

```rust
pub fn newton_solve_2d<T, F>(
	system: F,
	initial: (T, T),
	max_iter: usize,
	tol: T,
) -> Option<(T, T)>
where
	T: Scalar,
	F: Fn(T, T) -> (T, T, [[T; 2]; 2]),
```

この形を採る理由:

- 既存の doctest / test / 将来の downstream 呼び出しとの互換性が最も高い
- Issue #619 の対象を「2変数 solver の generic 化」に限定できる
- 内部で `Matrix2x2<T>` / `Vector2<T>` を使っても、公開 API まで同時に再設計する必要がない

#### 内部表現方針

- 公開 API では `(T, T)` と `[[T; 2]; 2]` を受け取る
- 関数内部では residual を `Vector2<T>`、Jacobian を `Matrix2x2<T>` へ即時変換する
- 更新量 `delta` も `Vector2<T>` として扱い、最終戻り値のみ `(T, T)` に戻す

内部変換イメージ:

```rust
let (f1, f2, jacobian_raw) = system(x, y);
let residual = Vector2::new(f1, f2);
let jacobian = Matrix2x2::from(jacobian_raw);
```

この形を採る理由:

- 既存の analysis 線形代数基盤と整合する
- residual norm / step norm の表現を `Vector2<T>` に集約できる
- 将来の多変数 solver 設計時に、ベクトル・行列表現へ段階的に寄せやすい

#### 線形更新の設計

- 2x2 線形更新は private helper に切り出す
- helper は `Matrix2x2<T>` と `Vector2<T>` を受け取り、`Option<Vector2<T>>` を返す
- 特異判定は `det.abs() < derivative_zero_threshold::<T>()` を用いる
- `Matrix2x2::inverse()` の `is_zero()` 判定には寄せず、Newton solver 専用の閾値判定を維持する

想定 helper:

```rust
fn solve_linear_2x2<T: Scalar>(
	jacobian: Matrix2x2<T>,
	residual: Vector2<T>,
) -> Option<Vector2<T>>
```

この helper で行うこと:

- 行列式 `det` を計算する
- `det` がしきい値未満なら `None` を返す
- クラメル展開または 2x2 の陽な逆行列式を使って更新量 `delta` を返す

#### 収束判定方針

- 既存実装と同じく residual norm と step norm の両方を使う
- 判定式は `residual.norm() < tol && delta.norm() < tol` を基本とする
- `tol` は `T: Scalar` の値として受け取り、`f32` / `f64` で同一 API を共有する

この方針を維持する理由:

- 現行 `f64` 実装の意味論を維持できる
- residual だけ、または step だけで判定するよりも既存挙動からの乖離が少ない
- #619 では generic 化を主目的とし、収束判定ロジックの再設計は行わない

#### 特異判定と閾値方針

- 1変数 generic solver と同様に `DERIVATIVE_ZERO_THRESHOLD` を `T::from_f64` で変換して利用する
- helper 名は既存との整合を優先し、`derivative_zero_threshold::<T>()` を再利用する
- 2変数 solver 専用の別閾値はこの段階では導入しない

この方針を採る理由:

- 既存 Newton solver 群との数値基準を揃えられる
- #619 で新しい定数設計論点を増やさずに済む

#### doctest / test 移行方針

- 既存の `f64` doctest はそのまま維持できる形を優先する
- `foundation/analysis` の既存 2D テストは継続利用する
- 追加テストとして `f32` の収束ケースを導入する
- 追加テストとして `f32` の特異 Jacobian ケースを導入する

最低限追加するテスト:

- `test_newton_solve_2d_circle_line_f32`
- `test_newton_solve_2d_singular_jacobian_f32`

#### 互換方針

- 既存の `newton_solve_2d` 呼び出しは、関数名・引数順・戻り値タプルを維持するため source-compatible とみなす
- doctest と既存 unit test は型注釈を `f64` のまま維持しても問題ない設計にする
- 今回は deprecated API を新設しない

#### 今回やらないこと

- 境界付き 2変数 Newton solver の追加
- 数値微分ベースの 2変数 Newton solver の追加
- Jacobian を trait で抽象化すること
- `Vector2<T>` / `Matrix2x2<T>` をそのまま公開 API に露出する再設計
- 多変数一般 Newton solver の同時実装

#### 実装順序案

1. `newton_solve_2d` のシグネチャを `T: Scalar` 化する
2. residual / Jacobian を `Vector2<T>` / `Matrix2x2<T>` へ変換する
3. 2x2 線形更新 helper を導入する
4. 収束判定を `Vector2<T>::norm()` ベースへ置き換える
5. 既存 `f64` doctest / test を通す
6. `f32` テストを追加する

#### Phase C への接続条件

- `newton_solve_2d` の generic 化が完了し、2変数 solver だけが `f64` 固定で残る不整合が解消されていること
- その上で、多変数一般 solver は `newton_solve_2d` の置換ではなく、別入口 API として設計開始する
- 将来的に `newton_solve_2d` を convenience wrapper に寄せる場合も、#619 の完了条件には含めない

### #624 の比較案

#### 案1: `Vector<T>` + `Vec<Vec<T>>` で最小構成の多変数 Newton solver を設計する

概要:

- residual は既存の動的ベクトル `Vector<T>` を使う
- Jacobian は既存の線形 solver 群と接続しやすい `Vec<Vec<T>>` で受ける
- 線形更新は `GaussianSolver<T>` または `LUSolver<T>` を使って求める

利点:

- 既存基盤だけで設計を完結しやすい
- `newton_solve_2d` より先に、多変数 solver の入口を比較的小さく定義できる
- 線形 solver 群との役割分担が明確で、責務重複を避けやすい

欠点:

- Jacobian だけ `Vec<Vec<T>>` で、residual は `Vector<T>` という表現の非対称性が残る
- 動的行列の型安全な抽象がないため、次元整合性チェックを API 側で丁寧に扱う必要がある
- 長期的には理想形というより移行しやすい中間形になりやすい

評価:

- #624 の初手として最も現実的

#### 案2: 多変数 Newton の内部 core だけ定義し、公開 API は薄く始める

概要:

- outward-facing には最小限の関数入口だけを置き、内部に residual / Jacobian / linear solve / convergence 判定を担う core を分離する
- Jacobian 表現は当面 `Vec<Vec<T>>` としつつ、内部 core の責務分離を優先する

利点:

- 将来の境界付き版や数値微分版へ拡張しやすい
- `newton_solve_2d` を後で convenience wrapper に寄せる導線を作りやすい
- 公開 API の再設計コストを一度で抱え込まずに済む

欠点:

- #624 の設計文書としては、公開 API と内部 core の二層説明が必要になりやや重い
- 現時点で内部 core の分割粒度を決めるには材料がまだ少ない

評価:

- 案1よりは重いが、中期の拡張性は高い

#### 案3: 動的行列 abstraction まで先に整備してから多変数 Newton solver を設計する

概要:

- `Vector<T>` に加えて、動的 Jacobian を安全に表現する行列 abstraction を先に導入する
- その上で residual / Jacobian / step を同じ抽象で扱う多変数 Newton solver を設計する

利点:

- 数値解析基盤としては最も整った形に近い
- 次元情報を持ちつつ内部を 1 次元配列で保持する設計を採りやすく、メモリ効率と走査効率の両面で有利
- residual と Jacobian の表現が揃い、長期の保守性・拡張性が高い
- 将来の多変数最適化・反復法にも流用しやすい

欠点:

- #624 の設計スコープを Newton solver から行列基盤設計へ拡大してしまう
- 先に解くべき論点が増え、短期で合意しづらい
- `analysis` の既存 solver 群との接続も含めて検討範囲が大きくなる

評価:

- 初手の設計量は増えるが、数値解析基盤としての整合を優先するなら最も筋が良い

### #624 の現時点の推奨案

- 初手は案3を推奨する
- つまり、多変数 Newton solver の前段として、次元情報を持ち内部を 1 次元配列で保持する `DynamicMatrix<T>` 相当の動的行列 abstraction を先に設計する
- その上で、その abstraction を前提に multivariate Newton solver の公開 API と内部責務を定義する
- 既存の `newton_solve_2d` は即時置換せず、将来的に convenience wrapper として接続可能な位置付けに保つ

この判断を採る理由:

- `Vector<T>` は既にある一方、Jacobian だけが `Vec<Vec<T>>` に留まる状態は基盤として非対称であり、多変数 solver の入口として弱い
- 次元情報を持つ動的行列 abstraction を先に整えることで、residual / Jacobian / step の表現を同じ設計思想で揃えられる
- 内部を 1 次元配列で持つ行列は、メモリ効率、連続配置、将来の数値計算最適化の観点で `Vec<Vec<T>>` より有利である
- 既存の一部 solver 実装が `Vec<Vec<T>>` ベースであることは現状では成立しているが、多変数 Newton 基盤へ拡張する上では先に返すべき設計負債と位置付ける方が妥当である

### #624 の段階方針

#### Phase A: 動的行列 abstraction の設計

- `DynamicMatrix<T>` 相当の型を定義し、行数・列数・内部 1 次元配列の保持方針を決める
- row-major を基本にするか、列優先を採るかを決める
- 最小 API として `new`、`rows`、`cols`、`get`、`set`、行列式ではなく一般アクセスと shape 検証を優先する
- `Vector<T>` との整合を取り、residual / step と Jacobian を同じ abstraction family へ乗せる

#### Phase A 実装スコープ（2026年4月8日着手）

- 初手の実装は `DynamicMatrix<T>` 単体に限定し、既存 `LinearSolver<T>` trait と `GaussianSolver<T>` / `LUSolver<T>` / `CramerSolver<T>` の受け取り型は変更しない
- 内部表現は row-major の `Vec<T>` とし、shape は `rows` と `cols` を明示保持する
- 初回に入れる API は `new`、`zeros`、`from_rows`、`rows`、`cols`、`shape`、`get`、`set`、`as_slice`、`to_vec2d`、`transpose`、`mul_vector` を基本とする
- `Vec<Vec<T>>` からの完全移行は Phase C で扱い、Phase A では変換補助と `Vector<T>` 連携までに留める
- したがって、この段階の目的は「多変数 Newton 用 Jacobian の基盤型を先に正規化すること」であり、「既存線形 solver の全面置換」ではない

#### Phase B: multivariate Newton solver API の設計

- residual を `Vector<T>`、Jacobian を `DynamicMatrix<T>` で受ける公開 API を定義する
- 収束判定を residual norm / step norm の両方で行うかを定義する
- 特異判定と失敗時の返り値方針を定義する
- `newton_solve_2d` との差分を明示し、2変数専用 convenience wrapper へ寄せる余地を残す

#### Phase B 実装スコープ（2026年4月8日着手）

- 新規入口として `newton_solve_multivariate` と `newton_solve_multivariate_with_solver` を追加する
- `system` は `&Vector<T>` を受け取り、`(Vector<T>, DynamicMatrix<T>)` を返す形に統一する
- 既定の更新ステップは `GaussianSolver<T>` + `DynamicMatrixLinearSolver<T>` adapter を使って解く
- solver 差し替えが必要な利用者向けに `..._with_solver` を併設し、`LUSolver<T>` などを注入可能にする
- 返り値は既存 Newton 系との整合を優先して `Option<Vector<T>>` とし、shape 不整合・特異 Jacobian・非収束は `None` で表す
- 収束判定は residual norm と step norm の両方を使い、既存 2変数 solver より明示的な多変数収束条件を採る

#### Phase C: 線形 solver 接続方針の設計

- 動的 Jacobian を既存の `GaussianSolver<T>` / `LUSolver<T>` へどう接続するかを整理する
- 既存 solver を `Vec<Vec<T>>` のまま維持するか、`DynamicMatrix<T>` 受け取りへ拡張するかを比較する
- 多変数 Newton の更新ステップ専用 helper を設けるか、既存 solver を直接使うかを決める

#### Phase C 接続方針（2026年4月8日合意）

- 初手では既存 `LinearSolver<T>` trait の `Vec<Vec<T>>` 受け取りを維持し、破壊的変更を避ける
- その代わり、`DynamicMatrix<T>` と `Vector<T>` を既存 solver へ橋渡しする adapter 層を追加し、多変数 Newton 側からは `DynamicMatrix<T>` ベースで呼べる経路を先に整える
- 将来の新規 generic solver は `DynamicMatrix<T>` ベースで追加し、既存 solver 群とは一定期間共存させる
- その後、利用実績と API 安定性を見ながら `Vec<Vec<T>>` ベース solver から段階移行する
- したがって現段階では「既存 solver を変える」のではなく、「DynamicMatrix ベースの新規利用経路を先に正本化する」を優先する

#### Phase D: follow-up 実装計画への分割

- 動的行列 abstraction
- multivariate Newton solver 入口
- 線形 solver 接続整理
- `newton_solve_2d` の convenience wrapper 再位置付け

- 上記を別 phase issue に分割できる状態まで落とし込む

### 境界付き多変数 Newton の型設計（2026年4月8日）

#### 設計対象

- 既存の `newton_solve_multivariate` / `newton_solve_multivariate_with_solver` を基準に、境界拘束付きの多変数 Newton 入口を追加する
- 初手では line search や active-set 法までは導入せず、各反復後にパラメータを境界内へ clamp する単純境界方式を採る
- 既存 1変数 bounded Newton と同様に、「更新候補を計算して境界内へ戻す」思想を多変数へ拡張する

#### 比較案

案1: `lower: Vector<T>` と `upper: Vector<T>` を関数引数へ直接追加する

利点:

- 実装は最小
- 既存 `newton_solve_multivariate` に近い呼び出し形を保てる

欠点:

- 引数が増え、shape 検証責務が API 呼び出しごとに漏れる
- 将来 `step_tol` や damping などの設定を追加すると関数シグネチャが肥大化する

評価:

- 初速は出るが、長期の API 安定性では弱い

案2: 境界だけを専用型へ分離する

利点:

- shape 検証と clamp 責務を境界型へ閉じ込められる
- 引数増加を一定程度抑えられる

欠点:

- 反復設定が将来増えると別の引数肥大が起きる

評価:

- 実用的だが、境界と反復設定の責務分離がまだ不十分

案3: 境界型と options 型を分離する

利点:

- 境界と反復設定の責務を分離できる
- 将来 `residual_tol` / `step_tol` / damping / line search 追加時も破壊的変更を抑えやすい
- builder 併用方針とも整合する

欠点:

- 初手の型数は増える

評価:

- 長期拡張性と整合性の観点で最も妥当

#### 推奨案

- 境界付き多変数 Newton では案3を採用する
- つまり、`MultivariateNewtonBounds<T>` と `MultivariateNewtonOptions<T>` を分離し、bounded API はそれらを受け取る形にする
- solver 差し替えの考え方は既存の unbounded 版に合わせ、既定 solver 版と `..._with_solver` 版を併設する

#### 推奨型

```rust
pub struct MultivariateNewtonBounds<T: Scalar> {
	lower: Vector<T>,
	upper: Vector<T>,
}

pub struct MultivariateNewtonOptions<T: Scalar> {
	pub max_iter: usize,
	pub tol: T,
}
```

補足:

- `MultivariateNewtonBounds<T>` は境界の shape 検証と clamp 責務を持つ
- `MultivariateNewtonOptions<T>` は初手では `max_iter` と `tol` のみを持ち、将来の `residual_tol` / `step_tol` 分離余地を残す

#### 推奨 API 形状

```rust
pub fn newton_solve_multivariate_bounded<T, F>(
	system: F,
	initial: Vector<T>,
	bounds: &MultivariateNewtonBounds<T>,
	options: &MultivariateNewtonOptions<T>,
) -> Option<Vector<T>>
where
	T: Scalar,
	F: Fn(&Vector<T>) -> (Vector<T>, DynamicMatrix<T>);

pub fn newton_solve_multivariate_bounded_with_solver<T, F, S>(
	system: F,
	initial: Vector<T>,
	bounds: &MultivariateNewtonBounds<T>,
	solver: &S,
	options: &MultivariateNewtonOptions<T>,
) -> Option<Vector<T>>
where
	T: Scalar,
	F: Fn(&Vector<T>) -> (Vector<T>, DynamicMatrix<T>),
	S: DynamicMatrixLinearSolver<T>;
```

#### `MultivariateNewtonBounds<T>` の責務

- `new(lower, upper) -> Result<Self, String>`
- `lower.len() == upper.len()` の検証
- 各軸で `lower[i] <= upper[i]` の検証
- `clamp(&self, point: &Vector<T>) -> Result<Vector<T>, String>`
- `contains(&self, point: &Vector<T>) -> bool`

この責務分離により、bounded Newton 本体は「候補点を計算し、境界へ clamp して次反復へ進む」ことだけに集中できる。

#### 反復アルゴリズム方針

- 初期値は開始時に `bounds.clamp(...)` で境界内へ正規化する
- 各反復で residual と Jacobian を評価し、更新ステップを解く
- `next = current - step` を計算したあと、`bounds.clamp(&next)` を適用する
- 収束判定は unbounded 版と同様に residual norm と step norm の両方を使う
- 初手では line search や active-set 法は導入しない

#### unbounded 版との関係

- unbounded 版の `system` 署名はそのまま維持する
- bounded 版は `bounds` と `options` を追加した薄い拡張入口として位置付ける
- 返り値は既存 Newton 群との整合を優先して `Option<Vector<T>>` のままとする
- 失敗理由の型分離は別論点として保持し、この段階では導入しない

#### 実装順序案

1. `MultivariateNewtonBounds<T>` を追加する
2. `MultivariateNewtonOptions<T>` を追加する
3. bounded API 2 本を `newton.rs` に追加する
4. clamp 動作、shape 不整合、境界端収束のテストを追加する
5. 必要なら follow-up で `tol` を `residual_tol` / `step_tol` に分離する

### 外部利用者向け移行方針

- 新規コードでは `newton_solve_generic` / `newton_solve_bounded_generic` / `newton_solve_with_numeric_derivative_bounded_generic` / `newton_inverse_generic` を優先する
- 既存コードが `f64` 固定で十分な場合は旧ラッパー API を当面そのまま利用してよい
- `T: Scalar` ベースの geometry / analysis 接続では、`T <-> f64` 変換を追加せず generic API へ直接寄せる
- 旧ラッパー API は互換維持のため残すが、workspace 内では正本利用経路ではないため、新しい利用箇所を増やさない
- `newton_solve_2d` は今回の generic core 移行対象外であり、2変数 solver の扱いは follow-up Issue 側で整理する

## 8. 設計上の注意

- `geo_nurbs` 側に Newton 本体を再実装して責務を重複させない
- `analysis` generic core は 1変数 root solve に責務を絞る
- 2変数 solver の一般化は別 issue へ切り出せるよう、今回の設計文書では意図的に深追いしない
- `T: Scalar` と `f64` の相互変換を残す場合も、それは互換ラッパ層に閉じ込める

## 9. 結論

#603 の結論は次の通りとする。

- 採用案は「generic core 追加 + 既存 `f64` API 温存」の二層構成
- 第1段の対象は 1変数 Newton solver 群
- 並行移行の初手は `newton_solve_with_numeric_derivative_bounded` と、その唯一の production 利用箇所である `geo_algorithms` の NURBS collision 経路
- `newton_solve_2d` の generic 化は follow-up として分離する

この方針により、短期の NURBS 需要と、中長期の analysis 共通数値基盤化を両立する。