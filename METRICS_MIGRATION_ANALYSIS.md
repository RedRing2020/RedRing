# analysis/metrics の geo_core 移行可能性検証レポート

## 📊 検証結果：部分移行を推奨

### 🔄 移行推奨機能 (geo_core へ)

#### 1. 幾何形状専用計算
- `arc_length()` - 円弧長計算（既に geo_core に重複存在）
- `ellipse_arc_length_approximation()` - 楕円弧長近似（既に geo_core に重複存在）
- `circle_area()`, `ellipse_area()` - 基本図形面積
- `triangle_area_2d()`, `triangle_area_heron()` - 三角形面積
- `polygon_area()` - 多角形面積（シューレース公式）
- `box_volume()`, `sphere_volume()`, `cylinder_volume()` - 基本立体体積

**理由**: これらは具体的な幾何形状に特化した計算で、geo_core の役割に適している

#### 2. 重複解消
現在 `foundation/analysis/metrics` と `model/geo_core/metrics` で同一実装が重複している機能を統一

### 🏠 Foundation に残留推奨機能

#### 1. 汎用数値計算
- `point_distance()` - N次元点間距離（汎用）
- `point_distance_squared()` - 二乗距離（最適化用）  
- `manhattan_distance()` - L1ノルム
- `chebyshev_distance()` - L∞ノルム
- `minkowski_distance()` - Lpノルム
- `vector_length()` - 汎用ベクトル長
- `vector_length_squared()` - 二乗長

**理由**: これらはドメイン非依存の純粋な数値計算アルゴリズム

#### 2. 最適化版関数
- `point_distance_2d()`, `point_distance_3d()` - 次元特化最適化
- `vector_length_2d()`, `vector_length_3d()` - 次元特化最適化

**理由**: 幾何計算の高速化のため、Foundation層での提供が適切

### 📋 推奨移行プラン

## Phase 1: 重複解消
1. `geo_core/metrics` の重複実装を analysis から import に変更
2. ビルド・テスト確認

## Phase 2: 幾何専用機能の移行
1. 基本図形の面積・体積計算を geo_core に移行
2. analysis では汎用計算のみ保持
3. 依存関係更新・テスト実行

## Phase 3: 最終整理
1. 不要な重複ファイル削除
2. ドキュメント更新
3. CI/CD確認

### 🎯 期待される効果

1. **責務の明確化**: Foundation（汎用数値）vs Geometry（形状特化）
2. **重複削除**: 同一機能の重複解消
3. **性能向上**: 適切な層での最適化
4. **保守性向上**: 機能の所在が明確

### ⚠️ 注意点

1. **依存関係の循環回避**: geo_core → analysis の単方向維持
2. **既存コード影響**: import文の更新が必要
3. **テスト移行**: 対応するテストも同時移行

## 結論

**部分移行を推奨**します。幾何形状専用の計算は geo_core へ、汎用数値計算は analysis に残すことで、適切な責務分離が実現できます。