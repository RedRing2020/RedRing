# RedRing アーキテクチャ依存性チェック項目

## 📋 **依存性ルール概要**

### **基本原則**

- **View → ViewModel → Model** の一方向依存
- **Model 層命名規則**: `geo_` プレフィックス必須
- **analysis**: 完全独立（数値計算専用）
- **逆方向依存**: 全面禁止

---

## 🏗️ **層別依存性マトリックス**

### **Model 層 (geo\_\*)**

```
geo_foundation → analysis ✅
geo_core → geo_foundation, analysis ✅
geo_primitives → geo_foundation, geo_core, analysis ✅
geo_algorithms → geo_foundation, geo_core, geo_primitives, analysis ✅
geo_io → geo_foundation, geo_core, geo_primitives, geo_algorithms, analysis ✅
```

### **ViewModel 層**

```
converter → geo_*, analysis ✅ (例外: geo_io直接参照許可)
graphics → geo_foundation, geo_core, geo_primitives, analysis ✅
```

### **View 層**

```
render → analysis ✅ (GPU層独立性保持)
stage → render, analysis ✅
app → converter, graphics, render, stage, analysis ✅
```

### **Analysis 層**

```
analysis → なし ✅ (完全独立)
```

---

## ❌ **禁止された依存パターン**

### **Model → ViewModel/View 禁止**

- `geo_*` → `converter`, `graphics` ❌
- `geo_*` → `render`, `stage`, `app` ❌

### **ViewModel → View 禁止**

- `converter` → `render`, `stage`, `app` ❌
- `graphics` → `render`, `stage`, `app` ❌

### **View → Model 禁止 (例外あり)**

- `render` → `geo_*` ❌
- `stage` → `geo_*` ❌
- `app` → `geo_*` ❌
- **例外**: `View` → `geo_foundation` (将来許可予定)

### **Analysis 完全独立**

- `analysis` → 他すべて ❌

---

## 🔍 **チェック項目詳細**

### **1. 命名規則チェック**

- [ ] Model 層クレートに `geo_` プレフィックス
- [ ] 必須クレート存在確認:
  - [ ] `geo_foundation` (抽象化レイヤー)
  - [ ] `geo_core` (基盤)
  - [ ] `geo_primitives` (プリミティブ)
  - [ ] `geo_algorithms` (アルゴリズム)
  - [ ] `geo_io` (I/O 専用)

### **2. 依存方向チェック**

- [ ] View → ViewModel → Model の一方向性
- [ ] 逆方向依存の検出・排除
- [ ] 循環依存の検出・排除

### **3. 例外ルールチェック**

- [ ] `ViewModel` → `geo_io` 直接参照 (データ変換効率化)
- [ ] `render` の `model` 非依存 (GPU 層独立性)
- [ ] `analysis` の完全独立性

### **4. 具体的依存性検証**

- [ ] `Cargo.toml` の `[dependencies]` セクション解析
- [ ] ワークスペース内クレート間依存の抽出
- [ ] 外部クレート依存と内部依存の分離

---

## 🛠️ **自動チェックスクリプト**

### **実行方法**

```powershell
# 基本チェック
.\scripts\check_architecture_dependencies.ps1

# 詳細モード
.\scripts\check_architecture_dependencies.ps1 -Verbose

# ルール確認
.\scripts\check_architecture_dependencies.ps1 -Rules

# CI/CD用 (エラー時終了)
.\scripts\check_architecture_dependencies.ps1 -ExitOnError
```

### **チェック内容**

1. **命名規則検証**: Model 層 `geo_*` プレフィックス
2. **依存性解析**: `Cargo.toml` から依存関係抽出
3. **ルール照合**: 許可/禁止パターンとの照合
4. **層別サマリー**: 各層の依存関係可視化
5. **結果レポート**: エラー・警告の統計

---

## 📊 **期待される依存グラフ**

```
┌─────────────┐
│   analysis  │ (完全独立)
└─────────────┘

┌─────────────────────────────────────┐
│              Model層                │
│  ┌─────────────┐                   │
│  │geo_foundation│ ← 抽象化レイヤー    │
│  └─────┬───────┘                   │
│        │                           │
│  ┌─────▼───────┐                   │
│  │  geo_core   │ ← 基盤            │
│  └─────┬───────┘                   │
│        │                           │
│  ┌─────▼───────┐                   │
│  │geo_primitives│ ← プリミティブ     │
│  └─────┬───────┘                   │
│        │                           │
│  ┌─────▼───────┐ ┌─────────────┐   │
│  │geo_algorithms│ │   geo_io    │   │
│  └─────────────┘ └─────────────┘   │
└─────────────────────────────────────┘
            ▲                ▲
            │                │
┌───────────┴────────────────┴───────┐
│           ViewModel層              │
│  ┌─────────────┐ ┌─────────────┐  │
│  │  converter  │ │  graphics   │  │
│  └─────────────┘ └─────────────┘  │
└─────────────┬─────────────────────┘
              │
┌─────────────▼─────────────────────┐
│             View層                │
│  ┌────────┐ ┌───────┐ ┌───────┐  │
│  │ render │ │ stage │ │  app  │  │
│  └────────┘ └───────┘ └───────┘  │
└───────────────────────────────────┘
```

---

## 🎯 **実装指針**

### **View 側でのジェネリック型変換**

```rust
// ❌ 悪い例: ViewがModel具象型に直接依存
use geo_primitives::Circle3D;

// ✅ 良い例: ViewModelが橋渡し
impl<T> ViewConverter<T> {
    fn convert_from_viewmodel(&self, data: ViewModelData) -> T {
        // View側でジェネリック型変換を実装
    }
}
```

### **ViewModel 層での具体的 Model 参照**

```rust
// ✅ 許可: ViewModelは geo_* への具体的参照
use geo_primitives::Circle3D;
use geo_algorithms::IntersectionAlgorithm;
use geo_io::StlLoader;  // 例外: 効率的データ変換

impl GeometryViewModel {
    fn process_geometry(&self) -> ViewData {
        // geo_* の具体実装を使用
    }
}
```

### **Model 層の独立性**

```rust
// ✅ Model層: ViewModel/Viewへの参照なし
// geo_primitives/src/circle.rs
impl Circle3D {
    fn calculate_area(&self) -> f64 {
        // 純粋な幾何計算、UI非依存
    }
}
```

---

## 🚨 **よくある違反パターン**

### **1. 逆方向依存**

```rust
// ❌ geo_primitives から graphics への依存
use crate::graphics::Camera;  // 禁止
```

### **2. 層跨ぎ依存**

```rust
// ❌ View から Model への直接依存
use geo_primitives::Circle3D;  // 禁止 (app内)
```

### **3. Analysis の依存性破綻**

```rust
// ❌ analysis から他クレートへの依存
use geo_foundation::GeometryTrait;  // 禁止
```

この依存性チェック項目により、RedRing のアーキテクチャ整合性が保たれます！
