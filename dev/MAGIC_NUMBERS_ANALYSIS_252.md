# analysis クレート マジックナンバー分析レポート

**日付**: 2026年2月23日  
**対象**: Issue #252 完了条件（マジックナンバー排除）  
**ステータス**: 特定完了 → 定数化実装待ち

---

## 📊 検出されたマジックナンバー

### 🔴 **最優先対処（units.rs - 5件）**

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

### � **テスト許容誤差値（98件）**

テストファイル全体で許容誤差値がハードコードされており、精度要件の一元管理ができない。

#### **検出パターン**

| 許容誤差値 | 出現回数 | 主な用途 | 推奨定数 |
|----------|---------|---------|---------|
| `1e-10` | 65件 | 標準的な数値精度検証 | `consts::test_constants::TOLERANCE_F64` (既存) |
| `1e-6` | 10件 | やや緩めの精度検証 | `consts::test_constants::TOLERANCE_F32` (既存) |
| `1e-15` | 9件 | 非常に高精度（ソルバー） | `consts::test_constants::SOLVER_TOLERANCE_F64` (新規) |
| `1e-4` | 2件 | 数値積分結果の精度 | `consts::test_constants::INTEGRATION_TOLERANCE` (新規) |
| `1e-3` | 1件 | 積分の緩めの精度 | `consts::test_constants::TOLERANCE_LOOSE` (新規) |
| `1e-12` | 1件 | 角度の高精度 | `consts::GEOMETRIC_ANGLE_TOLERANCE` (既存) |
| `1e-8`, `1e-16` | 各1件 | ソルバー許容誤差設定 | 個別定数化 |

#### **影響範囲**
- solver テスト（cramer, gaussian, lu, newton）: 36件
- matrix テスト（matrix2, matrix3, matrix4）: 22件
- vector テスト: 11件
- numerics テスト（integration, vector_distance）: 10件
- その他（consts, units）: 19件

**推奨対応**: `consts::test_constants` モジュールを拡張し、統一された許容誤差定数を提供

```rust
/// テスト用定数
pub mod test_constants {
    // 既存
    pub const TOLERANCE_F64: f64 = 1e-10;
    pub const TOLERANCE_F32: f32 = 1e-6;
    
    // 新規追加推奨
    /// ソルバー用高精度許容誤差
    pub const SOLVER_TOLERANCE_F64: f64 = 1e-15;
    
    /// 数値積分用許容誤差
    pub const INTEGRATION_TOLERANCE: f64 = 1e-4;
    
    /// 緩めの許容誤差（積分粗い分割）
    pub const TOLERANCE_LOOSE: f64 = 1e-3;
}
```

---

### �🟢 **対処不要（文脈上明確）**

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
### 3. テスト許容誤差値の統一 ⭐ **新規**

```rust
// foundation/analysis/src/consts.rs の test_constants モジュールを拡張

/// テスト用定数
pub mod test_constants {
    // 既存定数（そのまま）
    pub const TOLERANCE_F64: f64 = 1e-10;
    pub const TOLERANCE_F32: f32 = 1e-6;
    
    // 新規追加
    /// ソルバー用高精度許容誤差
    pub const SOLVER_TOLERANCE_F64: f64 = 1e-15;
    
    /// ソルバー用通常許容誤差
    pub const SOLVER_TOLERANCE_NORMAL: f64 = 1e-10;
    
    /// 数値積分用許容誤差（標準精度）
    pub const INTEGRATION_TOLERANCE: f64 = 1e-4;
    
    /// 数値積分用許容誤差（緩い精度）
    pub const INTEGRATION_TOLERANCE_LOOSE: f64 = 1e-3;
    
    /// 数値積分用許容誤差（高精度）
    pub const INTEGRATION_TOLERANCE_STRICT: f64 = 1e-6;
}
```

**修正箇所**（98件）:
- `foundation/analysis/src/linalg/solver/*_tests.rs`: 36件
- `foundation/analysis/src/linalg/matrix/*_tests.rs`: 22件
- `foundation/analysis/src/linalg/vector/*_tests.rs`: 11件
- `foundation/analysis/src/numerics/*_tests.rs`: 10件
- その他テストファイル: 19件

**修正例**:
```rust
// Before
assert!((result - expected).abs() < 1e-10);

// After
use crate::consts::test_constants::TOLERANCE_F64;
assert!((result - expected).abs() < TOLERANCE_F64);
```

---

## 📋 検証チェックリスト

- [ ] units.rs: 単位変換係数を定数化（5件）
- [ ] quaternion.rs: 閾値2件を定数化
- [ ] consts.rs: test_constants モジュールを拡張（5つの新定数追加）
- [ ] テストファイル: 許容誤差値を統一定数に置換（98件）
  - [ ] solver テスト（36件）
  - [ ] matrix テスト（22件）
  - [ ] vector テスト（11件）
  - [ ] numerics テスト（10件）
  - [ ] その他（19件）
    /// メートル → ミリメートル変換係数
    pub const METER_TO_MM_FACTOR: f64 = 1000.0;
    （テストファイル含む）  
**除外**: コメント行, use 文  
**検出パターン**: 
- 浮動小数点リテラル（`\d+\.\d+`）
- 科学的記法（`1e-\d+`） ← **98件検出**
- 特殊な閾値（0.9, 0.9995）
- 単位変換係数（25.4, 1000.0, 10.0）

**ツール**: grep_search + 手動コードレビュー

---

## 📊 統計サマリ

| カテゴリ | 検出数 | 優先度 |
|---------|-------|-------|
| 単位変換係数（units.rs） | 5件 | 🔴 最優先 |
| 四元数閾値（quaternion.rs） | 2件 | 🟡 推奨 |
| テスト許容誤差値 | **98件** | 🟠 重要 |
| 文脈上明確（対処不要） | - | 🟢 保留 |
| **合計** | **105件** | - |
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

**~~テストコード内のマジックナンバーは対象外~~（削除: 実際には98件の許容誤差値を検出し、対処推奨とした）
- テストの許容誤差値は統一定数化により、精度要件の一元管理と可読性向上が期待できる
- consts.rs 内の定数定義は既に適切に管理されているが、test_constants モジュールを拡張する必要がある
- 0, 1, -1 などの基本的な数値は文脈上自明なため対象外
- テストデータの具体値（例: `Point3::new(1.0, 2.0, 3.0)` の座標）は意図的な値として定数化不要
- 浮動小数点リテラル（`\d+\.\d+`）
- 特殊な閾値（0.9, 0.9995）
- 単位変換係数（25.4, 1000.0, 10.0）

**ツール**: grep_search + 手動コードレビュー

---

## 📝 備考

- テストコード内のマジックナンバーは対象外（テストデータは明示的な値が望ましい）
- consts.rs 内の定数定義は既に適切に管理されているため除外
- 0, 1, -1 などの基本的な数値は文脈上自明なため対象外
