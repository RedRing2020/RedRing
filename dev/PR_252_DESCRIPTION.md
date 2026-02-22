# analysis クレート: テスト統一とマジックナンバー排除

Closes #252

## 📝 概要

`foundation/analysis` クレートのテストファイル整理とマジックナンバー排除を実施し、保守性と可読性を大幅に向上させました。

## 🎯 実施内容

### 1. テストファイル統一 (d400f1d)

**課題**:
- quaternion テストが `#[cfg(test)] mod tests` と外部ファイルで二重管理
- 他のモジュールでも同様の不整合が散見

**対応**:
- 重複テストを削除し、外部テストファイル (`*_tests.rs`) に統合
- ファイルネスティング設定を追加（`.vscode/settings.json`）
- 22個のテストファイルを整理、137件のユニットテスト全合格

### 2. マジックナンバー排除 (b518dc1)

**課題**:
- テスト内に `1e-10`, `1e-6`, `1e-15` などの許容誤差値がハードコード
- units.rs に単位変換係数がハードコード（25.4, 1000.0 など）
- quaternion.rs に閾値がハードコード（0.9, 0.9995）

**対応**:

#### 2.1 consts.rs - テスト用定数拡張
```rust
pub mod test_constants {
    // 既存
    pub const TOLERANCE_F64: f64 = 1e-10;
    pub const TOLERANCE_F32: f32 = 1e-6;
    
    // 新規追加
    pub const SOLVER_TOLERANCE_F64: f64 = 1e-15;
    pub const INTEGRATION_TOLERANCE: f64 = 1e-4;
    pub const INTEGRATION_TOLERANCE_LOOSE: f64 = 1e-3;
    pub const INTEGRATION_TOLERANCE_STRICT: f64 = 1e-6;
}
```

#### 2.2 テストファイル - 103箇所の許容誤差値を統一
- solver テスト: 37箇所
- matrix テスト: 26箇所
- vector テスト: 9箇所
- numerics テスト: 8箇所
- その他: 23箇所

#### 2.3 units.rs - 単位変換係数の定数化
```rust
mod conversion {
    pub const MM_TO_MM_FACTOR: f64 = 1.0;
    pub const METER_TO_MM_FACTOR: f64 = 1000.0;
    pub const CM_TO_MM_FACTOR: f64 = 10.0;
    pub const INCH_TO_MM_FACTOR: f64 = 25.4;
}

const DEFAULT_TOLERANCE_MM: f64 = 0.01;
```

#### 2.4 quaternion.rs - 閾値の定数化
```rust
mod thresholds {
    pub const PERPENDICULAR_THRESHOLD: f64 = 0.9;
    pub const SLERP_THRESHOLD: f64 = 0.9995;
}
```

## 📊 変更統計

- **変更ファイル数**: 71ファイル
- **変更量**: +1,461 / -3,327行（コード削減）
- **コミット数**: 13個

## ✅ テスト結果

### ユニットテスト
```
cargo test -p analysis
running 137 tests
test result: ok. 137 passed; 0 failed
```

### Docテスト
```
Doc-tests analysis
running 22 tests
test result: ok. 22 passed; 0 failed
```

### Clippy
```
cargo clippy -p analysis -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.00s
```

### アーキテクチャチェック
```
.\scripts\check_architecture_dependencies_simple.ps1
SUCCESS: All architecture dependency checks passed!
```

### ワークスペース全体
```
cargo test --workspace
42 test suites: all passed
```

## 🎉 効果

### 保守性向上
- 許容誤差値の変更が1箇所で完結
- 重複テストの削除によりメンテナンスコスト削減

### 可読性向上
- `1e-10` → `TOLERANCE_F64` により意味が明確化
- ファイルネスティングでテストファイルが整理

### 一貫性向上
- 同じ用途で同じ定数を使用（統一された精度要件）

## 📚 参考資料

- 詳細分析レポート: [dev/MAGIC_NUMBERS_ANALYSIS_252.md](../dev/MAGIC_NUMBERS_ANALYSIS_252.md)
- Issue: #252

## ✅ 受け入れ条件チェック

- [x] quaternion テストが重複せず、1系統で管理される
- [x] `1e-10` 等のテスト許容誤差リテラルが定数参照化される
- [x] `cargo test -p analysis` が通過する
- [x] `cargo clippy -p analysis -- -D warnings` が通過する

## 🔍 レビューポイント

1. **テスト統一の妥当性**: 外部テストファイルへの移行が適切か
2. **定数名の適切性**: `SOLVER_TOLERANCE_F64`, `INTEGRATION_TOLERANCE` などの命名
3. **ドキュメント品質**: consts.rs の定数説明は十分か

## 📝 備考

- 今回は analysis クレートに限定
- 他のクレートへの横展開は別Issueで対応予定
- breaking change なし（全て内部リファクタリング）
