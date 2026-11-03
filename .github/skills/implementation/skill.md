# geo_primitives 実装パターン

## 最終更新日: 2026年10月12日

geo_primitives における幾何プリミティブの実装パターンと規約を定義します。trait定義の構成は `.github/skills/foundation-pattern/skill.md` を参照してください。

---

## ファイル構成

各幾何プリミティブは、形状と同名の親ファイルと、同名のディレクトリの子モジュールで実装する（詳細は `dev/architecture/GEO_PRIMITIVES_MODULE_STRUCTURE_RULES.md`）。

```
{shape}_3d.rs          # 親: 構造体定義（private フィールド）・基本メソッド・trait定義の実装
{shape}_3d/
  extensions.rs        # 拡張機能（必要な場合のみ）
  bounds.rs            # 境界ボックス
  transform.rs         # 相似変換
  tests.rs             # テスト（#[cfg(test)]）
```

- `mod.rs` は使わない（Clippy の `mod_module_files` で確認する）
- `lib.rs` では形状のモジュールを private にし（`mod {shape}_3d;`）、型だけを再エクスポートする（`pub use {shape}_3d::{Shape}3D;`）
- 子モジュールは親の private なフィールドをそのまま使えるため、`pub(crate)` のフィールドや `*_internal` アクセサを置かない

---

## 責務分離の原則

### 1. 親ファイル（`{shape}_3d.rs`）

**責務**:
- 構造体定義（フィールドは private）
- 基本の生成メソッド（`new` 等）と、定義パラメータの取得
- `geo_contracts` の trait定義の実装（タプルを型付きの値に変換して固有メソッドへ委譲する）
- `PrimitiveMetadata` の実装

**例**:
```rust
// line_segment_3d.rs
pub struct LineSegment3D<T: Scalar> {
    line: InfiniteLine3D<T>,
    start_param: T,
    end_param: T,
    start_point: Point3D<T>,
    end_point: Point3D<T>,
}

impl<T: Scalar> LineSegment3D<T> {
    /// 始点と終点から線分を作成（同じ点の場合は None）
    pub fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        let line = InfiniteLine3D::from_two_points(start, end)?;
        Self::from_support_line_and_constraint_points(line, start, end)
    }
}
```

### 2. 拡張機能（`{shape}_3d/extensions.rs`）

**責務**:
- 親に置くほど基本的ではないが、呼び出し元のある固有メソッド
- 呼び出し元のないメソッドを先回りして置かない

### 3. 境界ボックス（`{shape}_3d/bounds.rs`）

**責務**:
- `Bounded` の実装（`geo_core::Aabb2D` / `Aabb3D` を返す）

### 4. 相似変換（`{shape}_3d/transform.rs`）

**責務**:
- `SimilarityTransformable2D` / `SimilarityTransformable3D` の実装
- 定義パラメータを変換し、長さはスケール係数倍、角度・重みは保持する

### 5. テスト（`{shape}_3d/tests.rs`）

**責務**:
- 形状のすべてのテスト（親・子のファイルにテストを置かない）

---

## テストコード配置ルール

テストの置き場所は `dev/architecture/TEST_PLACEMENT_RULES.md` に従う。形状のモジュールでは次のとおり。

1. **形状のテストは `{shape}_3d/tests.rs` にまとめる**
   - 親・子のファイル（`bounds.rs` / `extensions.rs` 等）にテストを置かない
   - `use super::*;` で親の要素を使い、`mod tests { ... }` で包まない
2. **形状をまたぐテストは結合テスト（`tests/cross_shape/`）に置く**
   - 公開 API だけを使う
3. **兄弟のテストファイル（`{shape}_3d_tests.rs`）と `#[path]` は使わない**

---

## 使用例コメントの配置ルール（CI整合）

### 目的

- ソースコード内コメントと CI ルールの乖離を防ぐ
- 実装ファイルを責務中心に保ち、使用例はドキュメントへ集約する

### 基本方針

1. **実装ソース内の使用例は最小限**
    - API の意図説明に必要な最小例のみ
    - 詳細な手順・複数パターンは `manual/` 側へ記載

2. **`use` を含む詳細サンプルは `manual/` に移す**
    - 特に import 規約に影響する例（`use geo_primitives::...` など）は、実装コメントへ残さない
    - 実装コメントには「詳細は manual を参照」と記載

3. **ドキュメント側の参照先を明示する**
    - 例: `manual/voxel_examples.md`
    - `manual/SUMMARY.md` に追加して導線を維持

4. **CI で禁止される記述はコメントでも避ける**
    - 実装ファイルでは、規約違反と誤認される import 記述を置かない

### 推奨テンプレート

実装ファイルのコメント例:

```rust
/// # Examples
///
/// 詳細な使用例は `manual/<topic>_examples.md` を参照してください。
```

### 追加チェック（変更時）

- 実装コメントに不要な `use ...` が残っていないか確認
- `manual` 側に同等以上の使用例があるか確認
- 目次（`manual/SUMMARY.md`）に追加されているか確認

---

---

## エラーハンドリングパターン

### 生成の失敗は `Option`

形状として成立しない入力（長さ 0 の線分、半径 0 以下の円など）は `None` を返す。

```rust
impl<T: Scalar> Circle3D<T> {
    pub fn new(center: Point3D<T>, axis: Direction3D<T>, radius: T) -> Option<Self> { /* ... */ }
}
```

### 変換の失敗は `TransformError`

相似変換は `Result<Self, TransformError>` を返す（`geo_contracts::TransformError`）。

| バリアント | 使う場面 |
| --- | --- |
| `Unsupported(String)` | 扱わない変換・形状が受け付けない変換（AABB の回転など） |
| `InvalidGeometry(String)` | 変換結果が形状として成立しない |
| `InvalidParameter(String)` | 変換の構築に不正な引数が渡された（0 以下のスケールなど） |

### 評価の範囲外は `Option`

パラメータの評価で、範囲から許容誤差を超えて外れた値や有限でない値は `None` を返す（例: `LineSegment3DEvaluation::point_at_parameter`）。値を黙って範囲内に丸めない。

---

## 型安全パターン

### Direction と Vector の分離

方向は `geo_core::Direction3D`（単位ベクトルであることを型で保証）、大きさを持つベクトルは `Vector3D` を使う。

```rust
// ゼロベクトルの場合は None
let axis = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0))?;
let v: Vector3D<f64> = axis.as_vector();
```

- 正規化済みであることを前提にする引数は `Direction3D` で受け取る
- 方向の成分は `x()` / `y()` / `z()` で取得する

### 許容誤差

- 許容誤差は `analysis` に集約したものを使う（`default_distance_tolerance::<T>()` / `default_kernel_numerical_zero_tolerance::<T>()` 等、`geo_contracts` から再エクスポート）
- モジュール内に独自の許容誤差の定数を置かない（`dev/architecture/GEOMETRIC_TOLERANCE_USAGE_RULES.md`）

---

## 実装チェックリスト

新規幾何プリミティブ実装時：

### 1. ファイル作成
- [ ] `{shape}_3d.rs` - 親（構造体定義・基本メソッド・trait定義の実装）
- [ ] `{shape}_3d/bounds.rs` - 境界ボックス
- [ ] `{shape}_3d/transform.rs` - 相似変換
- [ ] `{shape}_3d/tests.rs` - テスト

### 2. lib.rs への登録
- [ ] `mod {shape}_3d;`（private）
- [ ] `pub use {shape}_3d::{Shape}3D;`

### 3. テスト実装
- [ ] 生成（成立しない入力で `None` になること）
- [ ] 定義パラメータの取得・派生量
- [ ] 評価・距離・包含（境界上の扱いを含む）
- [ ] 相似変換（変換後の点が点の変換と一致すること）

### 4. 検証
- [ ] `pwsh scripts/check_all_before_push.ps1`（fmt / clippy / テスト / 呼び出し元のない公開メソッドの警告）

---

## 参照文書

- **trait定義の構成**: `.github/skills/foundation-pattern/skill.md`
- **モジュールの構成**: `dev/architecture/GEO_PRIMITIVES_MODULE_STRUCTURE_RULES.md`
- **テストの置き場所**: `dev/architecture/TEST_PLACEMENT_RULES.md`
- **API の追加**: `dev/architecture/GEO_API_ADDITION_RULES.md`
- **許容誤差**: `dev/architecture/GEOMETRIC_TOLERANCE_USAGE_RULES.md`
