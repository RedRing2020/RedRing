# analysis クレート マジックナンバー分析レポート

**日付**: 2026年2月23日  
**対象**: Issue #252 完了条件（マジックナンバー排除）  
**ステータス**: 特定完了 → 定数化実装待ち

---

## 📊 検出されたマジックナンバー

### 🔴 **最優先対処（units.rs）**

単位変換係数がハードコードされており、保守性と可読性に影響。

#### **ファイル**: `foundation/analysis/src/units.rs`

| 行 | 現在のコード | 用途 | 推奨定数名 |
|---|-------------|------|----------|
| 37 | `1.0` | ミリメートル基準値 | `MM_TO_MM_FACTOR` |
| 38 | `1000.0` | メートル→ミリメートル | `METER_TO_MM_FACTOR` |
| 39 | `10.0` | センチメートル→ミリメートル | `CM_TO_MM_FACTOR` |
| 40 | `25.4` | インチ→ミリメートル | `INCH_TO_MM_FACTOR` |
| 173 | `0.01` | デフォルトトレランス (mm) | `DEFAULT_TOLERANCE_MM` |

**影響範囲**: 単位変換系統全体  
**推奨対応**: モジュールレベル定数として定義

---

### 🟡 **推奨対処（quaternion.rs）**

四元数計算の閾値がハードコードされており、調整時に説明が必要。

#### **ファイル**: `foundation/analysis/src/linalg/quaternion.rs`

| 行 | 現在のコード | 用途 | 推奨定数名 |
|---|-------------|------|----------|
| 104 | `0.9` | ベクトル垂直判定閾値 | `PERPENDICULAR_THRESHOLD` |
| 358 | `0.9995` | SLERP線形補間閾値 | `SLERP_THRESHOLD` |

**影響範囲**: 四元数補間・回転計算  
**推奨対応**: impl ブロック内の const または モジュール定数

---

### 🟢 **対処不要（文脈上明確）**

以下は数学的・文脈的に明確なため、定数化は任意。

#### **angle.rs** (π の分割)
- `2.0`, `3.0`, `4.0`, `6.0` → π/2, π/3, π/4, π/6 の計算に使用
- 文脈: `T::PI / T::from_f64(2.0)` のような形式で、意味が自明
- 現状維持推奨（過度な定数化は可読性低下）

#### **integration.rs** (台形公式)
- `0.5` → 台形公式の数学的係数
- 文脈: `0.5 * (v0.norm() + v1.norm())` で明確
- 現状維持推奨

#### **その他の基本値**
- `1.0`, `0.0` → ゼロ・単位値として自明
- `-1.0` → 方向反転として文脈上明確

---

## 🎯 実装推奨事項

### 1. units.rs の定数化

```rust
// foundation/analysis/src/units.rs 冒頭に追加

/// 単位変換係数
mod conversion {
    /// ミリメートル基準値
    pub const MM_TO_MM_FACTOR: f64 = 1.0;
    
    /// メートル → ミリメートル変換係数
    pub const METER_TO_MM_FACTOR: f64 = 1000.0;
    
    /// センチメートル → ミリメートル変換係数
    pub const CM_TO_MM_FACTOR: f64 = 10.0;
    
    /// インチ → ミリメートル変換係数 (1 inch = 25.4 mm)
    pub const INCH_TO_MM_FACTOR: f64 = 25.4;
}

/// デフォルトトレランス (ミリメートル単位)
const DEFAULT_TOLERANCE_MM: f64 = 0.01;
```

**修正箇所**:
- `to_millimeter_factor()` メソッド内の switch 文
- `Tolerance::default()` 実装

### 2. quaternion.rs の定数化

```rust
// foundation/analysis/src/linalg/quaternion.rs 冒頭に追加

/// 四元数計算用閾値
mod thresholds {
    /// ベクトル垂直判定閾値
    /// x成分の絶対値がこの値未満の場合、x軸と垂直と見なす
    pub const PERPENDICULAR_THRESHOLD: f64 = 0.9;
    
    /// SLERP閾値
    /// 内積がこの値以上の場合、線形補間（LERP）を使用
    pub const SLERP_THRESHOLD: f64 = 0.9995;
}
```

**修正箇所**:
- `from_two_vectors()` メソッド (L104)
- `slerp()` メソッド (L358)

---

## 📋 検証チェックリスト

- [ ] units.rs: 単位変換係数を定数化
- [ ] units.rs: デフォルトトレランスを定数化
- [ ] quaternion.rs: 閾値2件を定数化
- [ ] cargo test -p analysis で全テスト合格
- [ ] cargo clippy で警告なし
- [ ] ドキュメントコメント追加（定数の意味と出典）

---

## 🔍 分析手法

**検索対象**: `foundation/analysis/src/**/*.rs`  
**除外**: `*_tests.rs`, コメント行, use 文  
**検出パターン**: 
- 浮動小数点リテラル（`\d+\.\d+`）
- 特殊な閾値（0.9, 0.9995）
- 単位変換係数（25.4, 1000.0, 10.0）

**ツール**: grep_search + 手動コードレビュー

---

## 📝 備考

- テストコード内のマジックナンバーは対象外（テストデータは明示的な値が望ましい）
- consts.rs 内の定数定義は既に適切に管理されているため除外
- 0, 1, -1 などの基本的な数値は文脈上自明なため対象外
