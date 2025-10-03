# geo_core LineSegment3D vs model Line 構造分析と統合設計案

## 現在の構造比較

### geo_core::LineSegment3D

```rust
pub struct LineSegment3D {
    start: Point3D,     // Scalar型によるトレラント座標
    end: Point3D,       // Scalar型によるトレラント座標
}

// 特徴:
// - 有限線分（start-end間のみ）
// - ParametricCurve3D trait実装
// - Scalar型による高精度座標
// - ToleranceContext統合済み
// - パラメータ t ∈ [0,1]
```

### model::geometry3d::Line

```rust
pub struct Line {
    origin: Point,      // f64型座標
    direction: Direction, // 正規化方向ベクトル
    start: Point,       // f64型座標
    end: Point,         // f64型座標
}

// 特徴:
// - 無限直線（origin + direction）+ 有限区間（start-end）
// - Curve3D trait実装
// - f64型による座標
// - CurveKind3D::Line分類
// - パラメータ t ∈ [0,1]（実際はstart-end区間）
```

### model 関連構造

```rust
// InfiniteLine: 無限直線
pub struct InfiniteLine {
    origin: Point,
    direction: Direction,
}

// Ray: 半無限線（射線）
pub struct Ray {
    start: Point,
    direction: Direction,
}
```

## 意味的差異分析

### 1. 概念的違い

- **geo_core::LineSegment3D**: **純粋な線分**（2 点間の最短パス）
- **model::Line**: **直線の一部を切り取った区間**（無限直線の概念を持つ）

### 2. データ構造の違い

- **geo_core**: `start + end` のみ（ミニマル設計）
- **model**: `origin + direction + start + end`（豊富な情報）

### 3. 用途の違い

- **geo_core**: 数値計算基盤（線形補間、距離計算）
- **model**: CAD 幾何要素（直線概念、トリミング状態）

## 統合設計案

### 案 1: 階層統合設計（推奨）

```rust
// geo_core: 数値計算基盤
pub struct LineSegment3D {
    start: Point3D,
    end: Point3D,
}

// model: 高レベル幾何抽象化
pub struct Line {
    segment: LineSegment3D,        // geo_coreの数値基盤を内包
    infinite_line: InfiniteLine,   // 無限直線情報
    trimming_info: TrimmingInfo,   // トリミング状態
}

pub struct InfiniteLine {
    origin: Point,
    direction: Direction,
}

pub struct TrimmingInfo {
    is_trimmed: bool,
    parameter_start: f64,  // 無限直線上でのstart位置
    parameter_end: f64,    // 無限直線上でのend位置
}
```

### 案 2: アダプターベース統合

```rust
// geometry_adapter_line.rs
pub struct AdaptedLine {
    core_segment: geo_core::LineSegment3D,
    geometric_context: LineGeometricContext,
    tolerance: ToleranceContext,
}

pub struct LineGeometricContext {
    infinite_line: Option<InfiniteLine>,
    is_trimmed: bool,
    original_domain: Option<(f64, f64)>,
}

impl Curve3D for AdaptedLine {
    fn kind(&self) -> CurveKind3D { CurveKind3D::Line }
    fn evaluate(&self, t: f64) -> Point {
        let geo_point = self.core_segment.evaluate(Scalar::new(t));
        Point::from_geo_core(geo_point)
    }
}
```

### 案 3: 統一プリミティブ設計

```rust
// 統一された線分概念
pub struct LineSegment3D {
    start: Point3D,
    end: Point3D,
    geometric_properties: GeometricProperties,
}

pub struct GeometricProperties {
    infinite_line: Option<InfiniteLine3D>,
    parametric_domain: (Scalar, Scalar),
    semantic_type: LineSemanticType,
}

pub enum LineSemanticType {
    FiniteSegment,      // 純粋な線分
    TrimmedLine,        // トリミングされた直線
    InfiniteProjection, // 無限直線の射影
}
```

## 推奨統合方針

### Phase 1: アダプターパターン導入

1. 既存の `model::Line` を保持
2. `AdaptedLine` で geo_core との橋渡し
3. 段階的にパフォーマンス改善

### Phase 2: 階層統合への移行

1. `LineSegment3D` を数値計算エンジンとして内包
2. `Line` を高レベル幾何抽象として再設計
3. CAD 固有の概念（トリミング、無限性）を保持

### Phase 3: 意味論的統一

1. 線分、直線、射線の関係を明確化
2. パラメータ化の一貫性確保
3. CAM 用途への最適化

## 実装上の注意点

### 1. 型変換の一貫性

```rust
impl From<model::Line> for geo_core::LineSegment3D {
    fn from(line: model::Line) -> Self {
        Self::new(
            Point3D::from_model(line.start()),
            Point3D::from_model(line.end())
        )
    }
}
```

### 2. パラメータ化の統一

- geo_core: t ∈ [0,1]（線形補間）
- model: t ∈ [0,1]（start-end 区間）
- 両者の意味論的一致を保証

### 3. 許容誤差の統合

```rust
impl Line {
    pub fn is_aligned_tolerant(&self, tolerance: &ToleranceContext) -> bool {
        // geo_coreの許容誤差を使用した方向判定
    }
}
```

## 結論

**geo_core::LineSegment3D** と **model::Line** は**似て非なる概念**です：

- **LineSegment3D**: 純粋な数値計算対象（2 点間の線形補間）
- **Line**: CAD 幾何要素（直線概念 + トリミング状態）

**推奨**: **階層統合設計**により、geo_core の数値堅牢性を活用しつつ、model の CAD 概念を保持する統合アーキテクチャを構築。
