# 形状可視化システム設計仕様

**作成日**: 2025年12月26日  
**最終更新**: 2025年12月26日  
**関連Issue**: [#188 形状可視化システムの実装](https://github.com/RedRing2020/RedRing/issues/188)

## 📋 概要

geo_primitivesで実装されている各種形状をGPU描画可能な形式に変換し、デバッグや可視化に利用できるシステムの設計仕様書。

## 🎯 設計目標

1. **統一的な変換フロー**: 全ての形状に対して一貫した変換処理を提供
2. **MVVMアーキテクチャ遵守**: Model層への依存をViewModel層で吸収
3. **Foundation Pattern統合**: ExtensionFoundationトレイトを活用した型判別
4. **パフォーマンス**: 1000個の形状を60FPSで描画可能

## 🏗️ アーキテクチャ

### データフロー

```
Model層 (geo_primitives)
  ├─ LineSegment3D
  ├─ Circle3D
  ├─ Arc3D
  ├─ Triangle3D
  └─ その他の形状
         ↓
ViewModel層 (viewmodel/converter)
  shape_converter::shape_to_vertices()
  ├─ 形状種別判定 (PrimitiveKind)
  ├─ テッセレーション処理
  └─ VertexData生成 (position + normal)
         ↓
View層 (view/render, view/stage)
  ├─ MeshVertex変換 (GPU形式)
  ├─ ShapeStage (新規 or MeshStage拡張)
  └─ GPU描画
```

### レイヤー責務

- **Model層**: 幾何データの定義と計算
- **ViewModel層**: 形状からGPU頂点データへの変換
- **View層**: GPU描画とレンダリングステージ管理

## 📐 形状別可視化方式

### 優先度1: 線形形状（最優先実装）

#### LineSegment3D（線分）
- **表示方式**: ワイヤーフレーム（線）
- **テッセレーション**: 不要（始点・終点の2頂点）
- **頂点数**: 2
- **法線**: ゼロベクトルまたはカメラ方向
- **品質パラメータ**: 線の太さ（ピクセル単位）

```rust
// 疑似コード
fn line_segment_to_vertices(segment: &LineSegment3D<f64>) -> Vec<VertexData> {
    vec![
        VertexData::new(segment.start_position(), [0.0, 0.0, 0.0]),
        VertexData::new(segment.end_position(), [0.0, 0.0, 0.0]),
    ]
}
```

#### Ray3D / InfiniteLine3D（半直線・無限直線）
- **表示方式**: ワイヤーフレーム（有限長さで表現）
- **テッセレーション**: 境界ボックスまたは固定長で切断
- **頂点数**: 2
- **表示長さ**: デフォルト100単位、カメラ視野に基づく動的調整
- **方向インジケータ**: 矢印または色グラデーション

#### Plane3D（平面）
- **表示方式**: グリッド表示
- **テッセレーション**: グリッド分割（デフォルト 10x10）
- **頂点数**: 121 (11x11 vertices)
- **サイズ**: デフォルト 10x10単位
- **法線表示**: オプションで法線ベクトル表示

### 優先度2: 曲線形状

#### Circle3D（円）
- **表示方式**: ワイヤーフレーム（多角形近似）
- **テッセレーション**: セグメント分割
- **セグメント数**: デフォルト 32（半径に応じて動的調整可）
- **頂点数**: セグメント数
- **法線**: 円の法線方向（平面の法線）

**品質計算式**:
```
segments = max(8, min(64, int(2 * π * radius / pixel_size)))
```

#### Arc3D（円弧）
- **表示方式**: ワイヤーフレーム（多角形近似）
- **テッセレーション**: 角度範囲に応じたセグメント分割
- **セグメント数**: 円の場合と同様だが、角度範囲で比例調整
- **エンドポイント**: 始点・終点にマーカー表示（オプション）

#### Ellipse3D / EllipseArc3D（楕円・楕円弧）
- **表示方式**: ワイヤーフレーム（多角形近似）
- **テッセレーション**: パラメトリック分割
- **セグメント数**: デフォルト 48（長軸に応じて調整）
- **アスペクト比**: 正確に保持

### 優先度3: 面形状

#### Triangle3D（三角形）
- **表示方式**: ソリッド（両面描画）
- **テッセレーション**: 不要（3頂点）
- **頂点数**: 3
- **法線**: 外積から計算（CCW順序）
- **ワイヤーフレームモード**: エッジのみ表示

#### CylindricalSurface3D（円柱面）
- **表示方式**: ソリッド（UVパラメトリック）
- **テッセレーション**: UV分割
- **U分割**: 周方向 32セグメント
- **V分割**: 高さ方向 8セグメント
- **頂点数**: (U+1) * (V+1) = 33 * 9 = 297
- **LOD対応**: 距離に応じてセグメント数を削減

#### SphericalSurface3D（球面）
- **表示方式**: ソリッド
- **テッセレーション**: UV-sphere方式
- **緯度分割**: 16
- **経度分割**: 32
- **頂点数**: 約 513
- **極点処理**: 北極・南極の退化三角形を適切に処理

### 優先度4: ソリッド形状

#### CylindricalSolid3D（円柱ソリッド）
- **表示方式**: ソリッド（側面 + 上下面）
- **側面**: CylindricalSurface3Dと同様
- **上下面**: 円形の多角形（扇形分割）
- **頂点数**: 側面 + 上面 + 下面

#### SphericalSolid3D（球ソリッド）
- **表示方式**: ソリッド
- **テッセレーション**: SphericalSurface3Dと同様
- **半透明オプション**: アルファ値対応

#### TorusSolid3D / TorusSurface3D（トーラス）
- **表示方式**: ソリッド
- **テッセレーション**: UVパラメトリック
- **メジャー分割**: 32セグメント（大円）
- **マイナー分割**: 16セグメント（小円）
- **頂点数**: 約 513

## ⚙️ テッセレーション品質パラメータ

### デフォルト設定

```rust
pub struct TessellationQuality {
    /// 線形形状の最小セグメント数
    pub min_segments: usize,        // default: 8
    
    /// 線形形状の最大セグメント数
    pub max_segments: usize,        // default: 64
    
    /// 円形状のデフォルトセグメント数
    pub circle_segments: usize,     // default: 32
    
    /// 球面のU方向分割数
    pub sphere_u_divisions: usize,  // default: 32
    
    /// 球面のV方向分割数
    pub sphere_v_divisions: usize,  // default: 16
    
    /// 平面グリッドの分割数
    pub plane_grid_size: usize,     // default: 10
    
    /// LOD有効化フラグ
    pub enable_lod: bool,           // default: false
    
    /// LOD距離閾値
    pub lod_distance_threshold: f32, // default: 100.0
}
```

### 適応的品質調整

カメラからの距離、画面上のサイズ、ズームレベルに応じて動的にセグメント数を調整：

```
screen_size = projected_radius * viewport_width
segments = clamp(screen_size / 5.0, min_segments, max_segments)
```

## 🎨 表示優先順位（実装順序）

### Phase 2.1: 線形形状（Week 1）
1. **LineSegment3D** - 最もシンプル、デバッグに最適
2. **Ray3D** - 方向確認に重要
3. **InfiniteLine3D** - Ray3Dと類似
4. **Plane3D** - グリッド表示の基礎

### Phase 2.2: 曲線形状（Week 2）
5. **Circle3D** - 基本的なテッセレーション
6. **Arc3D** - Circle3Dの拡張
7. **Ellipse3D** - より複雑なテッセレーション
8. **EllipseArc3D** - Ellipse3Dの拡張

### Phase 2.3: 面形状（Week 3）
9. **Triangle3D** - 最もシンプルな面
10. **CylindricalSurface3D** - UVパラメトリックの基礎
11. **SphericalSurface3D** - 球面テッセレーション

### Phase 2.4: ソリッド形状（Week 4）
12. **CylindricalSolid3D** - 複合メッシュの基礎
13. **SphericalSolid3D** - ソリッド表現
14. **TorusSolid3D / TorusSurface3D** - 高度なテッセレーション

## 🔧 技術実装詳細

### Foundation Patternとの統合

```rust
use geo_foundation::{ExtensionFoundation, PrimitiveKind};

pub fn shape_to_vertices<T: Scalar>(
    shape: &dyn ExtensionFoundation<T>
) -> Vec<VertexData> {
    match shape.primitive_kind() {
        PrimitiveKind::LineSegment => {
            // LineSegment3D専用の変換
        },
        PrimitiveKind::Circle => {
            // Circle3D専用の変換
        },
        // ... その他の形状
        _ => vec![], // 未対応の形状
    }
}
```

### VertexData構造

既存のmesh_converterと同じ構造を使用：

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
```

### エラーハンドリング

```rust
#[derive(Debug, thiserror::Error)]
pub enum ShapeConversionError {
    #[error("Unsupported primitive kind: {0:?}")]
    UnsupportedPrimitive(PrimitiveKind),
    
    #[error("Invalid tessellation parameters")]
    InvalidTessellation,
    
    #[error("Shape has degenerate geometry")]
    DegenerateGeometry,
}
```

## 📊 パフォーマンス目標

### ベンチマーク指標

- **LineSegment3D**: 10,000個を60FPS（各2頂点）
- **Circle3D**: 1,000個を60FPS（各32頂点）
- **Triangle3D**: 5,000個を60FPS（各3頂点）
- **CylindricalSurface3D**: 100個を60FPS（各約300頂点）

### 最適化戦略

1. **頂点バッファキャッシュ**: 同一形状の再利用
2. **インスタンシング**: 同一形状の大量描画
3. **フラスタムカリング**: 視野外形状の描画スキップ
4. **LOD切り替え**: 距離に応じた品質調整

## 🔗 関連実装ファイル

### 新規作成予定
- `viewmodel/converter/src/shape_converter.rs` - 形状変換の中核
- `viewmodel/converter/src/tessellation.rs` - テッセレーション処理
- `view/stage/src/shape_stage.rs` - 形状描画ステージ（検討中）

### 既存参考実装
- `viewmodel/converter/src/mesh_converter.rs` - TriangleMesh変換ロジック
- `view/stage/src/mesh_stage.rs` - メッシュ描画ステージ
- `view/render/src/mesh.rs` - GPU描画リソース管理

## 📝 実装時の注意事項

1. **geo_primitives直接依存の禁止**: View層からModel層への直接依存を避ける
2. **Foundation経由アクセス**: primitive_kind()を活用した型判別
3. **f32/f64変換**: Model層はf64、GPU描画はf32を使用
4. **法線の正規化**: 必ず正規化された法線を渡す
5. **CCW順序**: 三角形の頂点順序は反時計回り

## 🚀 次のステップ

1. **Task 1.2**: shape_converterモジュールの実装（LineSegment3D, Circle3Dのプロトタイプ）
2. **Task 1.3**: View層の受け入れ設計（MeshStage拡張 vs 新規ShapeStage）
3. **Phase 2**: 各形状の順次実装

---

**レビュー**: @TBD  
**承認**: @TBD
