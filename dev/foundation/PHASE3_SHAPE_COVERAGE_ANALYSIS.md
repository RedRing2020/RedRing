# Phase 3 交差判定・衝突判定 実装状況調査

**作成日**: 2025年12月21日

## 概要

Phase 3 では全ての形状の交差判定実装が対応範囲となっていましたが、実際には一部の形状のみ実装されています。本ドキュメントでは、実装状況を整理し、未対応の形状を明確化します。

## 実装状況サマリー

### ✅ 実装済み (16形状)

以下の形状は collision と intersection の両方が実装されています：

#### 2D形状 (8形状)
1. `Arc2D` - 円弧
2. `Circle2D` - 円
3. `Ellipse2D` - 楕円
4. `InfiniteLine2D` - 無限直線
5. `LineSegment2D` - 線分
6. `Ray2D` - 半直線
7. `Triangle2D` - 三角形
8. *(注: Point2D は衝突判定の主体ではなく対象となるため除外)*

#### 3D形状 (8形状)
1. `Arc3D` - 3D円弧
2. `Circle3D` - 3D円
3. `Ellipse3D` - 3D楕円
4. `InfiniteLine3D` - 3D無限直線
5. `LineSegment3D` - 3D線分
6. `Plane3D` - 平面
7. `Ray3D` - 3D半直線
8. `SphericalSurface3D` - 球面
9. `Triangle3D` - 3D三角形

### ❌ 未実装 (14形状)

以下の形状は collision と intersection が未実装です：

#### 2D形状 (1形状)
1. `EllipseArc2D` - 楕円弧 **(要実装)**

#### 3D形状 (13形状)

##### 曲線系 (1形状)
1. `EllipseArc3D` - 3D楕円弧 **(要実装)**

##### 曲面系 (5形状)
2. `ConicalSurface3D` - 円錐曲面 **(要実装)**
3. `CylindricalSurface3D` - 円筒曲面 **(要実装)**
4. `EllipsoidalSurface3D` - 楕円体曲面 **(要実装)**
5. `TorusSurface3D` - トーラス曲面 **(要実装)**

##### 立体系 (5形状)
6. `ConicalSolid3D` - 円錐立体 **(要実装)**
7. `CylindricalSolid3D` - 円筒立体 **(要実装)**
8. `SphericalSolid3D` - 球立体 **(要実装)**
9. `TorusSolid3D` - トーラス立体 **(要実装)**

##### メッシュ系 (1形状)
10. `TriangleMesh3D` - 三角形メッシュ **(要実装)**

#### プリミティブ型 (対象外)
- `Point2D`, `Point3D` - 点（衝突判定の主体ではなく対象）
- `Vector2D`, `Vector3D` - ベクトル（幾何オブジェクトではない）
- `Direction2D`, `Direction3D` - 方向（幾何オブジェクトではない）

## 優先度分析

### 高優先度 (CAD/CAMで頻繁に使用)

1. **EllipseArc2D / EllipseArc3D**
   - 円弧の次に重要な曲線要素
   - Arc と同様のパターンで実装可能

2. **CylindricalSurface3D / CylindricalSolid3D**
   - CAD/CAM で最も一般的な3D形状の一つ
   - 穴、軸などの表現に必須

3. **ConicalSurface3D / ConicalSolid3D**
   - テーパー、面取りなどで使用
   - 円筒と並んで重要

### 中優先度

4. **SphericalSolid3D**
   - SphericalSurface3D が実装済みなので、追加は比較的容易

5. **TorusSurface3D / TorusSolid3D**
   - フィレット、ブレンド面で使用
   - やや複雑だが重要

### 低優先度（特殊用途）

6. **EllipsoidalSurface3D**
   - 特殊な形状、使用頻度は低い

7. **TriangleMesh3D**
   - メッシュ対メッシュの衝突は別の専門的なアルゴリズムが必要
   - 通常は BVH などの加速構造と組み合わせる

## 実装の進め方の提案

### ステップ1: 楕円弧対応
- `EllipseArc2D` の collision/intersection 実装
- `EllipseArc3D` の collision/intersection 実装

### ステップ2: 円筒・円錐系対応
- `CylindricalSurface3D` の collision/intersection 実装
- `CylindricalSolid3D` の collision/intersection 実装
- `ConicalSurface3D` の collision/intersection 実装
- `ConicalSolid3D` の collision/intersection 実装

### ステップ3: 球・トーラス系対応
- `SphericalSolid3D` の collision/intersection 実装
- `TorusSurface3D` の collision/intersection 実装
- `TorusSolid3D` の collision/intersection 実装

### ステップ4: 特殊形状対応
- `EllipsoidalSurface3D` の collision/intersection 実装
- `TriangleMesh3D` の collision/intersection 実装（BVH統合検討）

## Phase 3 完了報告との整合性

Phase 3 完了報告 (`PHASE3_COMPLETION_REPORT.md`) では「すべての形状の交差判定実装」と記載されていますが、実際には以下のように理解すべきです：

- **実装完了**: 基本的な幾何形状（直線、円、楕円、球面、平面、三角形など）
- **未対応**: 曲面系（円筒、円錐、トーラス、楕円体）、立体系、メッシュ系

Phase 3 の本来の目標は「交差判定の基本フレームワークと主要形状の実装」であり、全形状の網羅ではなかったと考えられます。

## 今後の対応方針

1. **Phase 3 の範囲の明確化**
   - 完了報告を修正し、実際の実装範囲を正確に記載
   - 未実装の形状は Phase 4 または別のタスクとして管理

2. **段階的な実装**
   - 優先度に基づき、高頻度で使用される形状から順次実装
   - 各形状の実装時には十分なテストケースを作成

3. **アーキテクチャの維持**
   - 既存の collision/intersection パターンを踏襲
   - Foundation パターンとの整合性を保つ

## 参考情報

- 実装済み形状の例: [ellipse_3d_collision.rs](../../model/geo_primitives/src/ellipse_3d_collision.rs)
- 実装済み形状の例: [ellipse_3d_intersection.rs](../../model/geo_primitives/src/ellipse_3d_intersection.rs)
- Phase 3 完了報告: [PHASE3_COMPLETION_REPORT.md](PHASE3_COMPLETION_REPORT.md)
