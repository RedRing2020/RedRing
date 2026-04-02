# geo_core / geo_contracts trait Source of Truth Design

## 背景

Issue #533 では、`geo_core` と `geo_contracts` にまたがって残っている基本型 trait定義の二重管理を整理する。

`#339` の `H-2` 調査時点で、少なくとも以下の状態が確認されている。

- `Point` / `Vector` の trait定義は `geo_contracts` と `geo_core` の双方に存在する
- `geo_contracts` 側の方が既に公開窓口として整っており、既定メソッドも一部多い
- `geo_core` 側ではローカル trait を `Point2D` / `Point3D` / `Vector2D` / `Vector3D` 実装の足場として保持している
- `AABB` 系 trait は現時点で `geo_core` にしか存在せず、`geo_contracts` 側には窓口がない

この状態では、基本型 trait定義の source of truth が曖昧であり、個別メソッド整理や利用側の import 整理を進めても根本解決にならない。

## 前提

本設計では、`geo_contracts` の責務を次のように扱う。

- `geo_contracts` は trait定義の正本を置くクレートとする
- `geo_contracts` には実装ロジックを置かない
- 具体型の実装、変換共通核、エラー型などの処理責務は `geo_core` に残す
- `geo_primitives` / `geo_nurbs` は、trait定義だけを知る用途では `geo_core` を知らなくてもよい形を目指す

## 観測事実

### Point / Vector

- `geo_contracts` には `Point2DCore` / `Point3DCore` / `Vector2DCore` / `Vector3DCore` が既に存在する
- `geo_core` にも同名の trait が残っている
- `geo_contracts` 側には `position` / `dimension` / `is_zero` / `is_unit` / `area` / `volume` / `length` など、`geo_core` 側にない既定メソッドがある
- `geo_core` 側 trait は主に `geo_core` 自身の concrete type 実装にしか使われていない

### AABB

- `geo_core` には `Aabb2DTrait` / `Aabb3DTrait` と concrete type `Aabb2D` / `Aabb3D` が存在する
- `geo_contracts` には `AABB` 系 trait定義が存在しない
- `Bounded` は `geo_contracts` 側に存在し、associated type `Aabb` を介して concrete type を返せる設計になっている

このため、AABB も trait定義だけを `geo_contracts` に追加し、concrete type を `geo_core` に残す構成が成立する。

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
- `geo_core::vector_traits` の役割を再エクスポートへ縮退するか削除する
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
- `geo_core::point_traits` を縮退または削除する
- `geo_core::Point2D` / `Point3D` が `geo_contracts` trait を実装するよう切り替える
- Point に依存する AABB 実装が次段で切り替えられるようにする

完了条件:

- `Point` trait定義の正本が `geo_contracts` に一本化されている
- `geo_core` 側の Point concrete type 実装が `geo_contracts` trait に統一されている

### Phase 3 影響ファイル: AABB

目的:

- AABB についても trait定義と concrete type を分離し、Point / Vector と同じ責務構成に揃える

実施内容:

- `geo_contracts` に `aabb_traits.rs` を追加する
- `geometry/core/mod.rs` と `lib.rs` から公開する
- `geo_core::Aabb2D` / `Aabb3D` が `geo_contracts` 側 AABB trait を実装するよう切り替える
- `Bounded::Aabb` は引き続き `geo_core::Aabb2D` / `geo_core::Aabb3D` を返してよい
- `geo_core::aabb_traits` は互換再エクスポートへ縮退するか最終的に削除する

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
- `model/geo_contracts/src/geometry/core/aabb_traits.rs` 新規
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
- `geo_contracts` に AABB trait を追加する配置先を `geometry/core` で確定する
- 互換再エクスポートを許容する期間を PR 単位で定義する
- 各 Phase を個別 PR に分ける
