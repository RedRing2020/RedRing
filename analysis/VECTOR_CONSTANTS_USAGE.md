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

### Vector4（.NET 準拠）

```rust
use analysis::linalg::Vector4;

// 基本定数（f64型に固定）
let x_axis = Vector4::<f64>::X_AXIS; // (1.0, 0.0, 0.0, 0.0)
let y_axis = Vector4::<f64>::Y_AXIS; // (0.0, 1.0, 0.0, 0.0)
let z_axis = Vector4::<f64>::Z_AXIS; // (0.0, 0.0, 1.0, 0.0)
let w_axis = Vector4::<f64>::W_AXIS; // (0.0, 0.0, 0.0, 1.0)
let zero = Vector4::<f64>::ZERO;     // (0.0, 0.0, 0.0, 0.0)
let one = Vector4::<f64>::ONE;       // (1.0, 1.0, 1.0, 1.0)

// .NET準拠のエイリアス定数
let unit_x = Vector4::<f64>::UNIT_X; // X_AXISのエイリアス
let unit_y = Vector4::<f64>::UNIT_Y; // Y_AXISのエイリアス

// メソッド（任意のScalar型で使用可能）
let x_axis_f32 = Vector4::<f32>::x_axis(); // (1.0, 0.0, 0.0, 0.0)
let y_axis_f64 = Vector4::<f64>::y_axis(); // (0.0, 1.0, 0.0, 0.0)
let z_axis_f64 = Vector4::<f64>::z_axis(); // (0.0, 0.0, 1.0, 0.0)
let w_axis_f64 = Vector4::<f64>::w_axis(); // (0.0, 0.0, 0.0, 1.0)

// .NET準拠の特殊コンストラクタ
let color = Vector4::<f32>::from_rgba(1.0, 0.5, 0.25, 0.8); // RGBA色空間
let homogeneous_point = Vector4::<f64>::from_point(1.0, 2.0, 3.0); // 同次座標系の点（w=1）
let direction = Vector4::<f64>::from_direction(0.0, 1.0, 0.0); // 方向ベクトル（w=0）
let custom_w = Vector4::<f64>::from_xyz(1.0, 2.0, 3.0, 0.5); // カスタムw成分
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

### 4D ベクトルの実用例（.NET 準拠）

```rust
use analysis::linalg::Vector4;

// RGBA色空間での色表現
let red = Vector4::<f32>::from_rgba(1.0, 0.0, 0.0, 1.0);
let semi_transparent_blue = Vector4::<f32>::from_rgba(0.0, 0.0, 1.0, 0.5);

// 距離と演算
let v1 = Vector4::new(1.0, 2.0, 3.0, 4.0);
let v2 = Vector4::new(5.0, 6.0, 7.0, 8.0);
let distance = v1.distance(&v2);
let reflected = v1.reflect(&Vector4::Y_AXIS);

// 要素ごとの最小値/最大値
let min_components = Vector4::min_components(&v1, &v2);
let max_components = Vector4::max_components(&v1, &v2);

// 値のクランプ
let clamped = v1.clamp(&Vector4::ZERO, &Vector4::ONE);

// 同次座標系での3D変換（後方互換性）
let point_3d = Vector3::new(1.0, 2.0, 3.0);
let point_4d = Vector4::from_point(point_3d.x(), point_3d.y(), point_3d.z());
let direction_4d = Vector4::from_direction(0.0, 1.0, 0.0);

// 同次座標の検証
if point_4d.is_point() && point_4d.is_valid_homogeneous() {
    let euclidean = point_4d.to_euclidean().unwrap();
}
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
