# geo_contracts の trait定義の型付けと形状 API の一本化

最終更新: 2026-10-12
適用範囲: `model/geo_core` / `model/geo_contracts` / 形状のクレート（`geo_primitives` / `geo_nurbs`）と、その呼び出し元

## 1. 目的

`geo_contracts` の trait定義は、点・ベクトル・方向をタプル（`(T, T)` / `(T, T, T)`）で受け渡している。値型（`Point2D/3D`・`Vector2D/3D`・`Direction2D/3D`）を定義する `geo_core` が `geo_contracts` に依存しており、`geo_contracts` から値型を参照できないためである。その結果、次の問題が生じている。

- **API の二重化**: 形状が、型付きの固有メソッドとタプルの trait定義の実装の両方を持ち、相互に変換している（同じ名前のメソッドが両方にあるものが 30 形状で 216 個）
- **名前の衝突**: 固有メソッドと trait定義のメソッドが同名で型が異なる場合、メソッド呼び出しでは固有メソッドが優先され、trait定義のメソッドを呼んでいた箇所の型が変わる
- **型安全性の欠如**: タプルは点・ベクトル・方向を区別できない

`geo_core` を `geo_contracts` の下位に置き、trait定義を値型で型付けして、形状の API を trait定義に一本化する。

これにより、当初の構成で意図していた抽象化と実装の責務の分離を実現する。

- **抽象化（`geo_contracts`）**: 形状が何をできるかを trait定義で定める。実装を持たない
- **値型の実装（`geo_core`）**: 抽象化に依存しない具体的な値型（点・ベクトル・方向・相似変換・AABB）と、その操作を実装する
- **形状（`geo_primitives` / `geo_nurbs`）**: `geo_core` の値型を使って `geo_contracts` の抽象化を実装し、上位の層（`geo_topology` / `geo_algorithms` / application 等）から trait定義を通じて呼ばれる

現在の構成では、`geo_core` が `geo_contracts` に依存するため、抽象化（trait定義）が具体的な値型を使えず、タプルで受け渡していた。依存の向きを入れ替えることで、抽象化が値型で型付けされ、形状の API が抽象化の一つに揃う。

## 2. クレートの依存関係

```text
analysis
  ← geo_core（点・ベクトル・方向・相似変換・AABB・点の分類・許容誤差の既定値・変換のエラー）
    ← geo_contracts（形状の trait定義。値型で型付け）
      ← geo_primitives / geo_nurbs / geo_topology / geo_algorithms …
```

- 新しいクレートは作らない。`geo_core` は `analysis` のみに依存する
- `geo_core` は trait定義を持たず、値型の操作は固有メソッドで提供する
- `geo_contracts` は `analysis` と `geo_core` に依存する
- `geo_contracts` は `geo_core` の値型（点・ベクトル・方向・相似変換・AABB）を再エクスポートする。trait定義の引数・戻り値に現れる型を、trait定義と同じクレートから使えるようにするため。Model 層の外からも `geo_contracts::Point3D` のように値型を作れる

### 2.0 抽象化と実装の分離

| クレート | 持つもの |
| --- | --- |
| `geo_contracts` | 抽象化: trait定義だけ（実装を持たない） |
| `geo_core` | 値型の実装: 値型と、その操作の固有メソッド（相似変換の適用を含む） |
| 形状のクレート（`geo_primitives` / `geo_nurbs`） | 形状の実装: `geo_core` の値型を使った trait定義の実装 |
| 上位の層（`geo_topology` / `geo_algorithms` / application 等） | trait定義を通じて形状を使う |

Rust の orphan rule では、trait の実装は trait定義のクレートか型のクレートに置く。`geo_core` は `geo_contracts` より下位にあり trait定義を参照できないため、`geo_core` の値型には `geo_contracts` の trait定義を実装しない。値型の操作はすべて固有メソッドとし、`geo_contracts` には実装を置かない。

### 2.1 `geo_core` に置くもの

| 要素 | 現在の場所 | 扱い |
| --- | --- | --- |
| 点・ベクトル・方向・相似変換・AABB | `geo_core` | そのまま |
| 値型の trait定義（`Point*` / `Vector*` / `Direction*` / `Aabb*` の Constructor / Properties 等） | `geo_contracts` | 削除する。境界・UFCS として使われておらず、固有メソッドで足りる |
| 許容誤差の既定値（`default_distance_tolerance` 等・`ToleranceSettings`） | `geo_contracts/src/tolerance.rs` | `geo_core` に移す。`geo_contracts` から再エクスポートし、呼び出し側のパスを変えない。使われていない `GeometryContext` は削除する |
| `PointClassification` | `geo_contracts` | `geo_core` に移し、`geo_contracts` から再エクスポートする |
| `TransformError` | `geo_contracts` | `geo_core` に移す（相似変換の生成が返す）。`geo_contracts` から再エクスポートする |

許容誤差の既定値は幾何のカーネルの値であり、`analysis`（純粋な数値計算）には移さない。

### 2.2 相似変換

- `SimilarityTransformable2D/3D` の trait定義は `geo_contracts` に残し、引数を具体型（`&SimilarityTransform2D<T>` / `&SimilarityTransform3D<T>`）にする
- 実装が 1 つずつしかない `SimilarityTransform2D/3DCore` の trait定義は削除する
- `SimilarityTransformable*` は形状のクレートだけが実装する。`geo_core` の値型には実装しない（ジェネリクスの境界として使われていない）
- 値型の変換は、相似変換の型の固有メソッドで行う
  - 点・ベクトル・方向: `apply_point(Point3D<T>) -> Point3D<T>` / `apply_vector` / `apply_direction`（失敗しないため値を返す）
  - 形状の相似変換の実装は、値型のフィールドを `transform.apply_point(self.center)` のように変換する
- AABB の相似変換は、呼び出し元がないため提供しない（回転後の境界ボックスは形状の側で再構築する）
- AABB の `Contains` の実装は削除し、固有メソッド（`contains_point` 等）を使う。他に実装のない `Contains` の trait定義も削除する

## 3. 形状の API の正本

- 形状の公開 API は `geo_contracts` の trait定義のメソッドとする
- trait定義と同じ名前の固有メソッドは削除する。固有メソッドは実装の補助（private）とし、trait定義にない公開メソッドが必要になった場合は trait定義に追加する
- 生成（`new` 等）も Constructor の trait定義を正本とする
- 値型（`geo_core`）は trait定義を持たないため、固有メソッドを API とする
- 形状の具体型を引数・戻り値に使う API は、関連型を持つ trait定義にする（`Bounded` の `type Aabb`、`LineSegment3DSupportLine` の `type Line` 等）

### 3.1 prelude

trait のメソッドを呼ぶには、その trait定義が `use` されている必要がある。`geo_contracts::prelude` で形状の trait定義・共通の capability・形状をまたぐ演算の trait定義を再エクスポートし、呼び出し側は `use geo_contracts::prelude::*;` の 1 行で使えるようにする。

- 1 つの型が同じ名前のメソッドを持つ 2 つの trait定義を実装すると、prelude の一括 import で呼び出しが曖昧になる。trait定義の型付けの際に、同じ型に同名のメソッドが重ならないようにする

## 4. 署名の規約

| 種類 | 引数 | 戻り値 |
| --- | --- | --- |
| 値型（点・ベクトル・方向。`Copy`） | 値（`Point3D<T>` / `Vector3D<T>` / `Direction3D<T>`） | 値 |
| スカラー | 値（`T`） | 値 |
| 角度 | 値（`Angle<T>`） | 値（`Angle<T>`） |
| 形状（線分・円・NURBS 等） | 参照（`&LineSegment3D<T>`） | 値 |
| 点の並び | スライス（`&[Point3D<T>]`） | `Vec<Point3D<T>>` |

- 値型は `Copy` で大きさも同じ（`Point3D<f64>` は 24 バイト）ため、点・ベクトル・方向を区別せず値で渡す。読むだけの引数を参照にする理由はなく、フィールドに保存する生成でも複製が要らない
- 二項のメソッドの相手（`other`）も、値型であれば値で渡す
- 角度（開始角・終了角・掃引角・角度指定の評価等）は `Angle<T>` とし、ラジアンの `T` と区別する
- 円弧・楕円弧の角度範囲は、開始角から反時計回りに終了角までとする（掃引角・評価点・角度範囲の判定で同じ解釈を使う）
- AABB・相似変換の型の同士の引数（`intersects(&other)` / `then(&next)`）と、`analysis` の trait定義の実装（`TolerantEq::tolerant_eq`）は参照のままとする

- 単位ベクトルであることを前提とする引数・戻り値は `Direction*` とし、`Vector*` と区別する
- 形状として成立しない入力の生成は `Option`、変換の失敗は `Result<_, TransformError>`、範囲外の評価は `Option` を返す（既存の規約のまま）
- タプルを受け渡す trait定義のメソッドは残さない
- `contains_point(point)` は既定の距離の許容誤差で判定する。呼び出し元が許容誤差を指定する場合は `distance_to_point(point) <= tolerance` と書く。専用のメソッド（`contains_point_within`）は、それで書けない呼び出し元が現れた時点で追加する

## 5. 移行の順序

| 段階 | 内容 | 形状の API |
| --- | --- | --- |
| D-0 | 本設計書 | 変更なし |
| D-1 | `geo_core` を下位に移す（値型の trait定義の削除、許容誤差・`PointClassification`・`TransformError` の移動、相似変換の型付け、`ARCHITECTURE.md` と依存のルールの更新）。完了 | 変更なし |
| D-2〜 | 形状の系統ごと（直線 → 円・楕円 → 平面・矩形・三角形 → 解析曲面・立体 → NURBS → 形状をまたぐ演算）に、trait定義の型付け・重複する固有メソッドの削除・呼び出し元の更新。最初の系統で prelude を追加する | 系統ごとに変更 |
| D-2a | 無限直線（`InfiniteLine2D/3D`）の型付けと prelude の追加。完了 | 無限直線 |
| D-2a-2 | `geo_core` の値型の固有メソッドの引数を値渡しに揃える（`apply_point`・`Vector3D::from_points`・`distance_to` 等）。完了 | 変更なし |
| D-2b | 半直線（`Ray2D/3D`）の型付け。完了 | 半直線 |
| D-2c | 線分（`LineSegment2D/3D`）の型付け。完了 | 線分 |
| D-3a | 円（`Circle2D/3D`）の型付け。完了 | 円 |
| D-3b | 円弧（`Arc2D/3D`）の型付け。完了 | 円弧 |
| D-3c | 楕円（`Ellipse2D/3D`） | 楕円 |
| D-3d | 楕円弧（`EllipseArc2D/3D`） | 楕円弧 |
| 最後 | `geo_algorithms` 等の Point / タプルの境界の残りの整理と、規約の文書の仕上げ | — |

- 旧 API の別名・互換用の変換は置かない（各段階で呼び出し元まで更新する）
- 段階の作業で呼び出し元がなくなった公開 API は、その段階の系統（と、その系統からしか使われていなかった値型のメソッド）の分を削除する。他の系統の分は、その系統の段階で扱う（それまでは未使用の公開 API の検出の警告として残る）
- 各段階で `scripts/check_all_before_push.ps1` と依存関係のチェックを通す

## 6. 関連

- `ARCHITECTURE.md`（クレートの責務と依存）
- `GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md`（trait定義の構成）
- `GEO_API_ADDITION_RULES.md`（API の追加の規則）
- `TRANSFORM_TRAIT_DESIGN.md`（変換の trait定義）
