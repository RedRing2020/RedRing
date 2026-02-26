# GitHub Issue テンプレート集

**作成日**: 2026年2月8日  
**用途**: ROADMAP_2026_Q1_Q2.md の各項目をIssue化する際のテンプレート

---

## Issue #1: CAM可視化システム設計・実装

```markdown
## 概要
CAM演算結果（工具経路、オフセット結果）を視覚的に確認・デバッグするための専用可視化システムを設計・実装する。

## 背景・目的
- CAM機能開発には演算結果の視覚的確認が不可欠
- 工具経路の検証・デバッグ機能が必要
- オフセット結果の正確性を視覚的に確認したい

## 実装ステップ

### ステップ1: 要件定義・技術調査（1日）

**要件定義**:
- [ ] 表示対象の明確化
  - [ ] 工具経路（ToolPath）の線分表示
  - [ ] オフセット結果（2D輪郭、3Dメッシュ）
  - [ ] 工具位置・姿勢の可視化
  - [ ] 加工シミュレーション結果
- [ ] 表示モードの定義
  - [ ] ワイヤーフレーム
  - [ ] ソリッドカラー
  - [ ] 工具径表示（円筒表示）
- [ ] インタラクション設計
  - [ ] パス上の任意点選択
  - [ ] パラメータ値表示
  - [ ] アニメーション再生

**技術調査項目**:
- [ ] 大量パス表示の最適化手法
  - [ ] インスタンシング調査
  - [ ] LOD（Level of Detail）実装方法
- [ ] 工具形状の効率的な描画方法
  - [ ] 円筒プリミティブの描画
  - [ ] GPU インスタンシングの活用
- [ ] アニメーション制御のアーキテクチャ
  - [ ] タイムライン管理
  - [ ] 再生速度制御

**成果物**:
- [ ] 要件定義書（`dev/architecture/CAM_VISUALIZATION_REQUIREMENTS.md`）
- [ ] 技術調査レポート（同ファイル内）

### ステップ2: アーキテクチャ設計（1日）

**設計項目**:

**2.1 データモデル設計**:
- [ ] `model/geo_algorithms/src/toolpath.rs` 設計
  ```rust
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

**2.2 ViewModel層変換設計**:
- [ ] `viewmodel/converter/src/toolpath_converter.rs` 設計
  ```rust
  pub fn toolpath_to_vertices(
      path: &ToolPath<f64>,
      options: &VisualizationOptions,
  ) -> ToolPathVertices;
  
  pub struct VisualizationOptions {
      pub show_rapid_moves: bool,
      pub tool_display: ToolDisplayMode,
      pub path_thickness: f32,
  }
  ```

**2.3 View層レンダリング設計**:
- [ ] `view/render/src/toolpath.rs` 設計
  ```rust
  pub struct ToolPathResources {
      path_pipeline: wgpu::RenderPipeline,
      tool_pipeline: wgpu::RenderPipeline,
      vertex_buffer: wgpu::Buffer,
      uniform_buffer: wgpu::Buffer,
  }
  ```

**2.4 Stage管理設計**:
- [ ] `view/stage/src/toolpath_stage.rs` 設計
  ```rust
  pub struct ToolPathStage {
      resources: ToolPathResources,
      animation_state: AnimationState,
      current_position: f64, // 0.0-1.0
  }
  ```

**成果物**:
- [ ] アーキテクチャ設計書（`dev/architecture/CAM_VISUALIZATION_DESIGN.md`）
- [ ] インターフェース定義
- [ ] データフロー図

### ステップ3: 基本実装（2日）

**Phase 1: 工具経路の線分表示（最優先）**:
- [ ] `model/geo_algorithms/src/toolpath.rs` 実装
  - [ ] `ToolPath` 構造体
  - [ ] `PathSegment` enum
  - [ ] 基本的なコンストラクタ
- [ ] `viewmodel/converter/src/toolpath_converter.rs` 実装
  - [ ] `toolpath_to_vertices()` 関数
  - [ ] 線分への変換ロジック
  - [ ] 早送り/切削の色分け
- [ ] `view/render/src/toolpath.rs` 実装
  - [ ] `ToolPathResources` 構造体
  - [ ] パイプライン構築
  - [ ] 描画メソッド
- [ ] `view/render/shaders/toolpath.wgsl` 実装
  - [ ] 頂点シェーダ
  - [ ] フラグメントシェーダ（色分け対応）
- [ ] `view/stage/src/toolpath_stage.rs` 実装
  - [ ] `ToolPathStage` 構造体
  - [ ] `RenderStage` トレイト実装

**Phase 2: オフセット結果表示**:
- [ ] 2D輪郭オフセット表示機能
- [ ] 3Dメッシュオフセット表示機能
- [ ] 既存の線分描画との統合

**Phase 3: 工具形状表示（後回し可）**:
- [ ] 円筒工具の簡易表示
- [ ] 工具位置アニメーション
- [ ] 工具径の視覚化

**テスト項目**:
- [ ] 単体テスト
  - [ ] `ToolPath` の生成・操作
  - [ ] 頂点変換の正確性
- [ ] 統合テスト
  - [ ] サンプル経路の描画確認

### ステップ4: 統合テスト・パフォーマンス検証（1日）

**統合テスト**:
- [ ] サンプル工具経路の描画確認
  - [ ] 単純パス（直線のみ）
  - [ ] 複雑パス（円弧含む）
  - [ ] 大規模パス（1000セグメント以上）
- [ ] オフセット結果の視覚的検証
  - [ ] 2D輪郭オフセット表示
  - [ ] 3Dメッシュオフセット表示
- [ ] アニメーション機能テスト
  - [ ] 再生・一時停止
  - [ ] 速度制御

**パフォーマンステスト**:
- [ ] 1000パスセグメント描画でのFPS測定
- [ ] GPUメモリ使用量確認
- [ ] CPUボトルネックの特定

**成果物**:
- [ ] テストスイート
- [ ] サンプルデータセット
- [ ] パフォーマンスレポート

## 完了条件
- [ ] サンプル工具経路が画面に描画される
- [ ] 早送り/切削が色分けされる
- [ ] 1000セグメント以上を60FPS以上で描画可能
- [ ] オフセット結果が視覚的に確認できる
- [ ] ドキュメントが整備されている

## 関連情報
- 関連Issue: #188 (形状可視化システム)
- 設計ドキュメント: `dev/foundation/ROADMAP_2026_Q1_Q2.md`
- 参考実装: `view/render/src/line.rs`, `view/stage/src/mesh_stage.rs`

## 工数見積
合計: 5日（1週間）

## 優先度
🔴 最優先（Tier 1）

## ラベル
- `enhancement`
- `visualization`
- `cam`
- `tier-1`
```

---

## Issue #2: レガシーAPI問題解決

```markdown
## 概要
10以上の形状で共存しているレガシーAPIとFoundation実装を統一し、Foundation Patternに完全移行する。

## 背景・目的
- メソッド名競合（`center()`, `arc_length()` など）の解消
- 型不一致（`Point2D<T>` vs `(T, T)`）の統一
- Foundation Patternの完全実現
- アーキテクチャの一貫性確保

## 対象形状
- point
- vector
- circle
- ellipse_arc
- direction
- bbox
- ray
- line_segment
- infinite_line

## 実装ステップ

### ステップ1: 影響範囲調査（0.5日）

**調査項目**:
- [ ] レガシーメソッド使用箇所のリストアップ
  - [ ] `grep_search` でワークスペース全体を調査
  - [ ] 各メソッドの使用頻度を集計
- [ ] Foundation対応トレイトの特定
  - [ ] 各レガシーメソッドの対応トレイト確認
  - [ ] 未対応メソッドの洗い出し
- [ ] 移行優先度の決定
  - [ ] 使用頻度ベースの優先順位付け
  - [ ] 依存関係の整理

**成果物**:
- [ ] 影響範囲調査レポート（`dev/foundation/LEGACY_API_ANALYSIS.md`）
  - [ ] 形状別メソッドリスト
  - [ ] 使用箇所マップ
  - [ ] 移行順序の決定
- [ ] 移行計画書（同ファイル内）

### ステップ2: 段階的移行（4-5日）

**Phase 1: 高頻度使用形状（2日）**:
- [ ] Circle2D / Circle3D
  - [ ] `center()` メソッド移行
  - [ ] `radius()` メソッド移行
  - [ ] テスト更新
- [ ] Ellipse2D / Ellipse3D
  - [ ] `center()` メソッド移行
  - [ ] `semi_major_axis()` / `semi_minor_axis()` メソッド移行
  - [ ] テスト更新

**Phase 2: 中頻度形状（1.5日）**:
- [ ] Ray2D / Ray3D
  - [ ] `origin()` メソッド移行
  - [ ] `direction()` メソッド移行
- [ ] LineSegment2D / LineSegment3D
  - [ ] `start()` / `end()` メソッド移行
  - [ ] `length()` メソッド移行

**Phase 3: 低頻度形状（0.5-1日）**:
- [ ] BBox2D / BBox3D
  - [ ] `min()` / `max()` メソッド移行
- [ ] Direction2D / Direction3D
  - [ ] `x()` / `y()` / `z()` メソッド移行

**各形状の作業内容**:
1. [ ] レガシーメソッドを `pub fn` → `fn` に変更
2. [ ] Foundation トレイト実装を公開APIとして確立
3. [ ] 内部実装の共通化・最適化
4. [ ] テスト整備
   - [ ] Foundation トレイト経由のテスト
   - [ ] 境界値テスト
   - [ ] エラーケーステスト

**実装例（Circle2D）**:
```rust
// ❌ Before (レガシー)
impl<T: Scalar> Circle2D<T> {
    pub fn center(&self) -> (T, T) { 
        (self.center_x, self.center_y)
    }
}

// ✅ After (Foundation優先)
impl<T: Scalar> Circle2D<T> {
    // 内部化（外部からアクセス不可）
    fn legacy_center(&self) -> (T, T) { 
        (self.center_x, self.center_y)
    }
}

// Foundation トレイトを公開API化
impl<T: Scalar> Circle2DProperties<T> for Circle2D<T> {
    fn center(&self) -> Point2D<T> { 
        Point2D::new(self.center_x, self.center_y)
    }
}
```

### ステップ3: 検証・ドキュメント更新（0.5日）

**検証項目**:
- [ ] 全テスト通過確認
  - [ ] `cargo test --workspace`
  - [ ] 特定クレートのテスト: `cargo test -p geo_primitives`
- [ ] 破壊的変更のリストアップ
  - [ ] 変更されたメソッドシグネチャ
  - [ ] 削除された公開API
- [ ] コンパイルエラーの確認
  - [ ] `cargo build`
  - [ ] `cargo clippy`

**ドキュメント更新**:
- [ ] Migration Guide作成（`dev/foundation/LEGACY_API_MIGRATION_GUIDE.md`）
  - [ ] 変更一覧表
  - [ ] 移行手順
  - [ ] コード例（Before/After）
- [ ] Foundation Patternドキュメント更新
  - [ ] 新しい使用例の追加
  - [ ] ベストプラクティス記載

**成果物**:
- [ ] 全テスト通過
- [ ] Migration Guide
- [ ] 更新されたFoundation Patternドキュメント

## 完了条件
- [ ] 全ての対象形状でレガシーAPIが内部化されている
- [ ] Foundation トレイト経由のアクセスが標準化されている
- [ ] 全テストが通過している
- [ ] Migration Guideが作成されている
- [ ] ドキュメントが更新されている

## 関連情報
- 関連Issue: なし（新規）
- 設計ドキュメント: `dev/architecture/ARCHITECTURE.md`
- Foundation Pattern: `dev/foundation/FOUNDATION_REFACTORING_PLAN.md`

## 工数見積
合計: 5-6日（1週間）

## 優先度
🟠 高優先（Tier 2）

## ラベル
- `refactoring`
- `architecture`
- `foundation-pattern`
- `tier-2`
```

---

## Issue #3: EllipseArc交差判定実装

```markdown
## 概要
EllipseArc2D および EllipseArc3D に対する collision/intersection 機能を実装する。

## 背景・目的
- 円弧の次に重要な曲線要素
- CAD/CAMで頻繁に使用される形状
- 既存の Arc2D/Arc3D 実装パターンを踏襲可能

## 実装ステップ

### ステップ1: アーキテクチャ確認（0.25日）

**確認項目**:
- [ ] 既存の Arc2D/Arc3D 実装パターン確認
  - [ ] `arc_2d_collision.rs` レビュー
  - [ ] `arc_2d_intersection.rs` レビュー
  - [ ] `arc_3d_collision.rs` レビュー
  - [ ] `arc_3d_intersection.rs` レビュー
- [ ] EllipseArc固有の考慮事項
  - [ ] 楕円パラメータ方程式
  - [ ] 回転変換の扱い
  - [ ] 角度範囲の処理

**成果物**:
- [ ] 実装方針メモ
- [ ] テストケース設計

### ステップ2: EllipseArc2D実装（0.75日）

**実装ファイル**:

**2.1 Collision実装**:
- [ ] `model/geo_primitives/src/ellipse_arc_2d_collision.rs` (新規)
  - [ ] `BasicCollision<T, Point2D<T>>` 実装
  - [ ] `BasicCollision<T, LineSegment2D<T>>` 実装
  - [ ] `BasicCollision<T, Circle2D<T>>` 実装
  - [ ] `PointDistance<T>` 実装
    - [ ] `distance_to_point()`
    - [ ] `contains_point()`
    - [ ] `point_on_boundary()`
    - [ ] `closest_point()`
  - [ ] `BBoxCollision<T>` 実装

**2.2 Intersection実装**:
- [ ] `model/geo_primitives/src/ellipse_arc_2d_intersection.rs` (新規)
  - [ ] `BasicIntersection<T, LineSegment2D<T>>` 実装
  - [ ] `BasicIntersection<T, Circle2D<T>>` 実装
  - [ ] `MultipleIntersection<T, LineSegment2D<T>>` 実装
  - [ ] `MultipleIntersection<T, Circle2D<T>>` 実装

**2.3 テスト**:
- [ ] `model/geo_primitives/src/ellipse_arc_2d_collision_tests.rs` (新規)
  - [ ] 点との距離テスト
  - [ ] 線分との衝突テスト
  - [ ] 円との衝突テスト
  - [ ] エッジケーステスト
- [ ] `model/geo_primitives/src/ellipse_arc_2d_intersection_tests.rs` (新規)
  - [ ] 線分との交点テスト
  - [ ] 円との交点テスト
  - [ ] 複数交点テスト

### ステップ3: EllipseArc3D実装（0.75日）

**実装ファイル**:

**3.1 Collision実装**:
- [ ] `model/geo_primitives/src/ellipse_arc_3d_collision.rs` (新規)
  - [ ] `BasicCollision<T, Point3D<T>>` 実装
  - [ ] `BasicCollision<T, LineSegment3D<T>>` 実装
  - [ ] `BasicCollision<T, Plane3D<T>>` 実装
  - [ ] `PointDistance<T>` 実装
  - [ ] `BBoxCollision<T>` 実装

**3.2 Intersection実装**:
- [ ] `model/geo_primitives/src/ellipse_arc_3d_intersection.rs` (新規)
  - [ ] `BasicIntersection<T, LineSegment3D<T>>` 実装
  - [ ] `BasicIntersection<T, Plane3D<T>>` 実装
  - [ ] `MultipleIntersection<T, LineSegment3D<T>>` 実装
  - [ ] `MultipleIntersection<T, Plane3D<T>>` 実装

**3.3 テスト**:
- [ ] `model/geo_primitives/src/ellipse_arc_3d_collision_tests.rs` (新規)
- [ ] `model/geo_primitives/src/ellipse_arc_3d_intersection_tests.rs` (新規)

### ステップ4: 統合テスト・検証（0.25日）

**テスト項目**:
- [ ] 全単体テスト通過確認
- [ ] エッジケース検証
  - [ ] 極小楕円弧
  - [ ] 完全楕円（360度）
  - [ ] 退化ケース（円弧化）
- [ ] パフォーマンステスト
  - [ ] 大量衝突判定の速度測定

**成果物**:
- [ ] テスト結果レポート
- [ ] パフォーマンスデータ

## 完了条件
- [ ] EllipseArc2D の collision/intersection 実装完了
- [ ] EllipseArc3D の collision/intersection 実装完了
- [ ] 全テストが通過している
- [ ] `.vscode/settings.json` のネスティング設定更新済み
- [ ] ドキュメント更新済み

## 関連情報
- 関連Issue: なし（新規）
- 参考実装: 
  - `model/geo_primitives/src/arc_2d_collision.rs`
  - `model/geo_primitives/src/arc_3d_collision.rs`
- 設計ドキュメント: `dev/foundation/PHASE3_COMPLETION_REPORT.md`

## 工数見積
合計: 2日

## 優先度
🟠 高優先（Tier 2）

## ラベル
- `enhancement`
- `geometry`
- `collision`
- `intersection`
- `tier-2`
```

---

## Issue #4: Cylindrical形状交差判定実装

```markdown
## 概要
CylindricalSurface3D および CylindricalSolid3D に対する collision/intersection 機能を実装する。

## 背景・目的
- CAD/CAMで最も頻出する3D形状
- 穴、軸、パイプなどの表現に必須
- 工具経路計算の基盤となる

## 実装ステップ

### ステップ1: アルゴリズム調査・設計（0.5日）

**調査項目**:
- [ ] 円筒面の数学的定義確認
  - [ ] パラメトリック表現
  - [ ] 陰関数表現
- [ ] 交差判定アルゴリズム調査
  - [ ] 直線と円筒の交点計算
  - [ ] 平面と円筒の交線計算
  - [ ] 円筒同士の交線計算（複雑 → 後回し）
- [ ] 距離計算アルゴリズム
  - [ ] 点と円筒面の最短距離
  - [ ] 点と円筒立体の最短距離

**設計項目**:
- [ ] データ構造の確認
  ```rust
  pub struct CylindricalSurface3D<T: Scalar> {
      axis: InfiniteLine3D<T>,
      radius: T,
      height: Option<T>, // None = 無限円筒
  }
  
  pub struct CylindricalSolid3D<T: Scalar> {
      axis: InfiniteLine3D<T>,
      radius: T,
      height: T,
  }
  ```

**成果物**:
- [ ] アルゴリズム調査レポート
- [ ] 実装方針書

### ステップ2: CylindricalSurface3D実装（1.25日）

**実装ファイル**:

**2.1 Collision実装**:
- [ ] `model/geo_primitives/src/cylindrical_surface_3d_collision.rs` (新規)
  - [ ] `BasicCollision<T, Point3D<T>>` 実装
  - [ ] `BasicCollision<T, LineSegment3D<T>>` 実装
  - [ ] `BasicCollision<T, Plane3D<T>>` 実装
  - [ ] `PointDistance<T>` 実装
    - [ ] 点から円筒面への最短距離計算
    - [ ] 最近点の取得
  - [ ] `BBoxCollision<T>` 実装

**2.2 Intersection実装**:
- [ ] `model/geo_primitives/src/cylindrical_surface_3d_intersection.rs` (新規)
  - [ ] `BasicIntersection<T, LineSegment3D<T>>` 実装
    - [ ] 直線と円筒の交点計算（0-2点）
  - [ ] `BasicIntersection<T, Plane3D<T>>` 実装
    - [ ] 平面と円筒の交線計算（楕円または円）
  - [ ] `MultipleIntersection<T, LineSegment3D<T>>` 実装

**2.3 テスト**:
- [ ] 包括的なテストスイート作成
  - [ ] 軸平行な円筒のテスト
  - [ ] 傾いた円筒のテスト
  - [ ] エッジケース（接触、交差なし）

### ステップ3: CylindricalSolid3D実装（1.25日）

**実装ファイル**:

**3.1 Collision実装**:
- [ ] `model/geo_primitives/src/cylindrical_solid_3d_collision.rs` (新規)
  - [ ] `BasicCollision<T, Point3D<T>>` 実装
  - [ ] `BasicCollision<T, LineSegment3D<T>>` 実装
  - [ ] `BasicCollision<T, Plane3D<T>>` 実装
  - [ ] `PointDistance<T>` 実装
    - [ ] 点が内部にある場合の処理
    - [ ] 端面の考慮
  - [ ] `BBoxCollision<T>` 実装

**3.2 Intersection実装**:
- [ ] `model/geo_primitives/src/cylindrical_solid_3d_intersection.rs` (新規)
  - [ ] `BasicIntersection<T, LineSegment3D<T>>` 実装
  - [ ] `BasicIntersection<T, Plane3D<T>>` 実装
  - [ ] `MultipleIntersection<T, LineSegment3D<T>>` 実装

**3.3 テスト**:
- [ ] 包括的なテストスイート作成

### ステップ4: 統合テスト・検証（0.5日）

**テスト項目**:
- [ ] Surface と Solid の整合性確認
- [ ] 組み合わせテスト
  - [ ] 円筒 × 平面
  - [ ] 円筒 × 直線（軸と平行/垂直/傾斜）
- [ ] パフォーマンステスト

**成果物**:
- [ ] テスト結果レポート
- [ ] 実装ドキュメント

## 完了条件
- [ ] CylindricalSurface3D の collision/intersection 実装完了
- [ ] CylindricalSolid3D の collision/intersection 実装完了
- [ ] 全テストが通過している
- [ ] `.vscode/settings.json` のネスティング設定更新済み

## 関連情報
- 関連Issue: なし（新規）
- 参考実装: `model/geo_primitives/src/spherical_surface_3d_collision.rs`
- 設計ドキュメント: `dev/foundation/PHASE3_SHAPE_COVERAGE_ANALYSIS.md`

## 工数見積
合計: 3.5日

## 優先度
🟠 高優先（Tier 2）

## ラベル
- `enhancement`
- `geometry`
- `collision`
- `intersection`
- `tier-2`
- `cad-essential`
```

---

## テンプレート使用ガイド

### 1. Issueタイトルの命名規則
```
[カテゴリ] 簡潔な説明
```

**カテゴリ例**:
- `[Visualization]` - 可視化関連
- `[Geometry]` - 幾何処理関連
- `[CAM]` - CAM機能関連
- `[Architecture]` - アーキテクチャ関連
- `[Refactoring]` - リファクタリング

**例**:
- `[Visualization] CAM可視化システム設計・実装`
- `[Geometry] EllipseArc交差判定実装`
- `[Architecture] レガシーAPI問題解決`

### 2. ラベルの使い分け

**優先度ラベル**:
- `tier-1` - 最優先
- `tier-2` - 高優先
- `tier-3` - 中優先
- `tier-4` - 中長期

**カテゴリラベル**:
- `enhancement` - 新機能
- `refactoring` - リファクタリング
- `bug` - バグ修正
- `documentation` - ドキュメント

**技術領域ラベル**:
- `visualization` - 可視化
- `geometry` - 幾何処理
- `cam` - CAM機能
- `collision` - 衝突判定
- `intersection` - 交差判定
- `architecture` - アーキテクチャ

### 3. マイルストーンの設定

**推奨マイルストーン**:
- `2026 Q1` (2026年1-3月)
- `2026 Q2` (2026年4-6月)
- `Phase 3 Completion` (特定フェーズ完了)
- `CAM Foundation` (CAM基盤構築)

### 4. 担当者の設定

- Issue作成時は担当者未設定でOK
- 着手時に自分自身を Assignee に設定
- 複数人で作業する場合は全員を Assignee に追加

### 5. プロジェクトボードへの追加

各Issueを作成後、以下のプロジェクトボードに追加：
- `RedRing Development Roadmap`
- 該当する Tier のボード（Tier 1, Tier 2 など）

ステータス設定:
- `Backlog` - 未着手
- `In Progress` - 作業中
- `In Review` - レビュー待ち
- `Done` - 完了

---

## 次のアクション

1. **Issue作成の優先順位**:
   1. CAM可視化システム（Tier 1）
   2. レガシーAPI問題（Tier 2）
   3. EllipseArc交差判定（Tier 2）
   4. Cylindrical形状交差判定（Tier 2）

2. **Issue作成手順**:
   ```bash
   # GitHub CLI を使用する場合
   gh issue create --title "[Visualization] CAM可視化システム設計・実装" \
                    --body-file issue_template_cam_viz.md \
                    --label "enhancement,visualization,cam,tier-1" \
                    --milestone "2026 Q1"
   ```

3. **ドキュメント参照**:
   - 各Issueに `dev/foundation/ROADMAP_2026_Q1_Q2.md` へのリンクを追加
   - 関連する既存Issueへの相互リンク設定
