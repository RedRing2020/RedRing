# geo_core / geo_contracts trait Source of Truth Design

## 背景

Issue #533 では、`geo_core` と `geo_contracts` にまたがって残っている基本型 trait定義の二重管理を整理する。

`#339` の `H-2` 調査時点では、少なくとも以下の状態が確認されていた。

- `Point` / `Vector` の trait定義は `geo_contracts` と `geo_core` の双方に存在する
- `geo_contracts` 側の方が既に公開窓口として整っており、既定メソッドも一部多い
- `geo_core` 側ではローカル trait を `Point2D` / `Point3D` / `Vector2D` / `Vector3D` 実装の足場として保持している
- `AABB` 系 trait は当時 `geo_core` にしか存在せず、`geo_contracts` 側には窓口がなかった

この状態では、基本型 trait定義の source of truth が曖昧であり、個別メソッド整理や利用側の import 整理を進めても根本解決にならない。

## 2026-04-05 時点の更新状況

- `geo_contracts` 側の `Point` / `Vector` trait定義を正本とする方針は維持する
- `geo_core::point_traits` / `geo_core::vector_traits` は duplicate trait定義ではなく、互換再エクスポート層として扱う
- 残課題は、`geo_core` の concrete type 実装が互換再エクスポート経由で trait を参照している点であり、最小整理ではここを `geo_contracts` 直接参照へ切り替える
- `Point2DMeasure` / `Point3DMeasure` / `Vector2DMeasure` / `Vector3DMeasure` 自体の capability 再設計は、2026-04-05 合意により別設計で破壊的変更として進める
- `geo_contracts` には既に `geometry/core/aabb_traits.rs` が追加済みであり、AABB trait定義の正本は contracts 側へ移管済みである
- AABB の残課題は、`geo_core::Aabb2D` / `Aabb3D` の concrete 実装とテストが互換再エクスポート経由で trait を参照している点であり、最小整理ではここを `geo_contracts` 直接参照へ切り替える

## 前提

本設計では、`geo_contracts` の責務を次のように扱う。

- `geo_contracts` は trait定義の正本を置くクレートとする
- `geo_contracts` には実装ロジックを置かない
- 具体型の実装、変換共通核、エラー型などの処理責務は `geo_core` に残す
- `geo_primitives` / `geo_nurbs` は、trait定義だけを知る用途では `geo_core` を知らなくてもよい形を目指す

## 観測事実

### Point / Vector

- `geo_contracts` には `Point2DCore` / `Point3DCore` / `Vector2DCore` / `Vector3DCore` が既に存在する
- `geo_core` 側の `point_traits` / `vector_traits` は互換再エクスポート層として残っている
- `geo_contracts` 側には `position` / `dimension` / `is_zero` / `is_unit` / `area` / `volume` / `length` など、`geo_core` 側にない既定メソッドがある
- `geo_core` の concrete type 実装はなお互換再エクスポート経由の import を一部残している

### AABB

- `geo_core` には concrete type `Aabb2D` / `Aabb3D` と互換再エクスポート層 `aabb_traits` が存在する
- `geo_contracts` には `Aabb2DProperties` / `Aabb2DDerived` / `Aabb3DProperties` / `Aabb3DDerived` が既に存在する
- `Bounded` は `geo_contracts` 側に存在し、associated type `Aabb` を介して concrete type を返せる設計になっている

このため、AABB も trait定義を `geo_contracts`、concrete type を `geo_core` に残す構成へ既に移行しており、残る整理は concrete 実装の参照先統一である。

## 2026-04-05 時点の補足

- source of truth 整理の最小段階は完了し、`geo_core` の concrete type 実装は Point / Vector / AABB とも `geo_contracts` 直接参照へ切り替え済みである
- 次段は source of truth の所在変更ではなく、`geo_contracts` 正本 trait 群そのものの capability 再分類である
- この capability 再分類では `Point2DMeasure` / `Point3DMeasure` / `Vector2DMeasure` / `Vector3DMeasure` を互換維持せず削除する
- 具体的な再分類方針は [GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md](GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md) を正本とする
- `geo_contracts::geometry::foundation` は facade を維持しつつ、内部実体を `metadata.rs` と `bounds.rs` へ分割する
- `ExtensionFoundation` は互換 surface としても維持せず削除する

## 採用方針

### 1. source of truth

- `Vector` trait定義の正本は `geo_contracts`
- `Point` trait定義の正本は `geo_contracts`
- `AABB` trait定義の正本も `geo_contracts`

### 2. `geo_core` の責務

- `Point2D` / `Point3D` / `Vector2D` / `Vector3D` / `Aabb2D` / `Aabb3D` の concrete type 実装を保持する
- 上記 concrete type が `geo_contracts` trait を実装する
- Transform 共通核、変換エラー、行列変換支援は引き続き `geo_core` に置く
- trait定義の正本は持たない

### 3. `geo_primitives` / `geo_nurbs` の見え方

- trait定義だけを参照する用途では `geo_contracts` を import すれば足りる状態を正とする
- concrete type を使う箇所だけ `geo_core` に依存する
- Foundation Pattern 上の「trait定義は contracts、実装は concrete type 側」という役割分担を徹底する

### 4. 移行期間の互換導線

- 移行期間中のみ `geo_core` 側に互換導線を残してよい
- 互換導線は再エクスポートを第一候補とし、trait定義の複製は増やさない
- 最終形では `geo_core` ローカルの Point / Vector / AABB trait定義は削除する

## 移行順序

順序は `Vector -> Point -> AABB` とする。

### Phase 1 影響ファイル: Vector

目的:

- 依存関係の最小ケースで `geo_core -> geo_contracts` 実装モデルを確立する
- `Vector2D` / `Vector3D` の concrete type 実装を `geo_contracts` trait 基準へ切り替える

実施内容:

- `geo_contracts` 側の `vector_traits.rs` を正本として確定する
- `geo_core::vector_traits` の役割を互換再エクスポートへ縮退した上で、concrete 実装からは直接参照しない
- `geo_core::Vector2D` / `Vector3D` が `geo_contracts` trait を実装するよう切り替える
- 利用側 import を `geo_contracts` 基準へ寄せる

完了条件:

- `Vector` trait定義の正本が `geo_contracts` に一本化されている
- `geo_core` にローカル実装専用の duplicate trait が残っていない、または互換再エクスポートに縮退している

### Phase 2 影響ファイル: Point

目的:

- `Point2D` / `Point3D` を `geo_contracts` trait 基準に切り替える
- `lerp` を含む Point 系メソッド責務を source of truth 上で明確にする

実施内容:

- `geo_contracts` 側の `point_traits.rs` を正本として確定する
- `geo_core::point_traits` を互換再エクスポート層として残しつつ、concrete 実装からは直接参照しない
- `geo_core::Point2D` / `Point3D` が `geo_contracts` trait を実装するよう切り替える
- Point に依存する AABB 実装が次段で切り替えられるようにする

完了条件:

- `Point` trait定義の正本が `geo_contracts` に一本化されている
- `geo_core` 側の Point concrete type 実装が `geo_contracts` trait に統一されている

### Phase 3 影響ファイル: AABB

目的:

- AABB についても trait定義と concrete type を分離し、Point / Vector と同じ責務構成に揃える

実施内容:

- `geo_contracts` 側の `aabb_traits.rs` を正本として維持する
- `geo_core::Aabb2D` / `Aabb3D` が `geo_contracts` 側 AABB trait を実装するよう切り替える
- `Bounded::Aabb` は引き続き `geo_core::Aabb2D` / `geo_core::Aabb3D` を返してよい
- `geo_core::aabb_traits` は互換再エクスポート層として残しつつ、concrete 実装からは直接参照しない

完了条件:

- AABB trait定義の正本が `geo_contracts` に存在する
- `Bounded` と concrete AABB の関係が `geo_contracts trait + geo_core concrete type` に整理されている

## 却下案

### 却下案 1: `geo_core` を正本のまま維持

却下理由:

- `geo_contracts` に trait定義を集約する全体方針と衝突する
- `geo_primitives` / `geo_nurbs` が trait定義を見るために `geo_core` を知る必要が残る
- Point / Vector の二重定義問題が解消しない

### 却下案 2: Point / Vector だけ `geo_contracts`、AABB は `geo_core` 正本のまま維持

却下理由:

- 基本型 trait定義の責務分担が再び分裂する
- `Bounded` だけ contracts にあり、AABB trait だけ core に残る構図は説明コストが高い
- 空間ラフチェック用の一般形状として AABB だけ扱いが別になる

### 却下案 3: duplicate trait を長期互換として維持

却下理由:

- source of truth 問題を恒久化する
- 個別メソッド追加や責務整理のたびに同じ論点が再発する

## 依存関係への影響

- `geo_primitives` / `geo_nurbs` は現時点でも `geo_contracts` と `geo_core` の双方に依存しているため、trait参照先を `geo_contracts` に寄せても利用不能にはならない
- `geo_core` は現時点で `analysis` のみに依存しているが、本方針では `geo_contracts` への依存を許可する必要がある
- 循環依存は作らないこと
- `geo_contracts` に実装ロジックを持ち込まないこと

このため、アーキテクチャ上の更新点は「`geo_core -> geo_contracts` を許可する」一点に集約される。

## 影響ファイルの初期候補

### Phase 1: Vector

- `model/geo_core/src/vector_traits.rs`
- `model/geo_core/src/vector_2d.rs`
- `model/geo_core/src/vector_3d.rs`
- `model/geo_contracts/src/geometry/core/vector_traits.rs`
- `model/geo_contracts/src/geometry/core/mod.rs`
- `model/geo_contracts/src/lib.rs`

### Phase 2: Point

- `model/geo_core/src/point_traits.rs`
- `model/geo_core/src/point_2d.rs`
- `model/geo_core/src/point_3d.rs`
- `model/geo_contracts/src/geometry/core/point_traits.rs`
- `model/geo_contracts/src/geometry/core/mod.rs`
- `model/geo_contracts/src/lib.rs`

### Phase 3: AABB

- `model/geo_core/src/aabb_traits.rs`
- `model/geo_core/src/aabb_2d.rs`
- `model/geo_core/src/aabb_3d.rs`
- `model/geo_contracts/src/geometry/core/mod.rs`
- `model/geo_contracts/src/lib.rs`
- `model/geo_contracts/src/geometry/foundation/mod.rs` 必要に応じて注記更新

## 受け入れ条件

- `Vector` / `Point` / `AABB` の trait定義の正本が `geo_contracts` に明文化されている
- `geo_core` は concrete type 実装と変換共通核に責務を限定している
- `geo_primitives` / `geo_nurbs` は trait定義参照のために `geo_core` を前提にしない
- 移行順序が `Vector -> Point -> AABB` で固定されている
- 互換期間の扱いが「再エクスポート優先、duplicate trait 増殖禁止」で整理されている

## 実装前チェック

- `geo_core -> geo_contracts` 依存追加をアーキテクチャルール上で許可するか確認する
- 互換再エクスポートを許容する期間を PR 単位で定義する
- 各 Phase を個別 PR に分ける
