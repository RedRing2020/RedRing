# Spherical Shapes: 正確な距離計算の設計

**作成日**: 2025年12月21日  
**関連Issue**: #180  
**対象**: SphericalSurface3D, SphericalSolid3D

## 概要

球面（SphericalSurface3D）と球体（SphericalSolid3D）に対する、Ray3D、InfiniteLine3D、LineSegment3D との正確な距離計算を実装する。

## 背景

PR #179 で Spherical shapes の衝突判定・交差判定を実装したが、以下のメソッドが簡易実装（始点/通過点のみ）となっている：

```rust
// ❌ 現状: 簡易実装
impl<T: Scalar> BasicCollision<T, Ray3D<T>> for SphericalSurface3D<T> {
    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.distance_to(&ray.origin_internal())  // 始点のみ
    }
}
```

これにより、7つのテストがTODOコメントアウトされている。

## 数学的基礎

### 1. 球と無限直線の距離

球の中心を $C$、半径を $r$、直線上の点を $P$、方向ベクトルを $\vec{d}$ とする。

**アルゴリズム**:

1. 中心 $C$ から直線への垂線の足 $F$ を求める
   - パラメータ $t = \frac{(C - P) \cdot \vec{d}}{|\vec{d}|^2}$
   - $F = P + t \vec{d}$

2. 中心から垂線の足までの距離 $d_{CF} = |C - F|$

3. 距離を計算:
   - **SphericalSolid3D** (球体): 
     - $d_{CF} \leq r$ なら $0$ (直線が球内部を通過)
     - $d_{CF} > r$ なら $d_{CF} - r$ (外側)
   
   - **SphericalSurface3D** (球面):
     - $|d_{CF} - r|$ (表面までの最短距離)

### 2. 球と光線（Ray3D）の距離

光線は始点 $O$ と方向 $\vec{d}$ で定義され、$t \geq 0$ の範囲のみ。

**アルゴリズム**:

1. 無限直線として垂線の足のパラメータ $t$ を計算
2. $t < 0$ の場合: 始点 $O$ との距離を使用
3. $t \geq 0$ の場合: 無限直線と同じ処理

**エッジケース**:
- 球の中心が光線の後方にある場合（$t < 0$）
- 光線が球に接する場合

### 3. 球と線分（LineSegment3D）の距離

線分は始点 $S$、終点 $E$ で定義され、$0 \leq t \leq 1$ の範囲のみ。

**アルゴリズム**:

1. 無限直線として垂線の足のパラメータ $t$ を計算
2. $t < 0$: 始点 $S$ との距離
3. $t > 1$: 終点 $E$ との距離
4. $0 \leq t \leq 1$: 無限直線と同じ処理

**エッジケース**:
- 線分が球内部を完全に通過する場合
- 線分の端点のみが球に近い場合

## 実装設計

### アーキテクチャ（ハイブリッドアプローチ）

```
model/geo_commons/src/metrics/
└── distance.rs (修正)                  # タプルベースのコアアルゴリズム

model/geo_primitives/src/
├── sphere_distance_helpers.rs (新規)  # 型安全なラッパー
├── spherical_surface_3d_collision.rs # 修正
└── spherical_solid_3d_collision.rs   # 修正
```

**責務分離**:
- **geo_commons**: 型に依存しないアルゴリズム実装（再利用性最大化）
- **geo_primitives**: 型安全な API 提供（可読性・保守性）

### 1. コアアルゴリズム (`geo_commons/src/metrics/distance.rs`)

タプルベースの型独立実装：

```rust
/// 球の中心から無限直線までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標 (x, y, z)
/// * `radius` - 球の半径
/// * `line_point` - 直線上の任意の点 (x, y, z)
/// * `line_direction` - 直線の方向ベクトル (dx, dy, dz) ※正規化不要
/// * `is_solid` - true なら球体（内部含む）、false なら球面（表面のみ）
///
/// # Returns
///
/// 球（面または体）から直線までの最短距離
///
/// # Algorithm
///
/// 1. 直線上の最近点のパラメータ t を計算: `t = (C - P) · d / |d|²`
/// 2. 最近点 F = P + t·d を求める
/// 3. 中心から最近点までの距離 d_CF を計算
/// 4. 球体の場合: d_CF ≤ r なら 0、さもなくば d_CF - r
/// 5. 球面の場合: |d_CF - r|
pub fn sphere_to_infinite_line_distance<T: Scalar>(
    center: (T, T, T),
    radius: T,
    line_point: (T, T, T),
    line_direction: (T, T, T),
    is_solid: bool,
) -> T {
    let (cx, cy, cz) = center;
    let (px, py, pz) = line_point;
    let (dx, dy, dz) = line_direction;
    
    // ベクトル to_center = center - line_point
    let to_cx = cx - px;
    let to_cy = cy - py;
    let to_cz = cz - pz;
    
    // direction の内積
    let dir_dot = dx * dx + dy * dy + dz * dz;
    
    // パラメータ t = to_center · direction / |direction|²
    let t = (to_cx * dx + to_cy * dy + to_cz * dz) / dir_dot;
    
    // 最近点 = line_point + t * direction
    let closest_x = px + t * dx;
    let closest_y = py + t * dy;
    let closest_z = pz + t * dz;
    
    // 中心から最近点までの距離
    let diff_x = cx - closest_x;
    let diff_y = cy - closest_y;
    let diff_z = cz - closest_z;
    let distance_from_center = (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt();
    
    if is_solid {
        // 球体: 内部なら0、外側なら表面までの距離
        if distance_from_center <= radius {
            T::ZERO
        } else {
            distance_from_center - radius
        }
    } else {
        // 球面: 表面までの最短距離
        (distance_from_center - radius).abs()
    }
}

/// 球の中心から光線（Ray）までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標 (x, y, z)
/// * `radius` - 球の半径
/// * `ray_origin` - 光線の始点 (x, y, z)
/// * `ray_direction` - 光線の方向ベクトル (dx, dy, dz) ※正規化不要
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から光線までの最短距離
///
/// # Algorithm
///
/// 1. 無限直線として最近点のパラメータ t を計算
/// 2. t < 0 の場合: 光線の始点との距離を使用
/// 3. t ≥ 0 の場合: 無限直線と同じ処理
pub fn sphere_to_ray_distance<T: Scalar>(
    center: (T, T, T),
    radius: T,
    ray_origin: (T, T, T),
    ray_direction: (T, T, T),
    is_solid: bool,
) -> T {
    let (cx, cy, cz) = center;
    let (ox, oy, oz) = ray_origin;
    let (dx, dy, dz) = ray_direction;
    
    // ベクトル to_center = center - origin
    let to_cx = cx - ox;
    let to_cy = cy - oy;
    let to_cz = cz - oz;
    
    // direction の内積
    let dir_dot = dx * dx + dy * dy + dz * dz;
    
    // パラメータ t
    let t = (to_cx * dx + to_cy * dy + to_cz * dz) / dir_dot;
    
    if t < T::ZERO {
        // 光線の始点が最近点
        let dist_to_origin = (to_cx * to_cx + to_cy * to_cy + to_cz * to_cz).sqrt();
        
        if is_solid {
            if dist_to_origin <= radius {
                T::ZERO
            } else {
                dist_to_origin - radius
            }
        } else {
            (dist_to_origin - radius).abs()
        }
    } else {
        // t ≥ 0: 無限直線と同じ処理
        let closest_x = ox + t * dx;
        let closest_y = oy + t * dy;
        let closest_z = oz + t * dz;
        
        let diff_x = cx - closest_x;
        let diff_y = cy - closest_y;
        let diff_z = cz - closest_z;
        let distance_from_center = (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt();
        
        if is_solid {
            if distance_from_center <= radius {
                T::ZERO
            } else {
                distance_from_center - radius
            }
        } else {
            (distance_from_center - radius).abs()
        }
    }
}

/// 球の中心から線分までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標 (x, y, z)
/// * `radius` - 球の半径
/// * `segment_start` - 線分の始点 (x, y, z)
/// * `segment_end` - 線分の終点 (x, y, z)
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から線分までの最短距離
///
/// # Algorithm
///
/// 1. 線分の方向ベクトル direction = end - start を計算
/// 2. 無限直線として最近点のパラメータ t を計算
/// 3. t < 0: 始点との距離
/// 4. t > 1: 終点との距離
/// 5. 0 ≤ t ≤ 1: 線分上の点との距離
pub fn sphere_to_line_segment_distance<T: Scalar>(
    center: (T, T, T),
    radius: T,
    segment_start: (T, T, T),
    segment_end: (T, T, T),
    is_solid: bool,
) -> T {
    let (cx, cy, cz) = center;
    let (sx, sy, sz) = segment_start;
    let (ex, ey, ez) = segment_end;
    
    // 線分の方向ベクトル
    let dx = ex - sx;
    let dy = ey - sy;
    let dz = ez - sz;
    
    // ベクトル to_center = center - start
    let to_cx = cx - sx;
    let to_cy = cy - sy;
    let to_cz = cz - sz;
    
    // direction の内積
    let dir_dot = dx * dx + dy * dy + dz * dz;
    
    // パラメータ t
    let t = (to_cx * dx + to_cy * dy + to_cz * dz) / dir_dot;
    
    // 最近点の座標を求める
    let (nearest_x, nearest_y, nearest_z) = if t < T::ZERO {
        // 始点が最近点
        segment_start
    } else if t > T::ONE {
        // 終点が最近点
        segment_end
    } else {
        // 線分上の点が最近点
        (sx + t * dx, sy + t * dy, sz + t * dz)
    };
    
    // 中心から最近点までの距離
    let diff_x = cx - nearest_x;
    let diff_y = cy - nearest_y;
    let diff_z = cz - nearest_z;
    let distance_from_center = (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt();
    
    if is_solid {
        if distance_from_center <= radius {
            T::ZERO
        } else {
            distance_from_center - radius
        }
    } else {
        (distance_from_center - radius).abs()
    }
}
```

### 2. 型安全ラッパー (`geo_primitives/src/sphere_distance_helpers.rs`)

geo_commons のアルゴリズムを geo_primitives の型で利用：

```rust
//! 球と直線系図形の距離計算ヘルパー（型安全ラッパー）

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D};
use geo_commons::metrics::{
    sphere_to_infinite_line_distance as commons_infinite_line,
    sphere_to_line_segment_distance as commons_line_segment,
    sphere_to_ray_distance as commons_ray,
};
use geo_foundation::{InfiniteLine3DProperties, Scalar};

/// 球と無限直線の距離（型安全版）
pub(crate) fn sphere_to_infinite_line_distance<T: Scalar>(
    center: Point3D<T>,
    radius: T,
    line: &InfiniteLine3D<T>,
    is_solid: bool,
) -> T {
    let (px, py, pz) = line.point();
    let (dx, dy, dz) = line.direction();
    
    commons_infinite_line(
        (center.x(), center.y(), center.z()),
        radius,
        (px, py, pz),
        (dx, dy, dz),
        is_solid,
    )
}

/// 球と光線の距離（型安全版）
pub(crate) fn sphere_to_ray_distance<T: Scalar>(
    center: Point3D<T>,
    radius: T,
    ray: &Ray3D<T>,
    is_solid: bool,
) -> T {
    let origin = ray.origin();
    let direction = ray.direction_vector();
    
    commons_ray(
        (center.x(), center.y(), center.z()),
        radius,
        (origin.x(), origin.y(), origin.z()),
        (direction.x(), direction.y(), direction.z()),
        is_solid,
    )
}

/// 球と線分の距離（型安全版）
pub(crate) fn sphere_to_line_segment_distance<T: Scalar>(
    center: Point3D<T>,
    radius: T,
    segment: &LineSegment3D<T>,
    is_solid: bool,
) -> T {
    let start = segment.start();
    let end = segment.end();
    
    commons_line_segment(
        (center.x(), center.y(), center.z()),
        radius,
        (start.x(), start.y(), start.z()),
        (end.x(), end.y(), end.z()),
        is_solid,
    )
}
```

```rust
// spherical_surface_3d_collision.rs

use crate::sphere_distance_helpers::*;

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for SphericalSurface3D<T> {
    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        sphere_to_infinite_line_distance(
            self.center_internal(),
            self.radius_internal(),
            line,
            false  // 球面（表面のみ）
        )
    }
}

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for SphericalSurface3D<T> {
    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        sphere_to_ray_distance(
            self.center_internal(),
            self.radius_internal(),
            ray,
            false  // 球面
        )
    }
}

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for SphericalSurface3D<T> {
    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        sphere_to_line_segment_distance(
            self.center_internal(),
            self.radius_internal(),
            segment,
            false  // 球面
        )
    }
}
```

### 3. SphericalSurface3D の修正

```rust
// spherical_surface_3d_collision.rs

use crate::sphere_distance_helpers::*;

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for SphericalSurface3D<T> {
    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        sphere_to_infinite_line_distance(
            self.center_internal(),
            self.radius_internal(),
            line,
            false  // 球面（表面のみ）
        )
    }
}

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for SphericalSurface3D<T> {
    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        sphere_to_ray_distance(
            self.center_internal(),
            self.radius_internal(),
            ray,
            false  // 球面
        )
    }
}

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for SphericalSurface3D<T> {
    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        sphere_to_line_segment_distance(
            self.center_internal(),
            self.radius_internal(),
            segment,
            false  // 球面
        )
    }
}
```

### 4. SphericalSolid3D の修正

```rust
// spherical_solid_3d_collision.rs

use crate::sphere_distance_helpers::*;

// InfiniteLine3D は既存実装を維持するか、ヘルパーに統一するか選択
// オプション1: 既存実装を維持（変更なし）
// オプション2: ヘルパーに統一（推奨）

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for SphericalSolid3D<T> {
    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        sphere_to_ray_distance(
            self.center_internal(),
            self.radius_internal(),
            ray,
            true  // 球体（内部含む）
        )
    }
}

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for SphericalSolid3D<T> {
    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        sphere_to_line_segment_distance(
            self.center_internal(),
            self.radius_internal(),
            segment,
            true  // 球体
        )
    }
}
```

## テスト戦略

### ヘルパー関数の単体テスト

```rust
// sphere_distance_helpers.rs の #[cfg(test)] セクション

#[test]
fn test_sphere_to_infinite_line_intersecting() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let radius = 1.0;
    let line = InfiniteLine3D::new(
        Point3D::new(-2.0, 0.0, 0.0),
        Vector3D::new(1.0, 0.0, 0.0)
    ).unwrap();
    
    // 球体（内部含む）
    let dist_solid = sphere_to_infinite_line_distance(center, radius, &line, true);
    assert!(dist_solid.abs() < 1e-10);
    
    // 球面（表面のみ）
    let dist_surface = sphere_to_infinite_line_distance(center, radius, &line, false);
    assert!((dist_surface - 1.0).abs() < 1e-10);
}

#[test]
fn test_sphere_to_ray_behind_origin() {
    // 光線の始点が球の後方にある場合
    let center = Point3D::new(-2.0, 0.0, 0.0);
    let radius = 1.0;
    let ray = Ray3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::new(1.0, 0.0, 0.0)  // X軸正方向
    ).unwrap();
    
    let dist = sphere_to_ray_distance(center, radius, &ray, true);
    assert!((dist - 1.0).abs() < 1e-10);  // 始点との距離 - 半径
}

#[test]
fn test_sphere_to_line_segment_endpoint() {
    // 端点が最近点の場合
    let center = Point3D::new(0.0, 2.0, 0.0);
    let radius = 1.0;
    let segment = LineSegment3D::new(
        Point3D::new(-1.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0)
    ).unwrap();
    
    let dist = sphere_to_line_segment_distance(center, radius, &segment, true);
    // sqrt(1^2 + 2^2) - 1 = sqrt(5) - 1 ≈ 1.236
    assert!((dist - (5.0_f64.sqrt() - 1.0)).abs() < 1e-10);
}
```

### 既存テストの有効化

TODOコメントアウトされている7テストを有効化：

1. `spherical_surface_3d_collision_tests.rs`:
   - `test_line_segment_intersection`
   - `test_ray_intersection`
   - `test_infinite_line_intersection`

2. `spherical_solid_3d_collision_tests.rs`:
   - `test_collision_with_ray`

(その他のテストも確認)

## エッジケースの処理

### 1. ゼロベクトル

- `InfiniteLine3D::new()`, `Ray3D::new()` がゼロベクトルを拒否するため、ヘルパー関数では考慮不要

### 2. 数値精度

- `is_zero()` を使用した tolerance チェック
- 浮動小数点演算の誤差を考慮

### 3. 特殊ケース

- 球の中心が直線/光線/線分上にある場合
- 光線/線分が球を完全に通過する場合
- 接する場合（距離 = 0）

## モジュール宣言

```rust
// lib.rs に追加
mod sphere_distance_helpers;  // 内部モジュール（pub 不要）
```

## パフォーマンス考慮事項

- ヘルパー関数のインライン化: `#[inline]` を検討
- 不要な計算の削減: 既に計算済みの値を再利用
- ベクトル演算の最適化: 可能な限り in-place 演算

## 実装の検証

### 成功基準

- ✅ 7つのTODOテスト全て成功
- ✅ ヘルパー関数の単体テスト全て成功
- ✅ 既存テストが全て成功（regression チェック）
- ✅ cargo clippy で警告なし

### レビューポイント

1. アルゴリズムの正確性
2. エッジケースの網羅性
3. コードの可読性と保守性
4. パフォーマンス

## 次のステップ

1. ✅ この設計ドキュメントのレビュー
2. ⬜ ヘルパー関数の実装
3. ⬜ SphericalSurface3D の修正
4. ⬜ SphericalSolid3D の修正
5. ⬜ テスト実行・検証
6. ⬜ PR作成

## 備考

- **Plane3D との衝突/交差判定**: 別Issue化を推奨（円交差の計算が必要なため）
- **参考実装**: SphericalSolid3D の InfiniteLine3D 実装（PR #179）
