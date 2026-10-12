# Foundation パターン

## 最終更新日: 2026年10月12日

形状の API を、`geo_contracts` の trait定義（抽象化）と、`geo_primitives` / `geo_nurbs` の形状の実装に分け、値型を `geo_core` に置く構成を定義します。

---

## 全体の構成

```
geo_primitives（解析形状）/ geo_nurbs（NURBS 曲線・曲面）   形状の実装
        ↓ trait定義を実装
geo_contracts（trait定義の正本・抽象化。実装を持たない）
  geometry/core/{shape}_traits.rs   形状ごとの capability（生成・定義パラメータ・評価・距離 等）
  geometry/foundation/              形状に共通の capability（PrimitiveMetadata / Bounded）
  geometry/operations/              形状をまたぐ演算（距離・衝突・関係・相似変換）
        ↓ 値型を使う
geo_core（点・ベクトル・方向・相似変換・AABB。trait定義を持たず、操作は固有メソッド）
```

- trait定義は `geo_contracts` だけに置く。具体型のクレートは trait定義を再エクスポートせず、利用側が `geo_contracts` から直接使う
- 形状をまたぐ高レベルの計算は `geo_algorithms`、形状の型に依存しない共通の数値計算は `geo_commons` に置く

---

## 1. 形状ごとの capability（`geometry/core/{shape}_traits.rs`）

trait定義は、責務ごとに分ける。形状がその責務を持つ場合だけ定義する。

| capability | 責務 | 例 |
| --- | --- | --- |
| `{Shape}Constructor` | 生成 | `new` |
| `{Shape}Properties` | 定義パラメータの取得 | `start` / `end` / `center` / `radius` |
| `{Shape}Derived` | 定義パラメータから導く量 | `length` / `area` / `volume` |
| `{Shape}Evaluation` | パラメータに対応する点の評価 | `point_at_parameter` / `point_at_uv` |
| `{Shape}Distance` | 点との距離 | `distance_to_point` |
| `{Shape}Containment` | 点の包含・分類 | `contains_point` / `classify_point` |
| `{Shape}Projection` | 点の最近点 | `closest_point` |

- 曲線の `contains_point` は「曲線上にあるか」を表す。領域を持つ形状は `classify_point` で `PointClassification`（`Inside` / `OnBoundary` / `Outside`）を返し、`contains_point(p)` は `classify_point(p) != Outside` と一致させる
- 他の trait定義をまとめるだけの trait定義（`{Shape}Core` 等）は置かない。境界が必要な場合は構成要素の trait定義を並べる
- 引数・戻り値の点・ベクトルは、現状はタプル（`(T, T, T)`）で受け渡す

### 例: LineSegment3D

```rust
// geo_contracts/src/geometry/core/linesegment_traits.rs
pub trait LineSegment3DConstructor<T: Scalar> {
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self>
    where
        Self: Sized;
}

pub trait LineSegment3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}
```

```rust
// geo_primitives/src/line_segment_3d.rs
impl<T: Scalar> LineSegment3DDistance<T> for LineSegment3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        self.distance_to_point(&p)
    }
}
```

trait定義の実装は、型付きの固有メソッドへタプルを変換して委譲する。固有メソッドと trait定義のメソッドが同名で型が異なる場合、メソッド呼び出しでは固有メソッドが優先されることに注意する。

---

## 2. 形状に共通の capability（`geometry/foundation/`）

```rust
pub trait PrimitiveMetadata {
    fn primitive_kind(&self) -> PrimitiveKind;
}

pub trait Bounded<T: Scalar = f64>: PrimitiveMetadata {
    type Aabb; // geo_core::Aabb2D / Aabb3D
    fn aabb(&self) -> Option<Self::Aabb>;
}
```

- `PrimitiveMetadata` は全形状が実装する
- `Bounded` は、有界な形状が `bounds.rs` で実装する（無限直線などは `None` を返すか実装しない）

---

## 3. 形状をまたぐ演算（`geometry/operations/`）

| ファイル | trait定義 |
| --- | --- |
| `distance.rs` | `CrossDistance`（形状どうしの距離）、`DistanceConvergenceError` |
| `collision.rs` | `BasicCollision` / `PointDistance` / `BBoxCollision` |
| `relation.rs` | 平行・垂直・同一直線・角度等の関係（`PointClassification` は `geo_core` から再エクスポート） |
| `transform.rs` | 相似変換（`SimilarityTransformable2D/3D`）。`TransformError` は `geo_core` から再エクスポート |

### 相似変換

形状の変換は、回転・平行移動・一様スケールの組み合わせ（相似変換）に限る。

```rust
pub trait SimilarityTransformable3D<T: Scalar>: Sized {
    fn transform_similarity(&self, transform: &SimilarityTransform3D<T>) -> Result<Self, TransformError>;
}

pub enum TransformError {
    Unsupported(String),      // 扱わない変換・形状が受け付けない変換
    InvalidGeometry(String),  // 変換結果が形状として成立しない
    InvalidParameter(String), // 変換の構築に不正な引数が渡された
}
```

```rust
// geo_primitives/src/line_segment_3d/transform.rs
impl<T: Scalar> SimilarityTransformable3D<T> for LineSegment3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Ok(Self {
            line: self.line.transform_similarity(transform)?,
            start_param: self.start_param * scale,
            end_param: self.end_param * scale,
            start_point: transform.apply_point(self.start_point),
            end_point: transform.apply_point(self.end_point),
        })
    }
}
```

- 値型（点・ベクトル・方向）は trait定義を実装せず、相似変換の型の固有メソッド（`apply_point` / `apply_vector` / `apply_direction`）で変換する

---

## 4. API を追加するときの原則

- 基本の形状定義と最低限のメソッドから始め、呼び出し元が必要になった時点で追加する
- 呼び出し元のない API・別名・結果が誤った簡易実装を置かない
- 呼び出し元のない公開メソッド・trait定義のメソッドは `scripts/check_unused_public_api.ps1` が警告する。意図して残す場合は `scripts/unused_public_api_allowlist.txt` に理由とともに記載する
- 許容誤差は `analysis` に集約したものを使い（`default_distance_tolerance` 等）、モジュール内に独自の定数を置かない

詳細は `dev/architecture/GEO_API_ADDITION_RULES.md` と `dev/architecture/GEOMETRIC_TOLERANCE_USAGE_RULES.md` を参照。

---

## 実装チェックリスト

新規幾何プリミティブ実装時：

### geo_contracts（trait定義）

- [ ] `geo_contracts/src/geometry/core/{shape}_traits.rs`
  - [ ] `{Shape}Constructor<T>` / `{Shape}Properties<T>`
  - [ ] 呼び出し元が必要な capability（`{Shape}Evaluation<T>` / `{Shape}Distance<T>` 等）だけを追加する
- [ ] `geo_contracts/src/geometry/core.rs` と `geo_contracts/src/lib.rs` に再エクスポートを追加
- [ ] `PrimitiveKind` に形状の種類を追加

### geo_primitives（具体実装）

- [ ] `geo_primitives/src/{shape}_3d.rs`（親: 構造体定義・基本メソッド・trait定義の実装）
- [ ] `geo_primitives/src/{shape}_3d/bounds.rs`（`Bounded`）
- [ ] `geo_primitives/src/{shape}_3d/transform.rs`（`SimilarityTransformable3D`）
- [ ] `geo_primitives/src/{shape}_3d/tests.rs`（テスト）
- [ ] `geo_primitives/src/lib.rs` に `mod {shape}_3d;` と `pub use {shape}_3d::{Shape}3D;`

構成の詳細は `dev/architecture/GEO_PRIMITIVES_MODULE_STRUCTURE_RULES.md`、テストの置き場所は `dev/architecture/TEST_PLACEMENT_RULES.md` を参照。

---

## 参照文書

- **trait定義の構成**: `dev/architecture/GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md`
- **形状の意味（包含・距離・パラメータ）**: `dev/architecture/GEOMETRY_SHAPE_SEMANTICS_DESIGN.md`
- **モジュールの構成**: `dev/architecture/GEO_PRIMITIVES_MODULE_STRUCTURE_RULES.md`
- **API の追加**: `dev/architecture/GEO_API_ADDITION_RULES.md`
- **相似変換**: `dev/architecture/TRANSFORM_TRAIT_DESIGN.md`

---

## 使用例コメントの配置方針（共通）

- 実装ファイルでは責務の説明を優先し、詳細な使用例は `manual/` に集約する
- import を含む実践サンプルはコメントで長文化せず、`manual/<topic>_examples.md` の参照へ置き換える
- 参照先ドキュメントを追加した場合は `manual/SUMMARY.md` に登録する
- 詳細は `.github/skills/implementation/skill.md` の「使用例コメントの配置ルール（CI整合）」に従う
