# geo_primitives 実装パターン

## 最終更新日: 2026年10月12日

geo_primitives における幾何プリミティブの実装パターンと規約を定義します。

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
- 構造体定義
- 基本コンストラクタ（`new`, `from_*`）
- 内部ヘルパーメソッド

**例**:
```rust
// line_segment_3d.rs
pub struct LineSegment3D<T: Scalar> {
    start: Point3D<T>,
    end: Point3D<T>,
}

impl<T: Scalar> LineSegment3D<T> {
    pub fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        if start == end {
            None
        } else {
            Some(Self { start, end })
        }
    }
}
```

### 2. Foundation 実装ファイル（`{shape}_3d_foundation.rs`）

**責務**:
- Core Traits 実装（Constructor/Properties/Measure）
- Extension Traits 実装（Bounded, etc.）

**例**:
```rust
// line_segment_3d_foundation.rs
impl<T: Scalar> LineSegment3DConstructor<T> for LineSegment3D<T> {
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self> {
        // ...
    }
}

impl<T: Scalar> ExtensionFoundation<T> for LineSegment3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
}
```

### 3. 拡張機能ファイル（`{shape}_3d/extensions.rs`）

**責務**:
- 便利メソッド
- ユーティリティ関数
- 特殊な計算機能

### 4. Transform 実装ファイル（`{shape}_3d/transform.rs`）

**責務**:
- AnalysisTransform3D 実装
- 変換操作の具体実装

**例**:
```rust
// line_segment_3d_transform.rs
impl<T: Scalar> AnalysisTransform3D<T> for LineSegment3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn translate_analysis(
        &self,
        offset: (T, T, T),
    ) -> Result<Self::Output, TransformError> {
        // ...
    }
}
```

### 5. 衝突判定ファイル（`{shape}_3d_collision.rs`）

**責務**:
- BasicCollision トレイト実装
- 衝突検出専用トレイト実装

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

### TransformError の使用

SafeTransform実装では以下のエラー型を使用：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    ZeroVector,           // ゼロベクトル
    InvalidScaleFactor,   // 無効なスケール倍率（0や負数）
    InvalidRotation,      // 無効な回転パラメータ
    InvalidGeometry,      // 変換後の幾何的無効性
}
```

**使用例**:
```rust
impl<T: Scalar> SafeTransform<T> for LineSegment3D<T> {
    fn safe_scale(
        &self,
        factor: T,
    ) -> Result<Self, TransformError> {
        if factor <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor);
        }
        // 正常な処理
        Ok(/* ... */)
    }
}
```

---

## 型安全パターン

### Direction と Vector の分離

```rust
// geo_primitives/src/direction_3d.rs
pub struct Direction3D<T: Scalar>(Vector3D<T>);

impl<T: Scalar> Direction3D<T> {
    /// ゼロベクトルの場合は None を返す
    pub fn from_vector(v: Vector3D<T>) -> Option<Self> {
        let len = v.norm();
        if len.is_zero() {
            None
        } else {
            Some(Direction3D(v.normalize()))
        }
    }

    // アクセサメソッド（内部Vectorへの直接アクセスは禁止）
    pub fn x(&self) -> T { self.0.x() }
    pub fn y(&self) -> T { self.0.y() }
    pub fn z(&self) -> T { self.0.z() }
    
    // 内部Vectorの取得（必要な場合のみ）
    pub fn as_vector(&self) -> &Vector3D<T> { &self.0 }
}
```

**利点**:
- 正規化されたベクトルであることを型で保証
- ゼロベクトルの混入を防止
- API の明確化

---

## Option/Result による失敗の明示化

### Constructor での使用

```rust
impl<T: Scalar> LineSegment3D<T> {
    /// 無効な線分（開始点=終了点）の場合は None
    pub fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self> {
        if start == end {
            None
        } else {
            Some(Self { start, end })
        }
    }
}
```

### Transform での使用

```rust
impl<T: Scalar> AnalysisTransform3D<T> for LineSegment3D<T> {
    fn rotate_analysis(
        &self,
        axis: (T, T, T),
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError> {
        // 軸ベクトルの正規化
        let axis_dir = Direction3D::from_vector(
            Vector3D::new(axis.0, axis.1, axis.2)
        ).ok_or(TransformError::ZeroVector)?;
        
        // 変換実行
        Ok(/* ... */)
    }
}
```

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

### 3. 公開 API の確認
- [ ] 呼び出し元のない公開メソッドを追加していない（`scripts/check_unused_public_api.ps1`）

### 4. テスト実装
- [ ] Constructor テスト
- [ ] Properties テスト
- [ ] Measure テスト
- [ ] Extension テスト
- [ ] Transform テスト

### 5. 検証
- [ ] `cargo test -p geo_primitives`
- [ ] `cargo build`
- [ ] `.\scripts\check_architecture_dependencies_simple.ps1`

---

## 参照文書

- **Foundation Pattern**: `skills/foundation-pattern.md`
- **Phase 1 実装**: `dev/foundation/PHASE1_COMPLETION_REPORT.md`
- **Phase 2 実装**: `dev/foundation/PHASE2_IMPLEMENTATION_PLAN.md`
