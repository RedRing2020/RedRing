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