# Foundation Implementation Pattern Comparison

## 現在の設計（問題がある）

```rust
// InfiniteLine2D の現在の実装
impl<T: Scalar> CoreFoundation<T> for InfiniteLine2D<T> { ... }
impl<T: Scalar> BasicContainment<T> for InfiniteLine2D<T> { ... }
impl<T: Scalar> BasicParametric<T> for InfiniteLine2D<T> { ... }
// → 設計意図が不明確、なぜこの組み合わせなのかわからない
```

## 提案A: マーカーInterface階層（推奨）

```rust
// 設計意図が明確
impl<T: Scalar> GeometryShape<T> for InfiniteLine2D<T> { ... }     // 基本形状
impl<T: Scalar> Shape2D<T> for InfiniteLine2D<T> {}               // 2D形状（マーカー）
impl<T: Scalar> CurveShape<T> for InfiniteLine2D<T> {}            // 曲線形状（マーカー）
impl<T: Scalar> ParametricShape<T> for InfiniteLine2D<T> {}       // パラメトリック（マーカー）

impl<T: Scalar> DataAccess<T> for InfiniteLine2D<T> { ... }       // レベル1：データアクセス
impl<T: Scalar> BasicMeasurement<T> for InfiniteLine2D<T> { ... } // レベル2：計量
impl<T: Scalar> BasicContainment<T> for InfiniteLine2D<T> { ... } // レベル3：包含
impl<T: Scalar> BasicParametric<T> for InfiniteLine2D<T> { ... }  // レベル4：パラメトリック

impl<T: Scalar> LinearFoundation<T> for InfiniteLine2D<T> { ... } // 特化：直線系
```

## 提案B: 統合Foundation（シンプル）

```rust
// Circle2D専用Foundation（EllipseArcCoreのような）
pub trait CircleFoundation<T: Scalar> {
    // 円に必要な全機能を統合
    fn center(&self) -> Point2D<T>;
    fn radius(&self) -> T;
    fn area(&self) -> T;
    fn contains_point(&self, point: &Point2D<T>) -> bool;
    // ...その他円専用メソッド
}

impl<T: Scalar> CircleFoundation<T> for Circle2D<T> { ... }
```

## 提案C: Trait Alias（Rust 1.82+で利用可能）

```rust
// 複数trait の組み合わせに名前をつける
trait Circle2DFoundation<T: Scalar> = 
    GeometryShape<T> + Shape2D<T> + SurfaceShape<T> + 
    DataAccess<T> + BasicMeasurement<T> + BasicContainment<T> + 
    CircularFoundation<T>;

impl<T: Scalar> Circle2DFoundation<T> for Circle2D<T> {}
```

## 各アプローチの比較

| アプローチ | 設計意図明確性 | 実装コスト | 拡張性 | 型安全性 |
|-----------|--------------|----------|--------|----------|
| 現在の設計 | ❌ 不明確 | ⭐⭐⭐ 中 | ⭐⭐ 低 | ⭐⭐⭐ 中 |
| マーカー階層 | ✅ 非常に明確 | ⭐⭐ 高 | ⭐⭐⭐ 高 | ⭐⭐⭐⭐ 高 |
| 統合Foundation | ⭐⭐⭐ 明確 | ⭐⭐⭐⭐ 低 | ⭐ 低 | ⭐⭐⭐ 中 |
| Trait Alias | ⭐⭐⭐⭐ 非常に明確 | ⭐⭐⭐⭐ 低 | ⭐⭐⭐ 高 | ⭐⭐⭐⭐ 高 |

## 推奨：マーカーInterface階層

### メリット
1. **設計意図が明確**: 各形状がどの「カテゴリ」に属するかが一目でわかる
2. **段階的実装**: レベル1→2→3→4 の順で機能追加可能
3. **型安全性**: コンパイル時に適切な組み合わせをチェック
4. **拡張性**: 新しいマーカーやレベルを後から追加可能

### 実装例での使い分け

```rust
// シンプルな形状：基本レベルのみ
Point2D: GeometryShape + Shape2D + DataAccess + BasicMeasurement

// 複雑な曲線：フルレベル
Circle2D: GeometryShape + Shape2D + SurfaceShape + ParametricShape +
          DataAccess + BasicMeasurement + BasicContainment + BasicParametric +
          CircularFoundation

// 直線系：線形特化
LineSegment2D: GeometryShape + Shape2D + CurveShape + ParametricShape +
               DataAccess + BasicMeasurement + BasicContainment + BasicParametric +
               LinearFoundation
```