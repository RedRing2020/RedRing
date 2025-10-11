# numerical_utils リファクタリング計画

## 現在の問題点
- numerical_utils.rs に様々な計算機能が混在
- 保守性が低下する恐れ
- 機能追加時の影響範囲が不明確

## 提案する分類構造

### 1. 角度関連 (angles/)
```
analysis/src/angles/
├── mod.rs              // 角度計算の統合モジュール
├── normalization.rs   // 角度正規化
├── comparison.rs       // 角度比較・範囲判定
└── operations.rs       // 角度演算（差分計算など）
```

### 2. 数値計算基盤 (numerics/)
```
analysis/src/numerics/
├── mod.rs              // 数値計算の統合モジュール
├── comparison.rs       // 許容誤差付き比較
├── precision.rs        // 精度管理・丸め処理
└── constants.rs        // 数学定数管理
```

### 3. 距離・計量 (metrics/)
```
analysis/src/metrics/
├── mod.rs              // 計量計算の統合モジュール
├── distance.rs         // 距離計算
├── length.rs           // 長さ計算
└── area_volume.rs      // 面積・体積計算
```

### 4. 幾何学的近似 (approximations/)
```
analysis/src/approximations/
├── mod.rs              // 近似計算の統合モジュール
├── ellipse.rs          // 楕円関連近似
├── curves.rs           // 曲線長計算
└── surfaces.rs         // 表面積近似
```

### 5. 空間構造 (spatial/)
```
analysis/src/spatial/
├── mod.rs              // 空間構造の統合モジュール
├── bbox.rs             // 境界ボックス計算
├── transforms.rs       // 座標変換
└── projections.rs      // 投影計算
```

## 移行手順

### Phase 1: 新構造の作成
1. 各カテゴリのディレクトリ作成
2. 機能別ファイル分割
3. mod.rs での統合

### Phase 2: 段階的移行
1. numerical_utils.rs から機能を分離
2. 後方互換性のためのre-export維持
3. 段階的なdeprecation

### Phase 3: クリーンアップ
1. numerical_utils.rs の削除
2. 新構造への完全移行
3. ドキュメント更新

## 利点

### 保守性向上
- 機能別の明確な分離
- 責務の明確化
- テストの分離

### 拡張性向上
- 新機能の追加先が明確
- 既存機能への影響最小化
- モジュラーな設計

### 可読性向上
- 機能の発見が容易
- コードの理解が簡単
- ドキュメント構造の改善