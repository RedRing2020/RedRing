# geo_algorithms モジュール分割ルール

最終更新: 2026-03-28
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

### 2.1 分割軸の定義（違和感対策）

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

### 2.2 正規命名（目標）

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

## 10. #338 再開時の戻り値移行ルール

`#338` では、公開関数の戻り値を `IntersectionResult<T>` へ直接置換する。
互換層として公開 API を二重化しない。

### 10.1 基本方針

- 既存の `*_intersection` 関数名は維持しつつ、戻り値型を `IntersectionResult<T>` に切り替える。
- 旧 `Option<Point3D<T>>` ロジックが必要な場合は private helper へ下げ、公開面には残さない。
- `IntersectionResult::from_option_point()` / `from_option_points()` を使用して変換規約を統一する。
- 一度に多数ペアを移行せず、1～2ペア単位の小PRで進める。

### 10.2 初期スライス（Phase C 再開の最小単位）

- `intersection/primitive_3d.rs` の point系ペアから開始する。
  - `arc3d_point3d_intersection`
  - `circle3d_point3d_intersection`

### 10.2.1 継続スライス（同一PR系列で拡張する範囲）

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

### 10.3 実装順序

1. 公開 `*_intersection` の戻り値を `IntersectionResult<T>` へ変更
2. private helper 化で旧単一点ロジックを局所化
3. 変換規約テスト追加（Disjoint / Crossing を最低限確認）
4. 次ペアへ同じ置換を展開

## 11. #466 多点関数の IntersectionResult 拡張戦略

### 11.1 概要

`#338` で単点関数（`Option<Point3D<T>>` → `IntersectionResult<T>`）の置換が完了した。
次段階として、複数孤立点を返す関数（`Vec<Point3D<T>>` → `IntersectionResult<T>`）の統一を進める。

### 11.2 既存 Points 実装の確認

`result.rs` には以下が既に実装済み:

- `IntersectionGeometry::Points(Vec<Point3D<T>>)` バリアント
- `IntersectionResult::points(points, is_tangent, tolerance) -> Self`
- `IntersectionResult::from_option_points(points, is_tangent, tolerance) -> Self`
- topology 自動分類: `is_tangent=true` → Touching、`false` → Crossing

#### 11.2.1 多点コンストラクタの仕様

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

### 11.3 多点関数の対象一覧（12 個）

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

### 11.4 Topology 分類ルール

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

### 11.5 実装順序（段階的移行）

### 11.5.1 Phase 1: 設計固定（現在 = #466）

- [ ] MultiPoint Topology ルール確定（本セクション 11.4）
- [ ] 多点関数群の一覧表を公開（上記 11.3）
- [ ] 変換規約テストの雛形作成（下記参照）
- [ ] ドキュメント整備完了

**出力**: GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md へこれらを記載（#466 で完了）

### 11.5.2 Phase 2: Ellipse3D 系実装（別 Issue 予定）

1. 関数宣言を `Vec<Point3D<T>>` → `IntersectionResult<T>` に変更
2. 内部計算は変わらず、返却時に `IntersectionResult::from_option_points()` で変換
3. テスト追加: Crossing / Touching 両ケース確認
4. 逆向け委譲関数も同時更新

**対象**: 7 個関数（ellipse3d_*）

### 11.5.3 Phase 3: EllipsoidalSolid3D + SphericalSurface 実装

同じプロセスで残り 5 個関数を置換。

### 11.6 変換規約テストの例

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

### 11.7 注意点

1. **Topology は形状ペア全体の関係** → 複数点でも単一分類
2. **空返却は必ず Disjoint** → 自動処理（`from_option_points()` 利用）
3. **is_tangent は局所判定 + 統一** → 2 点以上ある場合、全て同じ値
4. **逆向け委譲** → `{shape_b}_{shape_a}_intersections()` は `{shape_a}_{shape_b}_intersections()` へ委譲
