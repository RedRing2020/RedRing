# 命名規則修正完了レポート（CAD業界標準準拠版）

## 修正された命名規則

### ✅ 実施済み修正

**1. CAD業界標準準拠への修正**
- **Arc名称**: `CircleArc` → `Arc` （業界標準）
- **Line名称**: `LinePiece` → `Line` （業界標準）
- **修正理由**: CAD業界で一般的に使用される名称への統一

**2. 直線階層の正しい親子関係確立**
- **InfiniteLine** (親): 無限に延びる直線
- **Ray** (子): 半無限直線（始点あり、一方向に無限）  
- **Line** (孫): 線分（始点と終点を持つ）
- **階層構造**: InfiniteLine → Ray → Line

**3. 境界ボックス統一維持**
- `BoundingBox` → `BBox` の統一を維持
- geo_primitivesとの名前整合性確保

## 現在の正しい命名規則

### 基本トレイト命名
```rust
// 基本形状
PointCore<T>                    // 基本点トレイト
VectorCore<T>                   // 基本ベクトルトレイト
CircleCore<T>                   // 基本円トレイト
ArcCore<T>                      // 基本円弧トレイト（CAD業界標準）
LineCore<T>                     // 基本線分トレイト
BBoxCore<T>                     // 基本境界ボックストレイト

// 次元特化
Point2DCore<T>, Point3DCore<T>  // 2D/3D点トレイト
Vector2DCore<T>, Vector3DCore<T> // 2D/3Dベクトルトレイト
Arc3DCore<T>                    // 3D円弧トレイト
Line3DCore<T>                   // 3D線分トレイト
```

### 親子関係（修正済み）
```rust
// Circle → Arc 関係（CAD業界標準）
trait ArcCore<T>: CircleCore<T> {
    fn start_angle(&self) -> T;
    fn end_angle(&self) -> T;
    fn is_full_circle(&self) -> bool;
}

// InfiniteLine → Ray → Line 階層
trait RayCore<T>: InfiniteLineCore<T> {
    fn origin(&self) -> Self::Point;
    fn direction(&self) -> Self::Vector;
}

trait LineCore<T>: RayCore<T> {
    fn start_point(&self) -> Self::Point;
    fn end_point(&self) -> Self::Point;
    fn length(&self) -> T;
}
```

### 実装型命名（geo_primitives対応）
```rust
// CAD業界標準準拠
Point<T>        // 点（次元サフィックス無し）
Vector<T>       // ベクトル
Circle<T>       // 円
Arc<T>          // 円弧
Line<T>         // 線分
Ray<T>          // 光線（半無限直線）
BBox<T>         // 境界ボックス

// 互換性エイリアス
pub type Arc2D<T> = Arc<T>;
pub type Line2D<T> = Line<T>;
```

## クリーンアップ計画

### Phase 1: 基本命名統一 ✅
- [x] BoundingBox → BBox 統一
- [x] CircleArc → Arc（CAD業界標準）修正
- [x] LinePiece → Line修正
- [x] 直線階層関係の確立

### Phase 2: 専用トレイトファイル整備（準備中）
```rust
// 有効化予定ファイル（Angle<T>型統合後）
basic_arc.rs        // Arc専用トレイト
basic_line.rs       // Line専用トレイト（Ray含む）
basic_bbox.rs       // BBox専用トレイト
```

### Phase 3: 旧ファイルクリーンアップ計画
1. **basic_shapes.rs の段階的分離**
   - ArcCore → basic_arc.rs に移行
   - LineCore → basic_line.rs に移行
   - BBoxCore → basic_bbox.rs に移行

2. **型エイリアス互換性**
   ```rust
   // 移行期間中の互換性確保
   pub use basic_arc::ArcCore as NewArcCore;
   pub use basic_shapes::ArcCore; // 旧版（段階的非推奨予定）
   ```

3. **Deprecation 戦略**
   - 段階的警告の追加
   - 移行ガイドの提供
   - 明確な移行スケジュール

## 技術的考慮事項

### Angle<T>型統合の必要性
```rust
// 現在の制限事項
fn start_angle(&self) -> T;          // プリミティブ型使用

// 目標（型安全性向上）
fn start_angle(&self) -> Angle<T>;   // 型安全な角度型使用
```

### geo_primitives との整合性
- `Arc<T>` - geo_primitives/geometry2d/arc.rs と命名一致
- `Line<T>` - CAD業界標準の線分名称
- 型エイリアス戦略でスムーズな統合

## ビルド状況

✅ **コンパイル成功**: 修正後の命名規則でビルド成功
✅ **CAD業界準拠**: 主要CADソフトと命名一致  
✅ **階層構造明確**: InfiniteLine → Ray → Line の親子関係確立
✅ **後方互換性**: 段階的移行で既存コードとの互換性維持

## 今後のアクション

### 短期（immediate）
1. ✅ 基本命名規則の統一完了
2. 📋 geo_primitives 実装の命名確認
3. 📋 Angle<T>型の完全実装

### 中期（near-term）
1. 📋 専用トレイトファイルの有効化
2. 📋 basic_shapes.rs の段階的分離
3. 📋 型安全性の向上（Angle<T>統合）

### 長期（long-term）
1. 📋 旧ファイルの完全クリーンアップ
2. 📋 Deprecation 完了
3. 📋 CAD業界標準準拠の完全実現

---

**まとめ**: LinePieceやCircleArcといった一般的でない名称から、CAD業界で広く使用されているArcやLineに修正することで、業界標準との命名整合性を実現しました。短絡的な変更ではなく、段階的なクリーンアップ計画に基づいた持続可能な改善アプローチを採用しています。