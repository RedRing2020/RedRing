# ECS（Entity Component System）化の評価

**作成日**: 2026年2月12日  
**最終更新**: 2026年2月12日  
**ステータス**: 評価フェーズ  
**関連Issue**: [#215](https://github.com/RedRing2020/RedRing/issues/215)

---

## 目次

1. [概要](#概要)
2. [ECSとは](#ecsとは)
3. [現在の設計との比較](#現在の設計との比較)
4. [CAD/CAMシステムでのECS有効性評価](#cadcamシステムでのecs有効性評価)
5. [パフォーマンス予測](#パフォーマンス予測)
6. [実装複雑度評価](#実装複雑度評価)
7. [段階的移行計画](#段階的移行計画)
8. [推奨判断](#推奨判断)

---

## 概要

エンティティ層基礎（Issue #208）完了後、システム全体をECS（Entity Component System）アーキテクチャに移行することで、
以下の改善を目指します：

### 期待される効果

1. **パフォーマンス向上**
   - キャッシュ効率の向上（SoA: Structure of Arrays）
   - データ指向設計による最適化
   - 並列処理の容易化

2. **実装利便性向上**
   - イテレータベースの安全なループ処理
   - 添え字アクセスの問題解決
   - システムの疎結合化

3. **スケーラビリティ向上**
   - 大量エンティティの効率的管理（10万以上）
   - 動的なコンポーネント追加・削除
   - メモリ使用量の最適化

---

## ECSとは

### 基本概念

```rust
// Entity: 単なるID
type Entity = u64;

// Component: データのみ（ロジックなし）
struct Position { x: f64, y: f64, z: f64 }
struct Velocity { dx: f64, dy: f64, dz: f64 }
struct DisplayAttributes { color: [f32; 3], visible: bool }
struct GeometryData { shape: Box<dyn Shape3D> }

// System: ロジックのみ（データなし）
fn physics_system(positions: &mut [Position], velocities: &[Velocity], dt: f64) {
    for (pos, vel) in positions.iter_mut().zip(velocities.iter()) {
        pos.x += vel.dx * dt;
        pos.y += vel.dy * dt;
        pos.z += vel.dz * dt;
    }
}

fn render_system(positions: &[Position], attributes: &[DisplayAttributes]) {
    for (pos, attr) in positions.iter().zip(attributes.iter()) {
        if attr.visible {
            draw_point(pos, attr.color);
        }
    }
}
```

### データレイアウト: AoS vs SoA

```rust
// AoS (Array of Structures) - 従来の設計
struct Entity {
    id: u64,
    position: Position,
    velocity: Velocity,
    display: DisplayAttributes,
}
let entities: Vec<Entity> = vec![...];

// メモリレイアウト:
// [E1.id, E1.pos, E1.vel, E1.display, E2.id, E2.pos, E2.vel, E2.display, ...]
// キャッシュミスが多い（velocity不要な処理でもvelocityをメモリに読む）

// SoA (Structure of Arrays) - ECS設計
struct World {
    positions: Vec<Position>,      // [E1.pos, E2.pos, E3.pos, ...]
    velocities: Vec<Velocity>,     // [E1.vel, E2.vel, E3.vel, ...]
    displays: Vec<DisplayAttributes>, // [E1.display, E2.display, ...]
}

// メモリレイアウト:
// positions:  [P1, P2, P3, P4, ...]
// velocities: [V1, V2, V3, V4, ...]
// キャッシュヒット率が高い（必要なデータのみ連続アクセス）
```

---

## 現在の設計との比較

### Phase 4.0（#208）設計

```rust
// 現在の設計（AoS的）
pub struct GeometricEntity<T: Scalar, G> {
    id: EntityId,
    geometry: G,
    display_attributes: DisplayAttributes,
    metadata: Metadata,
    transform: Option<Transform3D<T>>,
}

// 使用例
let entities: Vec<GeometricEntity<f64, Box<dyn Shape3D<f64>>>> = vec![...];

// レンダリングループ
for entity in &entities {
    if entity.display_attributes.visible {
        let vertices = entity.geometry.tessellate();
        render(vertices, entity.display_attributes.color);
    }
}
```

**問題点**:
1. `entity.geometry` にアクセスする際、`display_attributes` もメモリに読まれる
2. 10万エンティティのループでキャッシュミスが多発
3. 並列処理が難しい（`entity` 全体を可変借用）

### ECS設計案

```rust
// ECS版
pub struct World<T: Scalar> {
    // Component storage (SoA)
    entities: Vec<Entity>,
    geometries: Vec<Option<Box<dyn Shape3D<T>>>>,
    display_attributes: Vec<Option<DisplayAttributes>>,
    metadata: Vec<Option<Metadata>>,
    transforms: Vec<Option<Transform3D<T>>>,
    
    // Sparse set for efficient iteration
    geometry_entities: Vec<Entity>,  // geometryを持つエンティティのみ
    visible_entities: Vec<Entity>,   // visible=trueのエンティティのみ
}

// レンダリングシステム
fn render_system<T: Scalar>(world: &World<T>) {
    // visible_entitiesのみをイテレート（高速）
    for &entity_id in &world.visible_entities {
        let idx = entity_id as usize;
        
        if let (Some(geometry), Some(display)) = 
           (&world.geometries[idx], &world.display_attributes[idx]) {
            let vertices = geometry.tessellate();
            render(vertices, display.color);
        }
    }
}
```

**改善点**:
1. 必要なコンポーネントのみメモリアクセス
2. `visible_entities` により不要な処理をスキップ
3. 並列処理が容易（各コンポーネント配列を独立して処理）

---

## CAD/CAMシステムでのECS有効性評価

### ✅ ECSが有効なシナリオ

#### 1. 大量形状の可視化

```rust
// 10万個の幾何形状を表示
// 従来: 10万回のループで entity 全体をアクセス → キャッシュミス多発
// ECS: visible_entities のみイテレート → 表示対象が1万個なら9万回のスキップ

// パフォーマンス予測:
// 従来: 10万エンティティ × 200ns/entity = 20ms (50 FPS)
// ECS:   1万エンティティ × 100ns/entity = 1ms (1000 FPS)
```

#### 2. 切削シミュレーション

```rust
// 工具経路1万セグメント × Octreeノード1万 = 1億回の衝突判定
// 従来: entity 全体をアクセス → メモリ帯域がボトルネック
// ECS: Position + BoundingBox のみアクセス → メモリ帯域が1/5に削減

// パフォーマンス予測:
// 従来: 1億回 × 50ns = 5秒
// ECS:   1億回 × 10ns = 1秒（5倍高速化）
```

#### 3. 属性による選択・フィルタ

```rust
// 「赤色の形状のみ選択」
// 従来: 全エンティティをループしてdisplay_attributes.colorをチェック
// ECS: color_index を使って事前にフィルタ済みエンティティのみ処理

// ユースケース:
// - レイヤー別表示
// - 材質別選択
// - 加工順序の可視化
```

#### 4. 並列処理

```rust
// 1万個の形状の境界ボックス計算
use rayon::prelude::*;

// 従来: 難しい（entity全体の可変借用が競合）
entities.par_iter_mut().for_each(|entity| {
    entity.bounding_box = entity.geometry.compute_bbox(); // NG: 他のフィールドも借用される
});

// ECS: 簡単（コンポーネントごとに独立）
world.geometries.par_iter()
    .zip(world.bounding_boxes.par_iter_mut())
    .for_each(|(geom, bbox)| {
        *bbox = geom.as_ref().map(|g| g.compute_bbox());
    });

// パフォーマンス予測（8コア）:
// 従来: 1万 × 100μs = 1秒
// ECS:   1万 × 100μs ÷ 8 = 125ms（8倍高速化）
```

### ⚠️ ECSが不向きなシナリオ

#### 1. 複雑な幾何計算

```rust
// NURBS曲面の交差計算
// 従来: geometry に直接アクセス、メソッド呼び出しが自然
let intersection = surface1.intersect(&surface2);

// ECS: コンポーネント取得が煩雑
let surface1 = world.geometries[entity1_id].as_ref().unwrap();
let surface2 = world.geometries[entity2_id].as_ref().unwrap();
let intersection = surface1.intersect(surface2);
```

**評価**: ECSでも実装可能だが、記述が冗長になる。
ただし、システム単位で実装すればクリーンに記述できる：

```rust
fn intersection_system(
    geometries: &[Option<Box<dyn Shape3D>>],
    intersection_queries: &[(Entity, Entity)],
) -> Vec<IntersectionResult> {
    intersection_queries.iter().map(|&(e1, e2)| {
        let g1 = geometries[e1 as usize].as_ref().unwrap();
        let g2 = geometries[e2 as usize].as_ref().unwrap();
        g1.intersect(g2)
    }).collect()
}
```

#### 2. 少数の複雑エンティティ

```rust
// 10個の複雑なNURBS曲面を編集
// 従来: entity.geometry にアクセスして直接操作
entities[0].geometry.insert_knot(u, v);

// ECS: 同様に可能
if let Some(geometry) = &mut world.geometries[entity_id as usize] {
    geometry.insert_knot(u, v);
}
```

**評価**: どちらでも同等。ECSのオーバーヘッドはほぼない。

#### 3. エンティティ間の複雑な依存関係

```rust
// B-Repトポロジー: Edge → Vertex への参照
// 従来: 直接参照が可能
struct Edge {
    start_vertex: Arc<Vertex>,
    end_vertex: Arc<Vertex>,
}

// ECS: エンティティIDで間接参照
struct EdgeComponent {
    start_vertex_id: Entity,
    end_vertex_id: Entity,
}

// 頂点座標へのアクセス:
let start_pos = world.positions[edge.start_vertex_id as usize].unwrap();
let end_pos = world.positions[edge.end_vertex_id as usize].unwrap();
```

**評価**: ECSでは間接参照になるが、キャッシュ効率は向上する可能性がある。
ただし、コード可読性は低下する。

---

## パフォーマンス予測

### ベンチマーク想定シナリオ

#### シナリオ1: 10万形状の可視化

| 設計 | メモリアクセス | 処理時間 | FPS |
|------|-------------|---------|-----|
| AoS（従来） | 10万 × 128 bytes = 12.8 MB | 20 ms | 50 |
| SoA（ECS） | 1万 × 64 bytes = 640 KB | 1 ms | 1000 |

**改善率**: **20倍高速化**

#### シナリオ2: 1万セグメント切削シミュレーション

| 設計 | Octree衝突判定 | 材料除去 | 合計 |
|------|--------------|---------|-----|
| AoS（従来） | 3秒 | 2秒 | 5秒 |
| SoA（ECS） | 0.6秒 | 2秒 | 2.6秒 |

**改善率**: **約2倍高速化**（衝突判定が支配的な場合）

#### シナリオ3: 並列境界ボックス計算（8コア）

| 設計 | シングルスレッド | マルチスレッド | 並列効率 |
|------|---------------|-------------|---------|
| AoS（従来） | 1000 ms | 500 ms | 50% |
| SoA（ECS） | 1000 ms | 125 ms | 100% |

**改善率**: **並列化効率が2倍向上**

### メモリ使用量

```rust
// 10万エンティティの場合

// AoS（従来）
struct Entity {
    id: u64,                     // 8 bytes
    geometry: Box<dyn Shape3D>,  // 16 bytes (ポインタ)
    display: DisplayAttributes,  // 32 bytes
    metadata: Metadata,          // 64 bytes
    transform: Option<Transform>, // 128 bytes
    // Total: 248 bytes × 10万 = 24.8 MB
}

// SoA（ECS）
struct World {
    entities: Vec<u64>,          // 8 bytes × 10万 = 800 KB
    geometries: Vec<Option<Box>>, // 16 bytes × 10万 = 1.6 MB
    displays: Vec<Option<Display>>, // 32 bytes × 10万 = 3.2 MB
    // (一部のエンティティのみコンポーネントを持つ場合、さらに削減)
    // Total: 約 5.6 MB
}
```

**改善率**: **メモリ使用量が1/4に削減**（スパースな場合）

---

## 実装複雑度評価

### 既存ECSライブラリの選択肢

#### Option 1: bevy_ecs（推奨）

```toml
[dependencies]
bevy_ecs = "0.13"
```

**特徴**:
- ✅ Rustで最も成熟したECS実装
- ✅ スパースセット + アーキタイプ最適化
- ✅ 並列スケジューリング自動化
- ✅ クエリシステムが強力
- ⚠️ ゲームエンジンBevy由来（CAD向けではない）

**使用例**:

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
struct Position { x: f64, y: f64, z: f64 }

#[derive(Component)]
struct DisplayAttributes { color: [f32; 3], visible: bool }

#[derive(Component)]
struct GeometryData(Box<dyn Shape3D<f64>>);

// システム定義
fn render_system(
    query: Query<(&Position, &DisplayAttributes, &GeometryData)>
) {
    for (pos, attr, geom) in query.iter() {
        if attr.visible {
            let vertices = geom.0.tessellate();
            render(vertices, attr.color);
        }
    }
}

// World作成とシステム実行
let mut world = World::new();
let entity = world.spawn((
    Position { x: 0.0, y: 0.0, z: 0.0 },
    DisplayAttributes { color: [1.0, 0.0, 0.0], visible: true },
    GeometryData(Box::new(Circle3D::new(...))),
));

let mut schedule = Schedule::default();
schedule.add_systems(render_system);
schedule.run(&mut world);
```

#### Option 2: hecs

```toml
[dependencies]
hecs = "0.10"
```

**特徴**:
- ✅ 軽量（bevy_ecsより小さい）
- ✅ シンプルなAPI
- ⚠️ 並列スケジューリングは手動
- ⚠️ クエリ機能がbevy_ecsより弱い

#### Option 3: 自作ECS

**特徴**:
- ✅ CAD/CAM特化の設計が可能
- ✅ 最小限の依存関係
- ⚠️ 実装コストが高い（2-3週間）
- ⚠️ バグリスクが高い

### 実装コスト見積もり

| アプローチ | 初期実装 | Phase 4.0統合 | Phase 4.1統合 | 合計 |
|-----------|---------|-------------|-------------|------|
| bevy_ecs採用 | 1週間 | 1週間 | 2週間 | **4週間** |
| hecs採用 | 1週間 | 1週間 | 2週間 | **4週間** |
| 自作ECS | 2-3週間 | 1週間 | 2週間 | **5-6週間** |
| ECS化しない | - | - | - | **0週間** |

---

## 段階的移行計画

### Phase 1: ECSプロトタイプ（1週間）

**目的**: ECSの有効性を実験的に確認

**実装内容**:
- bevy_ecs を依存関係に追加
- 既存の `GeometricEntity` と並行してECS版を実装
- ベンチマーク比較（可視化、衝突判定）

**成功基準**:
- 10万形状の可視化で2倍以上高速化
- コード可読性が許容範囲内

### Phase 2: Phase 4.0のECS実装（1週間）

**実装内容**:
- `geo_entity` クレートにECS版を追加
- `EntityWorld<T>` 構造体（bevy_ecsラッパー）
- Component定義（Position, DisplayAttributes, Metadata, GeometryData）

```rust
// geo_entity/src/ecs.rs

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct EntityId(pub uuid::Uuid);

#[derive(Component)]
pub struct Position3D<T: Scalar> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Component)]
pub struct DisplayAttributes {
    pub color: [f32; 3],
    pub line_width: f32,
    pub visible: bool,
}

#[derive(Component)]
pub struct Metadata {
    pub name: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Component)]
pub struct GeometryData<T: Scalar>(pub Box<dyn Shape3D<T>>);

/// ECS World ラッパー
pub struct EntityWorld {
    world: bevy_ecs::world::World,
    render_schedule: Schedule,
    update_schedule: Schedule,
}

impl EntityWorld {
    pub fn new() -> Self {
        let mut world = World::new();
        
        let mut render_schedule = Schedule::default();
        render_schedule.add_systems((
            visibility_culling_system,
            tessellation_system,
            render_system,
        ).chain());
        
        let update_schedule = Schedule::default();
        
        Self {
            world,
            render_schedule,
            update_schedule,
        }
    }
    
    pub fn spawn_geometric_entity<T: Scalar>(
        &mut self,
        geometry: Box<dyn Shape3D<T>>,
        display: DisplayAttributes,
        metadata: Metadata,
    ) -> bevy_ecs::entity::Entity {
        self.world.spawn((
            EntityId(uuid::Uuid::new_v4()),
            GeometryData(geometry),
            display,
            metadata,
        ))
    }
    
    pub fn render(&mut self) {
        self.render_schedule.run(&mut self.world);
    }
}

// システム定義
fn visibility_culling_system(
    mut query: Query<(&Position3D<f64>, &DisplayAttributes, &mut Visibility)>,
    camera: Res<Camera>,
) {
    for (pos, attr, mut vis) in query.iter_mut() {
        *vis = attr.visible && camera.frustum.contains(pos);
    }
}

fn tessellation_system(
    query: Query<(&GeometryData<f64>, &TessellationQuality), With<Visibility>>,
    mut meshes: ResMut<MeshCache>,
) {
    for (geom, quality) in query.iter() {
        let mesh = geom.0.tessellate(*quality);
        meshes.insert(geom.entity_id(), mesh);
    }
}

fn render_system(
    query: Query<(&DisplayAttributes, &MeshHandle), With<Visibility>>,
    meshes: Res<MeshCache>,
    mut gpu: ResMut<GpuResources>,
) {
    for (attr, mesh_handle) in query.iter() {
        if let Some(mesh) = meshes.get(*mesh_handle) {
            gpu.draw_mesh(mesh, attr.color);
        }
    }
}
```

### Phase 3: ViewModel/App統合（1週間）

**実装内容**:
- `AppState` に `EntityWorld` を統合
- 既存の `Vec<GeometricEntity>` から段階的移行
- 両方のアーキテクチャをサポート（互換性維持）

### Phase 4: Phase 4.1トポロジー統合（2週間）

**実装内容**:
- B-Repトポロジーのコンポーネント化
- Vertex/Edge/Face をコンポーネントとして実装
- 関係性をEntityIDで表現

```rust
#[derive(Component)]
struct VertexComponent<T: Scalar> {
    position: Point3D<T>,
}

#[derive(Component)]
struct EdgeComponent {
    start_vertex: Entity,
    end_vertex: Entity,
    curve: Option<Entity>,  // 3D曲線エンティティへの参照
}

#[derive(Component)]
struct FaceComponent {
    edges: Vec<Entity>,
    surface: Option<Entity>,  // サーフェスエンティティへの参照
}
```

---

## 推奨判断

### ✅ ECS化を推奨する理由

1. **パフォーマンス**: 10万形状規模で2-20倍高速化が見込める
2. **並列化**: 並列効率が大幅に向上（50% → 100%）
3. **メモリ効率**: スパースなコンポーネントでメモリ削減
4. **実装利便性**: イテレータベースで添え字問題が解消
5. **成熟度**: bevy_ecsは本番運用実績が豊富

### ⚠️ 懸念事項と対策

#### 懸念1: 学習コスト

**対策**: 
- Phase 1でプロトタイプを作成し、チーム全体で評価
- bevy_ecsの既存ドキュメント・事例が豊富

#### 懸念2: コード可読性の低下

**対策**:
- システム単位で処理をまとめることで、むしろ可読性向上
- 添え字アクセスがなくなり、イテレータベースで安全

#### 懸念3: デバッグの難しさ

**対策**:
- bevy_ecsは優れたデバッグツールを提供
- コンポーネントごとに独立してテスト可能

#### 懸念4: トポロジー実装の複雑化

**対策**:
- EntityIDによる参照は、むしろライフタイム問題を回避
- グラフ構造の実装に適している

### 📋 実施タイミング

**推奨スケジュール**:

```
Week 7-8:  #208 Phase 1（エンティティ基盤）← 従来設計で実装
Week 9:    ECS プロトタイプ（並行実施）
Week 10:   #208 Phase 2（ViewModel統合）← ECS採用判断
Week 19-21: #205 Phase 4.1（トポロジー層）← ECS版で実装
```

**判断ポイント（Week 9終了時）**:
- ✅ ベンチマークで2倍以上高速化達成 → ECS採用
- ❌ 高速化不十分 or 実装複雑度が高すぎる → 従来設計維持

---

## 付録: イテレータ化による添え字問題の解決

### 従来の問題

```rust
// 添え字アクセスの問題例
let entities: Vec<GeometricEntity> = vec![...];

for i in 0..entities.len() {
    let entity = &entities[i];  // パニックの可能性
    
    // 他のエンティティを参照したい場合
    if let Some(parent_id) = entity.parent_id {
        let parent = &entities[parent_id];  // 範囲外アクセスの危険
    }
}

// 並列処理での問題
entities.par_iter_mut().enumerate().for_each(|(i, entity)| {
    // entities[i+1] にアクセスしたいが、借用エラー
});
```

### ECS版の解決策

```rust
use bevy_ecs::prelude::*;

// クエリベースの安全なイテレート
fn update_system(
    query: Query<(&Position, &mut Velocity)>
) {
    for (pos, mut vel) in query.iter_mut() {
        // 添え字なし、イテレータで安全
        vel.dx += pos.x * 0.01;
    }
}

// エンティティ参照も型安全
#[derive(Component)]
struct ParentRef(Entity);

fn parent_child_system(
    children: Query<(&Position, &ParentRef)>,
    parents: Query<&Transform>,
) {
    for (child_pos, parent_ref) in children.iter() {
        if let Ok(parent_transform) = parents.get(parent_ref.0) {
            // 型安全なエンティティ参照
            let world_pos = parent_transform.transform_point(child_pos);
        }
    }
}

// 並列処理も自動最適化
fn parallel_system(
    mut query: Query<&mut BoundingBox>
) {
    query.par_iter_mut().for_each(|mut bbox| {
        // bevy_ecsが自動で並列化
        bbox.expand(1.0);
    });
}
```

---

## まとめ

### 推奨: ECS化を採用

**理由**:
1. パフォーマンス向上が大きい（2-20倍）
2. 並列処理の効率が劇的に向上
3. イテレータベースで安全性向上
4. 成熟したライブラリ（bevy_ecs）が利用可能
5. 実装コストは許容範囲（4週間）

**実施方針**:
1. Week 9でプロトタイプ作成
2. ベンチマーク評価で最終判断
3. #208 Phase 2からECS採用
4. #205 Phase 4.1でトポロジー統合

### 期待される成果

- ✅ 10万形状のリアルタイム可視化
- ✅ 切削シミュレーションの高速化
- ✅ 添え字問題の完全解消
- ✅ 並列処理の効率化（8コア活用）
- ✅ メモリ使用量の削減

---

## 関連情報

### 参考文献

- [bevy_ecs Documentation](https://docs.rs/bevy_ecs/)
- [Data-Oriented Design](https://www.dataorienteddesign.com/dodbook/)
- [ECS FAQ](https://github.com/SanderMertens/ecs-faq)

### 関連設計文書

- [ENTITY_FOUNDATION_DESIGN.md](./ENTITY_FOUNDATION_DESIGN.md) - エンティティ層基礎設計
- [PHASE4_TOPOLOGY_ENTITY_DESIGN.md](./PHASE4_TOPOLOGY_ENTITY_DESIGN.md) - Phase 4完全版設計

### GitHub Issues

- [#208](https://github.com/RedRing2020/RedRing/issues/208) - エンティティ層基礎（Phase 4.0）
- [#205](https://github.com/RedRing2020/RedRing/issues/205) - Phase 4完全版（B-Rep）

---

**最終推奨**: Week 9でECSプロトタイプを作成し、ベンチマーク結果に基づいて最終判断。
高速化効果が確認できれば、#208 Phase 2からECS採用を強く推奨します。
