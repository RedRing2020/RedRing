# Phase 4: トポロジー・エンティティ層の設計

**作成日**: 2026年2月8日  
**最終更新**: 2026年3月28日  
**ステータス**: 設計フェーズ  
**優先度**: 🟡 Tier 3（Phase 3完了後に着手）

---

## 📋 目次

1. [概要](#概要)
2. [背景・動機](#背景動機)
3. [アーキテクチャ設計](#アーキテクチャ設計)
4. [実装計画](#実装計画)
5. [ロードマップ統合](#ロードマップ統合)

関連設計ドキュメント:

- topology entity layer 設計正本: `dev/architecture/TOPOLOGY_ENTITY_LAYER_DESIGN.md`
- topo正規形不変条件凍結（#458 時点のスナップショット）: `dev/architecture/TOPO_NORMAL_FORM_INVARIANTS_FREEZE_458.md`

> 本ドキュメントは topology entity layer の実装計画・補助設計書であり、
> topology 設計の正本は `TOPOLOGY_ENTITY_LAYER_DESIGN.md` を参照する。
> `TOPO_NORMAL_FORM_INVARIANTS_FREEZE_458.md` は #458 時点の簡易実装前提でまとめた凍結スナップショットであり、
> 現行 topology 設計の単独正本とはみなさない。
> 2026年3月28日更新: #458 向けに「正規形不変条件 凍結候補 v0.1」を作成
> 2026年3月28日更新: #407 向けに「モデリング用正規形」と「表示用トリム表現」の境界を明文化
> **2026年3月追記**: PCurve・トリム表現・δ/2 トレランスモデルの設計方針を Phase 4.4 として追加

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
- LineSegment3D は線形 shape 実装であり、意味論の正本は `GEOMETRY_SHAPE_SEMANTICS_DESIGN.md` を参照する

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
│  基盤層 (geo_contracts, analysis)   │
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
use geo_contracts::Scalar;
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

    /// 開始頂点（トポロジー上の走査始点）
    start_vertex: Arc<Vertex<T>>,

    /// 終了頂点（トポロジー上の走査終点）
    end_vertex: Arc<Vertex<T>>,

    /// 母曲線への参照
    ///
    /// 母曲線自体の反転は行わない（Face の same_sense と同じ原則）。
    /// 母曲線を変更すると共有 Edge 経由で隣接面に影響が波及し
    /// 整合性破綻のリスクが高いため。
    curve: CurveRef<T>,

    /// パラメータ範囲 [t_start, t_end]
    parameter_range: (T, T),

    /// 走査向きフラグ: 母曲線の自然方向と Edge の走査方向が一致するか
    ///
    /// - `true` : t_start → t_end が start_vertex → end_vertex と一致
    /// - `false`: 母曲線の幾何は逆向き（ただしトポロジー上の向きは不変）
    ///
    /// 向き反転は常にこのフラグを切り替えるだけで実現する。
    same_sense: bool,
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
            same_sense: true,
        }
    }

    /// 辺上の点を取得（パラメータ t: 0.0-1.0）
    ///
    /// `same_sense` が false の場合はパラメータを反転して評価する。
    pub fn point_at(&self, t: T) -> Point3D<T> {
        let t_mapped = if self.same_sense { t } else { T::ONE - t };
        let (t0, t1) = self.parameter_range;
        let param = t0 + (t1 - t0) * t_mapped;

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
            CurveRef::Circle(arc) => arc.length(),
            CurveRef::Ellipse(ellipse) => ellipse.arc_length(),
            CurveRef::Nurbs(nurbs) => nurbs.arc_length_total(),
        }
    }

    /// 向き反転（母曲線は変更せず same_sense フラグのみ切り替える）
    pub fn reverse(&mut self) {
        self.same_sense = !self.same_sense;
        // start_vertex と end_vertex も入れ替える
        std::mem::swap(
            Arc::get_mut(&mut self.start_vertex).unwrap(),
            Arc::get_mut(&mut self.end_vertex).unwrap(),
        );
    }

    pub fn same_sense(&self) -> bool {
        self.same_sense
    }
}

/// Wire（ワイヤー）- 向きが揃った Edge の連続列
///
/// ## 向きの整合性ルール
///
/// Wire 内の Edge は以下を満たす必要がある：
/// - `edges[i].end_vertex` ≈ `edges[i+1].start_vertex`（δ/2 トレランス範囲内で一致）
///
/// これにより Wire を走査すると幾何的に連続したパスになる。
/// CCW（外周）か CW（穴）かの巻き方向は Wire 自体は保持しない。
/// 巻き方向の意味付けは Face に outer_loop/inner_loop として割り当てた時点で行う。
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

        // Edge の向きが揃っているか（end → next.start が接続）をチェック
        for i in 0..edges.len() - 1 {
            let current_end = &edges[i].end_vertex;
            let next_start = &edges[i + 1].start_vertex;

            if !current_end.is_coincident(next_start) {
                return Err(WireError::DiscontinuousEdges);
            }
        }

        // 閉じているかチェック（外周/内周ループの前提条件）
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
    /// Edge の end_vertex と次 Edge の start_vertex が一致しない（向きが揃っていない）
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

    /// 外側境界（常に 1 つのみ・型で強制）
    outer_loop: Arc<Wire<T>>,

    /// 内側境界（穴）のリスト（0 個以上）
    inner_loops: Vec<Arc<Wire<T>>>,

    /// 母曲面への参照
    ///
    /// 母曲面自体の表裏反転は行わない。
    /// 母曲面を変更すると他の Face・接続 Edge への影響が伴い
    /// 整合性破綻のリスクが高いため。
    surface: SurfaceRef<T>,

    /// 法線向きフラグ: 母曲面の表方向と Face の表方向が一致するか
    ///
    /// - `true` : 母曲面の法線と同じ向き（表向き）
    /// - `false`: 母曲面の法線と逆向き（裏向き）
    ///
    /// 表裏反転は常にこのフラグを切り替えるだけで実現する。
    /// 母曲面のデータは変更しない。
    same_sense: bool,
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
            same_sense: true, // 新規作成時は母曲面の表向きと一致
        })
    }

    /// 面の法線ベクトル（UV 座標で評価）
    ///
    /// `same_sense` フラグに従い、必要に応じて反転する。
    pub fn normal_at(&self, u: T, v: T) -> Vector3D<T> {
        let n = match &self.surface {
            SurfaceRef::Plane(plane) => plane.normal(),
            SurfaceRef::Cylinder(cyl) => cyl.normal_at(u, v),
            SurfaceRef::Cone(cone) => cone.normal_at(u, v),
            SurfaceRef::Sphere(sphere) => sphere.normal_at(u, v),
            SurfaceRef::Nurbs(nurbs) => nurbs.normal_at(u, v).unwrap_or_default(),
        };
        if self.same_sense { n } else { -n }
    }

    /// 表裏反転（母曲面は変更せず same_sense フラグのみ切り替える）
    pub fn reverse(&mut self) {
        self.same_sense = !self.same_sense;
    }

    pub fn same_sense(&self) -> bool {
        self.same_sense
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceError {
    OuterLoopNotClosed,
    InnerLoopNotClosed,
}

/// シェル（Shell）- 接続された面の集合（シート面・複合面）
///
/// ## ⚠️ 詳細設計は別 Issue で扱う
///
/// 以下の処理は Vertex/Edge/Wire/Face の基本実装完了後に別 Issue で設計する：
/// - **ニット処理**（複数 Face の Edge を縫合して Shell を構築）
/// - **多様体検証**（各 Edge に Face が 1〜2 枚接続されているか確認）
///
/// これらは Wire 接続性と Face の same_sense 管理が完成していることを前提とする。
#[derive(Debug, Clone)]
pub struct Shell<T: Scalar> {
    id: TopoId,
    faces: Vec<Arc<Face<T>>>,
    is_closed: bool,
}

impl<T: Scalar> Shell<T> {
    pub fn new(faces: Vec<Arc<Face<T>>>) -> Self {
        // TODO: シェルの閉じ性を計算（別 Issue で設計）
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

/// 立体（Solid）- 閉じたシェルで囲まれた体積（ボディ）
///
/// ## ⚠️ 詳細設計は別 Issue で扱う
///
/// 以下の処理は Shell の実装完了後に別 Issue で設計する：
/// - **閉じた Shell の検証**（全 Edge に Face が正確に 2 枚接続）
/// - **体積積分による法線向き確認**
///   （Divergence Theorem: $V = \frac{1}{6}\sum_{\text{face}} \mathbf{n} \cdot \mathbf{p} \cdot A$
///   で体積が正なら外側が表向き）
/// - **ボディ化**（シート面 Shell → 閉じた Solid への変換処理）
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
use geo_contracts::Scalar;
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
// model/geo_contracts/src/tolerance.rs

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

use geo_contracts::{ApplicationContext, ToleranceSettings, PrecisionMode};

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

### Phase 4.4: PCurve・トリム表現・トポロジー整合性モデル

> **背景**: 2026年3月の設計議論で確認された方針。トリム曲線/曲面を正しく扱うために必須。

#### 4.0 正規形の責務境界（#407 合意事項）

`geo_topology` におけるモデリング用の正規形と、表示・入出力向けのトリム表現を混在させないため、以下を固定ルールとする。

| 項目 | モデリング用正規形 | 表示用/入出力用表現 |
|---|---|---|
| Edge の幾何保持 | 母曲線 + parameter range + same_sense | 描画都合の分割済み線分列や専用トリム形状を許可 |
| 直接利用先 | 交差判定、接続判定、トポロジー比較、編集演算 | テッセレーション、表示、交換フォーマット変換 |
| ID の保持 | topo 要素が保持する | 派生表示形状は保持しない |
| トリム解除 | parameter range の変更で表現 | 母曲線差し替えは別操作 |
| 正本 | #458 の topo 正規形不変条件 | 本ドキュメント 4.4 節の派生表現 |

固定する境界は次の通り。

- モデリング演算は常に「母曲線 + parameter range」を入力とし、表示用トリム形状を直接受け取らない。
- `Arc` は primitive の形状として維持してよいが、topo の内部表現では「母曲線参照 + 区間」で比較・連結する。
- 向き反転は母曲線の破壊的変更ではなく `same_sense` で表現する。
- 同一性は topo 要素が保持し、描画用メッシュ・一時トリム曲線・交換フォーマット用投影形状には移譲しない。
- 本節の詳細は #458 の NFI-01〜NFI-05 に従う。

#### 4.1 設計原則: トリム表現の二重幾何管理

商用 CAD カーネルで実績のある設計を採用する。各 Edge は **3D 曲線** と **PCurve（面上の 2D パラメータ曲線）** の両方を独立して保持する。

```
Edge
  ├── 3D Curve     : 3次元空間での幾何（NurbsCurve3D 等）
  ├── PCurve on Face A : 面A のパラメータ空間での 2D 曲線
  └── PCurve on Face B : 面B のパラメータ空間での 2D 曲線

Face
  ├── Surface      : 母曲面の幾何（NurbsSurface3D 等）
  └── Loop
        └── Edge → PCurve （面境界の 2D 表現）
```

これにより：
- 面の境界をパラメータ空間で厳密に定義できる（トリム曲面）
- 形状変更後は PCurve を母曲面で再評価するだけで 3D 位置を復元できる（フィーチャ編集耐性）
- G1/G2 連続性確認は PCurve の微分をチェーンルール（曲面の Jacobian）で変換して実施できる

**運用境界**:

- PCurve は Face 側の境界表現であり、Edge の正規形そのものではない。
- `TrimmedCurve<T>` / `TrimmedSurface<T>` は表示・交換・評価補助の派生表現であり、トポロジー比較の正本として扱わない。
- 交差演算や編集演算で必要な場合は、まず母曲線/母曲面と parameter range に正規化してから処理する。
- `LineSegment` の `support_line` / 拘束点 / `length()` / `point_at_parameter()` の意味論は、本書ではなく `GEOMETRY_SHAPE_SEMANTICS_DESIGN.md` を正本とする。
- `#557` の段階では topology 現行構造の暫定維持を許容するが、vertex binding invariant と位相専用 tolerance は未確定論点として別管理する。

#### 4.2 PCurve の定義

```rust
/// 面上の 2D パラメータ曲線（PCurve）
///
/// 母曲面の UV パラメータ空間における曲線定義。
/// Edge が保持する PCurve と母曲面を組み合わせることで
/// 稜線の 3D 位置を完全に復元できる。
#[derive(Debug, Clone)]
pub struct PCurve<T: Scalar> {
    /// 母曲面の参照
    surface: Arc<SurfaceRef<T>>,

    /// UV パラメータ空間での曲線（常に [0,1] を使用）
    curve_2d: Curve2DRef<T>,

    /// パラメータ範囲
    parameter_range: (T, T),
}

/// 2D 曲線の種別
#[derive(Debug, Clone)]
pub enum Curve2DRef<T: Scalar> {
    Line(Line2D<T>),
    Nurbs(NurbsCurve2D<T>),
}

impl<T: Scalar> PCurve<T> {
    /// UV 座標を評価
    pub fn evaluate_uv(&self, t: T) -> (T, T) { /* ... */ }

    /// 母曲面上の 3D 点を評価
    pub fn evaluate_3d(&self, t: T) -> Point3D<T> {
        let (u, v) = self.evaluate_uv(t);
        self.surface.evaluate(u, v)
    }
}
```

#### 4.3 トリム曲線・トリム曲面の定義

```rust
/// トリム曲線
///
/// 元の曲線（母曲線）にパラメータ範囲 [t_start, t_end] を適用した部分曲線。
/// 幾何は母曲線で管理し、トリム情報として範囲のみを保持する。
#[derive(Debug, Clone)]
pub struct TrimmedCurve<T: Scalar> {
    /// 母曲線（[0,1] ドメイン）
    basis_curve: Arc<NurbsCurve3D<T>>,

    /// トリム範囲（母曲線のパラメータ空間で指定）
    t_start: T,
    t_end: T,
}

impl<T: Scalar> TrimmedCurve<T> {
    /// [0,1] ローカルパラメータで評価
    pub fn evaluate(&self, t: T) -> Point3D<T> {
        let param = self.t_start + (self.t_end - self.t_start) * t;
        self.basis_curve.evaluate(param)
    }
}

/// トリム曲面
///
/// 母曲面に閉じたループ（境界ワイヤー）を適用してトリムした曲面。
/// 境界は PCurve で面のパラメータ空間に記述する。
#[derive(Debug, Clone)]
pub struct TrimmedSurface<T: Scalar> {
    /// 母曲面
    basis_surface: Arc<SurfaceRef<T>>,

    /// 外側境界ループ（PCurve のリスト）
    outer_loop: Vec<PCurve<T>>,

    /// 内側境界ループ（穴）のリスト
    inner_loops: Vec<Vec<PCurve<T>>>,
}
```

#### 4.4 トレランス割り当て規則: δ/2 モデル

稜線（Edge）に対してトレランスを割り当てる際は、**δ/2 モデル**を採用する。

**原則**:
$$d\bigl(E_{\text{3D}},\; S_{uv \to xyz}\bigr) \leq \frac{\delta_{\text{dist}}}{2}$$

- Edge の 3D 曲線と PCurve を母曲面で評価した 3D 点の距離が `distance_tolerance / 2` 以内
- 隣接する 2 つの Face がそれぞれ `distance_tolerance / 2` の誤差を持っていても、稜線共有部での最大ギャップは `distance_tolerance` 以内に自動的に収まる

**効果**: 面間のウォータータイトネス（隙間なし条件）を代数的に保証できる。

```rust
impl<T: Scalar> Edge<T> {
    /// Edge の 3D 曲線と PCurve の整合性をチェック
    ///
    /// δ/2 モデルに基づき、3D 曲線と PCurve（→母曲面評価）の距離が
    /// distance_tolerance / 2 以内であることを確認する。
    pub fn validate_pcurve_consistency(
        &self,
        tolerance: &ToleranceSettings<T>,
        num_samples: usize,
    ) -> bool {
        let half_tol = tolerance.distance_tolerance / T::from_f64(2.0).unwrap();
        // サンプル点で 3D 曲線 と PCurve 評価点の距離を検証
        // ...
        true // TODO: 実装
    }
}
```

#### 4.5 G1/G2 連続性の確認アプローチ

パラメータ空間の PCurve に対してチェーンルールを適用し、3D 接続条件を得る。

$$\frac{d\mathbf{r}}{dt} = \frac{\partial \mathbf{S}}{\partial u} \cdot \frac{du}{dt} + \frac{\partial \mathbf{S}}{\partial v} \cdot \frac{dv}{dt}$$

- **G1 確認**: 隣接エッジ端点での接線方向（正規化）が一致するか
- **G2 確認**: 曲率ベクトルの一致まで確認（自動車意匠面 Class A サーフェス要件）

[0,1] 正規化はリパラメータ化であり G1・G2 は不変なので、正規化後の PCurve でそのまま確認できる。

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

### Phase 4.4: PCurve・トリム表現実装（2週間）

#### Week 7: PCurve とトリム曲線
- [ ] `PCurve<T>` 構造体実装（Curve2DRef, evaluate_uv, evaluate_3d）
- [ ] `TrimmedCurve<T>` 実装（母曲線 + パラメータ範囲）
- [ ] δ/2 整合性チェック（`validate_pcurve_consistency`）
- [ ] 単体テスト

#### Week 8: トリム曲面と連続性確認
- [ ] `TrimmedSurface<T>` 実装（母曲面 + 外側/内側ループ）
- [ ] `Edge` への PCurve フィールド追加（`pcurve_on_faces`）
- [ ] G1 連続性確認（接線方向比較）
- [ ] G2 連続性確認（曲率ベクトル比較、Class A 面要件）
- [ ] 統合テスト

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

