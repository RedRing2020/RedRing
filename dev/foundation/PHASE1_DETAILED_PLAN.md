# Phase 1: CylindricalSolid3D リファクタリング詳細計画

**作成日**: 2025年11月29日  
**対象図形**: `CylindricalSolid3D`  
**期間**: 1-2日  
**リスクレベル**: 低

---

## 📋 作業チェックリスト

### ステップ1: 現状確認 ✓

- [x] 既存メソッド一覧作成
- [x] Core Traits実装確認
- [x] テスト依存関係調査
- [x] ドキュメント現状確認

### ステップ2: レガシーメソッド内部化

#### 対象メソッド

| メソッド名 | 現在の公開性 | 変更後 | 理由 |
|-----------|------------|--------|------|
| `center()` | `pub` | `pub(crate) center_internal()` | Point3D型、Core Traitsと競合 |
| `axis()` | `pub` | `pub(crate) axis_internal()` | Direction3D型、Core Traitsと競合 |
| `ref_direction()` | `pub` | `pub(crate) ref_direction_internal()` | Direction3D型、Core Traitsと競合 |
| `radius()` | `pub` | `pub` | プリミティブ型、競合なし |
| `height()` | `pub` | `pub` | プリミティブ型、競合なし |
| `volume()` | `pub` | `pub(crate) volume_internal()` | Core Traitsと競合 |
| `surface_area()` | `pub` | `pub(crate) surface_area_internal()` | Core Traitsと競合 |
| `bounding_box()` | `pub` | `pub` | BBox3D型、Core Traitsにない |
| `contains_point(Point3D)` | `pub` | `pub(crate) contains_point_internal()` | Point3D型、Core Traitsと競合 |
| `distance_to_surface(Point3D)` | `pub` | `pub(crate) distance_internal()` | Point3D型、Core Traitsと競合 |

#### 作業項目

- [ ] 1.1 `center()` メソッドを `center_internal()` にリネーム＋`pub(crate)`化
- [ ] 1.2 `axis()` メソッドを `axis_internal()` にリネーム＋`pub(crate)`化
- [ ] 1.3 `ref_direction()` メソッドを `ref_direction_internal()` にリネーム＋`pub(crate)`化
- [ ] 1.4 `volume()` メソッドを `volume_internal()` にリネーム＋`pub(crate)`化
- [ ] 1.5 `surface_area()` メソッドを `surface_area_internal()` にリネーム＋`pub(crate)`化
- [ ] 1.6 `contains_point()` メソッドを `contains_point_internal()` にリネーム＋`pub(crate)`化
- [ ] 1.7 `distance_to_surface()` メソッドを `distance_internal()` にリネーム＋`pub(crate)`化

### ステップ3: Core Traits実装の直接化

#### 3.1 CylindricalSolid3DProperties トレイト

**Before（ラッパー型）**:
```rust
impl<T: Scalar> CylindricalSolid3DProperties<T> for CylindricalSolid3D<T> {
    fn center(&self) -> (T, T, T) {
        let c = self.center();       // 既存メソッド呼び出し
        (c.x(), c.y(), c.z())        // 型変換
    }
    
    fn radius(&self) -> T {
        self.radius()                // 転送のみ
    }
    
    fn height(&self) -> T {
        self.height()                // 転送のみ
    }
    
    fn axis(&self) -> (T, T, T) {
        let a = self.axis();
        (a.x(), a.y(), a.z())
    }
    
    fn ref_direction(&self) -> (T, T, T) {
        let r = self.ref_direction();
        (r.x(), r.y(), r.z())
    }
    
    fn diameter(&self) -> T {
        self.radius() * T::from_f64(2.0)
    }
}
```

**After（直接実装）**:
```rust
impl<T: Scalar> CylindricalSolid3DProperties<T> for CylindricalSolid3D<T> {
    #[inline]
    fn center(&self) -> (T, T, T) {
        // フィールド直接アクセス
        (self.center.x(), self.center.y(), self.center.z())
    }
    
    #[inline]
    fn radius(&self) -> T {
        self.radius  // フィールド直接アクセス
    }
    
    #[inline]
    fn height(&self) -> T {
        self.height  // フィールド直接アクセス
    }
    
    #[inline]
    fn axis(&self) -> (T, T, T) {
        (self.axis.x(), self.axis.y(), self.axis.z())
    }
    
    #[inline]
    fn ref_direction(&self) -> (T, T, T) {
        (self.ref_direction.x(), self.ref_direction.y(), self.ref_direction.z())
    }
    
    #[inline]
    fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)  // フィールド直接アクセス
    }
}
```

**作業項目**:
- [ ] 3.1.1 `center()` メソッドの直接実装＋`#[inline]`追加
- [ ] 3.1.2 `radius()` メソッドの直接実装＋`#[inline]`追加
- [ ] 3.1.3 `height()` メソッドの直接実装＋`#[inline]`追加
- [ ] 3.1.4 `axis()` メソッドの直接実装＋`#[inline]`追加
- [ ] 3.1.5 `ref_direction()` メソッドの直接実装＋`#[inline]`追加
- [ ] 3.1.6 `diameter()` メソッドの直接実装＋`#[inline]`追加

#### 3.2 CylindricalSolid3DMeasure トレイト

**Before（ラッパー型）**:
```rust
impl<T: Scalar> CylindricalSolid3DMeasure<T> for CylindricalSolid3D<T> {
    fn volume(&self) -> T {
        self.volume()  // 既存メソッド呼び出し
    }
    
    fn surface_area(&self) -> T {
        self.surface_area()  // 既存メソッド呼び出し
    }
    
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let point_3d = Point3D::new(point.0, point.1, point.2);  // 型変換
        self.contains_point(point_3d)  // 既存メソッド呼び出し
    }
    
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let point_3d = Point3D::new(point.0, point.1, point.2);  // 型変換
        self.distance_to_surface(point_3d).abs()  // 既存メソッド呼び出し
    }
}
```

**After（直接実装）**:
```rust
impl<T: Scalar> CylindricalSolid3DMeasure<T> for CylindricalSolid3D<T> {
    #[inline]
    fn volume(&self) -> T {
        // フィールド直接計算
        T::PI * self.radius * self.radius * self.height
    }
    
    #[inline]
    fn surface_area(&self) -> T {
        // フィールド直接計算
        let base_area = T::PI * self.radius * self.radius;
        let side_area = T::from_f64(2.0) * T::PI * self.radius * self.height;
        T::from_f64(2.0) * base_area + side_area
    }
    
    #[inline]
    fn contains_point(&self, point: (T, T, T)) -> bool {
        // タプルを直接使用（Point3D生成なし）
        let to_point_x = point.0 - self.center.x();
        let to_point_y = point.1 - self.center.y();
        let to_point_z = point.2 - self.center.z();
        
        let axis_projection = 
            to_point_x * self.axis.x() +
            to_point_y * self.axis.y() +
            to_point_z * self.axis.z();
        
        // 高さ範囲の確認
        if axis_projection < T::ZERO || axis_projection > self.height {
            return false;
        }
        
        // 半径範囲の確認
        let axis_x = self.axis.x() * axis_projection;
        let axis_y = self.axis.y() * axis_projection;
        let axis_z = self.axis.z() * axis_projection;
        
        let radial_x = to_point_x - axis_x;
        let radial_y = to_point_y - axis_y;
        let radial_z = to_point_z - axis_z;
        
        let radial_distance_sq = 
            radial_x * radial_x + 
            radial_y * radial_y + 
            radial_z * radial_z;
        
        radial_distance_sq <= self.radius * self.radius
    }
    
    #[inline]
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        // タプルを直接使用
        let to_point_x = point.0 - self.center.x();
        let to_point_y = point.1 - self.center.y();
        let to_point_z = point.2 - self.center.z();
        
        let axis_projection = 
            to_point_x * self.axis.x() +
            to_point_y * self.axis.y() +
            to_point_z * self.axis.z();
        
        // 軸方向の距離
        let axis_distance = if axis_projection < T::ZERO {
            -axis_projection
        } else if axis_projection > self.height {
            axis_projection - self.height
        } else {
            T::ZERO
        };
        
        // 半径方向の距離
        let axis_x = self.axis.x() * axis_projection;
        let axis_y = self.axis.y() * axis_projection;
        let axis_z = self.axis.z() * axis_projection;
        
        let radial_x = to_point_x - axis_x;
        let radial_y = to_point_y - axis_y;
        let radial_z = to_point_z - axis_z;
        
        let radial_distance = (
            radial_x * radial_x + 
            radial_y * radial_y + 
            radial_z * radial_z
        ).sqrt();
        
        let radial_excess = (radial_distance - self.radius).max(T::ZERO);
        
        // 軸方向と半径方向の距離を合成
        (axis_distance * axis_distance + radial_excess * radial_excess).sqrt()
    }
}
```

**作業項目**:
- [ ] 3.2.1 `volume()` メソッドの直接実装＋`#[inline]`追加
- [ ] 3.2.2 `surface_area()` メソッドの直接実装＋`#[inline]`追加
- [ ] 3.2.3 `contains_point()` メソッドの直接実装＋`#[inline]`追加（Point3D生成削除）
- [ ] 3.2.4 `distance_to_point()` メソッドの直接実装＋`#[inline]`追加（Point3D生成削除）

### ステップ4: テスト更新

#### 4.1 インポート追加

**Before**:
```rust
use geo_primitives::CylindricalSolid3D;
```

**After**:
```rust
use geo_primitives::CylindricalSolid3D;
use geo_foundation::{
    CylindricalSolid3DConstructor,
    CylindricalSolid3DProperties,
    CylindricalSolid3DMeasure,
    CylindricalSolid3DCore,
};
```

**作業項目**:
- [ ] 4.1.1 `cylindrical_solid_3d_tests.rs` インポート追加
- [ ] 4.1.2 `cylindrical_solid_3d_transform_safe_tests.rs` インポート追加（必要に応じて）

#### 4.2 メソッド呼び出し更新

**Before**:
```rust
#[test]
fn test_center() {
    let cylinder = CylindricalSolid3D::new_z_axis(
        Point3D::origin(), 
        1.0, 
        2.0
    ).unwrap();
    
    let center = cylinder.center();  // Point3D返却
    assert_eq!(center, Point3D::origin());
}
```

**After**:
```rust
#[test]
fn test_center() {
    let cylinder = CylindricalSolid3D::new_z_axis(
        Point3D::origin(), 
        1.0, 
        2.0
    ).unwrap();
    
    // Core Traits経由
    let center = <CylindricalSolid3D<f64> as CylindricalSolid3DProperties<f64>>::center(&cylinder);
    // または use文で
    // let center = cylinder.center();  // タプル返却
    
    assert_eq!(center, (0.0, 0.0, 0.0));
}
```

**作業項目**:
- [ ] 4.2.1 全テストケースの呼び出しパターン確認
- [ ] 4.2.2 Core Traits経由の呼び出しに更新
- [ ] 4.2.3 アサーション更新（Point3D → タプル比較）

### ステップ5: ドキュメント更新

#### 5.1 APIドキュメント

**作業項目**:
- [ ] 5.1.1 `cylindrical_solid_3d.rs` のdocコメント更新
- [ ] 5.1.2 非推奨メソッドにdeprecation警告追加（移行期間用）
- [ ] 5.1.3 Core Traits使用例追加

#### 5.2 マイグレーションガイド

**作成内容**:
```markdown
# CylindricalSolid3D マイグレーションガイド

## 変更内容

### Before (v0.1.x)
- レガシーメソッド（Point3D型）を直接使用

### After (v0.2.0)
- Core Traits経由での統一アクセス

## 移行例

### center() メソッド
\`\`\`rust
// Before
use geo_primitives::CylindricalSolid3D;
let cylinder = CylindricalSolid3D::new_z_axis(...)?;
let center: Point3D<f64> = cylinder.center();

// After
use geo_primitives::CylindricalSolid3D;
use geo_foundation::CylindricalSolid3DProperties;

let cylinder = CylindricalSolid3D::new_z_axis(...)?;
let center: (f64, f64, f64) = cylinder.center();
// Point3Dが必要な場合
let center_point = Point3D::from_tuple(center);
\`\`\`
```

**作業項目**:
- [ ] 5.2.1 マイグレーションガイド作成
- [ ] 5.2.2 全メソッドの移行例記載
- [ ] 5.2.3 よくある質問（FAQ）追加

### ステップ6: 検証

#### 6.1 ビルドテスト

**作業項目**:
- [ ] 6.1.1 `cargo build` 成功確認
- [ ] 6.1.2 `cargo build --release` 成功確認
- [ ] 6.1.3 警告・エラーなし確認

#### 6.2 テスト実行

**作業項目**:
- [ ] 6.2.1 `cargo test -p geo_primitives` 全通過確認
- [ ] 6.2.2 `cargo test --workspace` 全通過確認
- [ ] 6.2.3 テストカバレッジ確認

#### 6.3 Clippy検証

**作業項目**:
- [ ] 6.3.1 `cargo clippy` 警告なし確認
- [ ] 6.3.2 `cargo clippy -- -D warnings` 成功確認

#### 6.4 パフォーマンステスト

**作業項目**:
- [ ] 6.4.1 ベンチマーク実装（Criterion.rs）
- [ ] 6.4.2 Before/After比較測定
- [ ] 6.4.3 結果記録＋分析

---

## 🔍 詳細実装例

### contains_point() メソッドの完全実装

**現在の実装（cylindrical_solid_3d.rs）**:
```rust
pub fn contains_point(&self, point: Point3D<T>) -> bool {
    // 点から底面への投影を計算
    let to_point = Vector3D::new(
        point.x() - self.center.x(),
        point.y() - self.center.y(),
        point.z() - self.center.z(),
    );

    let axis_projection = to_point.dot(&self.axis.as_vector());

    // 高さ範囲の確認
    if axis_projection < T::ZERO || axis_projection > self.height {
        return false;
    }

    // 半径範囲の確認
    let axis_component = Vector3D::new(
        self.axis.x() * axis_projection,
        self.axis.y() * axis_projection,
        self.axis.z() * axis_projection,
    );
    let radial_component = Vector3D::new(
        to_point.x() - axis_component.x(),
        to_point.y() - axis_component.y(),
        to_point.z() - axis_component.z(),
    );
    let radial_distance = radial_component.magnitude();

    radial_distance <= self.radius
}
```

**リファクタリング後（直接実装）**:
```rust
impl<T: Scalar> CylindricalSolid3DMeasure<T> for CylindricalSolid3D<T> {
    #[inline]
    fn contains_point(&self, point: (T, T, T)) -> bool {
        // Vector3D生成なし、タプル直接使用
        let to_point_x = point.0 - self.center.x();
        let to_point_y = point.1 - self.center.y();
        let to_point_z = point.2 - self.center.z();
        
        // ドット積を直接計算（Vector3D::dot 呼び出しなし）
        let axis_projection = 
            to_point_x * self.axis.x() +
            to_point_y * self.axis.y() +
            to_point_z * self.axis.z();
        
        // 高さ範囲の確認
        if axis_projection < T::ZERO || axis_projection > self.height {
            return false;
        }
        
        // 半径方向成分の計算（Vector3D生成なし）
        let axis_x = self.axis.x() * axis_projection;
        let axis_y = self.axis.y() * axis_projection;
        let axis_z = self.axis.z() * axis_projection;
        
        let radial_x = to_point_x - axis_x;
        let radial_y = to_point_y - axis_y;
        let radial_z = to_point_z - axis_z;
        
        // magnitude() 呼び出しなし、平方根削減（平方同士で比較）
        let radial_distance_sq = 
            radial_x * radial_x + 
            radial_y * radial_y + 
            radial_z * radial_z;
        
        radial_distance_sq <= self.radius * self.radius
    }
}
```

**最適化ポイント**:
1. Point3D, Vector3D構造体生成削除（3箇所）
2. `dot()`, `magnitude()` メソッド呼び出し削除
3. 平方根計算削減（`magnitude()` → 平方同士比較）
4. `#[inline]` によるインライン化促進

---

## ⏱️ タイムライン

### Day 1

**午前**:
- [ ] ステップ1: 現状確認（完了済み）
- [ ] ステップ2.1-2.4: レガシーメソッド内部化（center, axis, ref_direction, volume）

**午後**:
- [ ] ステップ2.5-2.7: レガシーメソッド内部化（surface_area, contains_point, distance）
- [ ] ステップ3.1: CylindricalSolid3DProperties直接実装

### Day 2

**午前**:
- [ ] ステップ3.2: CylindricalSolid3DMeasure直接実装
- [ ] ステップ4: テスト更新

**午後**:
- [ ] ステップ5: ドキュメント更新
- [ ] ステップ6: 検証（ビルド、テスト、Clippy、パフォーマンス）

---

## 📝 完了基準

- [ ] 全てのレガシーメソッドが内部化（pub → pub(crate) + リネーム）
- [ ] Core Traits実装が直接実装に変更（ラッパー削除）
- [ ] 全テスト通過（301 tests passed維持）
- [ ] Clippy警告なし
- [ ] パフォーマンステスト実施＋20%以上改善確認
- [ ] マイグレーションガイド完成

---

## 🚨 リスク管理

### 高リスク

**リスク**: テスト大量更新でミス発生  
**対策**: 段階的commit、1メソッドずつ確認

### 中リスク

**リスク**: パフォーマンス改善が期待以下  
**対策**: ベンチマーク先行実施、測定結果に基づく調整

### 低リスク

**リスク**: ドキュメント更新漏れ  
**対策**: チェックリスト厳守

---

## 💡 次フェーズへの引き継ぎ

Phase 1完了後、以下を次フェーズに引き継ぐ：

1. **成功パターン**: CylindricalSolid3Dでの実装方法
2. **測定結果**: パフォーマンス改善の実測値
3. **問題点**: 遭遇した課題と解決策
4. **最適化ノウハウ**: インライン化、型変換削減のベストプラクティス

これらを基に、Phase 2（残り6図形）を効率的に実施。
