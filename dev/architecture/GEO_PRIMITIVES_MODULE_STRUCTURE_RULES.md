# geo_primitives モジュール構成ルール

最終更新: 2026-10-11
適用範囲: `model/geo_primitives/src`

## 1. 目的

形状の core（構造体・フィールド・補助関数）と責務ファイル（拡張・境界ボックス・変換・テスト）を親子のモジュールにし、形状の内部を形状モジュールの中に閉じる。兄弟のモジュールの構成では、責務ファイルから core の private な要素を使うために `pub(crate)` のフィールドや `*_internal` アクセサを増やす必要があった。

## 2. モジュール構成

形状ごとにディレクトリのモジュールを置き、core を親、責務ファイルを子とする。

```text
line_segment_3d/
  mod.rs         // 構造体（private フィールド）・core のメソッド・core の trait定義の実装
  extensions.rs  // 子モジュール: 親の private なフィールド・関数をそのまま使える
  bounds.rs
  transform.rs
  tests.rs       // #[cfg(test)]
```

- 子モジュールのファイル名は形状名を付けない（`extensions.rs` / `bounds.rs` / `transform.rs` / `metrics.rs` / `tests.rs`）
- `mod.rs` は先頭の doc コメントの直後で子モジュールを宣言する（`mod extensions;`、`#[cfg(test)] mod tests;` 等）
- `tests.rs` は `use super::*;` で親の要素を使い、`mod tests { ... }` で包まない（`tests::tests` の入れ子にしない）
- 形状をまたぐテスト（`transform_*_tests.rs` / `region_*_tests.rs` 等）は crate の直下に置き、公開 API だけを使う

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
| 楕円（楕円・楕円弧） | 移行済み。楕円弧は楕円の公開の取得メソッド（`Ellipse3D` の固有メソッド、`Ellipse2D` の trait定義 `Ellipse2DProperties`）を使い、`pub(crate)` は残さない。2D / 3D の楕円が共用する計算の補助モジュール（`ellipse_calculation_analysis` / `ellipse_calculation_strategy`）は crate の直下の private なモジュールとする |
| 平面・メッシュ（平面・矩形・三角形・三角形メッシュ） | 未移行 |
| 解析曲面・立体（球・楕円体・円柱・円錐・トーラス） | 未移行 |

`geo_nurbs` は、モジュール構成と命名規則を他の形状と揃える作業の中で別途扱う。
