# Phase 4: トポロジー・エンティティ層の設計

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月8日  
**ステータス**: 設計フェーズ  
**優先度**: 🟡 Tier 3（Phase 3完了後に着手）

---

## 📋 目次

1. [概要](#概要)
2. [背景・動機](#背景動機)
3. [アーキテクチャ設計](#アーキテクチャ設計)
4. [実装計画](#実装計画)
5. [ロードマップ統合](#ロードマップ統合)

---

## 概要

Phase 3（交差判定・衝突判定）の次に実装すべきコア機能として、以下の3層を導入します：

1. **トポロジー層（geo_topology）** - 境界表現（B-Rep）と形状の接続関係
2. **エンティティ層（geo_entity）** - 属性付き形状オブジェクトとメタデータ管理
3. **パラメータ管理層** - トレランス・精度のシステム全体管理

これにより、単なる幾何計算から**CADシステムとしての形状モデリング**への進化を実現します。

---

## 背景・動機

### なぜトポロジー層が必要か

現在の `geo_primitives` は**幾何情報のみ**を扱います：
- Circle3D は「数学的な円」
- LineSegment3D は「数学的な線分」

しかし、実際のCADシステムでは：
- 「この円弧は、この平面の境界の一部」
- 「この線分の端点は、別の線分の端点と共有されている」

このような**形状間の接続関係（トポロジー）**が必須です。

### 典型的なCADカーネルの構造

```text
┌─────────────────────────────────────┐
│  アプリケーション層                    │
│  (UI, コマンド, ファイルI/O)           │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│  エンティティ層 (geo_entity)          │ ← Phase 4.2
│  - 属性付き形状オブジェクト            │
│  - メタデータ、UUID管理               │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│  トポロジー層 (geo_topology)          │ ← Phase 4.1
│  - B-Rep (Vertex/Edge/Face/Solid)   │
│  - 形状間の接続関係                   │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│  幾何層 (geo_primitives, geo_nurbs)  │ ← 既存（Phase 1-3完了）
│  - 数学的な形状定義                   │
│  - 交差判定、距離計算                 │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│  基盤層 (geo_foundation, analysis)   │
│  - トレイト定義、数値計算             │
└─────────────────────────────────────┘
```

### 参考：商用CADカーネル

- **OpenCASCADE**: TopoDS (Topological Data Structure)
- **Parasolid**: PK (Body, Face, Edge, Vertex)
- **ACIS**: Entity (Body, Lump, Shell, Face, Loop, Edge, Vertex)

すべて**トポロジー層**を持っています。

---

## アーキテクチャ設計

### Phase 4.1: トポロジー層（geo_topology）

#### 1.1 B-Rep（Boundary Representation）の基本概念

CADの標準的な形状表現方式：

```text
Solid（立体）
  └─ Shell（殻）
      └─ Face（面）
          └─ Loop（ループ）
              └─ Edge（辺）
                  └─ Vertex（頂点）
```

**階層関係**:
- **Vertex（頂点）**: 0次元、点の位置
- **Edge（辺）**: 1次元、2つの頂点を結ぶ曲線
- **Wire（ワイヤー）**: 1次元、複数のエッジが接続された経路
- **Face（面）**: 2次元、境界ループで囲まれた曲面
- **Shell（殻）**: 2次元多様体、複数の面が接続された閉じた/開いた殻
- **Solid（立体）**: 3次元、閉じたシェルで囲まれた体積

#### 1.2 データ構造設計

```rust
// model/geo_topology/src/lib.rs

use geo_primitives::Point3D;
use geo_foundation::Scalar;
use std::sync::Arc;

/// トポロジー要素の一意識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopoId(u64);

impl TopoId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// 頂点（Vertex）
#[derive(Debug, Clone)]
pub struct Vertex<T: Scalar> {
    id: TopoId,
    point: Point3D<T>,
    tolerance: T, // 位置トレランス
}

impl<T: Scalar> Vertex<T> {
    pub fn new(point: Point3D<T>, tolerance: T) -> Self {
        Self {
            id: TopoId::new(),
            point,
            tolerance,
        }
    }
    
    pub fn id(&self) -> TopoId {
        self.id
    }
    
    pub fn point(&self) -> &Point3D<T> {
        &self.point
    }
    
    /// 2つの頂点が位置的に同一か判定
    pub fn is_coincident(&self, other: &Vertex<T>) -> bool {
        self.point.distance_to(&other.point) <= self.tolerance.max(other.tolerance)
    }
}

/// 曲線参照（Edgeが参照する実際の幾何曲線）
#[derive(Debug, Clone)]
pub enum CurveRef<T: Scalar> {
    Line(LineSegment3D<T>),
    Circle(Arc3D<T>),
    Ellipse(EllipseArc3D<T>),
    Nurbs(NurbsCurve3D<T>),
}

/// 辺（Edge）
#[derive(Debug, Clone)]
pub struct Edge<T: Scalar> {
    id: TopoId,
    
    /// 開始頂点
    start_vertex: Arc<Vertex<T>>,
    
    /// 終了頂点
    end_vertex: Arc<Vertex<T>>,
    
    /// 幾何曲線への参照
    curve: CurveRef<T>,
    
    /// パラメータ範囲 [t_start, t_end]
    parameter_range: (T, T),
    
    /// 向き（順方向 or 逆方向）
    orientation: Orientation,
}

impl<T: Scalar> Edge<T> {
    pub fn new(
        start: Arc<Vertex<T>>,
        end: Arc<Vertex<T>>,
        curve: CurveRef<T>,
        parameter_range: (T, T),
    ) -> Self {
        Self {
            id: TopoId::new(),
            start_vertex: start,
            end_vertex: end,
            curve,
            parameter_range,
            orientation: Orientation::Forward,
        }
    }
    
    /// 辺上の点を取得（パラメータ t: 0.0-1.0）
    pub fn point_at(&self, t: T) -> Point3D<T> {
        let (t0, t1) = self.parameter_range;
        let param = t0 + (t1 - t0) * t;
        
        match &self.curve {
            CurveRef::Line(line) => line.point_at_parameter(param),
            CurveRef::Circle(arc) => arc.point_at_parameter(param),
            CurveRef::Ellipse(ellipse) => ellipse.point_at_parameter(param),
            CurveRef::Nurbs(nurbs) => nurbs.evaluate(param),
        }
    }
    
    /// 辺の長さ
    pub fn length(&self) -> T {
        match &self.curve {
            CurveRef::Line(line) => {
                let (t0, t1) = self.parameter_range;
                line.length() * (t1 - t0).abs()
            },
            CurveRef::Circle(arc) => arc.arc_length(),
            CurveRef::Ellipse(ellipse) => ellipse.arc_length(),
            CurveRef::Nurbs(nurbs) => nurbs.arc_length_total(),
        }
    }
}

/// 向き
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Forward,  // 順方向
    Reversed, // 逆方向
}

/// ワイヤー（Wire）- 接続されたエッジの列
#[derive(Debug, Clone)]
pub struct Wire<T: Scalar> {
    id: TopoId,
    edges: Vec<Arc<Edge<T>>>,
    is_closed: bool,
}

impl<T: Scalar> Wire<T> {
    pub fn new(edges: Vec<Arc<Edge<T>>>) -> Result<Self, WireError> {
        if edges.is_empty() {
            return Err(WireError::EmptyEdges);
        }
        
        // エッジの接続性をチェック
        for i in 0..edges.len() - 1 {
            let current_end = &edges[i].end_vertex;
            let next_start = &edges[i + 1].start_vertex;
            
            if !current_end.is_coincident(next_start) {
                return Err(WireError::DiscontinuousEdges);
            }
        }
        
        // 閉じているかチェック
        let is_closed = edges.first().unwrap().start_vertex
            .is_coincident(&edges.last().unwrap().end_vertex);
        
        Ok(Self {
            id: TopoId::new(),
            edges,
            is_closed,
        })
    }
    
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
    
    pub fn total_length(&self) -> T {
        self.edges.iter().map(|e| e.length()).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    EmptyEdges,
    DiscontinuousEdges,
}

/// 曲面参照（Faceが参照する実際の幾何曲面）
#[derive(Debug, Clone)]
pub enum SurfaceRef<T: Scalar> {
    Plane(Plane3D<T>),
    Cylinder(CylindricalSurface3D<T>),
    Cone(ConicalSurface3D<T>),
    Sphere(SphericalSurface3D<T>),
    Nurbs(NurbsSurface3D<T>),
}

/// 面（Face）
#[derive(Debug, Clone)]
pub struct Face<T: Scalar> {
    id: TopoId,
    
    /// 外側境界
    outer_loop: Arc<Wire<T>>,
    
    /// 内側境界（穴）のリスト
    inner_loops: Vec<Arc<Wire<T>>>,
    
    /// 曲面への参照
    surface: SurfaceRef<T>,
    
    /// 法線の向き
    orientation: Orientation,
}

impl<T: Scalar> Face<T> {
    pub fn new(
        outer_loop: Arc<Wire<T>>,
        inner_loops: Vec<Arc<Wire<T>>>,
        surface: SurfaceRef<T>,
    ) -> Result<Self, FaceError> {
        if !outer_loop.is_closed() {
            return Err(FaceError::OuterLoopNotClosed);
        }
        
        for inner in &inner_loops {
            if !inner.is_closed() {
                return Err(FaceError::InnerLoopNotClosed);
            }
        }
        
        Ok(Self {
            id: TopoId::new(),
            outer_loop,
            inner_loops,
            surface,
            orientation: Orientation::Forward,
        })
    }
    
    /// 面の法線ベクトル（UV座標での法線）
    pub fn normal_at(&self, u: T, v: T) -> Vector3D<T> {
        match &self.surface {
            SurfaceRef::Plane(plane) => plane.normal(),
            SurfaceRef::Cylinder(cyl) => cyl.normal_at(u, v),
            SurfaceRef::Cone(cone) => cone.normal_at(u, v),
            SurfaceRef::Sphere(sphere) => sphere.normal_at(u, v),
            SurfaceRef::Nurbs(nurbs) => nurbs.normal_at(u, v).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceError {
    OuterLoopNotClosed,
    InnerLoopNotClosed,
}

/// シェル（Shell）- 接続された面の集合
#[derive(Debug, Clone)]
pub struct Shell<T: Scalar> {
    id: TopoId,
    faces: Vec<Arc<Face<T>>>,
    is_closed: bool,
}

impl<T: Scalar> Shell<T> {
    pub fn new(faces: Vec<Arc<Face<T>>>) -> Self {
        // TODO: シェルの閉じ性を計算
        let is_closed = false; // 暫定
        
        Self {
            id: TopoId::new(),
            faces,
            is_closed,
        }
    }
    
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
}

/// 立体（Solid）- 閉じたシェルで囲まれた体積
#[derive(Debug, Clone)]
pub struct Solid<T: Scalar> {
    id: TopoId,
    
    /// 外側シェル
    outer_shell: Arc<Shell<T>>,
    
    /// 内側シェル（空洞）のリスト
    inner_shells: Vec<Arc<Shell<T>>>,
}

impl<T: Scalar> Solid<T> {
    pub fn new(
        outer_shell: Arc<Shell<T>>,
        inner_shells: Vec<Arc<Shell<T>>>,
    ) -> Result<Self, SolidError> {
        if !outer_shell.is_closed() {
            return Err(SolidError::OuterShellNotClosed);
        }
        
        for inner in &inner_shells {
            if !inner.is_closed() {
                return Err(SolidError::InnerShellNotClosed);
            }
        }
        
        Ok(Self {
            id: TopoId::new(),
            outer_shell,
            inner_shells,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolidError {
    OuterShellNotClosed,
    InnerShellNotClosed,
}
```

#### 1.3 トポロジー操作（Topological Operations）

```rust
// model/geo_topology/src/operations.rs

/// トポロジー操作トレイト
pub trait TopologicalOperations<T: Scalar> {
    /// 隣接する面のリスト
    fn adjacent_faces(&self, edge: &Edge<T>) -> Vec<Arc<Face<T>>>;
    
    /// 境界のワイヤーを取得
    fn boundary(&self) -> Vec<Arc<Wire<T>>>;
    
    /// オイラー操作: 頂点を追加
    fn euler_make_vertex(&mut self, point: Point3D<T>) -> Arc<Vertex<T>>;
    
    /// オイラー操作: 辺を追加
    fn euler_make_edge(
        &mut self,
        start: Arc<Vertex<T>>,
        end: Arc<Vertex<T>>,
        curve: CurveRef<T>,
    ) -> Arc<Edge<T>>;
}

/// オイラー操作（Euler Operations）
/// 
/// B-Repの整合性を保ちながら形状を編集するための基本操作
pub mod euler {
    use super::*;
    
    /// MEV (Make Edge Vertex): 新しい辺と頂点を作成
    pub fn make_edge_vertex<T: Scalar>(
        wire: &mut Wire<T>,
        from_vertex: Arc<Vertex<T>>,
        new_point: Point3D<T>,
        curve: CurveRef<T>,
    ) -> Arc<Vertex<T>> {
        let new_vertex = Arc::new(Vertex::new(new_point, T::default_tolerance()));
        let new_edge = Arc::new(Edge::new(
            from_vertex,
            Arc::clone(&new_vertex),
            curve,
            (T::zero(), T::one()),
        ));
        
        wire.edges.push(new_edge);
        new_vertex
    }
    
    /// MEL (Make Edge Loop): ループで辺を作成
    pub fn make_edge_loop<T: Scalar>(
        face: &mut Face<T>,
        start: Arc<Vertex<T>>,
        end: Arc<Vertex<T>>,
        curve: CurveRef<T>,
    ) -> Arc<Edge<T>> {
        // TODO: 実装
        unimplemented!()
    }
    
    /// KEV (Kill Edge Vertex): 辺と頂点を削除
    pub fn kill_edge_vertex<T: Scalar>(
        wire: &mut Wire<T>,
        edge: &Edge<T>,
    ) {
        // TODO: 実装
        unimplemented!()
    }
}
```

---

### Phase 4.2: エンティティ層（geo_entity）

#### 2.1 エンティティの概念

**エンティティ** = トポロジー + 属性 + メタデータ

```rust
// model/geo_entity/src/lib.rs

use geo_topology::{Solid, Face, Edge, Vertex, TopoId};
use geo_foundation::Scalar;
use std::collections::HashMap;
use std::sync::Arc;

/// エンティティの一意識別子（UUID）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(uuid::Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

/// 属性値
#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Color([f32; 4]),
    Custom(Box<dyn std::any::Any + Send + Sync>),
}

/// 属性キー
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeKey(String);

impl AttributeKey {
    // システム属性（予約済み）
    pub const NAME: &'static str = "system.name";
    pub const DESCRIPTION: &'static str = "system.description";
    pub const MATERIAL: &'static str = "system.material";
    pub const COLOR: &'static str = "system.color";
    pub const LAYER: &'static str = "system.layer";
    pub const TOLERANCE: &'static str = "system.tolerance";
    
    /// アプリケーション属性（ユーザー定義）
    pub fn app(name: &str) -> Self {
        Self(format!("app.{}", name))
    }
    
    /// システム属性
    pub fn system(name: &str) -> Self {
        Self(format!("system.{}", name))
    }
}

/// 属性コンテナ
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    data: HashMap<AttributeKey, AttributeValue>,
}

impl Attributes {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set(&mut self, key: AttributeKey, value: AttributeValue) {
        self.data.insert(key, value);
    }
    
    pub fn get(&self, key: &AttributeKey) -> Option<&AttributeValue> {
        self.data.get(key)
    }
    
    pub fn remove(&mut self, key: &AttributeKey) -> Option<AttributeValue> {
        self.data.remove(key)
    }
    
    /// 名前を設定（便利メソッド）
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.set(
            AttributeKey::system(AttributeKey::NAME),
            AttributeValue::String(name.into()),
        );
    }
    
    /// 色を設定（便利メソッド）
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.set(
            AttributeKey::system(AttributeKey::COLOR),
            AttributeValue::Color(color),
        );
    }
}

/// ソリッドエンティティ
#[derive(Debug, Clone)]
pub struct SolidEntity<T: Scalar> {
    id: EntityId,
    topology: Arc<Solid<T>>,
    attributes: Attributes,
}

impl<T: Scalar> SolidEntity<T> {
    pub fn new(topology: Arc<Solid<T>>) -> Self {
        Self {
            id: EntityId::new(),
            topology,
            attributes: Attributes::new(),
        }
    }
    
    pub fn id(&self) -> EntityId {
        self.id
    }
    
    pub fn topology(&self) -> &Solid<T> {
        &self.topology
    }
    
    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }
    
    pub fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }
}

/// 面エンティティ
#[derive(Debug, Clone)]
pub struct FaceEntity<T: Scalar> {
    id: EntityId,
    topology: Arc<Face<T>>,
    attributes: Attributes,
}

/// 辺エンティティ
#[derive(Debug, Clone)]
pub struct EdgeEntity<T: Scalar> {
    id: EntityId,
    topology: Arc<Edge<T>>,
    attributes: Attributes,
}

/// 頂点エンティティ
#[derive(Debug, Clone)]
pub struct VertexEntity<T: Scalar> {
    id: EntityId,
    topology: Arc<Vertex<T>>,
    attributes: Attributes,
}
```

#### 2.2 エンティティコレクション（モデル）

```rust
// model/geo_entity/src/model.rs

use super::*;

/// CADモデル - エンティティの集合
#[derive(Debug, Clone)]
pub struct Model<T: Scalar> {
    id: EntityId,
    name: String,
    
    /// 全ソリッドエンティティ
    solids: HashMap<EntityId, SolidEntity<T>>,
    
    /// 全フェースティティ（スタンドアロンの面）
    faces: HashMap<EntityId, FaceEntity<T>>,
    
    /// メタデータ
    metadata: HashMap<String, String>,
}

impl<T: Scalar> Model<T> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: EntityId::new(),
            name: name.into(),
            solids: HashMap::new(),
            faces: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// ソリッドを追加
    pub fn add_solid(&mut self, solid: SolidEntity<T>) -> EntityId {
        let id = solid.id();
        self.solids.insert(id, solid);
        id
    }
    
    /// ソリッドを取得
    pub fn get_solid(&self, id: &EntityId) -> Option<&SolidEntity<T>> {
        self.solids.get(id)
    }
    
    /// ソリッドを削除
    pub fn remove_solid(&mut self, id: &EntityId) -> Option<SolidEntity<T>> {
        self.solids.remove(id)
    }
    
    /// 全ソリッドを取得
    pub fn solids(&self) -> impl Iterator<Item = &SolidEntity<T>> {
        self.solids.values()
    }
    
    /// 名前でエンティティを検索
    pub fn find_by_name(&self, name: &str) -> Vec<EntityId> {
        let mut result = Vec::new();
        
        for solid in self.solids.values() {
            if let Some(AttributeValue::String(n)) = solid.attributes()
                .get(&AttributeKey::system(AttributeKey::NAME))
            {
                if n == name {
                    result.push(solid.id());
                }
            }
        }
        
        result
    }
}
```

---

### Phase 4.3: パラメータ管理層

#### 3.1 トレランス管理

```rust
// model/geo_foundation/src/tolerance.rs

use analysis::Scalar;

/// グローバルトレランス設定
#[derive(Debug, Clone)]
pub struct ToleranceSettings<T: Scalar> {
    /// 位置精度（mm）
    pub position: T,
    
    /// 角度精度（radian）
    pub angle: T,
    
    /// 曲率精度
    pub curvature: T,
    
    /// テセレーション精度
    pub tessellation: T,
}

impl<T: Scalar> Default for ToleranceSettings<T> {
    fn default() -> Self {
        Self {
            position: T::from_f64(0.001).unwrap(),      // 0.001mm = 1μm
            angle: T::from_f64(0.0001).unwrap(),        // 約0.0057度
            curvature: T::from_f64(0.01).unwrap(),      // 曲率トレランス
            tessellation: T::from_f64(0.01).unwrap(),   // 表示用トレランス
        }
    }
}

impl<T: Scalar> ToleranceSettings<T> {
    /// CAM用の高精度設定
    pub fn cam_precision() -> Self {
        Self {
            position: T::from_f64(0.0001).unwrap(),     // 0.1μm
            angle: T::from_f64(0.00001).unwrap(),
            curvature: T::from_f64(0.001).unwrap(),
            tessellation: T::from_f64(0.001).unwrap(),
        }
    }
    
    /// 表示用の低精度設定
    pub fn display() -> Self {
        Self {
            position: T::from_f64(0.01).unwrap(),       // 0.01mm = 10μm
            angle: T::from_f64(0.001).unwrap(),
            curvature: T::from_f64(0.1).unwrap(),
            tessellation: T::from_f64(0.05).unwrap(),
        }
    }
}

/// アプリケーションコンテキスト
/// 
/// トレランスやその他のグローバル設定を管理
#[derive(Debug, Clone)]
pub struct ApplicationContext<T: Scalar> {
    tolerance: ToleranceSettings<T>,
    precision_mode: PrecisionMode,
}

impl<T: Scalar> Default for ApplicationContext<T> {
    fn default() -> Self {
        Self {
            tolerance: ToleranceSettings::default(),
            precision_mode: PrecisionMode::Standard,
        }
    }
}

impl<T: Scalar> ApplicationContext<T> {
    pub fn new(precision_mode: PrecisionMode) -> Self {
        let tolerance = match precision_mode {
            PrecisionMode::Display => ToleranceSettings::display(),
            PrecisionMode::Standard => ToleranceSettings::default(),
            PrecisionMode::CamPrecision => ToleranceSettings::cam_precision(),
        };
        
        Self {
            tolerance,
            precision_mode,
        }
    }
    
    pub fn tolerance(&self) -> &ToleranceSettings<T> {
        &self.tolerance
    }
    
    pub fn set_tolerance(&mut self, tolerance: ToleranceSettings<T>) {
        self.tolerance = tolerance;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionMode {
    Display,        // 表示用（低精度）
    Standard,       // 標準（中精度）
    CamPrecision,   // CAM用（高精度）
}
```

#### 3.2 アプリケーション層からのパラメータ受け渡し

```rust
// viewmodel/converter/src/context.rs

use geo_foundation::{ApplicationContext, ToleranceSettings, PrecisionMode};

/// ViewModel変換コンテキスト
/// 
/// アプリケーション層からの設定を受け取り、Model層に伝達
pub struct ConversionContext {
    /// 精度モード
    precision: PrecisionMode,
    
    /// トレランス設定
    tolerance: ToleranceSettings<f64>,
    
    /// 表示オプション
    display_options: DisplayOptions,
}

impl ConversionContext {
    pub fn new(precision: PrecisionMode) -> Self {
        let tolerance = match precision {
            PrecisionMode::Display => ToleranceSettings::display(),
            PrecisionMode::Standard => ToleranceSettings::default(),
            PrecisionMode::CamPrecision => ToleranceSettings::cam_precision(),
        };
        
        Self {
            precision,
            tolerance,
            display_options: DisplayOptions::default(),
        }
    }
    
    pub fn tolerance(&self) -> &ToleranceSettings<f64> {
        &self.tolerance
    }
    
    pub fn precision_mode(&self) -> PrecisionMode {
        self.precision
    }
}

#[derive(Debug, Clone)]
pub struct DisplayOptions {
    pub show_edges: bool,
    pub show_vertices: bool,
    pub wireframe_mode: bool,
    pub tessellation_quality: TessellationQuality,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            show_edges: true,
            show_vertices: false,
            wireframe_mode: false,
            tessellation_quality: TessellationQuality::Standard,
        }
    }
}
```

---

## 実装計画

### Phase 4.1: トポロジー層実装（2-3週間）

#### Week 1: 基本データ構造
- [ ] `geo_topology` クレート作成
- [ ] Vertex, Edge, Wire 実装
- [ ] 基本的な接続性チェック
- [ ] 単体テスト

#### Week 2: 高レベル構造
- [ ] Face, Shell, Solid 実装
- [ ] 境界チェック（閉じ性）
- [ ] トポロジーID管理
- [ ] 単体テスト

#### Week 3: オイラー操作
- [ ] MEV, MEL, KEV 実装
- [ ] トポロジー整合性検証
- [ ] 統合テスト

### Phase 4.2: エンティティ層実装（1-2週間）

#### Week 4: エンティティ基盤
- [ ] `geo_entity` クレート作成
- [ ] 属性システム実装
- [ ] EntityId, Attributes 実装
- [ ] SolidEntity, FaceEntity 実装

#### Week 5: モデル管理
- [ ] Model クラス実装
- [ ] エンティティ検索機能
- [ ] シリアライゼーション準備

### Phase 4.3: パラメータ管理層実装（1週間）

#### Week 6: トレランス・コンテキスト
- [ ] ToleranceSettings 実装
- [ ] ApplicationContext 実装
- [ ] ConversionContext 実装
- [ ] アプリケーション層統合

---

## ロードマップ統合

### ROADMAP_2026_Q1_Q2.md への追加

Phase 4 を **Tier 3（中優先）** として追加：

```markdown
## Tier 3: 形状モデリング基盤構築（中優先）

### 3.4 トポロジー層実装 (Phase 4.1)

**工数**: 15日（3週間）  
**内容**: B-Rep（Vertex/Edge/Face/Solid）、オイラー操作

### 3.5 エンティティ層実装 (Phase 4.2)

**工数**: 10日（2週間）  
**内容**: 属性システム、エンティティ管理、Modelクラス

### 3.6 パラメータ管理層実装 (Phase 4.3)

**工数**: 5日（1週間）  
**内容**: トレランス管理、アプリケーションコンテキスト
```

### スケジュール追加

```markdown
### 2026年7月（Week 21-24） - Phase 4着手

**Week 21** (6/27-7/3):
- トポロジー層設計完了
- Vertex, Edge, Wire 実装開始

**Week 22** (7/4-7/10):
- Wire実装完了
- Face, Shell実装開始

**Week 23** (7/11-7/17):
- Solid実装完了
- オイラー操作実装開始

**Week 24** (7/18-7/24):
- オイラー操作完了
- トポロジー層統合テスト

### 2026年8月（Week 25-28） - Phase 4完了

**Week 25** (7/25-7/31):
- エンティティ層実装開始
- 属性システム実装

**Week 26** (8/1-8/7):
- Modelクラス実装
- エンティティ検索機能

**Week 27** (8/8-8/14):
- パラメータ管理層実装
- トレランス・コンテキスト

**Week 28** (8/15-8/21):
- Phase 4 統合テスト
- ドキュメント整備
```

---

## 完了条件

### Phase 4.1 完了条件
- [ ] Vertex, Edge, Wire, Face, Shell, Solid が実装されている
- [ ] トポロジー整合性チェックが機能している
- [ ] オイラー操作（MEV, MEL, KEV）が実装されている
- [ ] 全単体テストが通過している

### Phase 4.2 完了条件
- [ ] 属性システムが実装されている
- [ ] SolidEntity, FaceEntity が実装されている
- [ ] Modelクラスでエンティティ管理ができる
- [ ] エンティティ検索機能が動作している

### Phase 4.3 完了条件
- [ ] ToleranceSettings, ApplicationContext が実装されている
- [ ] アプリケーション層から精度モードを指定できる
- [ ] ConversionContext でトレランスを受け渡せる
- [ ] ドキュメントが整備されている

---

## 次のアクション

1. **即座に実施**:
   - [ ] ROADMAP_2026_Q1_Q2.md に Phase 4 追加
   - [ ] GitHub Issue 作成（Phase 4.1, 4.2, 4.3）

2. **Phase 3完了後に着手**:
   - [ ] トポロジー層の詳細設計
   - [ ] プロトタイプ実装（Vertex, Edge, Wire）

3. **参考資料調査**:
   - [ ] OpenCASCADE の TopoDS 設計
   - [ ] Parasolid の PK ドキュメント
   - [ ] 「Geometric and Solid Modeling」（学術書）

**次回レビュー**: Phase 3完了時（2026年4月末予定）
