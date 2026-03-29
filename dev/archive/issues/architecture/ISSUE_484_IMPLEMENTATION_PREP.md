# Issue #484 実施準備チェックリスト

対象Issue: [#484 intersection返り値意味論の個別適用: Coincident/Segment2D を具体ペアへ反映](https://github.com/RedRing2020/RedRing/issues/484)

## 1. 目的

- #357 で定義した `IntersectionResult` 意味論を、2D 代表ペアへ具体適用する
- `Disjoint` / `Touching` / `Crossing` / `Coincident` と `Segment2D` の使い分けを実装で固定する
- 代表ケースの回帰テストを追加し、今後の退行を防ぐ

## 2. 現状確認（2026-03-29）

- `circle2d_circle2d_intersections_algo` は点群変換のみで、同一円と同心円半径違いを区別していない
- `line_segment2d_line_segment2d_intersection_algo` は `Option<Point2D>` 前提で、コリニア重複を `Segment2D` 化できない
- `ray2d_line_segment2d_intersection` / `infinite_line2d_line_segment2d_intersection` / `infinite_line2d_ray2d_intersection` は平行・共線時に空へ潰れる

## 3. 設計オプション

### Option A: `pair_base` 側で返り値を `IntersectionResult` 化

- 長所: 意味論が低層で一元化できる
- 短所: 既存呼び出し面の影響が大きく、差分が肥大化しやすい

### Option B: `primitive_2d` 入口で意味論を付与（採用）

- 長所: 公開APIの振る舞い修正に集中でき、最小差分で #484 の受け入れ条件を満たしやすい
- 短所: `pair_base` の幾何計算は旧形のまま残る

### Option C: 一部ペアのみ先行対応

- 長所: 最小リスク
- 短所: #484 の「主要線形ペア」条件が満たしにくい

## 4. 実装方針（Option B）

- `model/geo_algorithms/src/intersection/primitive_2d.rs` のみを修正対象にする
- 次の入口関数で共線/重複分岐を追加する
  - `circle2d_circle2d_intersections_algo`
  - `line_segment2d_line_segment2d_intersection_algo`
  - `ray2d_line_segment2d_intersection`
  - `infinite_line2d_line_segment2d_intersection`
  - `infinite_line2d_ray2d_intersection`
- 既存の非共線分岐は現行ロジックを維持する

## 5. 意味論ルール（本実装で固定）

- 同一円: `topology=Coincident`, `geometry=Coincident`
- 同心円・半径違い: `Disjoint`
- コリニア線分の部分重複: `topology=Coincident`, `geometry=Segment2D`
- コリニア線分の一点接触: `topology=Touching`, `geometry=Point2D`
- 直線×線分の共線: `topology=Coincident`, `geometry=Segment2D`
- 直線×半直線の共線: `topology=Coincident`, `geometry=Coincident`（連続重複を表現）
- 半直線×線分の共線重複: 一点は `Touching/Point2D`、区間は `Coincident/Segment2D`

## 6. 追加テスト

- 同一円と同心円半径違いの判定
- コリニア線分の部分重複 / 一点接触 / 完全一致
- 共線 `Ray2D×LineSegment2D` の区間重複
- 共線 `InfiniteLine2D×LineSegment2D` の `Segment2D`
- 共線 `InfiniteLine2D×Ray2D` の `Coincident`

## 7. 検証

- `cargo test -p geo_algorithms`
- `cargo clippy -p geo_algorithms -- -D warnings`
- `cargo fmt --all -- --check`
