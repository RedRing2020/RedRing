# Model/Geometry アーキテクチャ分析と geo_core 統合方針

## 現在の状況分析

### 重複している型の現状

1. **Vector 型**:

   - `model/geometry3d/vector.rs` (105 行) - 高レベル幾何演算、業務ロジック層
   - `geo_core/vector3d.rs` (286 行) - 数値計算基盤、許容誤差対応

2. **Point 型**:
   - `model/geometry3d/point.rs` (92 行) - 業務ロジック層
   - `geo_core/primitives.rs` Point3D - 数値計算層、Scalar 型使用

### model の洗練された設計パターン

#### 1. 抽象化レイヤー (`geometry_trait/`)

- **Curve3D trait**: Any ダウンキャスト + 統一 API
- **評価メソッド**: evaluate(), derivative(), length()
- **柔軟な拡張**: parameter_hint(), domain()
- **型安全性**: 実行時型判定の仕組み

#### 2. 分類システム (`geometry_kind/`)

- **階層的分類**: GeometryKind → CurveKind3D/SurfaceKind
- **網羅的列挙**: InfiniteLine, Ray, Line, Circle, Arc, Ellipse, NurbsCurve...
- **拡張性**: Unknown による将来対応

#### 3. 共通機能 (`geometry_common/`)

- **ジェネリック設計**: IntersectionResult<P> で Point2D/Point3D 両対応
- **意味論的分類**: IntersectionKind (Point, Tangent, Overlap, None)
- **許容誤差管理**: tolerance_used フィールド

## 統合戦略

### Phase 1: 基盤型の役割分担明確化

- **geo_core**: 数値計算基盤、許容誤差処理、スカラー演算
- **model**: 業務ロジック、幾何学的抽象化、トレイト設計

### Phase 2: アダプターパターンによる統合

- 既存 `geometry_adapter.rs` を拡張
- model API 互換性を保ちつつ、内部で geo_core 使用
- Curve3D trait の実装は維持

### Phase 3: 段階的移行

1. 新機能をアダプター経由で実装
2. 既存コードの互換性テスト
3. 重複コードの段階的削除

## 具体的な実装方針

### 1. 型変換レイヤー

```rust
// model → geo_core 変換
impl From<model::Point> for geo_core::Point3D { ... }
impl From<geo_core::Point3D> for model::Point { ... }
```

### 2. trait 実装の一貫性

```rust
// 既存のCurve3D traitを維持
impl Curve3D for AdaptedLine {
    fn kind(&self) -> CurveKind3D { CurveKind3D::Line }
    fn evaluate(&self, t: f64) -> Point {
        // 内部で LineSegment3D (from geo_core) を使用
    }
}
```

### 3. 許容誤差の統合

```rust
// geometry_commonのIntersectionResultをgeo_core::ToleranceContextと統合
pub struct IntersectionResult<P> {
    pub tolerance_used: ToleranceContext, // geo_coreの許容誤差型を使用
}
```

## 保持すべき設計原則

1. **型安全性**: Any ダウンキャストシステム
2. **抽象化**: Curve3D/Surface trait 設計
3. **分類システム**: geometry_kind 階層構造
4. **API 互換性**: 既存の evaluate/derivative/length メソッド
5. **拡張性**: parameter_hint/domain による柔軟性

## 次のステップ

1. `geometry_adapter.rs`の完成 (Vector3D, Point3D 適合層)
2. Curve3D trait 実装の統合テスト
3. IntersectionResult<P>の geo_core 許容誤差統合
4. 段階的移行計画の実行

この方針により、model の洗練された設計を保持しつつ、geo_core の数値的堅牢性を活用できます。
