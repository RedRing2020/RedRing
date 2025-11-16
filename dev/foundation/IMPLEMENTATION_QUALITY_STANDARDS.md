# 実装品質基準（Implementation Quality Standards）

**作成日**: 2025年11月16日  
**最終更新**: 2025年11月16日

## 🚨 **AI開発者への絶対遵守ルール**

このドキュメントは、AI開発者による実装品質違反を防止するための強制ルールです。
**違反は設計アーキテクチャの破綻を招くため、例外は一切認められません。**

## **【CRITICAL】derive マクロ統一基準**

### **Rule 1: derive マクロ強制使用**

**✅ 必須パターン**:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct {Shape}2D<T: Scalar> { /* fields */ }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct {Shape}3D<T: Scalar> { /* fields */ }
```

**❌ 絶対禁止**:
```rust
// 手動実装は一切禁止
impl<T: Scalar> Debug for Direction3D<T> { /* ... */ }
impl<T: Scalar> Clone for Direction3D<T> { /* ... */ }
impl<T: Scalar> Copy for Direction3D<T> { /* ... */ }
impl<T: Scalar> PartialEq for Direction3D<T> { /* ... */ }
```

### **Rule 2: 統一性検証の義務**

**実装前必須チェック**:
```powershell
# derive使用状況確認
grep -r "#\[derive.*Debug.*Clone.*Copy.*PartialEq" model/geo_primitives/src/

# 手動実装検出（あってはならない）
grep -r "impl.*Debug.*for\|impl.*Clone.*for\|impl.*Copy.*for\|impl.*PartialEq.*for" model/geo_primitives/src/
```

**違反発見時の即座対応**:
1. **手動実装を削除**
2. **derive マクロに置換**
3. **テスト実行で動作確認**
4. **統一性の再確認**

### **Rule 3: 例外禁止**

**以下の理由は無効**:
- ❌ "既存コードの尊重"
- ❌ "動作するから変更不要" 
- ❌ "一部だけ修正すれば十分"
- ❌ "影響範囲が大きい"

**統一性が最優先**: 一つの例外も認めない

## **【CRITICAL】Foundation Pattern ファイル配置基準**

### **Rule 4: 主ファイル集約の原則**

**✅ 正しい配置**:
```rust
// model/geo_primitives/src/direction_3d.rs （主ファイル）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction3D<T: Scalar> { /* fields */ }

impl<T: Scalar> DirectionConstructor<T> for Direction3D<T> { /* impl */ }
impl<T: Scalar> DirectionProperties<T> for Direction3D<T> { /* impl */ }
impl<T: Scalar> DirectionMeasure<T> for Direction3D<T> { /* impl */ }
```

**❌ 絶対禁止**:
```rust
// direction_3d_core_traits.rs などの分離ファイル作成
// extensions ファイルでのCore Traits実装
// 複数ファイルへの実装分散
```

### **Rule 5: レガシートレイト禁止**

**✅ 使用必須**:
```rust
// 新しいCore Traits（geo_foundation/src/core/）
impl<T: Scalar> DirectionConstructor<T> for Direction3D<T> { /* ... */ }
impl<T: Scalar> DirectionProperties<T> for Direction3D<T> { /* ... */ }
impl<T: Scalar> DirectionMeasure<T> for Direction3D<T> { /* ... */ }
```

**❌ 絶対禁止**:
```rust
// 古いlegacy traits
impl<T: Scalar> direction_traits::Direction<T> for Direction3D<T> { /* ... */ }
```

## **【CRITICAL】実装完整性基準**

### **Rule 6: 3-Function Pattern 強制**

**全ての幾何図形は以下3つを実装**:
1. **Constructor**: `new()`, `try_new()` メソッド
2. **Properties**: アクセサメソッド、Analysis変換
3. **Measure**: 距離、長さ、面積などの計量

**部分実装は違反**: 3つ全ての実装が必須

### **Rule 7: 品質チェック強制実行**

**実装完了後の必須検証**:
```powershell
cargo build                              # ビルド成功
cargo test --workspace                   # 全テスト成功  
cargo clippy --workspace -- -D warnings # Clippy警告ゼロ
```

**一つでも失敗した場合は実装不完全として修正必須**

## **【ENFORCEMENT】違反時の対応**

### **即座実行事項**

1. **違反コードの特定**
2. **標準パターンへの修正**
3. **統一性の確認**
4. **品質チェックの実行**
5. **完了の確認**

### **違反防止策**

- **実装前**: 既存パターンの確認
- **実装中**: ルールの遵守確認
- **実装後**: 品質チェックの実行
- **継続**: 定期的な統一性検証

## **結論**

これらのルールは**RedRingアーキテクチャの完整性を保持**するための最小限の基準です。
AI開発者は**システム的思考**を持ち、**統一性を最優先**として実装を行ってください。

**品質 > 速度 > 機能追加**

品質の統一なくして、持続可能な開発は不可能です。