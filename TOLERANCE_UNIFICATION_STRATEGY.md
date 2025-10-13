# Tolerance 統一戦略ドキュメント

**作成日**: 2025 年 10 月 13 日
**対象**: RedRing 幾何ライブラリ
**目的**: 許容誤差（tolerance）使用パターンの統一による長期的な設計方針確立

## 📋 目次

1. [現状分析](#現状分析)
2. [問題点と課題](#問題点と課題)
3. [設計方針](#設計方針)
4. [実装戦略](#実装戦略)
5. [移行計画](#移行計画)
6. [実装例](#実装例)
7. [評価基準](#評価基準)

## 📊 現状分析

### 現在の混在状況

RedRing コードベースでは、現在 3 つの tolerance アプローチが混在している：

#### Type A: Scalar 定数アプローチ（厳密制御）

```rust
// analysis/src/abstract_types/scalar.rs
pub trait Scalar {
    const EPSILON: Self;        // 機械的精度（f64: 2.22e-16）
    const TOLERANCE: Self;      // 数値計算用（f64: 1e-10）
    const ANGLE_TOLERANCE: Self; // 角度計算用（f64: 1e-8）
}
```

**使用例**:

- ゼロベクトル判定: `direction.is_zero(T::EPSILON)`
- 数値収束判定: `(a - b).abs() < T::TOLERANCE`

#### Type B: 引数指定アプローチ（柔軟制御）

```rust
// geo_primitives/src/ray_2d.rs
pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool
```

**使用例**:

- アプリケーション層での制御: `ray.contains_point(&point, user_tolerance)`
- 動的な精度調整: `segment.intersects(&other, context_tolerance)`

#### Type C: DefaultTolerances アプローチ（統一管理）

```rust
// geo_foundation/src/tolerance_migration.rs
impl DefaultTolerances {
    pub fn distance<T: Scalar>() -> T { /* f64: 1e-10, f32: 1e-6 */ }
    pub fn angle<T: Scalar>() -> T { /* f64: 1e-8, f32: 1e-4 */ }
}
```

**使用例**:

- 段階的移行: `DefaultTolerances::distance::<T>()`
- 型統一: `arc.is_degenerate()` → 内部で DefaultTolerances 使用

### 使用分布の詳細分析

| ファイル             | Scalar 定数                 | 引数指定           | DefaultTolerances       | 備考            |
| -------------------- | --------------------------- | ------------------ | ----------------------- | --------------- |
| ray_2d.rs            | T::EPSILON (1 箇所)         | tolerance (5 箇所) | 未使用                  | 引数指定主体    |
| infinite_line_2d.rs  | T::ANGLE_TOLERANCE (2 箇所) | tolerance (8 箇所) | 未使用                  | 混在パターン    |
| circle_2d.rs         | T::TOLERANCE (6 箇所)       | tolerance (3 箇所) | 未使用                  | Scalar 定数主体 |
| arc_2d_extensions.rs | 未使用                      | 未使用             | distance/angle (4 箇所) | 新方式採用      |

## ⚠️ 問題点と課題

### 1. 一貫性の欠如

- **同一機能での異なるアプローチ**: contains_point()メソッドで 3 パターンが混在
- **予測不可能な動作**: ユーザーがどの tolerance が使用されるか判断困難
- **保守性の低下**: 変更時の影響範囲が不明確

### 2. アプリケーション要求との乖離

- **厳密計算ニーズ**: CAD/CAM → 1e-12 レベルの高精度
- **リアルタイム計算ニーズ**: ゲーム → 1e-3 レベルの高速処理
- **現在の固定値**: 中間的な値で両方のニーズに未対応

### 3. 型安全性の問題

- **Scalar 定数の強制**: コンパイル時固定でランタイム調整不可
- **引数エラーリスク**: 不適切な tolerance 値の渡し方
- **単位不整合**: 距離 tolerance と角度 tolerance の混同リスク

## 🎯 設計方針

### 基本原則

#### 1. **階層化された制御レベル**

```
アプリケーション層    → 動的tolerance指定
↓
ドメイン層           → DefaultTolerances使用
↓
数値計算層           → Scalar定数使用
```

#### 2. **用途別 tolerance 分類**

| 用途分類         | 対象操作             | 推奨アプローチ | 例                                          |
| ---------------- | -------------------- | -------------- | ------------------------------------------- |
| **厳密幾何判定** | 平行・垂直・共線判定 | 引数指定       | `is_parallel_to(other, angle_tolerance)`    |
| **包含判定**     | 点の内外判定         | 引数指定       | `contains_point(point, distance_tolerance)` |
| **数値収束**     | 反復計算・近似       | Scalar 定数    | `newton_method()` → `T::TOLERANCE`          |
| **ゼロ判定**     | 特異点検出           | Scalar 定数    | `is_zero()` → `T::EPSILON`                  |

#### 3. **後方互換性の保証**

- 既存 API の動作変更禁止
- 新メソッド追加による段階的移行
- デフォルト実装での DefaultTolerances 活用

## 🔧 実装戦略

### Phase 1: Core Foundation 統一（推奨実装）

#### 現在の BasicContainment トレイト

```rust
trait BasicContainment<T> {
    fn contains_point(&self, point: &Self::Point) -> bool;
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool;
}
```

#### 提案する拡張実装

```rust
trait BasicContainment<T> {
    // レガシー互換（DefaultTolerances使用）
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point_with_tolerance(point, DefaultTolerances::distance())
    }

    // 厳密制御版（新規推奨）
    fn contains_point_with_tolerance(&self, point: &Self::Point, tolerance: T) -> bool;

    // 統一された境界判定
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point_with_tolerance(point, tolerance)
    }
}
```

**実装利点**:

- ✅ 既存コードの動作保証
- ✅ 新規コードでの厳密制御
- ✅ DefaultTolerances による統一管理

### Phase 2: Extension 方法統一

#### Ray2D Extension 統一例

```rust
impl<T: Scalar> Ray2D<T> {
    // デフォルト版（推奨パターン）
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.is_parallel_to_with_tolerance(other, DefaultTolerances::angle())
    }

    // 厳密制御版
    pub fn is_parallel_to_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.direction().is_parallel(&other.direction(), tolerance)
    }

    // 垂直判定も同様に統一
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.is_perpendicular_to_with_tolerance(other, DefaultTolerances::angle())
    }

    pub fn is_perpendicular_to_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.direction().is_perpendicular(&other.direction(), tolerance)
    }
}
```

### Phase 3: ToleranceSettings 活用

#### アプリケーション層での制御

```rust
use geo_foundation::{ToleranceSettings, GeometryContext};

// CAD/CAM用高精度設定
let precision_settings = ToleranceSettings::precision();
let cad_context = GeometryContext::new(precision_settings);

// ゲーム用高速設定
let relaxed_settings = ToleranceSettings::relaxed();
let game_context = GeometryContext::new(relaxed_settings);

// 使用例
let ray1 = Ray2D::new(origin, direction)?;
let ray2 = Ray2D::new(other_origin, other_direction)?;

// コンテキスト依存の判定
let is_parallel_cad = ray1.is_parallel_to_with_tolerance(&ray2, cad_context.angle_tolerance());
let is_parallel_game = ray1.is_parallel_to_with_tolerance(&ray2, game_context.angle_tolerance());
```

## 📅 移行計画

### Timeline

#### **2025 Q4: Phase 1 実装**

- [ ] BasicContainment トレイト拡張
- [ ] Ray2D 完全統一（既に 80%完了）
- [ ] Circle2D, LineSegment2D 統一
- [ ] 単体テスト充実

#### **2026 Q1: Phase 2 展開**

- [ ] InfiniteLine2D/3D 統一
- [ ] Arc2D/3D Extension 統一
- [ ] Ellipse2D Extension 統一
- [ ] 統合テスト実装

#### **2026 Q2: Phase 3 完成**

- [ ] ToleranceSettings 全面活用
- [ ] GeometryContext 実装
- [ ] ドキュメント完備
- [ ] パフォーマンス最適化

### 移行優先度

| 優先度 | 対象                 | 理由                   | 難易度 |
| ------ | -------------------- | ---------------------- | ------ |
| **高** | Ray2D, LineSegment2D | ユーザー利用頻度が高い | 低     |
| **中** | Circle2D, Arc2D      | 複雑な幾何計算が多い   | 中     |
| **低** | BBox2D, Ellipse2D    | 内部処理が主体         | 低     |

## 💡 実装例

### Example 1: Ray2D 完全実装（既に完了）

```rust
// geo_primitives/src/ray_2d.rs
impl<T: Scalar> Ray2D<T> {
    // Core Foundation（引数指定tolerance）
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let to_point = *point - self.origin;
        let t = to_point.dot(&self.direction);

        if t < T::ZERO { return false; }

        let projected_point = self.origin + self.direction * t;
        let distance = point.distance_to(&projected_point);
        distance <= tolerance
    }
}

// BasicContainment実装
impl<T: Scalar> BasicContainment<T> for Ray2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point(point, T::EPSILON)  // 現在の実装
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point(point, tolerance)   // 引数指定
    }
}
```

### Example 2: 提案する統一実装

```rust
// 新規BasicContainment実装案
impl<T: Scalar> BasicContainment<T> for Ray2D<T> {
    // レガシー互換版
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point_with_tolerance(point, DefaultTolerances::distance())
    }

    // 厳密制御版（推奨）
    fn contains_point_with_tolerance(&self, point: &Self::Point, tolerance: T) -> bool {
        let to_point = *point - self.origin;
        let t = to_point.dot(&self.direction);

        if t < T::ZERO { return false; }

        let projected_point = self.origin + self.direction * t;
        let distance = point.distance_to(&projected_point);
        distance <= tolerance
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point_with_tolerance(point, tolerance)
    }
}
```

## 📏 評価基準

### 成功指標

#### **技術的指標**

- [ ] **API 一貫性**: 同一機能で統一されたメソッド名・引数パターン
- [ ] **型安全性**: コンパイル時の tolerance 型チェック
- [ ] **パフォーマンス**: tolerance 計算のオーバーヘッド < 5%
- [ ] **後方互換性**: 既存テストケース 100%通過

#### **保守性指標**

- [ ] **コード可読性**: tolerance 使用箇所の明確な判別可能性
- [ ] **拡張性**: 新しい幾何プリミティブでの実装容易性
- [ ] **テスト網羅性**: tolerance 境界値での動作検証
- [ ] **ドキュメント**: 使用方針の明確な記述

### 品質基準

```rust
// 良い実装例
let intersection = ray1.intersection_with_tolerance(&ray2, app_context.distance_tolerance());

// 避けるべき実装
let intersection = ray1.intersection(&ray2); // どのtoleranceを使用するか不明
```

## 🔚 まとめ

### 長期戦略の核心

1. **段階的移行**: 一度に全てを変更せず、影響を最小化
2. **後方互換性**: 既存ユーザーコードの保護を最優先
3. **用途別最適化**: 数値計算とアプリケーション要求の両立
4. **統一管理**: DefaultTolerances による一元的な品質制御

### 今後の方針

- **Ray2D**: 既に理想的な実装済み（引数指定 tolerance）
- **他プリミティブ**: Ray2D パターンを踏襲した段階的統一
- **Core Foundation**: 慎重な拡張により互換性維持
- **Extension**: 積極的な統一により使いやすさ向上

**この文書は、RedRing の長期的な設計品質とユーザビリティ向上を目的とした戦略指針です。実装時は本文書を参照し、一貫した設計判断を行ってください。**

---

**Document Version**: 1.0
**Last Updated**: 2025 年 10 月 13 日
**Next Review**: 2025 年 12 月末
