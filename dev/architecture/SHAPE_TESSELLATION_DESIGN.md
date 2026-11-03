# 形状テッセレーション設計書

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月14日  
**対象Issue**: #204（形状可視化システム完成）  
**前提Issue**: #188（基本形状可視化、完了）  

## 1. 概要

Issue #204で追加する15個の幾何プリミティブの可視化方針を定義します。特に**無限要素の表示シンボル**と**曲面/ソリッドのテッセレーション方針**を明確化します。

## 2. 既存実装の確認

### 2.1 実装済み形状（Issue #188完了分）

| 形状 | ワイヤーフレーム | ソリッド | 備考 |
|------|----------------|---------|------|
| LineSegment3D | ✅ | - | 2頂点 |
| Circle3D | ✅ | ✅ | 扇形分割（center + segments*3 vertices） |
| Arc3D | ✅ | - | 円弧セグメント分割 |
| Triangle3D | ✅ | ✅ | 3頂点 + 法線計算 |
| NurbsCurve3D | ✅ | - | SVG経由表示 |

### 2.2 既存のTessellationQuality構造体

```rust
pub struct TessellationQuality {
    pub min_segments: usize,           // 8
    pub max_segments: usize,           // 64
    pub circle_segments: usize,        // 32
    pub sphere_u_divisions: usize,     // 32 (経度方向)
    pub sphere_v_divisions: usize,     // 16 (緯度方向)
    pub plane_grid_size: usize,        // 10 (グリッド線数)
    pub enable_lod: bool,              // false (Phase 3で使用予定)
    pub lod_distance_threshold: f32,   // 100.0 (Phase 3で使用予定)
}
```

**評価**: 既存パラメータで大部分をカバー可能。追加パラメータは不要。

### 2.3 geo_primitives実装状況

Issue #204で必要な全15形状は**既に実装済み**：

- ✅ Plane3D
- ✅ Ellipse3D / EllipseArc3D
- ✅ Ray3D / InfiniteLine3D
- ✅ CylindricalSurface3D / CylindricalSolid3D
- ✅ SphericalSurface3D / SphericalSolid3D
- ✅ ConicalSurface3D / ConicalSolid3D
- ✅ TorusSurface3D / TorusSolid3D
- ✅ EllipsoidalSurface3D / EllipsoidalSolid3D

## 3. Issue #42との関係整理

### 3.1 Issue #42の状態

- **ステータス**: `on-hold` + `needs-foundation-update`
- **目的**: トレランス指定のテセレーション機能（f32/f64統一制御）
- **判断**: Issue #204とは独立して進める

### 3.2 Issue #204の方針

- Issue #42の成果を待たずに、既存のTessellationQualityを使用
- 固定品質パラメータによる実装（適応的品質はPhase 3で導入）
- Issue #42完了時に、トレランスベースAPIへ移行可能な設計

### 3.3 NURBS適応的テッセレーション（Issue #210）

Issue #210では、NURBS曲線/サーフェスの適応的テッセレーションを追加する。Phase 1はハイブリッド方式（CPUで分割パラメータ生成 + GPUで点評価）で開始し、必要に応じてPhase 2でComputeベースの完全GPU適応へ移行する。

- 対象: NurbsCurve3D / NurbsSurface3D
- 誤差尺度: 3D空間の弦誤差（距離）
- 出力形式: 曲線はLineStrip、サーフェスはワイヤーフレーム

詳細設計: NURBS_ADAPTIVE_TESSELLATION_DESIGN.md

## 4. 無限要素の可視化方針

### 4.1 設計原則

無限要素（Plane3D, InfiniteLine3D, Ray3D）は視野内での**有限表示**とします：

- **表示シンボル**: 視覚的に無限性を示すシンボル表現
- **表示範囲**: カメラ視野またはAABB境界に基づく有限範囲
- **方向性**: 矢印等で方向を明示（必要に応じて）

### 4.2 Plane3D - 平面

**表示方法**: グリッド線による有限範囲表示

```text
┌──────────┬──────────┬──────────┐
│          │          │          │
├──────────┼──────────┼──────────┤  ← グリッド線
│          │  ●origin │          │     (plane_grid_size = 10)
├──────────┼──────────┼──────────┤
│          │          │          │
└──────────┴──────────┴──────────┘
```

**実装パラメータ**:
- `grid_extent`: f64 - グリッドの半径（デフォルト: 10.0）
- `plane_grid_size`: usize - グリッド分割数（TessellationQualityから取得）

**頂点生成**:
- 平面上の2つの直交基底ベクトル（u, v）を計算
- グリッド線を生成（U方向線 + V方向線）
- 法線は平面の法線ベクトル

**法線表示** (オプション):
- 中心点から法線方向への矢印（後回し可）

### 4.3 InfiniteLine3D - 無限直線

**表示方法**: 両方向へ延びる有限長の直線 + 端点シンボル

```text
    ⟵━━━━━━━━●━━━━━━━━⟶
              origin
```

**実装パラメータ**:
- `line_extent`: f64 - 表示範囲の半径（デフォルト: 100.0）
- 原点から両方向に `line_extent` の長さを表示

**頂点生成**:
- 原点 + 方向ベクトル × extent
- 原点 - 方向ベクトル × extent
- 2頂点の線分として表示

**端点シンボル** (オプション):
- 小さな矢印または点を配置（Phase 3で検討）

### 4.4 Ray3D - 光線

**表示方法**: 原点から一方向への有限長の直線 + 矢印

```text
    ●━━━━━━━━━━━━⟶
    origin
```

**実装パラメータ**:
- `ray_extent`: f64 - 表示範囲の長さ（デフォルト: 100.0）
- 原点から方向に `ray_extent` の長さを表示

**頂点生成**:
- 原点頂点
- 原点 + 方向ベクトル × extent
- 2頂点の線分として表示

**矢印シンボル** (オプション):
- 終点に矢印を配置（Phase 3で検討）

### 4.5 無限要素の表示範囲制御

**Phase 1（Issue #204）**: 固定範囲
- Plane: `grid_extent = 10.0`
- InfiniteLine: `line_extent = 100.0`
- Ray: `ray_extent = 100.0`

**Phase 3（将来）**: カメラ適応
- カメラからの距離に応じて表示範囲を調整
- AABB境界との交差計算で適切な範囲を決定

## 5. 曲面（Surface）のテッセレーション方針

### 5.1 UV パラメトリック分割

全てのSurface形状は**UV パラメトリック空間**で分割します：

```text
V方向 ↑
      │  ┌──┬──┬──┬──┐
      │  ├──┼──┼──┼──┤
      │  ├──┼──┼──┼──┤  ← UV グリッド
      │  ├──┼──┼──┼──┤     (u_divisions × v_divisions)
      │  └──┴──┴──┴──┘
      └─────────────────→ U方向
```

**分割パラメータ**:
- `u_divisions`: U方向の分割数（形状に応じて決定）
- `v_divisions`: V方向の分割数（形状に応じて決定）

### 5.2 CylindricalSurface3D - 円筒面

**UV マッピング**:
- U方向: 円周（0 → 2π）
- V方向: 高さ（0 → height）

**分割数**:
- `u_divisions = circle_segments` (32)
- `v_divisions = 16` (固定値、または高さに応じて調整)

**法線計算**:
- 外向き法線（中心軸からの放射方向）

### 5.3 SphericalSurface3D - 球面

**UV マッピング**:
- U方向: 経度（0 → 2π）
- V方向: 緯度（-π/2 → π/2）

**分割数**:
- `u_divisions = sphere_u_divisions` (32)
- `v_divisions = sphere_v_divisions` (16)

**極点退化処理**:
- 北極・南極で頂点を共有（退化三角形を避ける）

**法線計算**:
- 外向き法線（球の中心からの放射方向）

### 5.4 ConicalSurface3D - 円錐面

**UV マッピング**:
- U方向: 円周（0 → 2π）
- V方向: 高さ（0 → height）

**分割数**:
- `u_divisions = circle_segments` (32)
- `v_divisions = 16`

**頂点退化処理**:
- 頂点で全てのU方向頂点を共有

**法線計算**:
- 円錐の母線に対して垂直な法線

### 5.5 TorusSurface3D - トーラス面

**UV マッピング**:
- U方向: 主円周（0 → 2π）
- V方向: 副円周（0 → 2π）

**分割数**:
- `u_divisions = circle_segments` (32)
- `v_divisions = circle_segments / 2` (16)

**法線計算**:
- トーラス表面の外向き法線

### 5.6 EllipsoidalSurface3D - 楕円体面

**UV マッピング**:
- U方向: 経度（0 → 2π）
- V方向: 緯度（-π/2 → π/2）

**分割数**:
- `u_divisions = sphere_u_divisions` (32)
- `v_divisions = sphere_v_divisions` (16)

**極点退化処理**:
- SphericalSurface3Dと同様

**法線計算**:
- 楕円体表面の外向き法線（スケール補正が必要）

## 6. ソリッド（Solid）のテッセレーション方針

### 6.1 表示方針

Solid形状は**表面のみ**を描画します（内部は描画しない）：

- **ワイヤーフレームモード**: エッジのみ表示
- **ソリッドモード**: 表面メッシュを描画（法線による陰影）

### 6.2 各Solid形状の実装方針

| 形状 | 表面構成 | 実装方針 |
|------|---------|---------|
| CylindricalSolid3D | 側面 + 上面 + 下面 | CylindricalSurface + Circle × 2 |
| SphericalSolid3D | 球面 | SphericalSurface |
| ConicalSolid3D | 側面 + 底面 | ConicalSurface + Circle |
| TorusSolid3D | トーラス面 | TorusSurface |
| EllipsoidalSolid3D | 楕円体面 | EllipsoidalSurface |

### 6.3 エッジ vs 面の統合

**Phase 1（Issue #204）**: 独立実装
- Solidごとに表面メッシュ生成関数を実装
- SurfaceとCircleの組み合わせロジックを含む

**Phase 4（エンティティ層）**: B-Rep統合
- Edge/Face/Shellによるトポロジカル表現
- 統一的なメッシュ生成パイプライン

## 7. Ellipse3D / EllipseArc3D のテッセレーション

### 7.1 Ellipse3D - 楕円

**表示方法**: 楕円周上のセグメント分割

**パラメトリック表現**:
```
x(θ) = center + a*cos(θ)*u + b*sin(θ)*v
```
- `a`: 長軸半径
- `b`: 短軸半径
- `u`, `v`: 楕円平面上の直交基底

**分割数**:
- `circle_segments` (32) を使用

**法線計算**:
- 楕円の法線（平面の法線）

### 7.2 EllipseArc3D - 楕円弧

**表示方法**: 楕円弧のセグメント分割

**パラメトリック表現**:
```
x(θ) = center + a*cos(θ)*u + b*sin(θ)*v  (θ: start_angle → end_angle)
```

**分割数**:
- 角度範囲に応じて調整（Arc3Dと同様）

## 8. 法線計算の統一方針

### 8.1 線形要素

- **法線**: ゼロベクトル `[0.0, 0.0, 0.0]`
- 理由: 線には法線が定義されない（シェーダで無視）

### 8.2 曲線要素

- **法線**: 曲線の平面法線（平面曲線の場合）
- Circle/Arc/Ellipse: 平面の法線ベクトル
- 3D自由曲線: Frenet-Serret標構（Phase 3で検討）

### 8.3 曲面要素

- **法線**: 外向き法線（右手系）
- UVパラメータから偏微分ベクトルを計算し、外積で法線を取得
- 正規化して `[nx, ny, nz]` を格納

### 8.4 ソリッド要素

- **法線**: 各面の外向き法線
- 両面描画は行わない（カリング設定で制御）

## 9. 品質パラメータのチューニング

### 9.1 Phase 1（Issue #204）の品質

**目標**: 60FPS@100形状

| パラメータ | 値 | 理由 |
|-----------|---|------|
| circle_segments | 32 | 滑らかさと性能のバランス |
| sphere_u_divisions | 32 | 経度方向（視認性重視） |
| sphere_v_divisions | 16 | 緯度方向（頂点数削減） |
| plane_grid_size | 10 | グリッド線の視認性 |

**頂点数見積**:
- Circle: 32 segments → 96 vertices (solid)
- Sphere: 32 × 16 → ~1536 vertices
- 100形状 × 平均500 vertices = 50,000 vertices → 余裕でリアルタイム可能

### 9.2 Phase 3での改善予定

- **適応的品質**: カメラ距離に応じたLOD
- **曲率ベース分割**: 急激に曲がる部分を細かく分割
- **視錐台カリング**: 視野外の形状は描画しない

## 10. ワイヤーフレーム vs ソリッド表示

### 10.1 トポロジ選択

| モード | wgpu::PrimitiveTopology | 頂点生成方式 |
|--------|------------------------|-------------|
| ワイヤーフレーム | LineList | 各エッジを2頂点ペアで表現 |
| ソリッド | TriangleList | 各三角形を3頂点で表現 |

### 10.2 切り替え方式

**MeshStage での実装**:
```rust
pub struct MeshStage {
    pub wireframe_mode: bool,  // true: ワイヤーフレーム, false: ソリッド
    // ...
}
```

**パイプライン切り替え**:
- `wireframe_pipeline`: LineListトポロジ
- `render_pipeline`: TriangleListトポロジ

## 11. 実装スケジュール

### Step 1: テッセレーション基盤設計（1日）✅

- [x] この設計書作成
- [x] Issue #42との関係整理
- [x] 無限要素の表示方針決定

### Step 2: 基本形状実装（1.5日）

- [ ] Plane3D - グリッド表示
- [ ] Ellipse3D / EllipseArc3D - 楕円テッセレーション
- [ ] Ray3D / InfiniteLine3D - 有限長表示

### Step 3: 面形状（Surface）実装（2.5日）

- [ ] CylindricalSurface3D
- [ ] SphericalSurface3D
- [ ] ConicalSurface3D
- [ ] TorusSurface3D
- [ ] EllipsoidalSurface3D

### Step 4: ソリッド形状（Solid）実装（2.5日）

- [ ] CylindricalSolid3D
- [ ] SphericalSolid3D
- [ ] ConicalSolid3D
- [ ] TorusSolid3D
- [ ] EllipsoidalSolid3D

### Step 5: 統合テスト・最適化（1.5日）

- [ ] 全15形状のサンプルSVG作成
- [ ] パフォーマンステスト（100形状@60FPS）
- [ ] ドキュメント整備

## 12. 完了条件

- [ ] 全15形状が画面に正しく表示される
- [ ] ワイヤーフレーム/ソリッド表示の切り替えが可能
- [ ] 無限要素が適切な有限範囲で表示される
- [ ] 100個の形状を60FPS以上で描画可能
- [ ] Foundation Patternとの整合性が保たれている
- [ ] 全テストが通過している

## 13. 設計上の注意事項

### 13.1 Foundation Patternの遵守

- 各形状の変換関数は `shape_converter.rs` に集約
- `PrimitiveKind` を使った型消去パターンに対応
- `PrimitiveMetadata` / `Bounded` の再整理方針と整合させる

### 13.2 コード重複の回避

- 共通のUV分割ロジックはヘルパー関数として抽出
- SurfaceとSolidで共通化可能な部分は統合
- 法線計算ユーティリティの活用

### 13.3 テストコード配置

- 各形状の変換関数には単体テストを追加
- `shape_converter_tests.rs` に集約
- SVGファイル読み込みテストを含む

## 14. 補足: Issue #42完了時の移行計画

Issue #42（トレランス指定テセレーション）完了時の移行方針：

### 14.1 現在の実装（Issue #204）

```rust
pub fn sphere_surface_to_vertices(
    surface: &SphericalSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData>
```

### 14.2 Issue #42完了後

```rust
pub fn sphere_surface_to_vertices_with_tolerance(
    surface: &SphericalSurface3D<f64>,
    tolerance: f64,  // トレランス指定
) -> Vec<VertexData>
```

**移行方針**:
- 既存の `quality` ベース関数は維持（後方互換性）
- `tolerance` ベース関数を追加
- 内部で `tolerance → 品質パラメータ` の変換を行う

### 14.3 適応的品質計算（Phase 3）

```rust
pub fn sphere_surface_to_vertices_adaptive(
    surface: &SphericalSurface3D<f64>,
    camera: &Camera,
    tolerance: f64,
) -> Vec<VertexData>
```

- カメラからの距離に応じてLOD調整
- 画面上のピクセルサイズに基づく品質決定

---

**次のステップ**: Step 2（基本形状実装）に進む
