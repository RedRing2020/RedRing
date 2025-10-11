# 定数統合戦略

## 現在の重複状況

### 重複している定数

1. **基本数学定数**: π, e, τ が3箇所で定義
2. **角度定数**: 30°, 45°, 90° が2箇所で定義  
3. **許容誤差**: 幾何・角度許容誤差が2箇所で定義

### 重複の問題点

1. **保守性**: 同じ値を複数箇所で管理
2. **一貫性**: 値の不整合リスク
3. **可読性**: どこから取得すべきか不明確

## 統合戦略

### Phase 1: 役割分担の明確化

#### `analysis/consts.rs` (固定型定数)
- ✅ **保持**: f32/f64固定の高速アクセス用
- ✅ **用途**: パフォーマンス重視の計算
- ✅ **対象**: `game::PI`, `precision::PI` など

#### `analysis/numerics/constants.rs` (ジェネリック定数)
- ✅ **保持**: ジェネリック型パラメータ対応
- ✅ **用途**: 型安全なライブラリ設計
- ✅ **対象**: `MathConstants::golden_ratio<T>()` など

#### `analysis/Angle<T>` (角度専用)
- ✅ **保持**: 型安全な角度操作
- ✅ **用途**: 角度計算の型安全性確保
- ✅ **対象**: `Angle::deg_45()`, `Angle::right_angle()` など

### Phase 2: 重複排除ルール

#### 基本数学定数 (π, e, τ)
```rust
// ❌ 削除対象: numerics/constants.rs の重複
// pub fn euler_number<T: Scalar>() -> T

// ✅ 保持: consts.rs + Scalar トレイト
T::PI, T::E, T::TAU  // ジェネリック用
game::PI, precision::PI  // 固定型用
```

#### 角度定数
```rust
// ✅ 保持: 用途別に使い分け
Angle::deg_45()           // 型安全な角度操作
game::ANGLE_45           // 高速f32計算
precision::ANGLE_45      // 高精度f64計算
```

#### 許容誤差
```rust
// ✅ 統合: ToleranceConstants をメイン実装に
impl ToleranceConstants {
    pub fn geometric<T: Scalar>() -> T {
        if TypeId::of::<T>() == TypeId::of::<f32>() {
            T::from_f64(1e-6 as f64)  // game 相当
        } else {
            T::from_f64(1e-10 as f64)  // precision 相当
        }
    }
}

// ✅ consts.rs は ToleranceConstants への転送
pub const GEOMETRIC_TOLERANCE: f64 = 1e-10; // 後方互換性
```

### Phase 3: 統合後の使い分け

#### 高速計算 (ゲーム・リアルタイム)
```rust
use analysis::consts::game;
let angle = 90.0 * game::DEG_TO_RAD;
```

#### 高精度計算 (CAD・科学技術計算)
```rust
use analysis::consts::precision;
let angle = 90.0 * precision::DEG_TO_RAD;
```

#### 型安全角度計算 (ライブラリ設計)
```rust
use analysis::Angle;
let angle = Angle::<f64>::right_angle();
```

#### ジェネリック数値計算 (汎用ライブラリ)
```rust
use analysis::numerics::MathConstants;
let phi = MathConstants::golden_ratio::<f64>();
```

## 利点

1. **明確な役割分担**: 用途に応じた使い分け
2. **後方互換性**: 既存コードへの影響最小化
3. **型安全性**: Angle<T> による角度の型安全管理
4. **パフォーマンス**: 固定型による高速アクセス

## 実装順序

1. **Phase 1**: 重複の調査・マッピング ✅
2. **Phase 2**: 不要な重複の削除
3. **Phase 3**: 統合された定数システムのテスト
4. **Phase 4**: ドキュメント更新と移行ガイド作成