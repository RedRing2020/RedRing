# Helpers Migration Guide

## 背景

`geo_foundation/helpers.rs` がカオス状態になっていたため、責務を明確化しました。

## 変更内容

### 削除された機能

以下の再エクスポートを削除：

```rust
// ❌ 削除された（analysis から直接インポートしてください）
pub use analysis::numerics::{approx_equal, approx_equal_relative, is_near_zero};
pub use analysis::metrics::{point_distance, point_distance_squared, point_distance_2d, point_distance_3d};
pub use analysis::approximations::{ellipse_perimeter_ramanujan_i, ellipse_perimeter_ramanujan_ii};
pub use analysis::spatial::{bbox_2d_area, bbox_2d_perimeter, bbox_2d_diagonal};
pub use analysis::Angle;
```

### 移行方法

#### Before（削除前）
```rust
use geo_foundation::helpers::{approx_equal, point_distance, Angle};
```

#### After（移行後）
```rust
use analysis::{Angle, numerics::approx_equal, metrics::point_distance};
```

### 残存する機能

以下は `helpers.rs` に残存（geo_foundation固有）：

```rust
✅ normalize_parameter()     // パラメータ正規化
✅ parameter_in_range()      // パラメータ範囲チェック  
✅ lerp()                    // 線形補間
✅ inverse_lerp()            // 逆線形補間
✅ angle_to_parameter()      // 角度→パラメータ変換（新規）
✅ parameter_to_angle()      // パラメータ→角度変換（新規）
```

## 設計原則

### geo_foundation/helpers.rs
- ✅ geo_foundation固有のロジックのみ
- ✅ パラメトリック操作の支援
- ✅ 幾何学的変換の支援

### analysis クレート
- ✅ 汎用数値計算
- ✅ 角度操作（Angle<T>）
- ✅ 距離・面積計算
- ✅ 近似・空間計算

## 利点

1. **責務の明確化**: 各モジュールが単一責任
2. **依存関係の正常化**: 適切な階層構造
3. **保守性の向上**: 機能の所在が明確
4. **拡張性の向上**: geo_foundation固有の機能を安全に追加可能