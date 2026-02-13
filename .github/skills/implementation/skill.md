# geo_primitives 実装パターン

## 最終更新日: 2026年2月13日

geo_primitives における幾何プリミティブの実装パターンと規約を定義します。

---

## ファイル構成

各幾何プリミティブは以下の構成で実装：

```
{shape}_3d.rs                      # 基本実装（構造体定義、基本メソッド）
{shape}_3d_foundation.rs           # Foundation トレイト実装
{shape}_3d_extensions.rs           # 基本操作・拡張機能
{shape}_3d_transform.rs            # 変換操作（BasicTransform）
{shape}_3d_transform_safe.rs       # 安全な変換操作（SafeTransform）
{shape}_3d_collision.rs            # 衝突判定・交差判定
{shape}_3d_intersection.rs         # 交差計算
{shape}_3d_tests.rs                # 基本機能テスト
{shape}_3d_transform_safe_tests.rs # SafeTransform テスト
```

---

## 責務分離の原則

### 1. 基本実装ファイル（`{shape}_3d.rs`）

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

### 3. 拡張機能ファイル（`{shape}_3d_extensions.rs`）

**責務**:
- 便利メソッド
- ユーティリティ関数
- 特殊な計算機能

### 4. Transform 実装ファイル（`{shape}_3d_transform.rs`）

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

### 原則

1. **実装ファイル内のテスト禁止**
   - 全てのテストは独立したテストファイルに分離

2. **テストファイルの分類**
   - `{shape}_3d_tests.rs`: 基本機能テスト
   - `{shape}_3d_transform_safe_tests.rs`: SafeTransform テスト
   - `{shape}_3d_collision_tests.rs`: 衝突判定テスト（オプション）

### テストファイル構成

```rust
// line_segment_3d_tests.rs
#[cfg(test)]
mod line_segment_3d_tests {
    use super::*;

    // Constructor テスト
    mod constructor_tests {
        #[test]
        fn test_new_valid() { /* ... */ }
        
        #[test]
        fn test_new_invalid() { /* ... */ }
    }

    // Properties テスト
    mod properties_tests {
        #[test]
        fn test_start_end() { /* ... */ }
        
        #[test]
        fn test_length() { /* ... */ }
    }

    // Measure テスト
    mod measure_tests {
        #[test]
        fn test_distance_to_point() { /* ... */ }
    }

    // Extension テスト
    mod extension_tests {
        #[test]
        fn test_primitive_kind() { /* ... */ }
    }

    // Transform テスト
    mod transform_tests {
        #[test]
        fn test_translate() { /* ... */ }
    }
}
```

---

## VSCode ネスティング設定

### 設定ファイル: `.vscode/settings.json`

新規ファイル追加時は必ず更新：

```jsonc
"explorer.fileNesting.patterns": {
    "{shape}_3d.rs": [
        "{shape}_3d_extensions.rs",
        "{shape}_3d_foundation.rs",
        "{shape}_3d_transform.rs",
        "{shape}_3d_transform_safe.rs",
        "{shape}_3d_collision.rs",
        "{shape}_3d_intersection.rs",
        "{shape}_3d_tests.rs",
        "{shape}_3d_transform_safe_tests.rs"
    ].join(",")
}
```

**例**:
```jsonc
"line_segment_3d.rs": "line_segment_3d_extensions.rs,line_segment_3d_foundation.rs,line_segment_3d_transform.rs,line_segment_3d_transform_safe.rs,line_segment_3d_collision.rs,line_segment_3d_intersection.rs,line_segment_3d_tests.rs,line_segment_3d_transform_safe_tests.rs"
```

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
- [ ] `{shape}_3d.rs` - 基本実装
- [ ] `{shape}_3d_foundation.rs` - Foundation 実装
- [ ] `{shape}_3d_extensions.rs` - 拡張機能
- [ ] `{shape}_3d_transform.rs` - Transform 実装
- [ ] `{shape}_3d_tests.rs` - テスト

### 2. lib.rs への登録
- [ ] `pub mod {shape}_3d;`
- [ ] `pub mod {shape}_3d_foundation;`
- [ ] `pub mod {shape}_3d_extensions;`
- [ ] `pub mod {shape}_3d_transform;`
- [ ] `pub mod {shape}_3d_tests;`
- [ ] `pub use {shape}_3d::{Shape}3D;`

### 3. VSCode 設定更新
- [ ] `.vscode/settings.json` にネスティング追加

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
