# GitHub Project 作成・設定ガイド

**プロジェクト名**: RedRing Q1-Q3 Development (Weekly)

**作成方法**:
1. [GitHub Projects ページ](https://github.com/orgs/RedRing2020/projects) にアクセス
2. **New project** をクリック
3. テンプレート選択: **Table** を選択

---

## 📋 初期設定手順

### Step 1: 基本情報設定

**プロジェクト名**:
```
RedRing Q1-Q3 Development (Weekly)
```

**説明**:
```
RedRing 2026 Q1-Q3 開発ロードマップの進捗管理
- Week 3-28 の実施スケジュール追跡
- Tier 1/2/3 の並行進捗管理
- 依存関係の可視化
```

**アクセス**: Public

---

### Step 2: ビューの追加

#### ビュー1: Timeline View（推奨開始）
**名前**: Weekly Timeline
**テンプレート**: Timeline
**グループ化**: Week (カスタムフィールド)

#### ビュー2: Board View（進捗管理）
**名前**: Status Board
**テンプレート**: Board
**カラム**: Status (準備中 → 設計完了 → 実装中 → レビュー中 → 完了)

#### ビュー3: Table View（全体一覧）
**名前**: All Issues
**テンプレート**: Table

---

### Step 3: カスタムフィールドの追加

#### フィールド1: Week
**型**: Single select
**オプション**:
```
Week 3, Week 4, Week 5, Week 6, Week 7, Week 8, Week 9, Week 10,
Week 11, Week 12, Week 13, Week 14, Week 15, Week 16, Week 17, Week 18,
Week 19, Week 20, Week 21, Week 22, Week 23, Week 24
```

#### フィールド2: Tier
**型**: Single select
**オプション**:
```
tier-1 (最優先)
tier-2 (高優先)
tier-3 (中優先)
```

#### フィールド3: Phase
**型**: Single select
**オプション**:
```
Phase 1 (基礎)
Phase 2 (拡張)
Phase 3 (統合)
Phase 4 (トポロジー)
```

#### フィールド4: Dependencies
**型**: Text
**説明**: "Blocked by #206" など依存関係を記述

---

### Step 4: Issues の追加

**自動追加方法**:
1. **+ Add item** をクリック
2. Repository から Issue を検索・追加

**対象 Issue**:

#### 🔴 Tier 1（Week 3）

| Issue | Title | Week |
|-------|-------|------|
| #219 | Camera 行列計算修正 | Week 3 |
| #210 | NURBS GPU描画実装 | Week 6-9 |

#### 🟠 Tier 2（Week 4-11）

| Issue | Title | Week | Dependencies |
|-------|-------|------|--------------|
| #206 | Octree実装 | Week 4-5 | - |
| #207 | Octree可視化 | Week 6 | #206 |
| #208 | エンティティ層基礎 | Week 7-10 | #206, #207 |
| #214 | 切削シミュレーション | Week 7-11 | #206 |
| #215 | ECS化評価 | Week 9 | #208 |

#### 🟡 Tier 3（Week 12-24）

| Issue | Title | Week | Dependencies |
|-------|-------|------|--------------|
| #42 | テセレーション | Week 12-13 | #208 |
| #40 | 2D輪郭オフセット | Week 14 | #42 |
| #41 | 3Dメッシュオフセット | Week 15-16 | #42 |
| #205 | Phase 4完全版 | Week 19-24 | #208 |

---

### Step 5: ステータス設定

**ステータスフィールド**: Status（GitHub 標準）

**カラム設定**:
```
準備中（Backlog の色）
   ↓
設計完了（Ready の色）
   ↓
実装中（In Progress の色）
   ↓
レビュー中（In Review の色）
   ↓
完了（Done の色）
```

---

### Step 6: オートメーション設定

#### ルール1: Issue クローズ時に自動完了
**トリガー**: Repository の issue_closed イベント
**アクション**: Status を "完了" に変更

#### ルール2: PR マージ時に自動完了
**トリガー**: Repository の PR merged イベント
**アクション**: 対応 Issue の Status を "完了" に変更

---

## 📊 運用ルール

### 週間チェック（毎週月曜朝）

1. **完了 Issue の確認**
   - Status を "完了" に移動
   - 次週の "準備中" Issue を "実装中" に移動

2. **ボトルネック確認**
   - "準備中" Issue が多い場合は優先度見直し
   - "実装中" Issue が 3 個以上の場合は注意

3. **依存関係確認**
   - Dependencies フィールドで待機中の Issue 確認
   - Blocked by #206 のような記述で把握

### 実施効果の期待値

✅ 進捗が Week 単位で可視化される  
✅ ボトルネック（Octree など）が明確化  
✅ チーム全体で同じ Priority でアクションできる  
✅ バーンダウン計算が可能（完了/未完了の割合）

---

## 🔗 参考リンク

- [GitHub Projects ドキュメント](https://docs.github.com/en/issues/planning-and-tracking-with-projects)
- [Project テンプレートの選択](https://docs.github.com/en/issues/planning-and-tracking-with-projects/creating-a-project/creating-a-project)
- [カスタムフィールドの追加](https://docs.github.com/en/issues/planning-and-tracking-with-projects/customizing-your-project/about-custom-fields)
- [Issueラベル運用ガイド](../ISSUE_LABEL_OPERATION.md)

---

## 💡 代替案（CLI を使う場合）

GitHub CLI の project コマンドが完全に稼働すれば、以下で自動化可能：

```bash
# Project 作成
gh project create --title "RedRing Q1-Q3 Development (Weekly)" \
  --owner RedRing2020 \
  --description "Weekly progress tracking for Q1-Q3 2026"

# Issue 追加（API 呼び出し）
gh project item-add <PROJECT_ID> --issue 206 --format json
```

ただし現在の認証スコープ制限により、Web UI での作成をお勧めします。

---

**完了目安**: 10分（Project テンプレート選択 + Issue 3-4 個追加で開始可能）
