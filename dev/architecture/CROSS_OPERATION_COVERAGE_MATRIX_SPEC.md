# Cross Operation Coverage Matrix Spec (Draft v0.1)

対象Issue: #364
関連: #362, #366, #367, #365, #363, #350, #351

## 1. 目的

- 2D/3D の形状ペア演算（collision/intersection/distance）の網羅性を機械判定可能にする。
- 実装漏れ、対称漏れ、重複登録をテストで検出できるようにする。
- #351 の段階削減時に、削除後の回帰を同じ判定軸で検証できるようにする。

## 2. スコープ

- 演算: `collision`, `intersection`, `distance`
- 次元: `2d`, `3d`
- 対象層: `geo_algorithms` の正本エントリ（free-function / pair-base）

非スコープ:
- 個別アルゴリズムの高精度化
- Foundationパターンへの回帰

## 3. 用語

- `shape pair`: 形状型の組（shape_a, shape_b）
- `operation`: 演算種別（collision/intersection/distance）
- `required`: 必須実装であること
- `symmetric`: A-B と B-A の整合が必須であること
- `cardinality`: intersection の期待交点種別（none/single/multiple/optional）

## 4. データモデル（論理スキーマ）

1レコードは次の項目を持つ。

```text
id: string                      # 一意ID（例: 2d:circle-ray:intersection）
dimension: enum(2d, 3d)
operation: enum(collision, intersection, distance)
shape_a: string                 # 例: Circle2D
shape_b: string                 # 例: Ray2D
required: bool
symmetric: bool
cardinality: enum(none, single, multiple, optional) | null
tolerance_profile: enum(strict, standard, relaxed) | null
entrypoint_a_to_b: string | null
entrypoint_b_to_a: string | null
notes: string | null
```

制約:
- `operation=intersection` のとき `cardinality` は必須。
- `operation!=intersection` のとき `cardinality` は `null`。
- `symmetric=true` のとき `entrypoint_a_to_b` と `entrypoint_b_to_a` は原則必須。
- 同一 `(dimension, operation, shape_a, shape_b)` の重複登録は禁止。

## 5. 判定ルール

- 未登録検知: `required=true` レコードに対応エントリが未設定なら fail。
- 対称性検知: `symmetric=true` で A-B/B-A の片側欠落または結果不一致なら fail。
- 重複検知: 主キー重複を fail。
- 交点種別検知: `intersection` で `cardinality` が期待外なら fail。

## 6. 実装配置方針

#366 実装までの暫定方針:
- 仕様正本: 本ドキュメント
- 初期実装: Rust 定数配列（テストモジュール内）
- 将来拡張: TOML/JSON外部化（必要時）

## 7. 2Dシード（#350反映の最小セット）

| id | op | shape_a | shape_b | required | symmetric | cardinality | a_to_b | b_to_a |
|---|---|---|---|---|---|---|---|---|
| 2d:circle-ray:collision | collision | Circle2D | Ray2D | true | true | null | circle2d_ray2d_collides | ray2d_circle2d_collides |
| 2d:segment-ray:collision | collision | LineSegment2D | Ray2D | true | true | null | line_segment2d_ray2d_collides | ray2d_line_segment2d_collides |
| 2d:circle-ray:intersection | intersection | Circle2D | Ray2D | true | true | multiple | circle2d_ray2d_intersections | ray2d_circle2d_intersections |
| 2d:segment-ray:intersection | intersection | LineSegment2D | Ray2D | true | true | single | line_segment2d_ray2d_intersection | ray2d_line_segment2d_intersection |
| 2d:circle-ellipse:intersection | intersection | Circle2D | Ellipse2D | true | true | optional | circle2d_ellipse2d_intersection | ellipse2d_circle2d_intersection |
| 2d:line-circle:intersection | intersection | InfiniteLine2D | Circle2D | true | true | multiple | infinite_line2d_circle2d_intersections | circle2d_infinite_line2d_intersections |

## 8. #364 完了定義（DoD）

- 本仕様の項目定義と制約が合意される。
- 2Dシード（最小セット）が #350 成果と整合する。
- #366 がこの仕様を直接実装できるレベルで記述される。

## 9. 次アクション

1. #364 で本仕様へのレビューコメントを反映して v1.0 確定
2. #366 で「未登録/非対称/重複」検出テストを実装
3. #367/#365/#363 で演算ごとに登録範囲を拡張
