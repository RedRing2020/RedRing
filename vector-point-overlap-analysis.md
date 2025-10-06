# analysis と geo_primitives の Vector/Point 重複分析

## 📊 重複機能分析結果

### analysis Vector/Point の特徴

```rust
// geo_primitives/geometry3d/vector.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    x: f64, y: f64, z: f64,  // 直接f64型
}

// geo_primitives/geometry3d/point.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64, y: f64, z: f64,  // 直接f64型
}
```

**用途**: CAD 幾何形状、高レベル計算
**特徴**:

- ✅ Copy trait 実装（軽量）
- ✅ 直接的な f64 操作
- ✅ CAD 操作向け（translate, distance_to, between 等）
- ✅ 既存 Curve3D trait との統合

### analysis Vector/Point の特徴

```rust
// analysis/vector/vector3d.rs
#[derive(Debug, Clone)]  // Copy なし
pub struct Vector3D {
    components: [Scalar; 3],           // Scalar でラップ
    tolerance_context: ToleranceContext, // 許容誤差管理
}

// analysis/point3d.rs
#[derive(Debug, Clone)]  // Copy なし
pub struct Point3D {
    x: Scalar, y: Scalar, z: Scalar,  // Scalar 型
}
```

**用途**: 数値計算基盤、高精度演算
**特徴**:

- ✅ 直接的な f64 操作
- ✅ ToleranceContext 統合
- ✅ TolerantEq による許容誤差比較
- ✅ mm 単位での座標管理

## 🔍 機能重複状況

### ✅ 完全重複の基本機能

1. **座標アクセス**: `x()`, `y()`, `z()`
2. **基本演算**: `+`, `-`, `*` オペレータ
3. **距離計算**: `distance_to()`, `norm()`
4. **内積**: `dot()`
5. **外積**: `cross()` (Vector)

### 🔄 部分重複の機能

1. **正規化**: model は `normalize()` trait、geo_primitives は `Direction3D`
2. **ベクトル変換**: model は `to_vector()`、geo_primitives は異なる API
3. **コンストラクタ**: 異なる型システム（f64 vs Scalar）

### ⭐ geo_primitives 独自機能

1. **許容誤差処理**: `TolerantEq`, `ToleranceContext`
2. **高精度計算**: `Scalar` 型による数値安定性
3. **単位管理**: mm 単位の座標系統一
4. **堅牢性**: 数値誤差に対する堅牢な演算

### ⭐ model 独自機能

1. **CAD 特化操作**: `translate()`, `between()` 等
2. **trait 統合**: `Normed`, `Normalize`, `PointOps` trait
3. **軽量性**: Copy trait による効率的なメモリ使用
4. **業務ロジック**: 幾何学的意味論の抽象化

## 🎯 結論: **機能的重複だが目的が異なる**

### 重複の性質

- **基本機能**: 90% 重複（座標操作、基本演算）
- **設計思想**: 根本的に異なる
- **用途**: 階層的に補完的

### 役割分担の推奨

```
┌────────────────────┐
│   geo_primitives層 │ ← CAD業務ロジック、軽量操作
│  Vector/Point      │
└────────────────────┘
         │ 変換
         ▼
┌────────────────────┐
│  analysis層        │ ← 数値計算基盤、高精度演算
│  Vector3D/Point3D  │
└────────────────────┘
```

### 統合方針

1. **現状維持**: 両者の特性を活かし続ける
2. **アダプターパターン**: Phase 1 で実装済みの `TypeConverter` 活用
3. **段階的移行**: 高精度が必要な箇所で geo_core 優先使用
4. **共存**: 用途に応じた使い分け

## 📋 実装推奨事項

### 短期的対応

- ✅ 現在の `TypeConverter` で十分
- ✅ 両システムの共存を継続
- ✅ 用途別使い分けガイドラインの策定

### 長期的最適化

- 🔄 メモリ効率の改善（不要な Clone 削減）
- 🔄 変換コストの最小化
- 🔄 統一 API の検討（必要に応じて）

**結論**: 重複しているが、**相互補完的な設計**として価値があり、現在のアプローチが最適です。
