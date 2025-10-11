# プリミティブ幾何型ジェネリック化の失敗分析

## 理想 vs 現実

### 理想的な設計
```rust
// ✅ 完全ジェネリック: Scalar + Angle<T> のみで完結
fn circle_area<T: Scalar>(radius: T) -> T {
    T::PI * radius * radius
}

fn rotate_point<T: Scalar>(point: Point<T>, angle: Angle<T>) -> Point<T> {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    Point::new(
        point.x() * cos_a - point.y() * sin_a,
        point.x() * sin_a + point.y() * cos_a
    )
}
```

### 現実の妥協
```rust
// ❌ 妥協1: 数値リテラル変換
let two = T::from_f64(2.0);

// ❌ 妥協2: f64特化実装
impl BBox<f64> {
    pub fn diagonal_length_f64(&self) -> f64 { /* ... */ }
}

// ❌ 妥協3: 複数の定数システム
game::PI        // f32固定
precision::PI   // f64固定
T::PI          // ジェネリック
```

## 失敗の根本原因

### 1. Rustの型システムの制約
- **数値リテラルの型推論限界**: `2.0` が `T` として推論されない
- **const ジェネリクス不足**: 定数の型パラメータ化が困難
- **トレイト境界の複雑化**: `where` 句が肥大化

### 2. 段階的実装による妥協の蓄積
```rust
// Phase 1: とりあえず f64 で動くものを作る
pub fn calculate(radius: f64) -> f64 { /* ... */ }

// Phase 2: ジェネリック化を試みる  
pub fn calculate<T: Scalar>(radius: T) -> T { 
    let pi = T::from_f64(std::f64::consts::PI);  // 妥協開始
    pi * radius * radius
}

// Phase 3: f64特化が必要になる
impl Circle<f64> {
    pub fn legacy_method(&self) -> f64 { /* ... */ }  // 妥協固定化
}
```

### 3. 外部ライブラリとの互換性問題
- **標準ライブラリ**: `std::f64::consts::PI` は f64固定
- **数学関数**: `sin()`, `cos()` の戻り値型
- **既存コード**: f64前提の API との連携

## 回帰戦略

### 短期: 既存妥協の整理
1. **f64特化実装の削除**: ジェネリック実装に統一
2. **数値リテラル変換の標準化**: `T::from_f64()` パターンの統一
3. **定数アクセスの一本化**: `T::PI` を優先使用

### 中期: Scalar トレイトの完全化
1. **定数メソッドの追加**: `T::TWO`, `T::HALF` など頻用定数
2. **型安全な数値変換**: より直感的な API
3. **複合演算の追加**: よく使用される計算式の直接サポート

### 長期: 理想設計への回帰
1. **完全ジェネリック実装**: f32/f64 固定コードの根絶
2. **`Angle<T>` 統一**: 角度関連の定数重複解消  
3. **`Scalar` + `Angle<T>` のみのシンプル設計**

## 実装指針

### 新規コード
```rust
// ✅ 推奨: 完全ジェネリック
fn new_feature<T: Scalar>(value: T) -> T {
    T::PI * value  // ジェネリック定数使用
}

// ❌ 非推奨: f64固定
fn new_feature_f64(value: f64) -> f64 {
    std::f64::consts::PI * value
}
```

### 既存コード改修
```rust
// Before: 妥協コード
let two = T::from_f64(2.0);
let half = value / two;

// After: Scalar拡張で改善
let half = value / T::TWO;  // 将来追加される定数
```

## 結論

**現在の重複は設計の理想と実装現実のギャップから生まれた妥協の産物**

- ✅ **理想は正しい**: `Scalar` + `Angle<T>` による統一設計
- ❌ **実装で妥協**: 段階的な型固定コードの蓄積
- 🎯 **回帰目標**: 完全ジェネリック実装による重複解消

重複定数の統合は、この理想回帰の第一歩として重要な意味を持ちます。