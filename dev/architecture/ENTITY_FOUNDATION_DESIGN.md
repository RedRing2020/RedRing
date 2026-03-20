# エンティティ層基礎設計 - Phase 4 前倒し実装

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月8日  
**ステータス**: 設計フェーズ  
**優先度**: 🟠 Tier 2（形状可視化・CAM可視化完了後）  
**関連Issue**: #206（エンティティ層基礎）, #205（Phase 4完全版）

---

## 📋 目次

1. [概要](#概要)
2. [背景・動機](#背景動機)
3. [Phase 4との関係](#phase-4との関係)
4. [基礎設計](#基礎設計)
5. [実装計画](#実装計画)
6. [移行戦略](#移行戦略)

---

## 概要

Phase 4の完全なトポロジー層実装（B-Rep、Euler操作）の前に、**最小限のエンティティ概念**を先行導入します。

### 導入する機能（基礎版）

1. **EntityId**: UUID ベースの一意識別子
2. **DisplayAttributes**: 表示属性（色、線種、レイヤー）
3. **Metadata**: 基本メタデータ（名前、作成日時）
4. **CAMエンティティ**: 工具経路のエンティティ表現

### 導入しない機能（Phase 4完全版で実装）

- ❌ B-Rep トポロジー（Vertex/Edge/Face/Solid）
- ❌ Euler操作（MEV/MEL/KEV）
- ❌ トポロジー整合性検証
- ❌ パラメトリック依存関係
- ❌ フィーチャーツリー

---

## 背景・動機

### なぜエンティティ層を前倒しするのか

#### 1. デバッグ表示から本格CAD表示への移行

**現状の問題**:
```rust
// viewmodel/converter: 直接変換（IDなし）
fn circle_to_vertices(circle: &Circle3D) -> Vec<Vertex3D> {
    // IDの概念なし、選択不可、属性なし
}
```

**エンティティ導入後**:
```rust
pub struct GeometricEntity<T: Scalar> {
    id: EntityId,                    // ✅ 選択可能
    geometry: Circle3D<T>,           // ✅ 幾何データ
    display: DisplayAttributes,      // ✅ 色・線種
    metadata: Metadata,              // ✅ 名前・タグ
}

// 選択・ハイライト・色変更が可能に
```

#### 2. CAM可視化のエンティティ化

**現状の問題**:
```rust
// 単純なデータ構造
struct ToolPath {
    segments: Vec<PathSegment>,
    // IDなし、属性なし
}
```

**エンティティ導入後**:
```rust
pub struct ToolPathEntity {
    id: EntityId,                    // ✅ 工具経路の識別
    path: ToolPath,                  // ✅ 経路データ
    machining_params: MachiningAttr, // ✅ 工具番号、送り速度
    display: DisplayAttributes,      // ✅ 表示色
    metadata: Metadata,              // ✅ 加工名称
}

// 経路の選択、パラメータ編集、色分けが可能に
```

#### 3. Phase 4への段階的移行

```text
現在:                基礎版（Issue #206）:      Phase 4完全版（Issue #205）:
幾何データのみ  →  エンティティ+属性  →  トポロジー+エンティティ

Circle3D             GeometricEntity         TopologyEntity
├─ center                ├─ id                    ├─ id
├─ radius                ├─ geometry              ├─ topology (B-Rep)
└─ (データのみ)          │  └─ Circle3D           │  ├─ Vertex
                         ├─ display               │  ├─ Edge
                         │  ├─ color              │  └─ Face
                         │  ├─ line_style         ├─ geometry (参照)
                         │  └─ layer              ├─ display
                         └─ metadata              ├─ constraints
                            ├─ name               └─ metadata
                            └─ created_at
```

**段階的移行のメリット**:
- ✅ 一度に大きな変更を避ける（リスク低減）
- ✅ 基礎版で使い勝手を検証
- ✅ Phase 4で完全なトポロジーを追加

---

## Phase 4との関係

### Phase 4の3段階実装

| Phase | 内容 | Issue | 工数 | 実施時期 |
|-------|------|-------|------|----------|
| **4.0（基礎版）** | エンティティ+属性のみ | #206 | 3-4週間 | Week 7-10 |
| **4.1** | トポロジー層（B-Rep） | #205 | 3週間 | Week 19-22 |
| **4.2** | エンティティ統合 | #205 | 2週間 | Week 23-24 |
| **4.3** | パラメータ管理 | #205 | 1週間 | Week 25 |

### 基礎版（#206）のスコープ

**実装する内容**:
- ✅ `geo_entity` クレート作成（最小構成）
- ✅ `EntityId` - UUID ベース
- ✅ `DisplayAttributes` - 色、線種、レイヤー
- ✅ `Metadata` - 名前、タグ、作成日時
- ✅ `GeometricEntity<T, G>` - ジェネリックエンティティ
- ✅ `CAMEntity` - 工具経路エンティティ
- ✅ ViewModel層での変換対応

**実装しない内容（Phase 4.1-4.3で実装）**:
- ❌ B-Rep トポロジー（`geo_topology` クレート）
- ❌ Euler操作
- ❌ トポロジー整合性検証
- ❌ パラメトリック制約
- ❌ フィーチャー履歴

---

## 基礎設計

### 4.1 クレート構成

```text
model/
├── geo_entity/          ← 新規（基礎版）
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── entity_id.rs       # UUID管理
│       ├── display.rs         # DisplayAttributes
│       ├── metadata.rs        # Metadata
│       ├── geometric_entity.rs # GeometricEntity
│       └── cam_entity.rs      # CAMEntity
└── geo_algorithms/
    └── src/
        └── toolpath.rs        # ToolPath（既存）
```

### 4.2 データ構造

#### 4.2.1 EntityId - UUID ベース識別子

```rust
// model/geo_entity/src/entity_id.rs

use uuid::Uuid;

/// エンティティの一意識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(Uuid);

impl EntityId {
    /// 新しいIDを生成
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// 文字列から復元（永続化用）
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
    
    /// 文字列表現
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}
```

#### 4.2.2 DisplayAttributes - 表示属性

```rust
// model/geo_entity/src/display.rs

/// 表示属性
#[derive(Debug, Clone)]
pub struct DisplayAttributes {
    /// 色（RGB, 0.0-1.0）
    pub color: [f32; 3],
    
    /// 線種
    pub line_style: LineStyle,
    
    /// 線幅（ピクセル）
    pub line_width: f32,
    
    /// レイヤー
    pub layer: String,
    
    /// 可視性
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,       // 実線
    Dashed,      // 破線
    Dotted,      // 点線
    DashDot,     // 一点鎖線
    Hidden,      // 隠線
}

impl Default for DisplayAttributes {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],  // 白色
            line_style: LineStyle::Solid,
            line_width: 1.0,
            layer: "0".to_string(),  // デフォルトレイヤー
            visible: true,
        }
    }
}

impl DisplayAttributes {
    /// CAD標準色プリセット
    pub fn cad_red() -> Self {
        Self { color: [1.0, 0.0, 0.0], ..Default::default() }
    }
    
    pub fn cad_green() -> Self {
        Self { color: [0.0, 1.0, 0.0], ..Default::default() }
    }
    
    pub fn cad_blue() -> Self {
        Self { color: [0.0, 0.5, 1.0], ..Default::default() }
    }
    
    pub fn cad_yellow() -> Self {
        Self { color: [1.0, 1.0, 0.0], ..Default::default() }
    }
    
    /// CAM経路色プリセット
    pub fn cam_rapid() -> Self {
        // 早送り: 青色
        Self { color: [0.2, 0.5, 1.0], ..Default::default() }
    }
    
    pub fn cam_cutting() -> Self {
        // 切削送り: 白色
        Self { color: [1.0, 1.0, 1.0], ..Default::default() }
    }
    
    pub fn cam_approach() -> Self {
        // アプローチ: 緑色
        Self { color: [0.2, 1.0, 0.2], ..Default::default() }
    }
}
```

#### 4.2.3 Metadata - メタデータ

```rust
// model/geo_entity/src/metadata.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// メタデータ
#[derive(Debug, Clone)]
pub struct Metadata {
    /// エンティティ名称
    pub name: Option<String>,
    
    /// 説明文
    pub description: Option<String>,
    
    /// タグ（分類用）
    pub tags: Vec<String>,
    
    /// 作成日時
    pub created_at: DateTime<Utc>,
    
    /// 最終更新日時
    pub modified_at: DateTime<Utc>,
    
    /// カスタム属性（拡張用）
    pub custom: HashMap<String, String>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            name: None,
            description: None,
            tags: Vec::new(),
            created_at: now,
            modified_at: now,
            custom: HashMap::new(),
        }
    }
}

impl Metadata {
    /// 名前付きで作成
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }
    
    /// タグを追加
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }
    
    /// 最終更新日時を現在に設定
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }
}
```

#### 4.2.4 GeometricEntity - ジェネリックエンティティ

```rust
// model/geo_entity/src/geometric_entity.rs

use geo_contracts::Scalar;

/// 幾何エンティティ
#[derive(Debug, Clone)]
pub struct GeometricEntity<T: Scalar, G> {
    /// 一意識別子
    id: EntityId,
    
    /// 幾何データ
    geometry: G,
    
    /// 表示属性
    display: DisplayAttributes,
    
    /// メタデータ
    metadata: Metadata,
    
    /// 選択状態
    selected: bool,
    
    /// _phantom: PhantomData<T> (Scalarの型パラメータ保持用)
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Scalar, G> GeometricEntity<T, G> {
    /// 新規作成
    pub fn new(geometry: G) -> Self {
        Self {
            id: EntityId::new(),
            geometry,
            display: DisplayAttributes::default(),
            metadata: Metadata::default(),
            selected: false,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 表示属性付きで作成
    pub fn with_display(geometry: G, display: DisplayAttributes) -> Self {
        Self {
            display,
            ..Self::new(geometry)
        }
    }
    
    /// メタデータ付きで作成
    pub fn with_metadata(geometry: G, metadata: Metadata) -> Self {
        Self {
            metadata,
            ..Self::new(geometry)
        }
    }
    
    // アクセサ
    pub fn id(&self) -> EntityId { self.id }
    pub fn geometry(&self) -> &G { &self.geometry }
    pub fn geometry_mut(&mut self) -> &mut G { 
        self.metadata.touch();
        &mut self.geometry 
    }
    pub fn display(&self) -> &DisplayAttributes { &self.display }
    pub fn display_mut(&mut self) -> &mut DisplayAttributes { 
        self.metadata.touch();
        &mut self.display 
    }
    pub fn metadata(&self) -> &Metadata { &self.metadata }
    pub fn metadata_mut(&mut self) -> &mut Metadata { &mut self.metadata }
    
    /// 選択状態
    pub fn is_selected(&self) -> bool { self.selected }
    pub fn set_selected(&mut self, selected: bool) { self.selected = selected; }
    
    /// 色を変更
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.display.color = color;
        self.metadata.touch();
    }
}

// 型エイリアス（使いやすさ向上）
pub type CircleEntity<T> = GeometricEntity<T, geo_primitives::Circle3D<T>>;
pub type LineSegmentEntity<T> = GeometricEntity<T, geo_primitives::LineSegment3D<T>>;
pub type TriangleEntity<T> = GeometricEntity<T, geo_primitives::Triangle3D<T>>;
// ... 他の形状も同様
```

#### 4.2.5 CAMEntity - 工具経路エンティティ

```rust
// model/geo_entity/src/cam_entity.rs

use geo_algorithms::toolpath::ToolPath;
use geo_contracts::Scalar;

/// 加工属性
#[derive(Debug, Clone)]
pub struct MachiningAttributes {
    /// 工具番号
    pub tool_number: u32,
    
    /// 主軸回転数 (rpm)
    pub spindle_speed: f64,
    
    /// 送り速度 (mm/min)
    pub feed_rate: f64,
    
    /// 切込み深さ (mm)
    pub depth_of_cut: f64,
    
    /// 加工タイプ
    pub operation_type: OperationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Roughing,   // 荒加工
    Finishing,  // 仕上げ
    Drilling,   // 穴あけ
    Contouring, // 輪郭加工
}

/// CAMエンティティ（工具経路）
#[derive(Debug, Clone)]
pub struct CAMEntity<T: Scalar> {
    /// 一意識別子
    id: EntityId,
    
    /// 工具経路
    toolpath: ToolPath<T>,
    
    /// 加工属性
    machining: MachiningAttributes,
    
    /// 表示属性
    display: DisplayAttributes,
    
    /// メタデータ
    metadata: Metadata,
    
    /// 選択状態
    selected: bool,
}

impl<T: Scalar> CAMEntity<T> {
    /// 新規作成
    pub fn new(toolpath: ToolPath<T>, machining: MachiningAttributes) -> Self {
        Self {
            id: EntityId::new(),
            toolpath,
            machining,
            display: DisplayAttributes::cam_cutting(), // デフォルトは切削色
            metadata: Metadata::default(),
            selected: false,
        }
    }
    
    /// 加工タイプに応じた色設定
    pub fn with_operation_color(mut self) -> Self {
        self.display.color = match self.machining.operation_type {
            OperationType::Roughing => [0.8, 0.8, 0.8],   // グレー
            OperationType::Finishing => [1.0, 1.0, 1.0],  // 白
            OperationType::Drilling => [1.0, 0.5, 0.0],   // オレンジ
            OperationType::Contouring => [0.2, 1.0, 0.2], // 緑
        };
        self
    }
    
    // アクセサ
    pub fn id(&self) -> EntityId { self.id }
    pub fn toolpath(&self) -> &ToolPath<T> { &self.toolpath }
    pub fn machining(&self) -> &MachiningAttributes { &self.machining }
    pub fn display(&self) -> &DisplayAttributes { &self.display }
    pub fn metadata(&self) -> &Metadata { &self.metadata }
    
    /// 推定加工時間（秒）
    pub fn estimated_time(&self) -> f64 {
        let path_length = self.toolpath.total_length();
        (path_length / self.machining.feed_rate) * 60.0 // mm/min → sec
    }
}
```

---

## 実装計画

### Phase 1: エンティティ基盤実装（2週間）

#### ステップ1: クレート作成・基本構造（3日）

**実装ファイル**:
- `model/geo_entity/Cargo.toml`
  ```toml
  [package]
  name = "geo_entity"
  version = "0.1.0"
  edition = "2021"
  
  [dependencies]
  geo_contracts = { path = "../geo_contracts" }
  geo_primitives = { path = "../geo_primitives" }
  geo_algorithms = { path = "../geo_algorithms" }
  uuid = { version = "1.0", features = ["v4", "serde"] }
  chrono = { version = "0.4", features = ["serde"] }
  serde = { version = "1.0", features = ["derive"] }
  ```

- `model/geo_entity/src/lib.rs`
- `model/geo_entity/src/entity_id.rs`
- `model/geo_entity/src/display.rs`
- `model/geo_entity/src/metadata.rs`

**実装項目**:
- [x] EntityId 実装
- [x] DisplayAttributes 実装
- [x] Metadata 実装
- [x] 基本テスト

#### ステップ2: GeometricEntity 実装（4日）

**実装ファイル**:
- `model/geo_entity/src/geometric_entity.rs`
- `model/geo_entity/tests/geometric_entity_tests.rs`

**実装項目**:
- [x] GeometricEntity<T, G> 実装
- [x] 型エイリアスの定義（CircleEntity, etc.）
- [x] 選択・色変更のテスト
- [x] メタデータ自動更新のテスト

#### ステップ3: CAMEntity 実装（3日）

**実装ファイル**:
- `model/geo_entity/src/cam_entity.rs`
- `model/geo_entity/tests/cam_entity_tests.rs`

**実装項目**:
- [x] MachiningAttributes 実装
- [x] CAMEntity 実装
- [x] 推定加工時間の計算
- [x] 操作タイプによる色自動設定

#### ステップ4: ドキュメント・統合（3日）

**ドキュメント**:
- API ドキュメント（Rustdoc）
- 使用例（examples/entity_basic.rs）
- Phase 4への移行計画文書

**統合テスト**:
- 既存の幾何データとの統合
- シリアライズ/デシリアライズ

---

### Phase 2: ViewModel層統合（1週間）

#### ステップ1: 変換器の拡張（3日）

**実装ファイル**:
- `viewmodel/converter/src/entity_converter.rs` (新規)

**実装項目**:
```rust
// エンティティから頂点への変換
pub fn geometric_entity_to_vertices<T, G>(
    entity: &GeometricEntity<T, G>,
    options: &VisualizationOptions,
) -> Vec<Vertex3D>
where
    G: ToVertices<T>,
{
    let mut vertices = entity.geometry().to_vertices(options);
    
    // 表示属性を適用
    let color = entity.display().color;
    for vertex in &mut vertices {
        vertex.color = color;
    }
    
    vertices
}

pub fn cam_entity_to_vertices<T: Scalar>(
    cam_entity: &CAMEntity<T>,
    options: &ToolPathVisualizationOptions,
) -> Vec<Vertex3D> {
    // ToolPath → 頂点変換 + 表示属性適用
    // ...
}
```

#### ステップ2: App層での利用（2日）

**実装ファイル**:
- `view/app/src/entity_manager.rs` (新規)
- `view/app/src/app_state.rs` (修正)

**実装項目**:
```rust
// エンティティ管理
pub struct EntityManager<T: Scalar> {
    geometric_entities: HashMap<EntityId, GeometricEntity<T, Box<dyn Any>>>,
    cam_entities: HashMap<EntityId, CAMEntity<T>>,
}

impl<T: Scalar> EntityManager<T> {
    pub fn add_geometric(&mut self, entity: GeometricEntity<T, impl Any + 'static>) -> EntityId;
    pub fn add_cam(&mut self, entity: CAMEntity<T>) -> EntityId;
    pub fn select(&mut self, id: EntityId);
    pub fn set_color(&mut self, id: EntityId, color: [f32; 3]);
    pub fn remove(&mut self, id: EntityId);
}
```

#### ステップ3: テスト・検証（2日）

**テスト項目**:
- エンティティの追加・削除
- 選択・色変更
- 表示/非表示切り替え
- パフォーマンステスト（1000エンティティ）

---

### Phase 3: ドキュメント・完了報告（0.5週間）

**成果物**:
- 完了報告書（`dev/foundation/ENTITY_FOUNDATION_COMPLETION_REPORT.md`）
- マイグレーションガイド
- Phase 4への準備状況確認

**総工数**: 3.5週間 → **約1ヶ月**

---

## 移行戦略

### 既存コードからの移行

#### 段階的移行（3段階）

```text
Phase 1: デバッグ表示（現状）
  viewmodel/converter: 幾何データ → 頂点
  
  ↓ 移行1: エンティティラッパー追加

Phase 2: エンティティ基礎（Issue #206）
  viewmodel/converter: GeometricEntity → 頂点
  app: EntityManager で管理
  
  ↓ 移行2: トポロジー追加

Phase 3: Phase 4完全版（Issue #205）
  viewmodel/converter: TopologyEntity → 頂点
  app: CADドキュメントモデル
```

#### 互換性維持

Phase 2（基礎版）では、既存の直接変換も維持：

```rust
// 旧方式（デバッグ用）: そのまま利用可能
let vertices = circle_to_vertices(&circle, &options);

// 新方式（エンティティ経由）: 徐々に移行
let entity = GeometricEntity::new(circle);
let vertices = geometric_entity_to_vertices(&entity, &options);
```

---

## まとめ

### Issue #206の位置づけ

| 項目 | 内容 |
|------|------|
| **目的** | エンティティ基礎概念の先行導入 |
| **スコープ** | EntityId, DisplayAttributes, Metadata, GeometricEntity, CAMEntity |
| **除外** | B-Rep, Euler操作, トポロジー検証 |
| **工数** | 3.5週間（約1ヶ月） |
| **実施時期** | Issue #204, #203完了後（Week 7-10） |
| **優先度** | 🟠 Tier 2（高優先） |

### Phase 4へのスムーズな移行

```rust
// Phase 2（基礎版）:
GeometricEntity<T, Circle3D<T>>

// Phase 4（完全版）- シームレスに拡張:
TopologyEntity<T> {
    id: EntityId,              // ← そのまま流用
    topology: Solid,           // ← 新規追加
    display: DisplayAttributes,// ← そのまま流用
    metadata: Metadata,        // ← そのまま流用
}
```

**基礎版で培ったエンティティ管理ノウハウを、Phase 4で最大限活用**できます。

---

## 関連情報

- **設計文書**: `dev/architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md`
- **提案文書**: `dev/architecture/ENTITY_ATTRIBUTE_HIERARCHICAL_ID_PROPOSAL.md`
- **View統合設計**: `dev/architecture/ENTITY_VIEW_INTEGRATION_DESIGN.md`
- **ロードマップ**: `dev/foundation/ROADMAP_2026_Q1_Q2.md`
- **関連Issue**: #206（基礎版）, #205（Phase 4完全版）
- **将来バックログIssue**: #232（Persistent Naming / 自動再接続 / フィーチャー再実行堅牢化）

