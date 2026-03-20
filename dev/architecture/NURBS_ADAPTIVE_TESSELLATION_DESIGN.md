# NURBS適応的テッセレーション設計（GPU直接描画）

**作成日**: 2026年2月14日  
**最終更新**: 2026年2月14日  
**対象Issue**: #210（NURBS形状の適応的テッセレーション実装 - GPU直接描画）  
**関連**: SHAPE_TESSELLATION_DESIGN.md, NURBS_FOUNDATION_PATTERN.md  

## 1. 目的

NURBS曲線・サーフェスを対象に、適応的テッセレーションを導入する。Phase 1はハイブリッド方式（CPUで分割パラメータ生成 + GPUで点評価）で開始し、必要に応じてPhase 2で完全GPU適応（Compute）へ移行する。

## 2. スコープ

- 対象: NurbsCurve3D / NurbsSurface3D
- 誤差尺度: 3D空間の弦誤差（距離）
- 出力形式: LineStrip（曲線）、ワイヤーフレーム（サーフェス）

## 3. 設計方針（ハイブリッド方式）

### 3.1 全体フロー

1. CPU側で適応分割パラメータ列を生成（弦誤差ベース）
2. GPU側でNURBS評価し頂点列を生成
3. LineStripで描画（サーフェスはU/Vワイヤーフレーム）

### 3.2 CPU側: 適応分割パラメータ生成

- 入力: NURBS曲線/サーフェス（control points, knots, weights, degree）
- 出力: パラメータ列 `t_list`（曲線）/ `u_list`, `v_list`（サーフェス）
- 誤差判定: 弦誤差 $\varepsilon$（3D距離）

**曲線の弦誤差**:

$$
\varepsilon = \| P(\frac{t_0+t_1}{2}) - \frac{P(t_0) + P(t_1)}{2} \|
$$

- $\varepsilon$ が許容値以下なら分割を停止
- $\varepsilon$ が許容値超過なら区間を2分割
- 最大分割数を超えた場合は打ち切り

#### 3.2.1 曲線: 適応分割アルゴリズム

- 初期区間はパラメータドメイン全体 $[t_{min}, t_{max}]$
- キュー（またはスタック）で区間を管理し、弦誤差が大きい区間だけを再帰的に分割
- 目標セグメント数の下限を確保するため、`min_segments` 相当の初期分割も可能

**停止条件**:
- 弦誤差 $\varepsilon \leq chord\_error$
- 区間長がしきい値以下（`min_param_span`）
- 分割回数が `max_subdivisions` に到達

**パラメータ列の要件**:
- 単調増加
- 端点包含（`t_min`, `t_max`）
- 連続区間の重複なし

**擬似コード**:

```text
queue <- [(t_min, t_max, depth=0)]
params <- {t_min, t_max}

while queue not empty:
  (a, b, depth) <- pop
  m <- (a + b) / 2
  err <- chord_error(a, m, b)
  if err <= chord_error or depth >= max_subdivisions:
    continue
  else:
    params.add(m)
    push (a, m, depth+1)
    push (m, b, depth+1)

params <- sort(params)
ensure_min_segments(params, min_segments)
```

#### 3.2.2 サーフェス: U/V適応分割

- U方向/ V方向を独立に曲線として扱う
- 代表断面（境界線または中央断面）で曲線誤差評価
- 得られた `u_list`, `v_list` の直積でグリッド生成

**代表断面の選択**:
- まず境界線（u=const, v=const）を使用
- 必要に応じて中央断面を追加し、最大誤差を採用

**注意点**:
- U/Vの分割密度が大きく異なる場合に備え、`max_grid_points` を設定
- ワイヤーフレーム描画時はLineStripに分割して描く（U方向列とV方向列）

**サーフェスの分割**:
- U方向とV方向を独立に適応分割
- まず境界線（u=const / v=const）を曲線として評価
- 得られた `u_list`, `v_list` の直積でグリッド生成

### 3.3 GPU側: NURBS評価と描画

- GPUに `t_list`（曲線）または `u_list`, `v_list`（サーフェス）を渡す
- WGSL内でNURBS評価を行い、頂点位置を算出
- 出力はLineStrip（曲線）/ LineList（サーフェスの格子線）

## 4. データ構造案

```rust
pub struct AdaptiveTessellationSettings<T: Scalar> {
  pub chord_error: T,        // 許容弦誤差
  pub max_subdivisions: u32, // 最大分割数
  pub min_segments: u32,     // 最小分割数
  pub min_param_span: T,     // 最小パラメータ区間幅
}

pub struct AdaptiveParamList<T: Scalar> {
  pub params: Vec<T>, // 曲線用 t_list
}

pub struct AdaptiveParamGrid<T: Scalar> {
  pub u_params: Vec<T>,
  pub v_params: Vec<T>,
}
```

### 4.1.6 設定のデフォルト値（案）

```rust
impl<T: Scalar> AdaptiveTessellationSettings<T> {
  pub fn default_with_tolerance(display_tolerance: T) -> Self {
    Self {
      chord_error: display_tolerance,
      max_subdivisions: 12,
      min_segments: 16,
      min_param_span: display_tolerance, // 初期値は表示トレランス同等
    }
  }
}
```

**意図**:
- `chord_error`: アプリケーションの表示トレランスに一致させる
- `max_subdivisions`: 極端な分割を抑制
- `min_segments`: 低曲率でも最低限の滑らかさを確保
- `min_param_span`: パラメータ空間の過細分化防止

## 4.1 CPU側 関数/API設計（案）

### 4.1.1 主要API（曲線）

```rust
/// NURBS曲線の適応分割パラメータ列を生成する。
pub fn adaptive_params_curve<T: Scalar>(
  curve: &NurbsCurve3D<T>,
  settings: &AdaptiveTessellationSettings<T>,
) -> AdaptiveParamList<T>
```

### 4.1.2 主要API（サーフェス）

```rust
/// NURBSサーフェスのU/V適応分割パラメータ列を生成する。
pub fn adaptive_params_surface<T: Scalar>(
  surface: &NurbsSurface3D<T>,
  settings: &AdaptiveTessellationSettings<T>,
) -> AdaptiveParamGrid<T>
```

### 4.1.3 補助API

```rust
/// 弦誤差を計算する（曲線用）。
pub fn chord_error<T: Scalar>(
  curve: &NurbsCurve3D<T>,
  t0: T,
  t1: T,
) -> T

/// パラメータ列の最小分割数を保証する。
pub fn ensure_min_segments<T: Scalar>(
  params: &mut Vec<T>,
  t_min: T,
  t_max: T,
  min_segments: u32,
)
```

### 4.1.4 配置案

- `model/geo_nurbs/src/curve_3d_extensions.rs`
  - 既存の拡張ファイルに適応分割APIを追加
- `model/geo_nurbs/src/surface_3d_extensions.rs`（新規）
  - サーフェス向けの適応分割APIを追加
- `model/geo_nurbs/src/lib.rs`
  - `surface_3d_extensions` を公開（必要に応じて）

※ 依存関係は geo_nurbs → geo_contracts/geo_core を維持する。

### 4.1.5 ViewModel層の利用案

```rust
// ViewModel層での利用イメージ
let params = adaptive_params_curve(&curve, &settings);
let gpu_params: Vec<f32> = params.params.iter().map(|t| *t as f32).collect();
```

## 4.2 geo_nurbs 拡張API（推奨）

GPU向けに「パラメータ列」を返すAPIをgeo_nurbsの拡張として提供する。

```rust
pub trait NurbsAdaptiveParametricTessellation<T: Scalar> {
  fn adaptive_params_curve(
    &self,
    settings: &AdaptiveTessellationSettings<T>,
  ) -> AdaptiveParamList<T>;

  fn adaptive_params_surface(
    &self,
    settings: &AdaptiveTessellationSettings<T>,
  ) -> AdaptiveParamGrid<T>;
}
```

実装配置:
- `curve_3d_extensions.rs` に曲線用の実装
- `surface_3d_extensions.rs`（新規）にサーフェス用の実装

## 4.3 geo_contracts::NurbsTessellation との関係

`NurbsTessellation` は CPU側のメッシュ化用途として維持し、
GPU向けのパラメータ列APIとは別レイヤーで提供する。

推奨の型対応:

- `NurbsCurve3D<T>`
  - `Point = Point3D<T>`
  - `Triangle = [Point3D<T>; 3]`（未使用）
  - `Mesh = Vec<Point3D<T>>`（LineStrip相当）

- `NurbsSurface3D<T>`
  - `Point = Point3D<T>`
  - `Triangle = [Point3D<T>; 3]`
  - `Mesh = Vec<[Point3D<T>; 3]>`

※ GPUパスは `NurbsAdaptiveParametricTessellation` を使用する。

## 5. パイプライン設計（案D）

### 5.1 ViewModel層

- NurbsCurve3D / NurbsSurface3Dのパラメータ列を生成
- GPUに渡す最小限のデータに変換

### 5.2 View層

- `NurbsEvalResources` を新規追加
  - NURBS制御点/重み/ノットをGPUバッファ化
  - パラメータ列をGPUへ転送
- RenderPassでLineStrip/LineList描画

## 6. Phase 2（案C）への拡張方針

- Compute Shaderで適応分割をGPU化
- 出力をSSBOに書き込み、Indirect Drawで描画
- Phase 1のCPU分割ロジックと入出力形式を一致させ、移行コストを抑える

## 7. テスト方針

- CPU側の適応分割ロジックに単体テスト
  - 直線: 分割数が最小になる
  - 強い曲率: 分割数が増える
- パラメータ列の単調性と境界包含を検証

### 7.1 テストケース（曲線）

1. **直線NURBS**
  - 入力: 次数1の線分（2制御点, weight=1）
  - 期待: `params.len() == min_segments + 1`（min_segments強制を有効化した場合）
  - 単調性/端点包含を満たす

2. **円弧相当の曲線**
  - 入力: 円弧を近似するNURBS
  - 期待: `params.len()` が直線より増加
  - 弦誤差が閾値以下

3. **極端なトレランス**
  - 入力: chord_error を極端に小さく
  - 期待: `max_subdivisions` に到達して打ち切り

4. **最小区間幅**
  - 入力: `min_param_span` を大きく設定
  - 期待: 分割が抑制される

### 7.2 テストケース（サーフェス）

1. **平面サーフェス（unit_plane）**
  - 期待: `u_params` と `v_params` が最小分割に近い
  - グリッド生成後も単調性/端点包含

2. **曲率の高いサーフェス**
  - 期待: `u_params`/`v_params` が増加

### 7.3 検証ユーティリティ

- `is_strictly_increasing(params)`
- `includes_endpoints(params, t_min, t_max)`
- `max_segment_error(curve, params)`

## 8. 実装順序（Phase 1）

1. 設計ドキュメント更新
2. CPU側: 適応分割パラメータ生成
3. GPU側: NURBS評価用WGSL追加
4. 曲線描画（LineStrip）
5. サーフェス描画（ワイヤーフレーム）
6. テスト追加

## 9. 注意事項

- Foundation Patternの依存階層を厳守（geo_nurbs -> geo_core -> geo_contracts）
- 既存のSVG経由テッセレーションはデバッグ用途として維持
- 将来のCompute移行を見据えて、GPU入力形式は固定化する

