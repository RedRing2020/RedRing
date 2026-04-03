# NurbsCurve3D レガシーAPI非推奨化計画

**作成日**: 2026年1月10日  
**対象**: Issue #194 Task 5 - レガシーAPI非推奨化  
**関連**: [issue-170-collision-numerical-precision-archive-note.md](../archive/issues/foundation/issue-170-collision-numerical-precision-archive-note.md)

## 概要

NurbsCurve3DにFoundation Patternを導入し、Core Traitsベースの新APIを提供しました。
既存の`pub fn new`などのメソッドを段階的に非推奨化し、Core Traitsへ移行します。

## 現状分析

### Core Traits実装状況（✅ 完了）

**Foundation定義**:
- `geo_foundation/src/core/nurbs_curve_3d_core_traits.rs`
  - `NurbsCurve3DConstructor<T>`: 3メソッド（new, from_bezier, line_segment）
  - `NurbsCurve3DProperties<T>`: 6メソッド（degree, knot_vector等）
  - `NurbsCurve3DMeasure<T>`: 4メソッド（arc_length, evaluate等）

**geo_nurbs実装**:
- `curve_3d.rs`: Core Traits実装完了（commit ecaacf2）
- `curve_3d_foundation.rs`: 統合テスト完了（commit 571895d）
- `curve_3d_transform.rs`: AnalysisTransform3D完了（commit ddcf932）

### レガシーAPI使用箇所（13箇所）

#### 1. geo_nurbs内部テストコード（8箇所）
```
model/geo_nurbs/src/curve_3d.rs:299
model/geo_nurbs/src/curve_3d.rs:318
model/geo_nurbs/src/curve_3d.rs:346
model/geo_nurbs/src/curve_3d.rs:366
model/geo_nurbs/src/curve_3d_extensions.rs:172
model/geo_nurbs/src/curve_3d_extensions.rs:191
model/geo_nurbs/src/curve_3d_extensions.rs:211
model/geo_nurbs/src/curve_3d_extensions.rs:238
model/geo_nurbs/src/curve_3d_foundation.rs:69
```

#### 2. geo_nurbs内部実装（1箇所）
```
model/geo_nurbs/src/curve_3d_transform.rs:57
```

#### 3. geo_algorithms外部使用（1箇所）
```
model/geo_algorithms/src/collision/primitive_nurbs.rs:612
```

#### 4. ドキュメント参照（2箇所）
```
model/geo_foundation/src/core/nurbs_curve_3d_core_traits.rs:17 (サンプルコード)
dev/architecture/NURBS_PRIMITIVES_COLLISION_ARCHITECTURE.md:420 (例示)
```

## 非推奨化戦略

### Phase 1: 内部互換性維持（現在）

**方針**: 既存の`pub fn new`を**内部使用のために保持**
- 理由: Core Traitsの実装が内部的に`pub fn new`を呼び出している
- 状態: 内部実装として引き続き使用可能
- 公開API: Core Traitsを推奨

**実施内容**:
1. ✅ Core Traits実装完了
2. ✅ 既存`pub fn new`は変更せず維持
3. ⏳ ドキュメントでCore Traits使用を推奨

### Phase 2: ソフト非推奨（次のステップ）

**deprecation属性の追加**:
```rust
#[deprecated(
    since = "0.2.0",
    note = "Use NurbsCurve3DConstructor::new() from geo_foundation instead"
)]
pub fn new(...) -> Result<Self> {
    // 既存実装は変更なし
}
```

**影響範囲の限定**:
- geo_nurbs内部: 警告が出るが動作は継続
- geo_algorithms: 明示的な移行作業が必要
- テストコード: 段階的に更新

### Phase 3: 段階的移行（将来）

**優先順位**:
1. **高**: geo_algorithms外部使用（1箇所）
   - `primitive_nurbs.rs:612` → Core Traitsに移行
   
2. **中**: geo_nurbs内部テストコード（8箇所）
   - 新APIの動作検証を兼ねて更新
   
3. **低**: ドキュメント（2箇所）
   - サンプルコードをCore Traits版に更新

### Phase 4: 完全削除（未定）

**条件**:
- 全外部使用箇所が移行完了
- 内部実装がCore Traits実装から独立
- メジャーバージョンアップ時（1.0.0）

**実施内容**:
- `pub fn new`の削除
- 内部実装の再構成

## 実装タスク

### Task 5-1: Deprecation属性追加 ⏳

**対象メソッド**:
```rust
// curve_3d.rs
pub fn new(...) -> Result<Self>
```

**追加する属性**:
```rust
#[deprecated(
    since = "0.2.0",
    note = "Use NurbsCurve3DConstructor::new() trait method instead. \
            Import: use geo_foundation::NurbsCurve3DConstructor;"
)]
```

**ファイル**: `model/geo_nurbs/src/curve_3d.rs`

### Task 5-2: ドキュメント更新 ⏳

**geo_foundation Core Traitsドキュメント**:
- ファイル: `nurbs_curve_3d_core_traits.rs`
- 内容: 使用例を完全なCore Traits版に更新

**アーキテクチャドキュメント**:
- ファイル: `dev/architecture/NURBS_PRIMITIVES_COLLISION_ARCHITECTURE.md`
- 内容: サンプルコードをCore Traits版に更新

### Task 5-3: geo_algorithms移行 ⏳

**対象ファイル**: `model/geo_algorithms/src/collision/primitive_nurbs.rs:612`

**移行前**:
```rust
NurbsCurve3D::new(control_points, weights, knots, 3).unwrap()
```

**移行後**:
```rust
use geo_foundation::NurbsCurve3DConstructor;
<NurbsCurve3D<T> as NurbsCurve3DConstructor<T>>::new(
    control_points, weights, knots, 3
).unwrap()
```

### Task 5-4: 内部テスト移行（任意） 📝

geo_nurbs内部のテストコードを段階的に更新:
- 優先度: 低（内部使用のため）
- 目的: 新APIの動作検証
- 実施: 他の作業の合間に随時更新

## 移行ガイド

### 既存コードからの移行

**パターン1: 基本的な生成**
```rust
// Before (レガシー)
use geo_nurbs::NurbsCurve3D;
let curve = NurbsCurve3D::new(points, weights, knots, degree)?;

// After (Core Traits)
use geo_nurbs::NurbsCurve3D;
use geo_foundation::NurbsCurve3DConstructor;
let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
    points, weights, knots, degree
)?;
```

**パターン2: ジェネリック関数内**
```rust
// Before
fn create_curve<T: Scalar>(points: Vec<Vector3<T>>) -> Result<NurbsCurve3D<T>> {
    NurbsCurve3D::new(points, None, knots, 3)
}

// After
use geo_foundation::NurbsCurve3DConstructor;
fn create_curve<T: Scalar>(points: Vec<Vector3<T>>) -> Result<NurbsCurve3D<T>> {
    <NurbsCurve3D<T> as NurbsCurve3DConstructor<T>>::new(
        points, None, knots, 3
    )
}
```

### 推奨パターン

**トレイト境界の活用**:
```rust
fn process_curve<T, C>(curve_data: CurveData<T>) -> Result<C>
where
    T: Scalar,
    C: NurbsCurve3DConstructor<T>,
{
    C::new(curve_data.points, curve_data.weights, curve_data.knots, curve_data.degree)
}
```

## 成功基準

### Phase 2完了条件
- ✅ Deprecation属性が追加されている
- ✅ ドキュメントが更新されている
- ✅ cargo buildが警告付きで成功
- ✅ 全テストがパス

### Phase 3完了条件
- ✅ geo_algorithms使用箇所が移行済み
- ✅ Core Traits使用例がドキュメント化
- ✅ 移行ガイドが整備されている

## 次のステップ

1. **Task 5-1実施**: Deprecation属性追加
2. **Task 5-2実施**: ドキュメント更新  
3. **Task 5-3実施**: geo_algorithms移行
4. **コミット**: `refactor(nurbs): Deprecate legacy NurbsCurve3D::new, recommend Core Traits`

## 参考資料

- Issue #194: NURBS Foundation Pattern完全適用
- [FOUNDATION_CORE_EXTENSION_REDESIGN_PROPOSAL.md](./FOUNDATION_CORE_EXTENSION_REDESIGN_PROPOSAL.md)
- [PHASE3_COMPLETION_REPORT.md](./PHASE3_COMPLETION_REPORT.md)
- Rust Deprecation Guide: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-deprecated-attribute
