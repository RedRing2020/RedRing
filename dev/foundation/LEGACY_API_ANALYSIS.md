# レガシーAPI影響範囲調査レポート

**作成日**: 2026年2月13日  
**最終更新**: 2026年2月13日  
**Issue**: #202 レガシーAPI問題解決 - Foundation Patternへの完全移行

## 📋 調査概要

10以上の形状で共存しているレガシーAPIとFoundation実装を調査し、Foundation Patternへの完全移行計画を策定する。

## 🎯 調査目的

- レガシーメソッド使用箇所のリストアップ
- Foundation対応トレイトの特定
- 移行優先度の決定
- 破壊的変更の影響範囲の把握

## 🔍 対象形状（詳細調査結果）

| 形状 | ファイル | パターン分類 | 優先度 |
|------|----------|------------|--------|
| Circle2D/3D | `circle_2d.rs`, `circle_3d.rs` | ✅ 正しいパターン | - |
| Ray3D | `ray_3d.rs` | ✅ 正しいパターン | - |
| Arc2D/3D | `arc_2d.rs`, `arc_3d.rs` | ❌ 問題パターン | 高 |
| LineSegment2D/3D | `line_segment_2d.rs`, `line_segment_3d.rs` | ❌ 問題パターン | 最高 |
| Ellipse2D/3D | `ellipse_2d.rs`, `ellipse_3d.rs` | 🔶 混合パターン | 高 |
| Plane3D | `plane_3d.rs` | 🔶 混合パターン | 中 |
| EllipseArc2D/3D | `ellipse_arc_2d.rs`, `ellipse_arc_3d.rs` | 🔍 要確認 | 高 |
| Direction2D/3D | `direction_2d.rs`, `direction_3d.rs` | 🔍 要確認 | 中 |
| Point2D/3D | (geo_core) | ✅ 移行済み (Issue #218) | - |
| Vector2D/3D/4D | (geo_core) | ✅ 移行済み (Issue #218) | - |
| Aabb2D/3D | (geo_core) | ✅ 移行済み | - |
| InfiniteLine2D/3D | `infinite_line_*d.rs` | 🔍 調査未実施 | 低 |

### パターン分類の説明

- **✅ 正しいパターン**: `_internal()`メソッド使用、レガシーメソッドなし
- **❌ 問題パターン**: レガシーメソッドをFoundation実装が呼び出している
- **🔶 混合パターン**: 直接フィールドアクセス + レガシーメソッド共存
- **🔍 要確認**: Foundation実装の詳細未確認

## 📊 詳細調査結果

### ✅ 正しいパターン（移行完了/準拠）

#### 1. Circle2D/Circle3D
- **状態**: **Foundation Pattern準拠完了**（リファレンス実装）
- **パターン**: ✅ 内部メソッド方式
- **内部メソッド**:
  - `center_internal() -> Point2D<T>` - pub(crate)
  - `radius_internal() -> T` - pub(crate)
  - `ref_direction_internal()` - pub(crate)
- **Foundation実装** ([circle_2d.rs:283](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\circle_2d.rs#L283)):
  ```rust
  impl<T: Scalar> Circle2DProperties<T> for Circle2D<T> {
      fn center(&self) -> (T, T) {
          let c = self.center_internal();  // 内部メソッド呼び出し
          (c.x(), c.y())
      }
      fn radius(&self) -> T { self.radius_internal() }
  }
  ```
- **重要**: レガシーメソッドは存在せず、Foundation実装が内部メソッドを呼び出す正しいパターン

#### 2. Ray3D
- **状態**: **Foundation Pattern準拠完了**
- **パターン**: ✅ 内部メソッド方式
- **内部メソッド** ([ray_3d.rs:78-87](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\ray_3d.rs#L78-L87)):
  - `origin_internal() -> Point3D<T>` - pub(crate)
  - `direction_internal() -> Direction3D<T>` - pub(crate)
- **レガシーメソッド** ([ray_3d.rs:73](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\ray_3d.rs#L73)):
  - `pub fn origin(&self) -> Point3D<T>` - **存在するが、Foundation非依存**
- **Foundation実装** ([ray_3d.rs:300-310](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\ray_3d.rs#L300-L310)):
  ```rust
  impl<T: Scalar> Ray3DProperties<T> for Ray3D<T> {
      fn origin(&self) -> Point3<T> {
          let origin = self.origin_internal();  // 内部メソッド呼び出し
          Point3::new(origin.x(), origin.y(), origin.z())
      }
      fn direction(&self) -> Vector3<T> {
          let direction = self.direction_vector();
          Vector3::new(direction.x(), direction.y(), direction.z())
      }
  }
  ```
- **評価**: Foundation実装は内部メソッドを使用しており正しい（レガシーメソッドは今後非推奨化可能）

#### 3. Point2D/Point3D, Vector2/3/4, Aabb2D/3D
- **状態**: Issue #218で移行完了（2026年2月13日）
- **カプセル化**: dataフィールドをprivate化
- **アクセサ**: `x()`, `y()`, `z()`, `w()`メソッド

---

### ❌ 問題パターン（レガシーメソッド依存）

#### 1. Arc2D/3D - **最大の問題**
- **状態**: **Foundation実装がレガシーメソッドを呼び出している**
- **パターン**: ❌ 循環依存パターン
- **レガシーメソッド** ([arc_2d.rs:93-99](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\arc_2d.rs#L93-L99)):
  ```rust
  pub fn center(&self) -> Point2D<T> { self.circle.center() }  // Circle2Dに委譲
  pub fn radius(&self) -> T { self.circle.radius() }
  ```
- **Foundation実装** ([arc_2d.rs:296](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\arc_2d.rs#L296)):
  ```rust
  impl<T: Scalar> Arc2DProperties<T> for Arc2D<T> {
      fn center(&self) -> (T, T) {
          let c = self.center();  // ❌ レガシーメソッドを呼び出し
          (c.x(), c.y())
      }
      fn radius(&self) -> T { self.radius() }  // ❌ レガシーメソッドを呼び出し
  }
  ```
- **問題点**:
  1. Foundation実装がレガシーメソッドに依存
  2. `self.circle.center()` は実際にはCircle2Dの**計算メソッド**を呼んでいる（Circle2Dには`pub fn center()`が存在しない）
  3. 内部メソッド方式への移行が必要
- **使用箇所**:
  - `arc_2d_tests.rs`: `arc.center()`, `arc.radius()` (複数)
  - `arc_2d_tests.rs`: `arc.arc_length()` (複数)
  - テストコードの大規模修正が必要

#### 2. LineSegment2D/3D - **最多使用箇所**
- **状態**: **Foundation実装がレガシーメソッドを呼び出している**
- **パターン**: ❌ 循環依存パターン（最優先修正対象）
- **レガシーメソッド** ([line_segment_3d.rs:69-75](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\line_segment_3d.rs#L69-L75)):
  ```rust
  pub fn start(&self) -> Point3D<T> {
      self.line.point_at_parameter(self.start_param)
  }
  pub fn end(&self) -> Point3D<T> {
      self.line.point_at_parameter(self.end_param)
  }
  ```
- **Foundation実装** ([line_segment_3d.rs:200-210](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\line_segment_3d.rs#L200-L210)):
  ```rust
  impl<T: Scalar> LineSegment3DProperties<T> for LineSegment3D<T> {
      fn start(&self) -> (T, T, T) {
          let p = self.start();  // ❌ レガシーメソッドを呼び出し
          (p.x(), p.y(), p.z())
      }
      fn end(&self) -> (T, T, T) {
          let p = self.end();    // ❌ レガシーメソッドを呼び出し
          (p.x(), p.y(), p.z())
      }
  }
  ```
- **問題点**:
  1. Foundation実装がレガシーメソッドに依存
  2. **20回以上の外部使用**があり、影響範囲が最大
- **使用箇所**（一部抜粋）:
  - `voxel.rs:335-339`: `segment.start().x()`, `segment.end().y()` 等
  - `circle_3d_collision.rs`: `segment.start()`, `segment.end()`
  - 複数のテストファイル
- **移行計画**: Tier 1（最優先）として段階的移行が必要

---

### 🔶 混合パターン（直接アクセス + レガシー）

#### 1. Ellipse2D/3D
- **状態**: **フィールド直接アクセス + レガシーメソッド共存**
- **パターン**: 🔶 フィールドアクセス方式（準Foundation準拠）
- **レガシーメソッド** ([ellipse_2d.rs:75-90](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\ellipse_2d.rs#L75-L90)):
  ```rust
  pub fn center(&self) -> Point2D<T> { self.center }         // フィールド返却
  pub fn semi_major(&self) -> T { self.semi_major }          // フィールド返却
  pub fn semi_minor(&self) -> T { self.semi_minor }
  pub fn rotation(&self) -> T { self.rotation }
  ```
- **Foundation実装** ([ellipse_2d.rs:505-520](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\ellipse_2d.rs#L505-L520)):
  ```rust
  impl<T: Scalar> Ellipse2DProperties<T> for Ellipse2D<T> {
      fn center(&self) -> (T, T) {
          (self.center.x(), self.center.y())  // ✅ フィールド直接アクセス
      }
      fn semi_major_axis(&self) -> T { self.semi_major }  // ✅ フィールド直接アクセス
      fn semi_minor_axis(&self) -> T { self.semi_minor }
      fn rotation(&self) -> T { self.rotation }
  }
  ```
- **評価**:
  - **Good**: Foundation実装はフィールドに直接アクセス（レガシーメソッド非依存）
  - **Bad**: レガシーメソッドが公開されたまま（非推奨化が必要）
- **移行計画**: レガシーメソッドを `#[deprecated]` → 削除

#### 2. Plane3D
- **状態**: **フィールド直接アクセス + レガシーメソッド共存**
- **パターン**: 🔶 フィールドアクセス方式
- **レガシーメソッド** ([plane_3d.rs:160-167](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\plane_3d.rs#L160-L167)):
  ```rust
  pub fn origin(&self) -> Point3D<T> { self.origin }  // フィールド返却
  pub fn normal(&self) -> Direction3D<T> { self.normal }
  ```
- **Foundation実装** ([plane_3d.rs:352-370](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\plane_3d.rs#L352-L370)):
  ```rust
  impl<T: Scalar> Plane3DProperties<T> for Plane3D<T> {
      fn origin(&self) -> (T, T, T) {
          (self.origin.x(), self.origin.y(), self.origin.z())  // ✅ フィールド直接アクセス
      }
      fn normal(&self) -> (T, T, T) {
          (self.normal.x(), self.normal.y(), self.normal.z())
      }
  }
  ```
- **評価**: Ellipse2D同様、Foundation準拠だがレガシーメソッド共存
- **使用頻度**: 低（`plane_3d.rs:160`に1箇所のみ）

---

### 🔍 調査未完了（要確認）

#### 1. EllipseArc2D/3D
- **レガシーメソッド発見** ([ellipse_arc_2d.rs](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\ellipse_arc_2d.rs)):
  - L72: `pub fn center(&self) -> Point2D<T>`
  - L77: `pub fn semi_major(&self) -> T`
  - L82: `pub fn semi_minor(&self) -> T`
  - L133: `pub fn arc_length(&self) -> T`
- **Foundation実装**: 未確認
- **優先度**: 高（arc_length使用あり）

#### 2. Direction2D/3D
- **レガシーメソッド発見**:
  - [direction_2d.rs:78-83](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\direction_2d.rs#L78-L83): `pub fn x()`, `pub fn y()`
  - [direction_3d.rs:83-93](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\direction_3d.rs#L83-L93): `pub fn x()`, `pub fn y()`, `pub fn z()`
  - [direction_3d.rs:130](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\direction_3d.rs#L130): `pub fn normalize()`
- **Foundation実装** (確認済み):
  - `DirectionProperties` トレイト存在
  - [direction_2d.rs:176](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\direction_2d.rs#L176): `impl Direction2DProperties`
  - [direction_3d.rs:235](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\direction_3d.rs#L235): `impl Direction3DProperties`
- **詳細調査**: Foundation実装の内容確認が必要

#### 3. InfiniteLine2D/3D
- **調査状況**: 未着手
- **優先度**: 低

## 🎯 修正パターンの分類と優先度

### パターン1: ✅ 正しいパターン（修正不要）
**該当**: Circle2D, Ray3D, Point, Vector, Aabb

**特徴**:
- 内部メソッド (`_internal()`) を使用
- Foundation実装が内部メソッドを呼び出す
- レガシーメソッドが存在しないか、Foundation非依存

**リファレンス実装** (Circle2D):
```rust
// 内部メソッド（非公開）
pub(crate) fn center_internal(&self) -> Point2D<T> { self.center }

// Foundation実装
impl Circle2DProperties for Circle2D {
    fn center(&self) -> (T, T) {
        let c = self.center_internal();  // ✅ 内部メソッド呼び出し
        (c.x(), c.y())
    }
}
```

---

### パターン2: ❌ 循環依存パターン（最優先修正）
**該当**: Arc2D/3D, LineSegment2D/3D

**問題構造**:
```rust
// レガシーメソッド（公開）
pub fn center(&self) -> Point2D<T> { ... }

// Foundation実装
impl Arc2DProperties for Arc2D {
    fn center(&self) -> (T, T) {
        let c = self.center();  // ❌ レガシーメソッド呼び出し
        (c.x(), c.y())
    }
}
```

**修正方針**:
1. レガシーメソッド → `_internal()` に改名
2. Foundation実装 → 内部メソッド呼び出しに変更
3. 外部使用箇所 → Foundation トレイトに移行

**影響範囲**:
- **LineSegment2D/3D**: 20+ 箇所（最大規模）
- **Arc2D/3D**: 複数テストファイル

---

### パターン3: 🔶 フィールドアクセス方式（準Foundation準拠）
**該当**: Ellipse2D/3D, Plane3D

**現状**:
```rust
// レガシーメソッド（公開）- 存在するが使われていない
pub fn center(&self) -> Point2D<T> { self.center }

// Foundation実装
impl Ellipse2DProperties for Ellipse2D {
    fn center(&self) -> (T, T) {
        (self.center.x(), self.center.y())  // ✅ フィールド直接アクセス
    }
}
```

**修正方針**:
1. レガシーメソッドを `#[deprecated]` でマーク
2. 1バージョン後に削除
3. Foundation実装は変更不要

**影響範囲**: 小（レガシーメソッド使用が少ない）

---

### パターン4: 🔍 要調査パターン
**該当**: EllipseArc2D/3D, Direction2D/3D, InfiniteLine2D/3D

**調査事項**:
- Foundation実装の詳細確認
- レガシーメソッド使用頻度
- パターン1/2/3のどれに該当するか判定

---

## 📊 移行優先度マトリックス

| 優先度 | 形状 | パターン | 影響範囲 | 実装期間 |
|--------|------|----------|----------|----------|
| **Tier 0** (リファレンス) | Circle2D/3D, Ray3D | ✅ 正しい | - | - |
| **Tier 1** (最優先) | LineSegment2D/3D | ❌ 循環依存 | 20+ 箇所 | 2-3日 |
| **Tier 1** (最優先) | Arc2D/3D | ❌ 循環依存 | 複数ファイル | 1-2日 |
| **Tier 2** (高優先) | Ellipse2D/3D | 🔶 フィールド | 少 | 0.5日 |
| **Tier 2** (高優先) | Plane3D | 🔶 フィールド | 1箇所 | 0.3日 |
| **Tier 3** (要調査) | EllipseArc2D/3D | 🔍 未確認 | 不明 | 調査後決定 |
| **Tier 3** (要調査) | Direction2D/3D | 🔍 未確認 | 不明 | 調査後決定 |
| **Tier 4** (低優先) | InfiniteLine | 🔍 未着手 | 少 | 調査後決定 |

---

## 📈 実装戦略

### Phase 1: 調査完了（現在位置）
- [x] 主要形状の調査完了（Circle, Arc, LineSegment, Ellipse, Ray, Plane）
- [ ] EllipseArc, Direction, InfiniteLine の詳細確認（残り作業）

### Phase 2A: Tier 1 移行（循環依存解消）
**目標**: Foundation実装の独立性確保

#### Step 1: Arc2D/3D 修正（1-2日）
1. `center()` → `center_internal()` に改名
2. `radius()` → `radius_internal()` に改名
3. Foundation実装を内部メソッド呼び出しに変更
4. テストコード修正（Foundation トレイト使用）
5. `cargo test -p geo_primitives` で検証

#### Step 2: LineSegment2D/3D 修正（2-3日）
1. `start()` → `start_internal()` に改名
2. `end()` → `end_internal()` に改名
3. Foundation実装を内部メソッド呼び出しに変更
4. **20+ 箇所の外部使用**を Foundation トレイトに移行
5. 段階的コミット（形状ごと、ファイルグループごと）
6. `cargo test --workspace` で全体検証

### Phase 2B: Tier 2 移行（レガシー非推奨化）
#### Step 3: Ellipse2D/3D 修正（0.5日）
1. レガシーメソッドに `#[deprecated]` 追加
2. 非推奨警告の対応（使用箇所があれば修正）

#### Step 4: Plane3D 修正（0.3日）
1. レガシーメソッドに `#[deprecated]` 追加

### Phase 3: Tier 3-4 調査・移行
- EllipseArc, Direction, InfiniteLine の詳細調査
- パターン分類後、適切な修正方針決定

---

## 🔨 実装詳細例

### Arc2D 修正パターン

#### Before (現状):
```rust
// arc_2d.rs
pub fn center(&self) -> Point2D<T> {
    self.circle.center()  // Circle2Dの計算メソッド呼び出し
}

impl Arc2DProperties for Arc2D {
    fn center(&self) -> (T, T) {
        let c = self.center();  // ❌ レガシーメソッド依存
        (c.x(), c.y())
    }
}
```

#### After (修正後):
```rust
// arc_2d.rs
pub(crate) fn center_internal(&self) -> Point2D<T> {
    // Circle2D内部の計算ロジックを直接実装、または
    // self.circle の内部メソッド呼び出し
    Point2D::new(self.circle.center_x(), self.circle.center_y())
}

impl Arc2DProperties for Arc2D {
    fn center(&self) -> (T, T) {
        let c = self.center_internal();  // ✅ 内部メソッド呼び出し
        (c.x(), c.y())
    }
}
```

### LineSegment3D 修正パターン

#### Before (現状):
```rust
// line_segment_3d.rs
pub fn start(&self) -> Point3D<T> {
    self.line.point_at_parameter(self.start_param)
}

impl LineSegment3DProperties for LineSegment3D {
    fn start(&self) -> (T, T, T) {
        let p = self.start();  // ❌ レガシーメソッド依存
        (p.x(), p.y(), p.z())
    }
}

// 外部使用（voxel.rs等）
let sx = segment.start().x();  // Point3D型取得 → x()呼び出し
```

#### After (修正後):
```rust
// line_segment_3d.rs
pub(crate) fn start_internal(&self) -> Point3D<T> {
    self.line.point_at_parameter(self.start_param)
}

impl LineSegment3DProperties for LineSegment3D {
    fn start(&self) -> (T, T, T) {
        let p = self.start_internal();  // ✅ 内部メソッド呼び出し
        (p.x(), p.y(), p.z())
    }
}

// 外部使用（voxel.rs等）- Foundation トレイト使用
use geo_foundation::LineSegment3DProperties;
let (sx, sy, sz) = segment.start();  // タプル取得
// または
let sx = segment.start().0;  // x座標直接取得
```

---

## 📈 使用頻度分析（詳細版）

### 高頻度メソッド（20回以上使用）
| メソッド | 形状 | 使用箇所 | 影響範囲 |
|---------|------|---------|---------|
| `start()` / `end()` | LineSegment2D/3D | 20+ 箇所 | voxel.rs, collision系, tests |

### 中頻度メソッド（5-20回使用）
| メソッド | 形状 | 使用箇所 | 影響範囲 |
|---------|------|---------|---------|
| `center()` | Arc2D | arc_2d_tests.rs (複数) | テストのみ |
| `radius()` | Arc2D | arc_2d_tests.rs (複数) | テストのみ |
| `arc_length()` | Arc/EllipseArc | tests, examples | 限定的 |

### 低頻度メソッド（5回未満）
| メソッド | 形状 | 使用箇所 | 影響範囲 |
|---------|------|---------|---------|
| `origin()` | Plane3D | plane_3d.rs (1箇所) | 極小 |
| `origin()` | Ray3D | 要調査 | 不明 |
| `center()` | Ellipse2D | 要調査 | 不明 |

## 🚧 予想される破壊的変更

### 1. メソッド名の変更（Tier 1: 循環依存パターン）

#### Arc2D/3D
```rust
// ❌ Before (削除)
pub fn center(&self) -> Point2D<T>
pub fn radius(&self) -> T

// ✅ After (内部メソッド化)
pub(crate) fn center_internal(&self) -> Point2D<T>
pub(crate) fn radius_internal(&self) -> T
```

#### LineSegment2D/3D
```rust
// ❌ Before (削除)
pub fn start(&self) -> Point3D<T>
pub fn end(&self) -> Point3D<T>

// ✅ After (内部メソッド化)
pub(crate) fn start_internal(&self) -> Point3D<T>
pub(crate) fn end_internal(&self) -> Point3D<T>
```

---

### 2. レガシーメソッドの非推奨化（Tier 2: フィールドアクセス方式）

#### Ellipse2D/3D
```rust
// ⚠️ 非推奨マーク（削除予定）
#[deprecated(since = "0.x.0", note = "Use Ellipse2DProperties::center() instead")]
pub fn center(&self) -> Point2D<T> { self.center }

#[deprecated(since = "0.x.0", note = "Use Ellipse2DProperties::semi_major_axis() instead")]
pub fn semi_major(&self) -> T { self.semi_major }
```

#### Plane3D
```rust
// ⚠️ 非推奨マーク（削除予定）
#[deprecated(since = "0.x.0", note = "Use Plane3DProperties::origin() instead")]
pub fn origin(&self) -> Point3D<T> { self.origin }
```

---

### 3. 外部コードの修正パターン

#### パターンA: 単純なアクセサ置換（Arc2D例）

```rust
// ❌ Before
use geo_primitives::Arc2D;
let arc = Arc2D::new(...);
let center = arc.center();  // Point2D<f64>
let x = center.x();

// ✅ After
use geo_primitives::Arc2D;
use geo_foundation::Arc2DProperties;  // トレイトインポート必須
let arc = Arc2D::new(...);
let (x, y) = arc.center();  // タプル (f64, f64)
```

#### パターンB: 複雑なチェーン呼び出し（LineSegment3D例）

```rust
// ❌ Before (voxel.rs等)
let sx = segment.start().x();
let sy = segment.start().y();
let sz = segment.start().z();

// ✅ After - Option 1: タプル分解
use geo_foundation::LineSegment3DProperties;
let (sx, sy, sz) = segment.start();

// ✅ After - Option 2: 個別アクセス
let sx = segment.start().0;
let sy = segment.start().1;
let sz = segment.start().2;
```

#### パターンC: フィールド保持が必要な場合

```rust
// ❌ Before
let start_point = segment.start();  // Point3D<T>
some_function(start_point);         // Point3D型を引数に

// ✅ After - Point3D再構築
use geo_foundation::LineSegment3DProperties;
let (x, y, z) = segment.start();
let start_point = Point3D::new(x, y, z);
some_function(start_point);

// 🔧 Better - 関数側をFoundation対応に変更
fn some_function<T: Scalar, S: LineSegment3DProperties<T>>(seg: &S) {
    let (x, y, z) = seg.start();
    // ...
}
```

---

## 📝 移行ガイドライン

### 開発者向け修正手順

#### Tier 1 修正（Arc, LineSegment）

1. **内部メソッド作成**
   ```rust
   // 元のレガシーメソッドを pub(crate) で残す
   pub(crate) fn center_internal(&self) -> Point2D<T> {
       self.circle.center()  // 既存ロジック
   }
   ```

2. **Foundation実装修正**
   ```rust
   impl Arc2DProperties for Arc2D {
       fn center(&self) -> (T, T) {
           let c = self.center_internal();  // 内部メソッド呼び出し
           (c.x(), c.y())
       }
   }
   ```

3. **レガシーメソッド削除**
   ```rust
   // 以下を削除
   // pub fn center(&self) -> Point2D<T> { ... }
   ```

4. **テストコード修正**
   ```rust
   use geo_foundation::Arc2DProperties;
   let (cx, cy) = arc.center();  // タプル受け取り
   assert_eq!(cx, 5.0);
   ```

#### Tier 2 修正（Ellipse, Plane）

1. **非推奨マーク追加**
   ```rust
   #[deprecated(since = "0.x.0", note = "Use Ellipse2DProperties::center()")]
   pub fn center(&self) -> Point2D<T> { self.center }
   ```

2. **コンパイラ警告確認**
   ```bash
   cargo build --workspace 2>&1 | grep "warning: use of deprecated"
   ```

3. **警告箇所を修正** → Foundation トレイト使用に変更

4. **次バージョンで削除**

---

## 📊 進捗管理

### Phase 1: 影響範囲調査 ✅ 完了
- [x] 主要形状調査（Circle, Arc, LineSegment, Ellipse, Ray, Plane）
- [x] パターン分類（正しい/循環依存/フィールドアクセス/要調査）
- [x] 使用頻度分析
- [ ] 残り形状調査（EllipseArc, Direction, InfiniteLine）
- [ ] 調査レポート最終版作成

**期間**: 2026/02/13（0.5日目標 → 0.7日実績見込み）

### Phase 2: Tier 1 移行実装
- [ ] Arc2D/3D 修正（見積: 1-2日）
  - [ ] 内部メソッド作成
  - [ ] Foundation実装修正
  - [ ] テストコード修正
  - [ ] `cargo test -p geo_primitives`
- [ ] LineSegment2D/3D 修正（見積: 2-3日）
  - [ ] 内部メソッド作成
  - [ ] Foundation実装修正
  - [ ] 外部使用箇所修正（20+ 箇所）
  - [ ] `cargo test --workspace`

**期間**: 見積 3-5日

### Phase 3: Tier 2 移行実装
- [ ] Ellipse2D/3D 非推奨化（見積: 0.5日）
- [ ] Plane3D 非推奨化（見積: 0.3日）

**期間**: 見積 0.8日

### Phase 4: Tier 3-4 対応
- [ ] EllipseArc, Direction, InfiniteLine 調査・修正

**期間**: 調査後決定

---

## 🔗 関連ドキュメント・Issue

- **Issue #202**: [レガシーAPI問題解決 - Foundation Patternへの完全移行](https://github.com/your-repo/RedRing/issues/202)
- **Issue #218**: 行列演算の統一とカプセル化（完了）
- **参考実装**: [circle_2d.rs](c:\Users\takat\GitHub\RedRing\model\geo_primitives\src\circle_2d.rs) - 正しいパターンのリファレンス
- **Foundation定義**: `model/geo_foundation/src/core/` - Core Traits
- **アーキテクチャ**: `dev/architecture/ARCHITECTURE.md`
- **Foundation詳細**: `dev/foundation/FOUNDATION_REFACTORING_PLAN.md`

---

## 📌 次回作業タスク

### 即時実施（Phase 1 残り）
1. [ ] EllipseArc2D/3D の Foundation実装詳細確認
2. [ ] Direction2D/3D の Foundation実装詳細確認
3. [ ] この調査レポートを最終版にコミット

### Phase 2 開始準備
4. [ ] Arc2D 修正のための詳細設計書作成
5. [ ] LineSegment2D/3D 使用箇所の完全マップ作成（ファイル別リスト）
6. [ ] テスト戦略策定（段階的テスト方法）

---

**最終更新**: 2026年2月13日  
**調査実施者**: AI開発者  
**次回更新予定**: Phase 1 完全完了後（EllipseArc, Direction調査完了時）
