# エンティティ属性ID階層化 提案書（Phase 4.0 / #208 対応）

**作成日**: 2026年2月21日  
**最終更新**: 2026年2月21日  
**ステータス**: 提案  
**関連Issue**: #208（エンティティ層基礎）, #205（Phase 4完全版）

---

## 📋 目次

1. [目的](#目的)
2. [前提整理（#208との整合）](#前提整理208との整合)
3. [提案方針](#提案方針)
4. [ID体系設計](#id体系設計)
5. [階層IDの具体例（CAM穴あけパス）](#階層idの具体例cam穴あけパス)
6. [Rust実装方針（const中心）](#rust実装方針const中心)
7. [関係モデル（グループ・レイヤー）](#関係モデルグループレイヤー)
8. [ユーザーコマンド拡張方針](#ユーザーコマンド拡張方針)
9. [運用ルール](#運用ルール)
10. [段階的導入計画](#段階的導入計画)
11. [将来バックログ（失念防止）](#将来バックログ失念防止)
12. [機能十分性チェック](#機能十分性チェック)

---

## 目的

`geo_entity` の属性管理において、以下を同時に満たす設計を定義する。

- **保守性**: Enum肥大化を避け、定義追加を簡潔化
- **識別性**: システム属性とユーザ属性を明確分離
- **拡張性**: CAM/寸法/スケッチ等のドメイン階層をID自体に埋め込む
- **移行性**: #208（Phase 4.0）から #205（Phase 4.1-4.3）へ継続利用可能
- **再現性**: フィーチャー再実行時に同一エンティティを安定再構築できる

---

## 前提整理（#208との整合）

#208 の「Phase 4への段階的移行」図は、

- 幾何データのみ
- エンティティ + 属性
- トポロジー + エンティティ

への**モデル段階化**を示すものであり、ID内部形式（UUIDか階層コードか）を拘束する図ではない。

したがって本提案では、図の思想を維持しつつ、識別子を次の2層に分離する。

1. **EntityId（一意ID）**: エンティティ同一性（不変、ユーザー指定不可）
2. **AttributeCode（i32）**: 属性の意味分類（階層・運用可変）

### EntityId と AttributeCode の責務境界

- `EntityId`:
    - エンティティの主キー（同一性）
    - ユーザーは直接指定しない
    - フィーチャー再実行時の再現性を満たすため、**決定的生成**を採用
        - 例: `FeatureId + 出力インデックス + 形状ローカルキー` から生成
- `AttributeCode`:
    - 実データに紐づく属性の種類コード
    - system（負ID）/ user（正ID）を区別して付与

この分離により、

- エンティティ同一性は再現可能で安定
- 属性は運用上柔軟に追加・変更可能

を同時に達成できる。

---

## 提案方針

### 1) Enumではなく `const + 定義テーブル`

- 固定キー管理をEnum列挙で行わず、`const AttributeCode` と静的定義テーブルで管理する
- 理由:
  - 追加が軽い（定数1件 + 定義テーブル1件）
  - 外部連携時にID値を直接扱いやすい
  - レンジ予約管理（負ID/正ID）が明確

### 2) 符号で責務分離

- **負ID**: システム属性（予約済み・アプリ共通）
- **正ID**: ユーザ属性（拡張・プロジェクト依存）
- **0**: 未使用（無効）

### 3) 階層構造をID部位で表現

属性IDを「ドメイン / カテゴリ / サブカテゴリ / 項目」の階層部位で構成し、
CAM穴あけや寸法拘束など、用途別の意味をコードから復元可能にする。

---

## ID体系設計

### 形式（推奨）

`i32` を以下の4部位に分割して運用する（10進4ブロック表記）。

- 符号: system/user の区別
- `DD`: ドメイン（CAM, DIM, SKETCH, DISPLAY など）
- `CC`: カテゴリ（Path, Tool, Tolerance など）
- `SS`: サブカテゴリ（Drill, Pocket, Contour など）
- `II`: 項目（FeedRate, SpindleSpeed など）

表現イメージ:

- `-DDCCSSII` = システム属性
- `+DDCCSSII` = ユーザ属性

### ドメイン例

- `01`: 共通メタ（name, description, tag）
- `10`: CAM
- `20`: 寸法（Dimension）
- `30`: スケッチ
- `40`: 表示

---

## 階層IDの具体例（CAM穴あけパス）

例として `DD=10 (CAM)`, `CC=01 (Path)`, `SS=02 (Drill)` を割り当てる。

- `-10010201`: 穴あけサイクル種別（例: peck / deep / gun）
- `-10010202`: 穴あけ送り速度
- `-10010203`: 穴あけ主軸回転数
- `-10010204`: 穴あけステップ量（peck量）
- `-10010205`: 穴底処理（dwell / chamfer）

このように、IDの上位部位を見れば「CAM Path Drill 系」であることが即判別できる。

---

## Rust実装方針（const中心）

### コア型

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttributeCode(i32);

impl AttributeCode {
    pub const INVALID: Self = Self(0);

    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    pub const fn is_system(self) -> bool {
        self.0 < 0
    }

    pub const fn is_user(self) -> bool {
        self.0 > 0
    }
}
```

### 定義テーブル（例）

```rust
pub struct SystemAttributeDef {
    pub code: AttributeCode,
    pub name: &'static str,
    pub value_kind: AttributeValueKind,
    pub description: &'static str,
}

pub const CAM_PATH_DRILL_FEED_RATE: AttributeCode = AttributeCode::new(-10010202);

pub const SYSTEM_ATTRIBUTE_DEFS: &[SystemAttributeDef] = &[
    SystemAttributeDef {
        code: CAM_PATH_DRILL_FEED_RATE,
        name: "cam.path.drill.feed_rate",
        value_kind: AttributeValueKind::Float,
        description: "穴あけ加工の送り速度",
    },
];
```

### 属性値

`AttributeValue` は型付き中心 + 任意バイナリ拡張。

```rust
pub enum AttributeValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Color([f32; 4]),
    Binary(Vec<u8>),
}
```

---

## 関係モデル（グループ・レイヤー）

本提案では、以下を明確に分離する。

- **属性（Attribute）**: エンティティ自身の性質（寸法値、公差、送り速度など）
- **関係（Relation）**: エンティティ間・分類間の所属（グループ所属、レイヤー所属）

### 1) グループ所属

- 寸法エンティティがグループに属することは、エンティティ概念として正しい
- 実装は属性ではなく `EntityId ↔ GroupId` の関係で保持する
- 基本は多対多（1寸法が複数グループ所属可能）

### 2) レイヤー所属（単一/複数の区別）

エンティティ種別ごとに所属制約を持たせる。

- `SingleLayer`: 最大1レイヤー
- `MultiLayer`: 複数レイヤー可

```rust
pub enum LayerMembershipPolicy {
    SingleLayer,
    MultiLayer,
}
```

追加時バリデーションでポリシーを強制する。

- `SingleLayer` で既存所属あり → エラーまたは置換（運用設定）
- `MultiLayer` は重複登録のみ禁止

### 3) 最小データ構造（提案）

```rust
pub struct GroupEntity {
    pub id: GroupId,
    pub name: String,
}

pub struct EntityGroupMembership {
    pub entity_id: EntityId,
    pub group_id: GroupId,
    pub role: Option<String>,
    pub order: Option<i32>,
}

pub struct EntityLayerMembership {
    pub entity_id: EntityId,
    pub layer_id: LayerId,
}
```

---

## ユーザーコマンド拡張方針

目標要件: 「コアで組み込んだ属性に加え、後からユーザーがコマンドで識別コードと属性値を自由付与できる」

### 1) 2系統の付与モード

- **systemモード**: 負IDのみ許可、定義テーブル照合必須
- **userモード**: 正IDのみ許可、ユーザー定義コードを許可

### 2) コマンドI/F（概念）

```text
entity attr set --entity <EntityId> --code <i32> --type <kind> --value <...>
entity attr get --entity <EntityId> [--code <i32>]
entity attr remove --entity <EntityId> --code <i32>
entity rel group add --entity <EntityId> --group <GroupId>
entity rel layer add --entity <EntityId> --layer <LayerId>
```

補足:

- コマンドで `EntityId` を「新規指定」する操作は提供しない
- `EntityId` は生成器が発行し、ユーザーは参照キーとして利用する

### 3) バリデーション

- `code == 0` を拒否
- `code < 0` かつ未定義systemコードは拒否
- `code > 0` は user範囲として許可（必要ならプロジェクト予約レンジを設定）
- 値型不一致（例: Float項目へString投入）を拒否
- `LayerMembershipPolicy` 違反を拒否

### 4) 監査・保守

- 属性変更コマンドを監査ログ化（who/when/what）
- userコード使用実績の集計を可能にし、将来のsystem昇格候補を抽出

---

## 運用ルール

1. **システム属性は必ず負ID**
2. **ユーザ属性は必ず正ID**
3. **0は禁止**（入力時バリデーション）
4. **同一コード再定義禁止**（CIテストで検出）
5. **定義テーブル未登録のsystemコード禁止**
6. ドメイン/カテゴリ番号は予約表で重複管理
7. レイヤー所属は `LayerMembershipPolicy` に従って検証
8. グループ/レイヤーは属性ではなく関係テーブルで管理
9. コマンド経由変更は監査ログを記録

---

## 段階的導入計画

### Phase 4.0（#208）

- `AttributeCode` 型と `AttributeValue` を `geo_entity` に導入
- `const + 定義テーブル` で system 属性を先行定義
- CAM/寸法/スケッチの最小コードセット作成
- Group/Layer 関係モデルを導入（所属データ分離）
- `LayerMembershipPolicy` による単一/複数レイヤー制約を導入
- ユーザーコマンドの最小機能（set/get/remove）を導入

### Phase 4.1-4.3（#205）

- `geo_topology` の Entity と統合
- トポロジー由来属性（face/edge/vertex 系）を追加
- 必要に応じて ECS 評価結果を反映
- トポロジー要素に対するグループ/レイヤー関係適用を拡張

---

## 将来バックログ（失念防止）

本提案の次段階（かなり未来の実装）は、GitHub Issue で追跡管理する。

- 親Issue: [#232 Future Backlog: Persistent Naming / 自動再接続 / フィーチャー再実行堅牢化](https://github.com/RedRing2020/RedRing/issues/232)

### 将来実装テーマ（#232で管理）

1. **Persistent Naming**
    - 決定的なトポロジ命名規則
    - split/merge/replace 後の参照解決

2. **Lineage Graph（親子系譜）**
    - `1 -> N` 分割の親子関係保持
    - `N -> 1` 統合時の親集合記録
    - tombstone（失効ID）管理

3. **Auto Reattach（自動再接続）**
    - 参照消失時の候補探索
    - 幾何/トポロジ/属性ヒントで順位付け

4. **履歴再生の堅牢化**
    - トランザクション境界
    - ロールバック/再試行
    - 失敗診断ログ

5. **検証基盤**
    - 決定性テスト（同入力同結果）
    - 回帰テスト（分割・結合・削除）
    - 大規模モデル安定性評価

---

## #208 基礎実装の着手スコープ（2026-02-21）

本ドキュメントを根拠に、Issue #208 の基礎実装は以下を対象に着手する。

- `model/geo_entity` クレート新設
- `EntityId`（ユーザー指定不可、決定的生成を含む）
- `AttributeCode`（負=system、正=user、0禁止）
- `AttributeValue`（型付き + `Binary(Vec<u8>)`）
- `Attributes`（型検証付き set/get/remove）
- Group/Layer 関係モデル
- `LayerMembershipPolicy`（SingleLayer / MultiLayer）
- 単体テスト（正負ID、0拒否、型検証、レイヤー制約、決定的ID）

本着手では、ViewModel/App統合およびB-Repトポロジー統合は対象外とする。

---

## 機能十分性チェック

以下を満たせば、Phase 4.0 の属性ID基盤として十分。

- [ ] システム属性（負ID）とユーザ属性（正ID）の判別が機能する
- [ ] `0` を属性コードとして拒否できる
- [ ] CAM穴あけ属性の階層IDを定義・取得できる
- [ ] `AttributeCode -> 定義情報` の逆引きができる
- [ ] 重複コードをテストで検知できる
- [ ] 任意データ（`Binary(Vec<u8>)`）を保持できる
- [ ] `EntityId(UUID)` と独立して運用できる
- [ ] 寸法エンティティのグループ所属（多対多）を管理できる
- [ ] 単一レイヤー対象で複数所属を拒否できる
- [ ] 複数レイヤー対象で複数所属を許可できる
- [ ] コマンドで user属性コード（正ID）を付与・取得・削除できる
- [ ] system属性コード（負ID）に未定義コードを拒否できる
- [ ] 値型バリデーションが機能する

---

## 結論

#208 の段階移行思想に対し、

- `EntityId(UUID)` は「同一性」
- `AttributeCode(i32)` は「意味分類（階層）」

として分離する本提案は整合的である。  
特にCAMの穴あけパスのような階層ドメインでは、**階層IDの部位分割 + const管理**が、Enum中心運用より高い保守性を期待できる。
