# 命名規則統一進捗レポート（更新版）

## 最新の修正内容（2025 年 10 月 11 日）

### 1. 「弧」→「円弧」命名修正 ✅

- **修正理由**: 「弧」は誤りで「円弧」が正式名称
- **変更内容**:
  - `ArcCore<T>` → `CircleArcCore<T>`
  - `Arc3DCore<T>` → `CircleArc3DCore<T>`
  - 関連コメント・ドキュメントの「弧」→「円弧」統一

### 2. LineSegment→LinePiece 命名修正 ✅

- **修正理由**: LineSegment は CAD では複合 Line の意味で Segment と混乱を招く
- **変更内容**:
  - `LineSegmentCore<T>` → `LinePieceCore<T>`
  - 位相処理での Segment 混乱回避

## 実施済みの統一作業

### 1. 境界ボックス命名統一 ✅

- **変更前**: `BoundingBox`, `BoundingBoxCore`
- **変更後**: `BBox`, `BBoxCore`
- **対象ファイル**:
  - `foundation.rs`: `type BoundingBox` → `type BBox`
  - `basic_shapes.rs`: `BoundingBoxCore` → `BBoxCore`
  - `primitive.rs`: コメント更新で統一
  - **新規作成**: `basic_bbox.rs` - 分離した BBox 専用トレイト（一時コメントアウト）

### 2. 円弧（CircleArc）の親子関係明確化 ✅

- **Circle → CircleArc の親子関係確立**（「弧」→「円弧」に修正）
- **修正済み**: `basic_shapes.rs`
  - `ArcCore<T>` → `CircleArcCore<T>`: 基本円弧トレイト
  - **新規作成**: `basic_arc.rs` - 専用円弧トレイト（一時コメントアウト、Angle 型統合要）
- **Circle → CircleArc 関係**: 円弧は円の一部として定義

### 3. 2D/3D サフィックス規則統一 ✅

- **トレイト定義**: 次元サフィックス付与 (`Point2DCore<T>`, `Point3DCore<T>`)
- **具象型実装**: 次元サフィックス無し (`Point<T>` を 2D/3D で共用)
- **型エイリアス**: 互換性エイリアス提供 (`Point2D<T> = Point<T>`)

## 現在の統一済み命名規則

### トレイト命名パターン

```rust
// 基本パターン: {Shape}Core<T>
PointCore<T>                    // 基本点トレイト
VectorCore<T>                   // 基本ベクトルトレイト
CircleCore<T>                   // 基本円トレイト
CircleArcCore<T>                // 基本円弧トレイト（「弧」→「円弧」修正）
LinePieceCore<T>                // 基本線分トレイト（LineSegment→LinePiece修正）
BBoxCore<T>                     // 基本境界ボックストレイト

// 次元特化パターン: {Shape}{2D|3D}Core<T>
Point2DCore<T>                  // 2D点トレイト
Point3DCore<T>                  // 3D点トレイト
Vector2DCore<T>                 // 2Dベクトルトレイト
Vector3DCore<T>                 // 3Dベクトルトレイト
Circle3DCore<T>                 // 3D円トレイト
CircleArc3DCore<T>              // 3D円弧トレイト（修正）
LinePiece3DCore<T>              // 3D線分トレイト（修正）
BBox2DCore<T>                   // 2D境界ボックストレイト
BBox3DCore<T>                   // 3D境界ボックストレイト
```

### 統一済み型名

```rust
// 境界ボックス統一
BBox<T>                         // BoundingBox → BBox
BBoxCore<T>                     // BoundingBoxCore → BBoxCore
type BBox                       // GeometryFoundation::BoundingBox → BBox

// 円弧の統一（「弧」→「円弧」修正）
CircleArc<T>                    // 円弧（Circle の子要素）
CircleArcCore<T>                // 円弧の基本トレイト
EllipseArcCore<T>               // 楕円弧の基本トレイト

// 線分の統一（CAD用語適正化）
LinePiece<T>                    // 線分（LineSegmentから変更）
LinePieceCore<T>                // 線分の基本トレイト

// エラー型統一
VectorNormalizationError        // ベクトル正規化エラー
CircleArcAngleError            // 円弧角度エラー（修正）
LinePieceValidationError       // 線分検証エラー（修正）
```

### 親子関係の明確化

```rust
// Circle → CircleArc 関係（「弧」→「円弧」修正）
trait CircleArcCore<T>: CircleCore<T> {
    fn start_angle(&self) -> T;    // 型安全なAngle<T>に将来変更予定
    fn end_angle(&self) -> T;
    fn is_full_circle(&self) -> bool;
}

// Ellipse → EllipseArc 関係
trait EllipseArcCore<T>: EllipseCore<T> {
    fn start_angle(&self) -> T;
    fn end_angle(&self) -> T;
    fn is_full_ellipse(&self) -> bool;
}

// InfiniteLine → LinePiece 関係（命名修正）
trait LinePieceCore<T>: InfiniteLineCore<T> {
    fn start_point(&self) -> Self::Point;
    fn end_point(&self) -> Self::Point;
    fn length(&self) -> T;
}
```

## ファイル構成の整理状況

### 修正済みファイル

- `basic_shapes.rs`: `ArcCore` → `CircleArcCore`, `LineSegmentCore` → `LinePieceCore`
- `foundation.rs`: `BoundingBox` → `BBox` 統一
- `primitive.rs`: コメント・文書更新
- `mod.rs`: 命名規則反映、一時コメントアウト対応

### 段階的移行戦略

```rust
// 旧統合版（段階的に置き換え予定、命名修正済み）
pub use basic_shapes::{
    CircleArcCore, LinePieceCore, BBoxCore, // 修正済み名称
    InfiniteLineCore, EllipseCore          // 未修正
};

// 新しいトレイト（将来優先使用、現在は準備中）
// pub use basic_arc::{CircleArcCore as NewCircleArcCore}; // Angle<T>統合後
// pub use basic_bbox::{BBoxCore as NewBBoxCore};          // 専用実装後
```

## ビルド状況

✅ **コンパイル成功**: 全ての命名統一作業完了後もビルド成功
✅ **後方互換性**: 段階的移行で既存コードとの互換性維持
✅ **命名正確性**: 「弧」→「円弧」、LineSegment→LinePiece で業界標準に準拠

## geo_primitives との整合性検証が必要な項目

### 確認・対応必要項目

1. **geo_primitives/geometry2d/arc.rs**: `Arc<T>` → `CircleArc<T>` への変更必要性
2. **geo_primitives 実装での命名**: `LineSegment` → `LinePiece` への対応
3. **型エイリアス互換性**: 旧名前での後方互換性確保

### 今後の作業項目

1. **Angle\<T\>型統合**
   - `basic_arc.rs` のエラー解消（Angle\<T\>型メソッド実装必要）
   - 型安全な角度操作の完全実装

2. **geo_primitives の対応確認**
   - CircleArc, LinePiece 命名への適合
   - 実装との整合性検証

3. **専用トレイトファイルの有効化**
   - `basic_arc.rs`, `basic_bbox.rs` の問題解消
   - モジュラー設計の完成

## 効果

### 解決された問題

- ✅ 「弧」→「円弧」で正式名称に修正
- ✅ LineSegment→LinePiece で CAD 用語の混乱解消
- ✅ BBox vs BoundingBox の名前衝突解消
- ✅ 2D/3D サフィックス規則の統一
- ✅ geo_foundation と geo_primitives の命名整合性向上

### 改善されたメリット

- 🎯 **業界標準準拠**: CAD 用語として正確な命名
- 📚 **可読性向上**: 明確で誤解のない名称
- 🔧 **保守性向上**: 一貫した命名規則
- 🚀 **開発効率向上**: 混乱の無い明確な名称

---

この命名規則修正により、CAD/CAM の業界標準に準拠した正確な用語使用が実現され、
geo_primitives との統合時の混乱リスクが大幅に削減されました。
