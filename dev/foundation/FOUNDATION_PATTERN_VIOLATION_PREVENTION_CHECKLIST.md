# Foundation Pattern 違反防止チェックリスト

**作成日**: 2025年11月16日  
**最終更新**: 2025年11月16日

## 🚨 **実装前必須チェック（Pre-Implementation Checklist）**

### **Step 1: 既存実装状況確認**

```powershell
# 対象図形の現在の実装状況確認
ls model/geo_primitives/src/{shape}_*

# derive使用状況確認
grep -n "#\[derive" model/geo_primitives/src/{shape}_*.rs

# Core Traits実装状況確認
grep -n "impl.*Constructor\|impl.*Properties\|impl.*Measure" model/geo_primitives/src/{shape}_*.rs

# Legacy traits使用状況確認（あってはならない）
grep -n "direction_traits\|legacy" model/geo_primitives/src/{shape}_*.rs
```

### **Step 2: アーキテクチャ適合性確認**

```powershell
# Foundation Pattern遵守確認
.\scripts\check_architecture_dependencies_simple.ps1

# ビルド状態確認  
cargo build

# テスト状態確認
cargo test -p geo_primitives --tests {shape}
```

## 🚨 **実装中必須チェック（During Implementation）**

### **Step 3: 強制実装パターン確認**

**✅ 必須確認項目**:

1. **derive マクロ使用**: `#[derive(Debug, Clone, Copy, PartialEq)]`
2. **主ファイル配置**: `{shape}_3d.rs` に全Core Traits実装
3. **3-Function Pattern**: Constructor + Properties + Measure
4. **レガシートレイト排除**: 古いtraitsの使用なし

**❌ 禁止パターン検出**:

```powershell
# 手動実装検出（禁止）
grep -n "impl.*Debug.*for\|impl.*Clone.*for\|impl.*Copy.*for\|impl.*PartialEq.*for" model/geo_primitives/src/{shape}_*.rs

# 分離ファイル検出（禁止）  
ls model/geo_primitives/src/{shape}_*core_traits.rs 2>$null

# レガシートレイト検出（禁止）
grep -n "direction_traits\|legacy_traits" model/geo_primitives/src/{shape}_*.rs
```

## 🚨 **実装後必須チェック（Post-Implementation Validation）**

### **Step 4: 品質基準適合確認**

```powershell
# ビルド成功確認
cargo build
if ($LASTEXITCODE -ne 0) { Write-Error "ビルド失敗 - 修正必須"; exit 1 }

# テスト成功確認
cargo test --workspace  
if ($LASTEXITCODE -ne 0) { Write-Error "テスト失敗 - 修正必須"; exit 1 }

# Clippy警告ゼロ確認
cargo clippy --workspace -- -D warnings
if ($LASTEXITCODE -ne 0) { Write-Error "Clippy警告 - 修正必須"; exit 1 }

# アーキテクチャ遵守確認
.\scripts\check_architecture_dependencies_simple.ps1
if ($LASTEXITCODE -ne 0) { Write-Error "アーキテクチャ違反 - 修正必須"; exit 1 }
```

### **Step 5: 統一性最終確認**

```powershell
# 全図形のderive統一性確認
$deriveCount = (grep -r "#\[derive.*Debug.*Clone.*Copy.*PartialEq" model/geo_primitives/src/ | wc -l)
$manualCount = (grep -r "impl.*Debug.*for\|impl.*Clone.*for\|impl.*Copy.*for\|impl.*PartialEq.*for" model/geo_primitives/src/ | wc -l)

Write-Host "derive使用: $deriveCount 件"
Write-Host "手動実装: $manualCount 件"

if ($manualCount -gt 0) {
    Write-Error "手動実装が残存 - 全てderiveに統一必須"
    exit 1
}
```

## 🎯 **チェックリスト実行例**

### **Direction3D修正時の実行例**:

```powershell
# Step 1: 現状確認
ls model/geo_primitives/src/direction_*
# → direction_3d.rs, direction_3d_core_traits.rs (問題: 分離ファイル存在)

# Step 2: アーキテクチャ確認  
.\scripts\check_architecture_dependencies_simple.ps1
# → OK

# Step 3: 実装実行
# direction_3d_core_traits.rs の内容を direction_3d.rs にマージ
# derive マクロ追加
# 手動実装削除

# Step 4: 品質確認
cargo build && cargo test --workspace && cargo clippy --workspace -- -D warnings && .\scripts\check_architecture_dependencies_simple.ps1
# → 全てOK

# Step 5: 統一性確認
grep -r "impl.*Debug.*for" model/geo_primitives/src/direction_*
# → 結果なし (OK)
```

## 🚨 **違反時緊急対応手順**

### **違反発見時の即座対応**:

1. **作業停止**: 現在の作業を一時停止
2. **違反特定**: 具体的な違反内容を特定  
3. **標準修正**: 強制実装パターンに修正
4. **検証実行**: 上記チェックリスト再実行
5. **完了確認**: 全項目クリア後に作業再開

### **よくある違反パターンと対応**:

| 違反パターン | 検出方法 | 修正方法 |
|-------------|----------|----------|
| 手動Debug実装 | `grep "impl.*Debug"` | derive マクロに置換 |
| 分離ファイル | `ls *_core_traits.rs` | 主ファイルにマージ・削除 |
| レガシートレイト | `grep "direction_traits"` | Core Traitsに置換 |
| 部分実装 | Core Traits実装数確認 | 不足分を追加実装 |

## **結論**

このチェックリストは**実装品質の最低基準**です。
全ての項目をクリアしなければ、実装完了とは認められません。

**品質基準に妥協はありません。**