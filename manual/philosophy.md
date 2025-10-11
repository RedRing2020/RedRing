# 設計思想 / Design Philosophy

RedRing の設計は、以下の原則に基づいて構築されています。

## 1. 責務分離と語義整合

- モジュール・型・関数の命名は、語義的に明確であることを最優先
- 表現と実装の責務を分離し、保守性と拡張性を両立

## 2. 型安全性と抽象化

- Rust の型システムを活用し、誤用を防ぐ API 設計
- トレイトによる抽象化で、汎用性と柔軟性を確保

## 3. 国際化と可読性

- 英語圏の開発者にも直感的に理解できる命名と構成
- ドキュメントは日本語と英語の併記を基本とし、国際貢献を促進

## 4. 将来拡張への備え

- STEP 対応、NURBS 実装、mdBook 多言語化などを視野に入れた構造設計

## 5. エラー処理ガイドライン

RedRing では型安全性と保守性を重視したエラー処理パターンを採用しています。

### 専用エラー型の使用

各幾何要素は独自のエラー型を定義し、具体的なエラー情報を提供します：

```rust
// ✅ 推奨: 専用エラー型
impl Ellipse<T> {
    pub fn from_radii(rx: T, ry: T) -> Result<Self, EllipseError> {
        // 楕円固有の検証とエラー報告
    }
}

// ❌ 非推奨: 汎用エラー型
impl Ellipse<T> {
    pub fn from_radii(rx: T, ry: T) -> Result<Self, GeometryError> {
        // エラーの詳細が不明確
    }
}
```

### 統合トレイトの活用

重複する操作は統合トレイトで抽象化し、型安全性を保ちます：

```rust
// 正規化操作の統合
pub trait Normalizable<T> {
    type Output;
    type Error;
    fn normalize(&self) -> Result<Self::Output, Self::Error>;
}

// 距離計算の統合
pub trait DistanceCalculation<T, Target> {
    fn distance_to(&self, other: &Target) -> T;
}
```

### 実装原則

- **責務分離**: 各モジュールは単一の責務を持つ
- **型安全性**: コンパイル時エラー検出を最大化
- **トレイト設計**: 共通操作は統合トレイトで抽象化
- **エラー情報**: 具体的で actionable なエラーメッセージ
