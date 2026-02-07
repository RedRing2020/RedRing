# RedRing 開発ロードマップ 2026 Q1-Q3

**作成日**: 2026年2月8日  
**最終更新日**: 2026年2月8日  
**対象期間**: 2026年2月〜2026年8月  
**関連ドキュメント**: [PHASE3_COMPLETION_REPORT.md](PHASE3_COMPLETION_REPORT.md), [PHASE4_TOPOLOGY_ENTITY_DESIGN.md](../architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md)

---

## 📋 目次

1. [優先順位マトリクス](#優先順位マトリクス)
2. [Tier 1: 表示UX完成（最優先）](#tier-1-表示ux完成最優先)
3. [Tier 2: 幾何形状処理の完成度向上（高優先）](#tier-2-幾何形状処理の完成度向上高優先)
4. [Tier 3: 3DCAM基本機能（中優先）](#tier-3-3dcam基本機能中優先)
5. [Tier 4: 高度な幾何演算・UI機能（中長期）](#tier-4-高度な幾何演算ui機能中長期)
6. [実施スケジュール](#実施スケジュール)

---

## 優先順位マトリクス

| Tier | 領域 | 項目 | 工数見積 | 優先理由 |
|------|------|------|----------|----------|
| 🔴 1 | 表示UX | [形状可視化システム完成](#11-形状可視化システム完成-issue-188-phase-2) | 2-3日 | ユーザーフィードバックループ構築 |
| 🔴 1 | 表示UX | [CAM可視化システム設計・実装](#12-cam可視化システム設計実装) | 1週間 | CAM開発の前提条件 |
| 🟠 2 | 幾何処理 | [レガシーAPI問題解決](#21-レガシーapi問題解決) | 1週間 | アーキテクチャ一貫性 |
| 🟠 2 | 幾何処理 | [CAD必須形状の交差判定](#22-cad必須形状の交差判定実装) | 2週間 | CAD/CAM頻出形状対応 |
| 🟡 3 | CAM基本 | [テセレーション機能](#31-テセレーション機能-issue-42) | 1-2週間 | 描画とCAM演算の橋渡し |
| 🟡 3 | CAM基本 | [2D輪郭線オフセット](#32-2d輪郭線オフセット-issue-40) | 1週間 | CAM最基礎機能 |
| 🟡 3 | CAM基本 | [3Dメッシュオフセット](#33-3dメッシュオフセット-issue-41) | 1-2週間 | 荒加工・仕上げ基盤 |
| � 3 | 形状基盤 | [トポロジー層実装](#34-トポロジー層実装-phase-41) | 3週間 | B-Rep基盤構築 |
| 🟡 3 | 形状基盤 | [エンティティ層実装](#35-エンティティ層実装-phase-42) | 2週間 | 属性・メタデータ管理 |
| 🟡 3 | 形状基盤 | [パラメータ管理層実装](#36-パラメータ管理層実装-phase-43) | 1週間 | トレランス管理 |
| �🟢 4 | 高度機能 | [境界ボックス最適化](#41-境界ボックス最適化-issue-43) | 1週間 | 空間計算高速化 |
| 🟢 4 | 高度機能 | [Octree空間分割](#42-octree空間分割-issue-44) | 2週間 | CAM演算効率化 |
| 🟢 4 | 高度機能 | [ブール演算統合](#43-ブール演算統合-issue-33-39) | 3週間 | CAD編集機能 |
| 🟢 4 | UI強化 | [アプリケーションUI](#44-アプリケーションui強化) | 4週間 | ユーザビリティ向上 |

---

## Tier 1: 表示UX完成（最優先）

### 1.1 形状可視化システム完成 (Issue #188 Phase 2)

**現状**: ViewModel層でのデータ変換のみ完了、画面表示は未実装

#### ステップ1: 技術調査・設計（0.5日）

**調査項目**:
- wgpu の `PrimitiveTopology::LineList` 最適な使用方法
- 既存 MeshResources との共通化可能な箇所
- シェーダのユニフォーム共有方式

**成果物**:
- 技術調査レポート（Markdown）
- LineResources の設計仕様書

**完了条件**:
- 実装方針が明確化
- 既存コードとの整合性確認完了

#### ステップ2: View層実装（1日）

**実装項目**:
1. `view/render/src/line.rs`
   - `LineResources` 構造体実装
   - `update_line_data()` メソッド
   - `render()` メソッド

2. `view/render/shaders/line.wgsl`
   - 頂点シェーダ（カメラ変換）
   - フラグメントシェーダ（白色固定 → 後に色指定対応）

3. `view/stage/src/mesh_stage.rs`
   - `RenderMode::Lines` 対応
   - LineResources との統合

**テスト項目**:
- LineResources の単体テスト
- シェーダコンパイル確認

#### ステップ3: アプリケーション層統合（0.5-1日）

**実装項目**:
1. `view/app/src/app_state.rs`
   - デバッグキーバインド追加
     - `L` キー: LineSegment3D 表示
     - `C` キー: Circle3D 表示
     - `T` キー: Triangle3D 表示
     - `A` キー: Arc3D 表示
   - ViewModel 変換呼び出し
   - MeshStage へのデータ受け渡し

**テスト項目**:
- エンドツーエンド動作確認
- 各形状の描画確認
- スクリーンショット記録

#### ステップ4: パフォーマンス検証（0.5日）

**検証項目**:
- 1000形状描画での FPS 測定
- GPU メモリ使用量確認
- 頂点バッファ更新オーバーヘッド測定

**成果物**:
- パフォーマンスレポート
- 最適化が必要な箇所のリスト

**総工数**: 2.5-3日  
**GitHub Issue**: #188 (既存)

---

### 1.2 CAM可視化システム設計・実装

**背景**: CAM演算結果の確認・デバッグには専用の可視化機能が必要

#### ステップ1: 要件定義・技術調査（1日）

**要件定義**:
- 表示対象
  - 工具経路（ToolPath）の線分表示
  - オフセット結果（2D輪郭、3Dメッシュ）
  - 工具位置・姿勢の可視化
  - 加工シミュレーション結果
- 表示モード
  - ワイヤーフレーム
  - ソリッドカラー
  - 工具径表示（円筒表示）
- インタラクション
  - パス上の任意点選択
  - パラメータ値表示
  - アニメーション再生

**技術調査項目**:
- 大量パス表示の最適化手法（インスタンシング、LOD）
- 工具形状の効率的な描画方法
- アニメーション制御のアーキテクチャ

**成果物**:
- 要件定義書（`dev/architecture/CAM_VISUALIZATION_REQUIREMENTS.md`）
- 技術調査レポート

#### ステップ2: アーキテクチャ設計（1日）

**設計項目**:
1. **データモデル**
   ```rust
   // model/geo_algorithms/src/toolpath.rs
   pub struct ToolPath<T: Scalar> {
       segments: Vec<PathSegment<T>>,
       tool_diameter: T,
       feed_rate: T,
       metadata: PathMetadata,
   }
   
   pub enum PathSegment<T: Scalar> {
       Linear(LineSegment3D<T>),
       Arc(Arc3D<T>),
       Rapid(LineSegment3D<T>), // 早送り
   }
   ```

2. **ViewModel層変換**
   ```rust
   // viewmodel/converter/src/toolpath_converter.rs
   pub fn toolpath_to_vertices(
       path: &ToolPath<f64>,
       options: &VisualizationOptions,
   ) -> ToolPathVertices;
   ```

3. **View層レンダリング**
   ```rust
   // view/render/src/toolpath.rs
   pub struct ToolPathResources {
       path_pipeline: wgpu::RenderPipeline,
       tool_pipeline: wgpu::RenderPipeline,
       // ...
   }
   ```

4. **Stage管理**
   ```rust
   // view/stage/src/toolpath_stage.rs
   pub struct ToolPathStage {
       resources: ToolPathResources,
       animation_state: AnimationState,
       // ...
   }
   ```

**成果物**:
- アーキテクチャ設計書（`dev/architecture/CAM_VISUALIZATION_DESIGN.md`）
- インターフェース定義

#### ステップ3: 基本実装（2日）

**実装優先順位**:
1. **Phase 1**: 工具経路の線分表示（最低限）
   - 線分ベースのパス描画
   - 早送り/切削の色分け

2. **Phase 2**: オフセット結果表示
   - 2D輪郭オフセット表示
   - 3Dメッシュオフセット表示

3. **Phase 3**: 工具形状表示（後回し可）
   - 円筒工具の簡易表示
   - 工具位置アニメーション

**実装ファイル**:
- `model/geo_algorithms/src/toolpath.rs` (新規)
- `viewmodel/converter/src/toolpath_converter.rs` (新規)
- `view/render/src/toolpath.rs` (新規)
- `view/render/shaders/toolpath.wgsl` (新規)
- `view/stage/src/toolpath_stage.rs` (新規)

#### ステップ4: 統合テスト（1日）

**テスト項目**:
- サンプル工具経路の描画確認
- オフセット結果の視覚的検証
- パフォーマンステスト（1000パスセグメント）

**成果物**:
- テストスイート
- サンプルデータセット

**総工数**: 5日（1週間）  
**GitHub Issue**: 新規作成予定

---

## Tier 2: 幾何形状処理の完成度向上（高優先）

### 2.1 レガシーAPI問題解決

**現状**: 10以上の形状でレガシーAPIとFoundation実装が共存

#### ステップ1: 影響範囲調査（0.5日）

**調査項目**:
- レガシーメソッドの使用箇所全リストアップ
- 各メソッドの Foundation 対応トレイト特定
- 移行優先度の決定

**対象形状**:
- point, vector, circle, ellipse_arc, direction
- bbox, ray, line_segment, infinite_line

**成果物**:
- 影響範囲調査レポート
- 移行計画書

#### ステップ2: 段階的移行（4-5日）

**移行戦略**:
1. **Phase 1**: 高頻度使用形状（Circle, Ellipse）
2. **Phase 2**: 中頻度形状（Ray, LineSegment）
3. **Phase 3**: 低頻度形状（BBox, Direction）

**各形状の作業内容**:
1. レガシーメソッドを `pub fn` → `fn` に変更
2. Foundation トレイト実装を優先APIとして公開
3. 内部実装の共通化・最適化
4. テスト整備

**例: Circle2D の移行**:
```rust
// Before (レガシー)
impl<T: Scalar> Circle2D<T> {
    pub fn center(&self) -> (T, T) { ... }  // 競合！
}

// After (Foundation優先)
impl<T: Scalar> Circle2D<T> {
    fn legacy_center(&self) -> (T, T) { ... }  // 内部化
}

impl<T: Scalar> Circle2DProperties<T> for Circle2D<T> {
    fn center(&self) -> Point2D<T> { ... }  // 公開API
}
```

#### ステップ3: 検証・ドキュメント更新（0.5日）

**検証項目**:
- 全テスト通過確認
- 破壊的変更のリストアップ
- マイグレーションガイド作成

**成果物**:
- Migration Guide（`dev/foundation/LEGACY_API_MIGRATION_GUIDE.md`）
- 更新された Foundation Pattern ドキュメント

**総工数**: 5-6日（1週間）  
**GitHub Issue**: 新規作成予定

---

### 2.2 CAD必須形状の交差判定実装

**現状**: 14形状の collision/intersection が未実装

#### ステップ1: アーキテクチャ再確認（0.5日）

**確認項目**:
- 既存実装パターンの整理（Ellipse3D を参考）
- テストケース設計方針の統一
- エラーハンドリング戦略

**成果物**:
- 実装テンプレート
- テストケース設計ガイド

#### ステップ2: 高優先度形状実装（8-9日）

**実装順序**:

1. **EllipseArc2D / EllipseArc3D**（2日）
   - 円弧と類似パターンで実装可能
   - ファイル:
     - `model/geo_primitives/src/ellipse_arc_2d_collision.rs`
     - `model/geo_primitives/src/ellipse_arc_2d_intersection.rs`
     - `model/geo_primitives/src/ellipse_arc_3d_collision.rs`
     - `model/geo_primitives/src/ellipse_arc_3d_intersection.rs`

2. **CylindricalSurface3D / CylindricalSolid3D**（3日）
   - CAD/CAM最頻出形状
   - 直線との交差が重要
   - ファイル:
     - `model/geo_primitives/src/cylindrical_surface_3d_collision.rs`
     - `model/geo_primitives/src/cylindrical_surface_3d_intersection.rs`
     - `model/geo_primitives/src/cylindrical_solid_3d_collision.rs`
     - `model/geo_primitives/src/cylindrical_solid_3d_intersection.rs`

3. **ConicalSurface3D / ConicalSolid3D**（3-4日）
   - テーパー形状で重要
   - 円錐の2次曲面方程式処理が必要
   - ファイル:
     - `model/geo_primitives/src/conical_surface_3d_collision.rs`
     - `model/geo_primitives/src/conical_surface_3d_intersection.rs`
     - `model/geo_primitives/src/conical_solid_3d_collision.rs`
     - `model/geo_primitives/src/conical_solid_3d_intersection.rs`

**各形状の実装内容**:
- `BasicCollision<T, Other>` 実装
- `PointDistance<T>` 実装
- `BBoxCollision<T>` 実装
- `BasicIntersection<T, Other>` 実装
- `MultipleIntersection<T, Other>` 実装
- 包括的なテストスイート

#### ステップ3: テスト・検証（1日）

**テスト項目**:
- 全形状の単体テスト
- 組み合わせテスト（円筒×平面、円錐×直線など）
- エッジケーステスト

**総工数**: 10日（2週間）  
**GitHub Issue**: 新規作成予定（形状ごとに分割可能）

---

## Tier 3: 3DCAM基本機能（中優先）

### 3.1 テセレーション機能 (Issue #42)

#### ステップ1: アーキテクチャ設計（1日）

**設計項目**:
1. **トレイト定義**
   ```rust
   // model/geo_foundation/src/extensions/tessellation.rs
   pub trait Tessellate<T: Scalar> {
       type Output;
       
       fn tessellate(&self, quality: TessellationQuality) -> Self::Output;
       fn tessellate_with_tolerance(&self, tolerance: T) -> Self::Output;
   }
   
   pub enum TessellationQuality {
       Draft,      // 粗い（表示用）
       Standard,   // 標準
       Fine,       // 高精度（CAM用）
       Custom(TessellationParams),
   }
   ```

2. **精度管理**
   - f32: 表示用（0.01mm トレランス）
   - f64: CAM演算用（0.001mm トレランス）

3. **曲率適応サンプリング**
   - 平坦部分: 粗い分割
   - 高曲率部分: 細かい分割

**成果物**:
- テセレーション設計書（`dev/foundation/TESSELLATION_DESIGN.md`）

#### ステップ2: プリミティブ形状実装（3日）

**実装対象**:
- Circle2D/3D
- Ellipse2D/3D
- Arc2D/3D
- SphericalSurface3D
- CylindricalSurface3D

**実装例**:
```rust
impl<T: Scalar> Tessellate<T> for Circle3D<T> {
    type Output = Vec<LineSegment3D<T>>;
    
    fn tessellate(&self, quality: TessellationQuality) -> Self::Output {
        let segment_count = match quality {
            TessellationQuality::Draft => 16,
            TessellationQuality::Standard => 32,
            TessellationQuality::Fine => 64,
            TessellationQuality::Custom(p) => p.segment_count,
        };
        // 円周を segment_count 個の線分に分割
    }
}
```

#### ステップ3: NURBS曲面実装（4-5日）

**実装項目**:
- NurbsCurve2D/3D のテセレーション
- NurbsSurface3D のテセレーション
- 曲率ベースの適応サンプリング

**技術課題**:
- ノット挿入アルゴリズム
- 曲率計算の数値安定性
- メッシュ品質の保証

#### ステップ4: テスト・検証（1日）

**テスト項目**:
- トレランス精度検証
- 視覚的品質確認
- パフォーマンス測定（大規模曲面）

**総工数**: 9-10日（1.5-2週間）  
**GitHub Issue**: #42 (既存)

---

### 3.2 2D輪郭線オフセット (Issue #40)

#### ステップ1: アルゴリズム調査・選定（1日）

**調査項目**:
- クリッパーアルゴリズム（Clipper2）
- Boost.Geometry のオフセット
- 自前実装の可能性

**検討内容**:
- 外部ライブラリ vs 自前実装
- トポロジー整合性の保証方法
- 自己交差の処理

**成果物**:
- アルゴリズム調査レポート
- 実装方針決定書

#### ステップ2: 基本実装（2日）

**実装項目**:
1. **データ構造**
   ```rust
   // model/geo_algorithms/src/offset_2d.rs
   pub struct Contour2D<T: Scalar> {
       segments: Vec<ContourSegment<T>>,
       is_closed: bool,
   }
   
   pub enum ContourSegment<T: Scalar> {
       Line(LineSegment2D<T>),
       Arc(Arc2D<T>),
       EllipseArc(EllipseArc2D<T>),
   }
   
   pub struct OffsetResult2D<T: Scalar> {
       contours: Vec<Contour2D<T>>,
       islands: Vec<Contour2D<T>>,  // 内部輪郭
   }
   ```

2. **オフセット処理**
   ```rust
   pub fn offset_contour_2d<T: Scalar>(
       contour: &Contour2D<T>,
       distance: T,
       tolerance: T,
   ) -> Result<OffsetResult2D<T>, OffsetError>;
   ```

#### ステップ3: 工具径補正との統合（1日）

**実装項目**:
- 工具径パラメータの取り扱い
- 左右オフセットの制御
- 複数輪郭の処理

#### ステップ4: テスト（1日）

**テスト項目**:
- 単純形状のオフセット
- 複雑形状（自己交差含む）
- エッジケース（極小R）

**総工数**: 5日（1週間）  
**GitHub Issue**: #40 (既存)

---

### 3.3 3Dメッシュオフセット (Issue #41)

#### ステップ1: アルゴリズム調査・設計（2日）

**調査項目**:
- 頂点法線ベースのオフセット
- ラプラシアンスムージング
- メッシュ品質維持手法

**設計項目**:
- TriangleMesh3D の拡張
- 法線計算・平滑化
- 自己交差検出・修正

**成果物**:
- 3Dオフセット設計書（`dev/foundation/3D_OFFSET_DESIGN.md`）

#### ステップ2: 基本実装（3-4日）

**実装項目**:
1. **データ構造**
   ```rust
   // model/geo_algorithms/src/offset_3d.rs
   pub struct MeshOffset3D<T: Scalar> {
       mesh: TriangleMesh3D<T>,
       normals: Vec<Vector3D<T>>,
   }
   
   impl<T: Scalar> MeshOffset3D<T> {
       pub fn offset(&self, distance: T) -> Result<TriangleMesh3D<T>, OffsetError>;
       pub fn smooth_normals(&mut self, iterations: usize);
   }
   ```

2. **オフセット処理**
   - 頂点法線計算
   - オフセット適用
   - メッシュ品質チェック

#### ステップ3: テスト・検証（1-2日）

**テスト項目**:
- 単純メッシュ（立方体、球）
- 複雑メッシュ（NURBS曲面テセレーション結果）
- パフォーマンステスト

**総工数**: 6-8日（1-1.5週間）  
**GitHub Issue**: #41 (既存)

---

### 3.4 トポロジー層実装 (Phase 4.1)

**背景**: 現在の幾何層は数学的な形状のみを扱い、形状間の接続関係（トポロジー）が欠けている。CADシステムとして不可欠な境界表現（B-Rep）を実装する。

#### ステップ1: アーキテクチャ設計・調査（1週間）

**調査項目**:
- OpenCASCADE の TopoDS 設計
- Parasolid の PK 仕様
- B-Rep 理論と実装パターン

**設計項目**:
- トポロジー階層（Vertex/Edge/Wire/Face/Shell/Solid）
- トポロジーID管理システム
- トポロジー整合性検証方式

**成果物**:
- トポロジー設計書（`dev/architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md`）
- データ構造設計

#### ステップ2: 低レベル構造実装（1週間）

**実装項目**:
1. **`geo_topology` クレート作成**
2. **Vertex実装**
   - `model/geo_topology/src/vertex.rs`
   - Point3D への参照
   - 位置トレランス管理
3. **Edge実装**
   - `model/geo_topology/src/edge.rs`
   - 曲線への参照（CurveRef）
   - パラメータ範囲管理
4. **Wire実装**
   - `model/geo_topology/src/wire.rs`
   - エッジの接続性検証
   - 閉じ性チェック

**テスト項目**:
- 頂点の同一性判定
- エッジの接続性テスト
- ワイヤーの閉じ性テスト

#### ステップ3: 高レベル構造実装（1週間）

**実装項目**:
1. **Face実装**
   - `model/geo_topology/src/face.rs`
   - 曲面への参照（SurfaceRef）
   - 外側・内側境界ループ
2. **Shell実装**
   - `model/geo_topology/src/shell.rs`
   - 面の集合
   - 閉じ性判定
3. **Solid実装**
   - `model/geo_topology/src/solid.rs`
   - シェルで囲まれた体積
   - 多重殻（空洞）対応

**テスト項目**:
- 面の境界ループ検証
- シェルの閉じ性テスト
- 立体の整合性テスト

#### ステップ4: オイラー操作実装（0.5-1週間）

**実装項目**:
- `model/geo_topology/src/operations.rs`
- MEV (Make Edge Vertex) - 辺と頂点を作成
- MEL (Make Edge Loop) - ループで辺を作成
- KEV (Kill Edge Vertex) - 辺と頂点を削除

**テスト項目**:
- オイラー操作の整合性テスト
- トポロジー不変量（Euler-Poincaré）の検証

**総工数**: 15-20日（3-4週間）  
**GitHub Issue**: 新規作成予定

---

### 3.5 エンティティ層実装 (Phase 4.2)

**背景**: トポロジーに属性・メタデータを付与し、アプリケーション層で扱いやすいエンティティシステムを構築する。

#### ステップ1: 属性システム設計（0.5週間）

**設計項目**:
- 属性キーの命名規則（system.*, app.*）
- 属性値の型システム（String, Integer, Float, Boolean, Color, Custom）
- 属性の永続化方式

**成果物**:
- 属性システム設計書

#### ステップ2: エンティティ基盤実装（1週間）

**実装項目**:
1. **`geo_entity` クレート作成**
2. **属性システム**
   - `model/geo_entity/src/attributes.rs`
   - AttributeKey, AttributeValue
   - Attributes コンテナ
3. **エンティティID管理**
   - `model/geo_entity/src/entity_id.rs`
   - UUID ベースの識別子
4. **エンティティ実装**
   - `model/geo_entity/src/solid_entity.rs`
   - `model/geo_entity/src/face_entity.rs`
   - `model/geo_entity/src/edge_entity.rs`
   - `model/geo_entity/src/vertex_entity.rs`

**テスト項目**:
- 属性の設定・取得
- エンティティIDの一意性
- エンティティの複製

#### ステップ3: モデル管理実装（0.5週間）

**実装項目**:
- `model/geo_entity/src/model.rs`
- エンティティのコレクション管理
- 名前・属性によるエンティティ検索
- シリアライゼーション準備

**テスト項目**:
- エンティティの追加・削除・検索
- 名前による検索
- 属性フィルタリング

**総工数**: 10日（2週間）  
**GitHub Issue**: 新規作成予定

---

### 3.6 パラメータ管理層実装 (Phase 4.3)

**背景**: アプリケーション層からトレランス・精度などのパラメータを受け渡す基盤を構築する。

#### ステップ1: トレランス設計（0.25週間）

**設計項目**:
- 位置精度、角度精度、曲率精度の定義
- 精度モード（Display/Standard/CamPrecision）
- デフォルト値の決定

**成果物**:
- トレランス設計書

#### ステップ2: トレランス実装（0.5週間）

**実装項目**:
- `model/geo_foundation/src/tolerance.rs`
- ToleranceSettings 構造体
- PrecisionMode 列挙型
- ApplicationContext 構造体

**テスト項目**:
- トレランス値の妥当性
- 精度モードの切り替え

#### ステップ3: コンテキスト統合（0.25週間）

**実装項目**:
- `viewmodel/converter/src/context.rs`
- ConversionContext 実装
- アプリケーション層からの設定受け渡し

**テスト項目**:
- アプリケーション層からのパラメータ伝達
- トレランスの適用確認

**総工数**: 5日（1週間）  
**GitHub Issue**: 新規作成予定

---

## Tier 4: 高度な幾何演算・UI機能（中長期）

### 4.1 境界ボックス最適化 (Issue #43)

**工数**: 5日（1週間）  
**内容**: AABB保持機能、空間インデックス基盤

### 4.2 Octree空間分割 (Issue #44)

**工数**: 10日（2週間）  
**内容**: 空間分割構造、CAM演算効率化

### 4.3 ブール演算統合 (Issue #33-39)

**工数**: 15日（3週間）  
**内容**: Union/Subtract/Intersect、トポロジー連携

### 4.4 アプリケーションUI強化

**工数**: 20日（4週間）  
**内容**: メニュー、コマンドパレット、ファイルI/O、STEP/IGES対応

---

## 実施スケジュール

### 2026年2月（Week 1-4）

**Week 1** (2/8-2/14):
- ✅ Issue #188 Phase 2 完了（形状可視化）
- アーキテクチャ設計: CAM可視化システム

**Week 2** (2/15-2/21):
- CAM可視化システム実装開始
- レガシーAPI問題調査

**Week 3** (2/22-2/28):
- CAM可視化システム実装完了
- レガシーAPI移行開始（Circle, Ellipse）

**Week 4** (2/29-3/6):
- レガシーAPI移行継続

---

### 2026年3月（Week 5-8）

**Week 5** (3/7-3/13):
- レガシーAPI移行完了
- EllipseArc2D/3D 交差判定実装開始

**Week 6** (3/14-3/20):
- EllipseArc 交差判定完了
- CylindricalSurface/Solid 交差判定開始

**Week 7** (3/21-3/27):
- CylindricalSurface/Solid 交差判定継続

**Week 8** (3/28-4/3):
- ConicalSurface/Solid 交差判定開始

---

### 2026年4月（Week 9-12）

**Week 9** (4/4-4/10):
- ConicalSurface/Solid 交差判定完了
- テセレーション設計開始

**Week 10** (4/11-4/17):
- テセレーション: プリミティブ実装

**Week 11** (4/18-4/24):
- テセレーション: NURBS実装

**Week 12** (4/25-5/1):
- テセレーション完了・検証

---

### 2026年5月（Week 13-16）

**Week 13** (5/2-5/8):
- 2D輪郭線オフセット調査・設計

**Week 14** (5/9-5/15):
- 2D輪郭線オフセット実装

**Week 15** (5/16-5/22):
- 2D輪郭線オフセット完了
- 3Dメッシュオフセット調査

**Week 16** (5/23-5/29):
- 3Dメッシュオフセット設計

---

### 2026年6月（Week 17-20）

**Week 17** (5/30-6/5):
- 3Dメッシュオフセット実装

**Week 18** (6/6-6/12):
- 3Dメッシュオフセット実装継続

**Week 19** (6/13-6/19):
- 3Dメッシュオフセット完了・検証

**Week 20** (6/20-6/26):
- Q1-Q2 レビュー・Q3計画策定

### Phase 4: トポロジー・エンティティ層（7-8月）

**Week 21** (7/1-7/7):
- Phase 4.1 トポロジー層設計・基本データ構造実装開始

**Week 22** (7/8-7/14):
- トポロジーAPI実装（Vertex/Edge/Wire）

**Week 23** (7/15-7/21):
- トポロジーAPI実装（Face/Shell/Solid）・Euler操作実装

**Week 24** (7/22-7/28):
- Phase 4.2 エンティティ層設計・基本実装

**Week 25** (7/29-8/4):
- エンティティ属性システム実装・UUID管理

**Week 26** (8/5-8/11):
- Phase 4.3 パラメータ管理層実装・完了

**Week 27** (8/12-8/18):
- Phase 4 統合テスト・ドキュメント整備

**Week 28** (8/19-8/25):
- Phase 4 検証・Q3計画策定

---

## GitHub Issue テンプレート

各マイルストーン項目を Issue 化する際のテンプレート：

```markdown
## 概要
[機能の簡潔な説明]

## 背景・目的
[なぜこの機能が必要か]

## 実装ステップ

### ステップ1: 技術調査・設計（X日）
**調査項目**:
- [ ] [調査項目1]
- [ ] [調査項目2]

**設計項目**:
- [ ] [設計項目1]
- [ ] [設計項目2]

**成果物**:
- [ ] 技術調査レポート
- [ ] 設計仕様書

### ステップ2: 実装（X日）
**実装項目**:
- [ ] [ファイル名1]: [実装内容]
- [ ] [ファイル名2]: [実装内容]

**テスト項目**:
- [ ] [テスト項目1]
- [ ] [テスト項目2]

### ステップ3: 統合・検証（X日）
**検証項目**:
- [ ] [検証項目1]
- [ ] [検証項目2]

**成果物**:
- [ ] [成果物1]
- [ ] [成果物2]

## 完了条件
- [ ] [条件1]
- [ ] [条件2]

## 関連情報
- 関連Issue: #xxx
- 設計ドキュメント: [リンク]
- 参考実装: [リンク]

## 工数見積
合計: X日
```

---

## 次のアクション

1. **即座に着手**:
   - Issue #188 Phase 2 の実装開始

2. **今週中に完了**:
   - CAM可視化システムの要件定義
   - GitHub Issue の作成（以下を個別Issue化）
     - CAM可視化システム設計・実装
     - レガシーAPI問題解決
     - EllipseArc交差判定
     - Cylindrical形状交差判定
     - Conical形状交差判定
     - テセレーション機能
     - 2D輪郭線オフセット
     - 3Dメッシュオフセット

3. **来週着手**:
   - CAM可視化システムの設計開始
   - レガシーAPI影響範囲調査

---

## レビューサイクル

- **週次レビュー**: 毎週金曜日に進捗確認
- **月次レビュー**: 各月末に成果確認・計画調整
- **四半期レビュー**: Q1終了時（3月末）、Q2終了時（6月末）

**次回レビュー**: 2026年2月14日（Week 1完了時）
