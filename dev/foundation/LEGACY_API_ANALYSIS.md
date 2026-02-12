# レガシーAPI影響範囲調査レポート

**作成日**: 2026年2月13日  
**Issue**: #202 レガシーAPI問題解決 - Foundation Patternへの完全移行

## 📋 調査概要

10以上の形状で共存しているレガシーAPIとFoundation実装を調査し、Foundation Patternへの完全移行計画を策定する。

## 🎯 調査目的

- レガシーメソッド使用箇所のリストアップ
- Foundation対応トレイトの特定
- 移行優先度の決定
- 破壊的変更の影響範囲の把握

## 🔍 対象形状

| 形状 | ファイル | 状態 | 優先度 |
|------|----------|------|--------|
| Circle2D/3D | `circle_2d.rs`, `circle_3d.rs` | ✅ 移行済み | - |
| Ellipse2D/3D | `ellipse_2d.rs`, `ellipse_3d.rs` | 🔍 調査中 | 高 |
| EllipseArc2D/3D | `ellipse_arc_2d.rs`, `ellipse_arc_3d.rs` | 🔍 調査中 | 高 |
| Arc2D/3D | `arc_2d.rs`, `arc_3d.rs` | 🔍 調査中 | 高 |
| Ray2D/3D | `ray_2d.rs`, `ray_3d.rs` | 🔍 調査中 | 中 |
| LineSegment2D/3D | `line_segment_2d.rs`, `line_segment_3d.rs` | 🔍 調査中 | 中 |
| Direction2D/3D | `direction_2d.rs`, `direction_3d.rs` | 🔍 調査中 | 中 |
| Point2D/3D | (geo_core) | ✅ 移行済み | - |
| Vector2D/3D/4D | (geo_core) | ✅ 移行済み | - |
| Aabb2D/3D | (geo_core) | ✅ 移行済み | - |
| Plane3D | `plane_3d.rs` | 🔍 調査中 | 低 |
| InfiniteLine2D/3D | `infinite_line_*d.rs` | 🔍 調査中 | 低 |

## 📊 初期調査結果

### ✅ 既に移行完了している形状

#### Circle2D/Circle3D
- **状態**: Foundation Pattern準拠完了
- **内部メソッド**:
  - `center_internal()` - 外部公開なし
  - `radius_internal()` - 外部公開なし
  - `ref_direction_internal()` - 外部公開なし
- **Foundation実装**:
  - `Circle2DProperties::center()` - Point2D返却
  - `Circle2DProperties::radius()` - T返却
  - `Circle2DMeasure::circumference()`
  - `Circle2DMeasure::area()`

#### Point2D/Point3D, Vector2/3/4
- **状態**: Issue #218で移行完了（2026年2月13日）
- **カプセル化**: dataフィールドをprivate化
- **アクセサ**: `x()`, `y()`, `z()`, `w()`メソッド

#### Aabb2D/Aabb3D
- **状態**: Foundation Pattern準拠
- **メソッド**:
  - `min_point()` / `max_point()` - レガシーの`min()`/`max()`は使用されていない
  - `center()` - 中心点取得

### 🔍 調査が必要な形状（詳細調査中）

#### 1. Arc2D/Arc3D
**レガシーメソッド候補**:
- `pub fn center(&self) -> Point2D<T>` / `Point3D<T>`
- `pub fn radius(&self) -> T`
- `pub fn arc_length(&self) -> T`

**使用箇所**（初期調査）:
```
arc_2d_tests.rs:27:  assert_eq!(arc.center(), center);
arc_2d_tests.rs:28:  assert_eq!(arc.radius(), 5.0);
arc_2d_tests.rs:75:  let length = arc.arc_length();
arc_2d_tests.rs:99:  let arc_length = full_arc.arc_length();
```

**Foundation対応**: `arc_traits.rs` 確認が必要

#### 2. EllipseArc2D/EllipseArc3D
**レガシーメソッド候補**:
- `pub fn center(&self) -> Point2D<T>` / `Point3D<T>`
- `pub fn arc_length(&self) -> T`

**使用箇所**（初期調査）:
```
ellipse_arc_2d.rs:72:   pub fn center(&self) -> Point2D<T>
ellipse_arc_2d.rs:133:  pub fn arc_length(&self) -> T
ellipse_arc_3d.rs:70:   pub fn center(&self) -> Point3D<T>
```

**Foundation対応**: `ellipse_arc_traits.rs` 確認が必要

#### 3. Ellipse2D/Ellipse3D
**レガシーメソッド候補**:
- `pub fn center(&self) -> Point2D<T>` / `Point3D<T>`
- `pub fn semi_major_axis(&self) -> T`
- `pub fn semi_minor_axis(&self) -> T`

**使用箇所**（初期調査）:
```
ellipse_2d.rs:75:  pub fn center(&self) -> Point2D<T>
ellipse_3d.rs:102: pub fn center(&self) -> Point3D<T>
```

**Foundation対応**: `ellipse_traits.rs` 確認が必要

#### 4. Ray2D/Ray3D
**レガシーメソッド候補**:
- `pub fn origin(&self) -> Point3D<T>`
- `pub fn direction(&self) -> Direction3D<T>`

**使用箇所**（初期調査）:
```
ray_3d.rs:73: pub fn origin(&self) -> Point3D<T>
```

**Foundation対応**: `ray_traits.rs` 既に定義済み

#### 5. LineSegment2D/LineSegment3D
**レガシーメソッド候補**:
- `pub fn start(&self) -> Point3D<T>`
- `pub fn end(&self) -> Point3D<T>`
- `pub fn length(&self) -> T`
- `pub fn direction(&self) -> Vector3D<T>`

**使用箇所**（初期調査） - 多数:
```
voxel.rs:335-339:   segment.start().x/y/z(), segment.end().x/y/z()
voxel.rs:441-446:   sx/sy/sz = segment.start/end()
circle_3d_collision.rs: segment.start(), segment.end()
... (20+ matches)
```

**Foundation対応**: `linesegment_traits.rs` 既に定義済み

#### 6. Direction2D/Direction3D
**レガシーメソッド候補**:
- `pub fn x(&self) -> T`
- `pub fn y(&self) -> T`
- `pub fn z(&self) -> T`
- `pub fn length(&self) -> T` (Direction3D)

**使用箇所**: 要詳細調査

**Foundation対応**: `direction_traits.rs` 確認が必要

#### 7. Plane3D
**レガシーメソッド候補**:
- `pub fn origin(&self) -> Point3D<T>`

**使用箇所**:
```
plane_3d.rs:160: pub fn origin(&self) -> Point3D<T>
```

**Foundation対応**: `plane_traits.rs` 確認が必要

#### 8. CylindricalSolid3D/CylindricalSurface3D
**レガシーメソッド候補**:
- `pub fn radius(&self) -> T`

**使用箇所**:
```
cylindrical_surface_3d.rs:177: pub fn radius(&self) -> T
cylindrical_solid_3d.rs:183:   pub fn radius(&self) -> T
```

**Foundation対応**: 対応トレイト確認が必要

## 📈 使用頻度分析

### 高頻度メソッド（20回以上使用）
- `segment.start()` / `segment.end()` - LineSegment系（20回以上）
- `arc.center()` - Arc2D系（複数ファイル）
- `arc.radius()` - Arc2D系（複数ファイル）

### 中頻度メソッド（5-20回使用）
- `arc.arc_length()` - Arc/EllipseArc系
- `center()` - Circle/Ellipse/Arc系

### 低頻度メソッド（5回未満）
- `plane.origin()`
- `ray.origin()`
- `direction.length()`

## 🎯 移行優先度

### Tier 1: 最優先（使用頻度が高く、Foundation対応済み）
1. **LineSegment2D/3D** - 20回以上使用、Foundation定義済み
2. **Arc2D/3D** - テストコードで多用

### Tier 2: 高優先（Foundation対応確認が必要）
3. **EllipseArc2D/3D** - `arc_length()`使用あり
4. **Ellipse2D/3D** - `center()`メソッド
5. **Ray2D/3D** - Foundation定義済み

### Tier 3: 中優先
6. **Direction2D/3D** - アクセサメソッド
7. **Plane3D** - 低頻度使用

## 🚧 予想される破壊的変更

### メソッドシグネチャの変更
```rust
// Before (レガシー)
pub fn center(&self) -> (T, T)           // タプル返却

// After (Foundation)
fn center(&self) -> Point2D<T>           // Point型返却（トレイト実装）
```

### アクセス方法の変更
```rust
// Before
let arc = Arc2D::new(...);
let c = arc.center();  // 直接アクセス

// After
use geo_foundation::Arc2DProperties;
let arc = Arc2D::new(...);
let c = arc.center();  // Foundation トレイト経由
```

## 📝 次のステップ

### Phase 1: 詳細調査（残り作業）
- [ ] Arc2D/3D のFoundation実装状況確認
- [ ] EllipseArc2D/3D のFoundation実装状況確認
- [ ] Ellipse2D/3D のFoundation実装状況確認
- [ ] Ray2D/3D のFoundation実装状況確認
- [ ] LineSegment2D/3D の全使用箇所マップ作成
- [ ] Direction2D/3D の詳細調査
- [ ] Plane3D の詳細調査
- [ ] 各形状のテストコード影響範囲確認

### Phase 2: 移行計画策定
- [ ] 形状別の移行手順書作成
- [ ] テストコード更新計画
- [ ] 破壊的変更の影響範囲リスト
- [ ] Migration Guide の骨組み作成

### Phase 3: 実装
- [ ] Tier 1 形状の移行実装
- [ ] Tier 2 形状の移行実装
- [ ] Tier 3 形状の移行実装
- [ ] テストコード更新
- [ ] ドキュメント更新

## 📊 進捗状況

- [x] 初期調査開始（2026年2月13日）
- [x] 対象形状リストアップ
- [x] 既存移行済み形状の確認
- [x] 主要レガシーメソッドの grep 調査
- [ ] Foundation トレイト定義の詳細確認
- [ ] 全使用箇所の完全マップ作成
- [ ] 移行計画書の完成

## 🔗 関連ドキュメント

- Issue #202: レガシーAPI問題解決 - Foundation Patternへの完全移行
- `dev/architecture/ARCHITECTURE.md` - アーキテクチャ構成
- `dev/foundation/FOUNDATION_REFACTORING_PLAN.md` - Foundation パターン詳細
- `model/geo_foundation/src/core/` - Core Traits 定義

---

**次回更新**: 各形状のFoundation実装状況確認後
