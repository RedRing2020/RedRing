# geo_primitives モジュール構成ルール

最終更新: 2026-10-12
適用範囲: `model/geo_primitives/src`、`model/geo_nurbs/src`（6 章）、`model/geo_core/src`（7 章）

## 1. 目的

形状の core（構造体・フィールド・補助関数）と責務ファイル（拡張・境界ボックス・変換・テスト）を親子のモジュールにし、形状の内部を形状モジュールの中に閉じる。兄弟のモジュールの構成では、責務ファイルから core の private な要素を使うために `pub(crate)` のフィールドや `*_internal` アクセサを増やす必要があった。

## 2. モジュール構成

形状ごとにディレクトリのモジュールを置き、core を親、責務ファイルを子とする。

```text
line_segment_3d.rs  // 親: 構造体（private フィールド）・core のメソッド・core の trait定義の実装
line_segment_3d/
  extensions.rs     // 子モジュール: 親の private なフィールド・関数をそのまま使える
  bounds.rs
  transform.rs
  tests.rs          // #[cfg(test)]
```

- 子モジュールのファイル名は形状名を付けない（`extensions.rs` / `bounds.rs` / `transform.rs` / `metrics.rs` / `tests.rs`）
- 親はディレクトリと同名のファイル（`line_segment_3d.rs`）に置き、`mod.rs` は使わない（Clippy の `mod_module_files` で確認する）
- 親は先頭の doc コメントの直後で子モジュールを宣言する（`mod extensions;`、`#[cfg(test)] mod tests;` 等）
- `tests.rs` は `use super::*;` で親の要素を使い、`mod tests { ... }` で包まない（`tests::tests` の入れ子にしない）
- 形状のテストは `tests.rs` にまとめ、親・子のファイルにテストを置かない（テストの置き場所の全体の規則は `TEST_PLACEMENT_RULES.md`）
- 形状をまたぐテストは結合テスト（`tests/cross_shape/main.rs` とその子モジュール）に置き、公開 API だけを使う。結合テストはクレートの外からコンパイルされるため、公開 API だけを使うことをコンパイラが確認する
- `src` の直下には `lib.rs` と形状のモジュール（親のファイルとディレクトリ）だけを置く

## 3. `lib.rs`

- 形状のモジュールは private とし（`mod line_segment_3d;`）、型だけを再エクスポートする（`pub use line_segment_3d::LineSegment3D;`）
- クレート外からのパスは型の再エクスポートのみとし、モジュールのパス（`geo_primitives::line_segment_3d::...`）は公開しない

## 4. 可視性

| 対象 | 規則 |
| --- | --- |
| フィールド | private とする。子モジュールは親の private なフィールドを直接使う |
| 補助関数 | 形状モジュールの中だけで使うものは private とする |
| `*_internal` アクセサ | 同じ形状の中で使うだけのものは置かない（フィールドを直接使う） |
| 形状をまたぐ内部の参照 | 公開の取得メソッドがあればそれを使う。ない場合に限り `pub(crate)` とし、doc コメントに理由（どの形状から使うか）を書く |

- 公開の取得メソッドを新たに追加する場合は、trait定義の同名のメソッド（タプルを返す `point()` / `direction()` 等）と名前が衝突しないことを確認する。固有メソッドが優先されるため、trait定義のメソッドを呼んでいた箇所の型が変わる
- 公開 API の追加は `GEO_API_ADDITION_RULES.md` に従う

## 5. 移行の状況

| 形状の系統 | 状況 |
| --- | --- |
| 直線（無限直線・半直線・線分） | 移行済み。`InfiniteLine2D/3D::point_internal` / `direction_internal` は線分から使うため `pub(crate)` |
| 円（円・円弧） | 移行済み。楕円の円からの生成は円の trait定義の取得メソッド（`Circle2DProperties` / `Circle3DProperties`）を使い、`pub(crate)` は残さない |
| 楕円（楕円・楕円弧） | 移行済み。楕円弧は楕円の公開の取得メソッド（`Ellipse3D` の固有メソッド、`Ellipse2D` の trait定義 `Ellipse2DProperties`）を使い、`pub(crate)` は残さない |
| 平面・メッシュ（平面・矩形・三角形・三角形メッシュ） | 移行済み。形状をまたぐテストは trait定義の取得メソッド（`Triangle2D/3DBoundaryAccess`・`Plane3DProperties`）を使う。法線を指定した三角形メッシュの生成（`from_parts`）は private とし、そのテストは三角形メッシュの `tests.rs` に置く |
| 解析曲面・立体（球・楕円体・円柱・円錐・トーラス） | 移行済み。形状をまたぐ参照はなく、`pub(crate)` は残さない。形状をまたぐテストは trait定義の取得メソッド（`TorusSurface3DProperties` 等）を使う |

## 6. `geo_nurbs`

`geo_nurbs` の形状（`NurbsCurve2D` / `NurbsCurve3D` / `NurbsSurface3D`）も 2〜4 章に従う。形状のディレクトリ名は型名の snake_case とする（`nurbs_curve_2d/` / `nurbs_curve_3d/` / `nurbs_surface_3d/`）。

`geo_primitives` と異なる点は次のとおり。

- 形状が共用する NURBS の計算（`basis` / `knot` / `weight_storage` / `error` / `adaptive_tessellation` / `operations`）は `src` の直下の private なモジュールとし、型・関数を `lib.rs` から再エクスポートする
- 形状をまたぐ内部の参照（テセレーションの補助関数等）は、共用のモジュールの `pub(crate)` とする
- 定数（`constants`）はクレート外から使うため、公開のモジュールとする
- 形状をまたぐテストは結合テスト（`tests/similarity_transform.rs`）に置く

## 7. `geo_core`

`geo_core` の値型（`Point2D/3D` / `Vector2D/3D` / `Direction2D/3D`）・相似変換（`SimilarityTransform2D/3D`）・AABB（`Aabb2D/3D`）も 2〜4 章に従う。モジュールは private とし、型だけを `lib.rs` から再エクスポートする。`geo_contracts` の trait定義は再エクスポートせず、利用側が `geo_contracts` から直接使う。
