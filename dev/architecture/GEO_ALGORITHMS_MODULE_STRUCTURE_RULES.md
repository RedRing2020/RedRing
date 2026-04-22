# geo_algorithms モジュール分割ルール

最終更新: 2026-04-23
適用範囲: `model/geo_algorithms/src/{collision,intersection,distance}`

## 1. 目的

`primitive_2d` / `primitive_3d` / `primitive_nurbs*` の分割基準を統一し、
実装追加時の迷いと配置ゆれを防止する。

## 2. ルールの原則

- 原則1: **演算種別でディレクトリを分ける**
  - `collision/`, `intersection/`, `distance/`
- 原則2: **次元と形状系でファイルを分ける**
  - 2D primitive, 3D primitive, NURBS curve, NURBS surface
- 原則3: **対称性は正規方向 + 逆方向委譲**
  - 正規方向だけ計算本体を持ち、逆方向はラッパー
- 原則4: **公開面は mod.rs で一元再エクスポート**
- 原則5: **新規ペア追加時は coverage matrix を同時更新**
- 原則6: **Point/Tuple 境界は Point 型を正本にする**
  - `geo_algorithms` の公開 free-function では `Point2D<T>` / `Point3D<T>` を優先する
  - 下位 trait定義や既存 shape method が tuple を要求する場合でも、tuple は内部 bridge に限定する
  - 呼び出し側が tuple を都度組み立てる経路を増やさず、Point 型の入口へ寄せる

### 2.1 Point/Tuple 境界の段階移行

- 第1段では `shape-point` 系の距離 entrypoint を Point 型正本として整える
- collision / intersection は、その Point 型 distance entrypoint を代表箇所から優先的に利用する
- `ToPoint2D<T>` / `ToPoint3D<T>` のような一般化 helper は、複数 shape family で同型の変換需要が確認できた段階で導入する

### 2.2 分割軸の定義（違和感対策）

本ルールは、次の2軸で分類する。

- 第1軸: 計算空間の次元（2D / 3D）
- 第2軸: 形状ファミリ（primitive / nurbs_curve / nurbs_surface）

整理すると以下になる。

- `primitive_2d.rs`: 2D × primitive
- `primitive_3d.rs`: 3D × primitive
- `nurbs_curve_3d.rs`: 3D × nurbs_curve
- `nurbs_surface_3d.rs`: 3D × nurbs_surface

ただし現行ファイル名 `primitive_nurbs*` は、
この2軸表現を名前に十分反映できておらず、命名上の一貫性は未達である。

理由:

- `geo_nurbs` 全体では `NurbsCurve2D` / `NurbsCurve3D` / `NurbsSurface3D` を提供する
- 一方 `geo_algorithms` の NURBS × Primitives 集約スコープ（本ルール対象）は現時点で 3D を主対象とする
- curve/surface でアルゴリズム責務（反復・収束特性・サンプリング戦略）が異なる
- `primitive_3d.rs` に混在させると責務過密になる

### 2.3 正規命名（目標）

分割軸に一致した正規命名を以下とする。

- `primitive_2d.rs`
- `primitive_3d.rs`
- `nurbs_curve_2d.rs`（必要時）
- `nurbs_curve_3d.rs`
- `nurbs_surface_3d.rs`

`primitive_nurbs.rs` / `primitive_nurbs_surface.rs` はレガシー互換名として扱う。

## 3. 標準ファイル構成

各演算ディレクトリで、以下を標準とする。

```text
{operation}/
  mod.rs
  pair_base.rs                 # 必要時のみ（共通計算）
  primitive_2d.rs
  primitive_3d.rs
  nurbs_curve_2d.rs            # NurbsCurve2D を扱う場合のみ
  nurbs_curve_3d.rs            # 既存 primitive_nurbs.rs の正規後継名
  nurbs_surface_3d.rs          # 既存 primitive_nurbs_surface.rs の正規後継名
```

補足:
- `distance` は現時点で `pair_base.rs` 不要。
- 既存互換のため、当面は `primitive_nurbs.rs` / `primitive_nurbs_surface.rs` を維持してよい。

## 4. 配置基準（どのファイルに置くか）

- `primitive_2d.rs`: 2D primitive 同士
- `primitive_3d.rs`: 3D primitive 同士
- `nurbs_curve_2d.rs`: NurbsCurve2D を含むペア（存在する場合）
- `nurbs_curve_3d.rs` (または `primitive_nurbs.rs`): NurbsCurve3D を含むペア
- `nurbs_surface_3d.rs` (または `primitive_nurbs_surface.rs`): NurbsSurface3D を含むペア
- `pair_base.rs`: collision/intersection の共通下位ロジック

補足（命名上の見え方について）:

- 既存ファイル名 `primitive_nurbs.rs` / `primitive_nurbs_surface.rs` は歴史的名称。
- 正規化後は `nurbs_curve_3d.rs` / `nurbs_surface_3d.rs` を推奨し、
  「primitive と NURBS で軸が違う」印象を無くす。

### 4.1 primitive_3d 系の責務分割方針

- `primitive_3d.rs` が shape family 見出しコメントに依存し始めた場合は、コメント追加ではなく module 分割を優先する。
- 分割単位は 1 shape 1 ファイルを既定にせず、shape family または line-like / planar-like の責務単位でまとめてよい。
- family 横断 helper は `shared` 系 module へ隔離し、各 family module に混在させない。
- 利用側の公開面は親 `primitive_3d` module と上位 `mod.rs` で一元化し、分割詳細を外へ漏らさない。
- コメント整理は分割後に不要になった section 見出しを落とす形で行い、コメント削除だけを先行させない。

## 5. 関数命名規約

- 通常距離: `{shape_a}_{shape_b}_distance`
- 失敗可能距離: `{shape_a}_{shape_b}_try_distance`
- 逆方向: `{shape_b}_{shape_a}_distance` は `{shape_a}_{shape_b}_distance` へ委譲

例:
- `nurbscurve3d_ray3d_distance`
- `nurbscurve3d_ray3d_try_distance`
- `ray3d_nurbscurve3d_distance` (委譲)

## 6. mod.rs の並び順（固定）

`mod.rs` は次順序で宣言・`pub use` する。

1. `pair_base`（ある場合）
2. `primitive_2d`
3. `primitive_3d`
4. `nurbs_curve_2d`（ある場合）
5. `nurbs_curve_3d`（現行は `primitive_nurbs`）
6. `nurbs_surface_3d`（現行は `primitive_nurbs_surface`）

## 7. 段階移行ルール（既存コード保護）

- Phase A (即時): 新規実装は本ルールに従って配置。
- Phase B (中期): `primitive_nurbs*` を `nurbs_curve_3d` / `nurbs_surface_3d` へリネーム。
- Phase C (中期): NurbsCurve2D を演算層で扱う場合は `nurbs_curve_2d.rs` を新設。
- Phase D (中期): `distance` の NURBS 実装完了後、演算間の命名を完全統一。

重要:
- 一括リネームは避ける。Issue 単位で段階的に行う。
- API 互換を壊す変更は PR 単位で明示する。

## 8. PR チェック項目（必須）

- [ ] 追加関数の配置先が本ルールと一致
- [ ] 逆方向関数が委譲実装
- [ ] `mod.rs` の宣言・再エクスポート順が規約準拠
- [ ] `coverage_matrix_engine.rs` へ対称エントリ追加
- [ ] `cargo clippy -p geo_algorithms -- -D warnings` 成功
- [ ] `cargo fmt --all` 実行
- [ ] `cargo test -p geo_algorithms` 成功
- [ ] `./scripts/check_architecture_dependencies_simple.ps1` 成功

## 9. #403 への適用メモ

- #403 Phase2 では、当面 `distance/primitive_3d.rs` に NURBS×3D primitive を集約する。
- ただし将来整理で `distance/nurbs_curve_3d.rs` / `distance/nurbs_surface_3d.rs` へ分離可能。
- その際は本ルールを正として移行する。

## 10. geo_primitives 側の交点実装に関するポリシー（Issue #697 追記）

最終更新: 2026-04-23

### 10.1 正本の所在

**交点計算・距離計算・衝突判定の正本は `geo_algorithms` に置く。**

`geo_primitives` 側のメソッド（例: `Ray2D::intersection_with_ray`、`InfiniteLine2D::intersection_with_line`）が同種の計算を行っている場合でも、それらは正本ではない。

### 10.2 `geo_primitives` 側の実装を許容するケース

以下の場合に限り、`geo_primitives` 側にロジックを持つことを許可する。

- **プリミティブ固有の補助演算**（例: `project_point`、`contains_point`）  
  ― 形状の定義に密接し、`geo_algorithms` へ委譲するほどではないもの。
- **convenience エイリアス**（正本実装を内部呼び出しするラッパー）  
  ― このケースでは「委譲先・正本 Issue を doc コメントに明記」することを必須とする。

### 10.3 禁止事項

- `geo_primitives` 側に、`geo_algorithms` と**並行した独立実装**を新規追加しない。
- `Option<Point2D<T>>` など「`IntersectionResult<T>` より情報が落ちる型」で、`geo_algorithms` 正本と同等のロジックを持つ実装を残存させない。

### 10.4 既存の重複実装に気づいた場合

1. `geo_algorithms` 側に正本実装が既存か確認する。
2. 既存なら、`geo_primitives` 側は Issue で棚卸し対象として記録する。
3. 「空返し」「コインシデント非対応」など情報落ちが確認できた時点で、Issue を起票して削除・委譲化を計画する。

**参照 Issue**: #697（2D交点重複の棚卸し）、#696（Ellipse3D スタブ廃止）

---

## 11. #338 再開時の戻り値移行ルール

`#338` では、公開関数の戻り値を `IntersectionResult<T>` へ直接置換する。
互換層として公開 API を二重化しない。

### 11.1 基本方針

- 既存の `*_intersection` 関数名は維持しつつ、戻り値型を `IntersectionResult<T>` に切り替える。
- 旧 `Option<Point3D<T>>` ロジックが必要な場合は private helper へ下げ、公開面には残さない。
- `IntersectionResult::from_option_point()` / `from_option_points()` を使用して変換規約を統一する。
- 一度に多数ペアを移行せず、1～2ペア単位の小PRで進める。

### 11.2 初期スライス（Phase C 再開の最小単位）

- `intersection/primitive_3d.rs` の point系ペアから開始する。
  - `arc3d_point3d_intersection`
  - `circle3d_point3d_intersection`

### 11.2.1 継続スライス（同一PR系列で拡張する範囲）

- point系の次は、同じ形状対の 1次元入力を対象に広げる。
  - `arc3d_line_segment3d_intersection`
  - `arc3d_ray3d_intersection`
  - `arc3d_infinite_line3d_intersection`
  - `circle3d_line_segment3d_intersection`
  - `circle3d_ray3d_intersection`
  - `circle3d_infinite_line3d_intersection`

選定理由:

- 既存実装が `Option<Point3D<T>>` の単一点返却で揃っている
- `IntersectionResult::from_option_point()` をそのまま適用できる
- `arc` / `circle` 系で命名・テストパターンを揃えやすい

### 11.3 実装順序

1. 公開 `*_intersection` の戻り値を `IntersectionResult<T>` へ変更
2. private helper 化で旧単一点ロジックを局所化
3. 変換規約テスト追加（Disjoint / Crossing を最低限確認）
4. 次ペアへ同じ置換を展開

## 12. #466 多点関数の IntersectionResult 拡張戦略

### 12.1 概要

`#338` で単点関数（`Option<Point3D<T>>` → `IntersectionResult<T>`）の置換が完了した。
次段階として、複数孤立点を返す関数（`Vec<Point3D<T>>` → `IntersectionResult<T>`）の統一を進める。

### 12.2 既存 Points 実装の確認

`result.rs` には以下が既に実装済み:

- `IntersectionGeometry::Points(Vec<Point3D<T>>)` バリアント
- `IntersectionResult::points(points, is_tangent, tolerance) -> Self`
- `IntersectionResult::from_option_points(points, is_tangent, tolerance) -> Self`
- topology 自動分類: `is_tangent=true` → Touching、`false` → Crossing

#### 12.2.1 多点コンストラクタの仕様

```rust
pub fn points(points: Vec<Point3D<T>>, is_tangent: bool, tolerance: T) -> Self {
    IntersectionResult {
        geometry: IntersectionGeometry::Points(points),
        topology: if is_tangent {
            IntersectionTopology::Touching
        } else {
            IntersectionTopology::Crossing
        },
        is_tangent,
        tolerance_used: tolerance,
    }
}

pub fn from_option_points(points: Vec<Point3D<T>>, is_tangent: bool, tolerance: T) -> Self {
    if points.is_empty() {
        return Self::disjoint(tolerance);
    }
    Self::points(points, is_tangent, tolerance)
}
```

規約:
- 空ベクタ → `Disjoint`
- 1点以上 → `Touching` または `Crossing`（`is_tangent` で制御）

### 12.3 多点関数の対象一覧（12 個）

`intersection/primitive_3d.rs` で複数孤立点を返す関数:

#### Ellipse3D 系 7 個
- L539:  `pub fn ellipse3d_circle3d_intersections`
- L553:  `pub fn ellipse3d_arc3d_intersections`
- L567:  `pub fn ellipse3d_line_segment3d_intersections`
- L592:  `pub fn ellipse3d_infinite_line3d_intersections`
- L606:  `pub fn ellipse3d_ray3d_intersections`
- L646:  `pub fn ellipse3d_triangle3d_intersections`
- L673:  `pub fn ellipse3d_ellipse3d_intersections`

#### EllipsoidalSolid3D 系 2 個
- L732:  `pub fn ellipsoidal_solid3d_infinite_line3d_intersections`
- L746:  `pub fn ellipsoidal_solid3d_ray3d_intersections`

#### SphericalSurface3D 系 3 個
- L1186: `pub fn ray3d_spherical_surface3d_intersections`
- L1362: `pub fn line_segment3d_spherical_surface3d_intersections`
- L1505: `pub fn infinite_line3d_spherical_surface3d_intersections`

### 12.4 Topology 分類ルール

**多点交差の position 判定基準** （参考: Ellipse3D との交差パターン）:

| パターン | 返却点数 | is_tangent | 結果 Topology | 寄与 |
|:------:|:------:|:---------:|:-----:|:---|
| 横断交差 | 2 | false | Crossing | 2 つの孤立点が交差 |
| 接線接触（2点） | 2 | true | Touching | 微分一致だが 1 点扱い（数値的に 2 重根） |
| 接線接触（1点）| 1 | true | Touching | 接線接触 |
| 横断交差（1点）| 1 | false | Crossing | 1 孤立点（例: 退化形状) |

**実装の原則**:

1. **幾何から is_tangent を決定**: 孤立点の法線ベクタを調べ、接線条件（`dot(tangent_A, normal_B)` 等）を検証
2. **空返却は Disjoint**: `from_option_points()` で自動処理
3. **複数点の場合は `is_tangent` を統一**: 2 つ以上の孤立点がある場合、全て同じ `is_tangent` フラグを付与
   - 理由: Topology は形状ペア全体の関係を表すため。一部だけ接線は想定外

### 12.5 実装順序（段階的移行）

### 12.5.1 Phase 1: 設計固定（現在 = #466）

- [ ] MultiPoint Topology ルール確定（本セクション 12.4）
- [ ] 多点関数群の一覧表を公開（上記 12.3）
- [ ] 変換規約テストの雛形作成（下記参照）
- [ ] ドキュメント整備完了

**出力**: GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md へこれらを記載（#466 で完了）

### 12.5.2 Phase 2: Ellipse3D 系実装（別 Issue 予定）

1. 関数宣言を `Vec<Point3D<T>>` → `IntersectionResult<T>` に変更
2. 内部計算は変わらず、返却時に `IntersectionResult::from_option_points()` で変換
3. テスト追加: Crossing / Touching 両ケース確認
4. 逆向け委譲関数も同時更新

**対象**: 7 個関数（ellipse3d_*）

### 12.5.3 Phase 3: EllipsoidalSolid3D + SphericalSurface 実装

同じプロセスで残り 5 個関数を置換。

### 12.6 変換規約テストの例

```rust
#[test]
fn multipoint_from_option_points_empty_is_disjoint() {
    let result = IntersectionResult::<f64>::from_option_points(vec![], false, 1e-9);
    assert!(!result.intersects());
    assert_eq!(result.topology, IntersectionTopology::Disjoint);
}

#[test]
fn multipoint_crossing() {
    let pts = vec![
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(-1.0, 0.0, 0.0),
    ];
    let result = IntersectionResult::from_option_points(pts, false, 1e-9);
    assert!(result.intersects());
    assert_eq!(result.topology, IntersectionTopology::Crossing);
    assert!(!result.is_tangent);
    assert_eq!(result.geometry.dimension(), 0); // points
}

#[test]
fn multipoint_tangent_is_touching() {
    let pts = vec![
        Point3D::new(1.0, 1.0, 0.0),
        Point3D::new(1.0, -1.0, 0.0),
    ];
    let result = IntersectionResult::from_option_points(pts, true, 1e-9);
    assert!(result.intersects());
    assert_eq!(result.topology, IntersectionTopology::Touching);
    assert!(result.is_tangent);
}
```

### 12.7 注意点

1. **Topology は形状ペア全体の関係** → 複数点でも単一分類
2. **空返却は必ず Disjoint** → 自動処理（`from_option_points()` 利用）
3. **is_tangent は局所判定 + 統一** → 2 点以上ある場合、全て同じ値
4. **逆向け委譲** → `{shape_b}_{shape_a}_intersections()` は `{shape_a}_{shape_b}_intersections()` へ委譲

## 13. #472 2D 公開 API の Result 型統一方針

### 13.1 現状棚卸し（`intersection/primitive_2d.rs`）

- `Option<Point2D<T>>` 返却: 18 関数
- `Vec<Point2D<T>>` 返却: 14 関数
- 合計 31 公開関数のうち 32 戻り値（委譲関数を含む）で Option/Vec が使われている

主な対象カテゴリ:

- point系: `circle2d_point2d_intersection`, `arc2d_point2d_intersection`, `ellipse2d_point2d_intersection` ほか
- 線分/直線/ray 系: `ray2d_line_segment2d_intersection`, `infinite_line2d_ray2d_intersection` ほか
- 円/楕円/弧 系: `circle2d_circle2d_intersections_algo`, `ellipse2d_circle2d_intersections` ほか

### 13.2 型設計オプション（A/B/C）

#### Option A: 既存 `IntersectionResult<T>` を 2D に直接流用

概要:
- 2D 点群を z=0 へ昇格して `Point3D<T>` として格納し、既存 `IntersectionGeometry` を再利用

利点:
- 呼び出し側インターフェイスを最短で統一可能
- 既存 Topology と helper (`from_option_point(s)`) を再利用できる

欠点:
- 2D の意味論に 3D 型を混入させるため、モデルの説明力が低下
- `z=0` 前提が暗黙になり、将来の 2D 層責務を曖昧にする

#### Option B: `IntersectionGeometry` を 2D/3D 共通に拡張

概要:
- `IntersectionGeometry` に `Point2D` / `Points2D`（必要なら `Segment2D`）を追加
- `IntersectionResult<T>` 本体は維持しつつ geometry を次元対応させる

利点:
- 2D/3D 共通 API を維持しながら型意味を保てる
- 呼び出し側は `topology` 統一を享受しつつ、幾何型も正確

欠点:
- `result.rs` と既存 match 分岐の改修範囲が広い
- 3D 既存コードへの影響確認が必要

#### Option C: 2D 専用 Result 型（`IntersectionResult2D<T>`）を新設

概要:
- 2D 専用 enum/struct を追加し、3D とは別系統で段階移行

利点:
- 2D の意味論を最も明瞭に保持
- 既存 3D 実装への影響が小さい

欠点:
- 2D/3D で API が再び分岐し、統一目的が弱まる
- 中長期で bridge trait か wrapper が必要

### 13.3 #472 の推奨案

推奨: **Option B（共通 Result の次元拡張）**

理由:

1. #338/#470 で進めた「公開 API の Result 型統一」という方向性と整合する
2. Option A の 2D->3D 昇格は短期回避策としては有効だが、幾何意味の劣化が大きい
3. Option C は短期安全だが、2D/3D 統一という #472 の目的を満たしにくい

#### 12.3.1 追加する `IntersectionGeometry` バリアント一覧

`result.rs` の `IntersectionGeometry<T>` に以下を追加する:

| バリアント | 型 | 用途 |
|---|---|---|
| `Point2D(Point2D<T>)` | `geo_primitives::Point2D<T>` | 単一交点（2D） |
| `Points2D(Vec<Point2D<T>>)` | `Vec<geo_primitives::Point2D<T>>` | 複数交点（2D、例: 円同士の 2 交点） |
| `Segment2D(LineSegment2D<T>)` | `geo_primitives::LineSegment2D<T>` | 部分重複区間（2D、例: コリニア線分の重複部分） |

既存の 3D バリアント（`Segment(LineSegment3D<T>)`, `Coincident` 等）との対称性を維持する。

#### 12.3.2 `Segment2D` バリアントの使用ルール（重複区間の扱い）

2D 線分同士がコリニア（同一直線上）かつ区間が部分重複する場合は「線が返却される」ケースとなる。
3D の `Segment(LineSegment3D<T>)` と同じルールで以下のように分類する:

| 状態 | `topology` | `geometry` |
|---|---|---|
| 独立（交差なし） | `Disjoint` | `None` |
| 端点のみ接触 | `Touching` | `Point2D(pt)` |
| 1 点で横断交差 | `Crossing` | `Point2D(pt)` |
| 部分重複（有限区間） | `Coincident` | `Segment2D(seg)` |
| 完全一致 | `Coincident` | `Coincident` |

- `Segment2D` は **部分重複（有限区間の重なり）** のみに使用する
- 形状全体が完全一致する場合は引き続き `Coincident` バリアントを使用する
- `dimension()` と `description()` 実装も 3D Segment と対称に更新する

### 13.4 Topology 対応ルール（2D）

- `Disjoint`: 交差なし（空集合）
- `Touching`: 接線接触または端点接触
- `Crossing`: 横断交差（交点 1 以上）
- `Coincident`: 同一直線・同一円弧区間など、連続重なり

補足:
- 2 点交差でも形状ペア全体の関係は `Crossing` として単一分類
- `is_tangent` は 3D と同様に `Touching/Crossing` の補助判定として維持

### 13.5 段階移行計画（実装反映済み）

Phase D1: Result 型拡張の最小導入（完了）

- `result.rs` に `Point2D` / `Points2D` / `Segment2D` の 3 バリアントを追加済み
- `dimension()` / `description()` / `is_empty()` / `is_multiple()` を 3D と対称に実装済み
- 変換 helper（2D 用 `from_option_point2d()` / `from_option_points2d()`）を導入済み

Phase D2: 2D point系から先行置換（完了）

- `*_point2d_intersection` の Result 型移行を完了
- 小粒度 PR で置換・回帰確認を実施済み

Phase D3: 線分/直線/ray 系置換（完了）

- `ray/line/segment` 系関数の Result 型移行を完了
- 逆方向委譲の更新を完了

Phase D4: 円/楕円/弧の多点関数置換（完了）

- `*_intersections` の Vec 返却を Result 型へ移行完了
- Touching/Crossing/Coincident の分類テストを追加済み
- PR #481 マージ時点で 2D 側公開 API の置換を完了

Phase D5: 状況同期と運用固定（本タスク）

- 目的:
  - D1〜D4 完了状態と実装実態の乖離をドキュメント上で解消する
  - フリーズ/再開時の再調査コストを最小化する
- 実施項目:
  - 本セクションの完了ステータス更新（D1〜D4）
  - `primitive_2d.rs` / `primitive_3d.rs` の返り値方針を `IntersectionResult<T>` へ統一済みとして明記
  - 旧 WIP ブランチの記録は履歴として保持し、現行正本は `develop` とする方針を明記
- 完了条件:
  - 本ドキュメントのステータスが実コードと一致していること
  - 次回作業開始時に「どこまで完了か」を本ドキュメント単体で判断できること

### 13.6 現在ステータス（2026-03-28）

- 現行正本: `develop`
- 2D/3D intersection 公開 API の返り値は `IntersectionResult<T>` へ統一済み
- D5 は「設計更新と運用同期」のタスクとして継続管理し、追加実装を要求しない
