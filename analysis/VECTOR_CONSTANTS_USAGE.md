# Vector 軸ベクトル定数の使用方法

analysis crate の linalg モジュールでは、Vector2、Vector3、Vector4 に軸ベクトルの定数とメソッドが追加されました。

## 利用可能な定数

### Vector2

```rust
use analysis::linalg::Vector2;

// 定数（f64型に固定）
let x_axis = Vector2::<f64>::X_AXIS; // (1.0, 0.0)
let y_axis = Vector2::<f64>::Y_AXIS; // (0.0, 1.0)
let zero = Vector2::<f64>::ZERO;     // (0.0, 0.0)

// メソッド（任意のScalar型で使用可能）
let x_axis_f32 = Vector2::<f32>::x_axis(); // (1.0, 0.0)
let y_axis_f64 = Vector2::<f64>::y_axis(); // (0.0, 1.0)
```

### Vector3

```rust
use analysis::linalg::Vector3;

// 定数（f64型に固定）
let x_axis = Vector3::<f64>::X_AXIS; // (1.0, 0.0, 0.0)
let y_axis = Vector3::<f64>::Y_AXIS; // (0.0, 1.0, 0.0)
let z_axis = Vector3::<f64>::Z_AXIS; // (0.0, 0.0, 1.0)
let zero = Vector3::<f64>::ZERO;     // (0.0, 0.0, 0.0)

// メソッド（任意のScalar型で使用可能）
let x_axis_f32 = Vector3::<f32>::x_axis(); // (1.0, 0.0, 0.0)
let y_axis_f64 = Vector3::<f64>::y_axis(); // (0.0, 1.0, 0.0)
let z_axis_f64 = Vector3::<f64>::z_axis(); // (0.0, 0.0, 1.0)
```

### Vector4

```rust
use analysis::linalg::Vector4;

// 定数（f64型に固定）
let x_axis = Vector4::<f64>::X_AXIS; // (1.0, 0.0, 0.0, 0.0)
let y_axis = Vector4::<f64>::Y_AXIS; // (0.0, 1.0, 0.0, 0.0)
let z_axis = Vector4::<f64>::Z_AXIS; // (0.0, 0.0, 1.0, 0.0)
let w_axis = Vector4::<f64>::W_AXIS; // (0.0, 0.0, 0.0, 1.0)
let zero = Vector4::<f64>::ZERO;     // (0.0, 0.0, 0.0, 0.0)

// メソッド（任意のScalar型で使用可能）
let x_axis_f32 = Vector4::<f32>::x_axis(); // (1.0, 0.0, 0.0, 0.0)
let y_axis_f64 = Vector4::<f64>::y_axis(); // (0.0, 1.0, 0.0, 0.0)
let z_axis_f64 = Vector4::<f64>::z_axis(); // (0.0, 0.0, 1.0, 0.0)
let w_axis_f64 = Vector4::<f64>::w_axis(); // (0.0, 0.0, 0.0, 1.0)
```

## 実用例

### numerical.rs での使用例

```rust
use analysis::linalg::Vector2;

// 直線フィッティングでのフォールバック値として使用
let direction = direction.normalize().unwrap_or(Vector2::x_axis());

// 垂直線の場合のデフォルト方向
return Ok((
    Point2D::from_f64(avg_x, avg_y),
    Vector2::y_axis()
));
```

### 3D 変換での使用例

```rust
use analysis::linalg::Vector3;

// カメラの初期向き
let forward = -Vector3::<f64>::Z_AXIS; // カメラは-Z方向を向く
let up = Vector3::<f64>::Y_AXIS;       // Y軸を上方向とする
let right = Vector3::<f64>::X_AXIS;    // X軸を右方向とする

// 回転軸として使用
let rotation_around_y = some_matrix.rotate_around_axis(&Vector3::y_axis(), angle);
```

### 4D 同次座標での使用例

```rust
use analysis::linalg::Vector4;

// 3D点を同次座標に変換
let point_3d = Vector3::new(1.0, 2.0, 3.0);
let point_4d = Vector4::from_point(point_3d.x(), point_3d.y(), point_3d.z());

// または定数から構築
let origin = Vector4::<f64>::ZERO + Vector4::<f64>::W_AXIS; // (0, 0, 0, 1)
```

## 軸ベクトルの利用方法

新しい軸ベクトル定数とメソッドは、直感的な名前と利便性を提供します。
メソッドは対応する定数のエイリアス（別名）として実装されており、メンテナンス性を向上させています。

```rust
// 定数（f64型、コンパイル時定数）
let x_const = Vector3::<f64>::X_AXIS;   // (1.0, 0.0, 0.0)

// メソッド（任意のScalar型で使用可能、定数のエイリアス）
let x_axis = Vector3::<f64>::x_axis();  // (1.0, 0.0, 0.0) - X_AXIS定数のエイリアス
let x_axis_f32 = Vector3::<f32>::x_axis(); // (1.0, 0.0, 0.0) f32版
```

### エイリアス設計の利点

- **単一責任**: 定数が真の値の定義となり、メソッドは型変換を提供
- **メンテナンス性**: 値の変更は定数のみで完結（実際には軸ベクトルの値は不変）
- **型安全性**: 定数は f64 固定、メソッドは任意の Scalar 型に対応
- **一貫性**: `zero()`, `x_axis()`, `y_axis()`等、すべて同じパターンで実装

定数は値が固定されており（f64）、メソッドは任意の Scalar 型で使用可能です。
