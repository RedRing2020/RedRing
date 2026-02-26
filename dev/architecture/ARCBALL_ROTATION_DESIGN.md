# Arcball回転設計ドキュメント

**最終更新**: 2026年2月21日
**対応Issue**: #242
**ブランチ**: `feature/issue-242-arcball-rotation`

## 概要

Octree可視化の際に報告されたカメラ回転の不安定性（Issue #242）を解決するために、
Arcball回転アルゴリズムを実装しました。
通常のマウスデルタ(Δx, Δy)ベースの回転から、
**画面座標をマップした仮想球面上の点**を使った直感的な回転に変更します。

## 問題背景

### 旧実装の課題
```rust
pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
    let sensitivity = 0.01;
    // Y軸回転（水平方向のマウス移動）
    let y_rotation = Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), -delta_x * sensitivity);
    // X軸回転（垂直方向のマウス移動）
    let x_rotation = Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -delta_y * sensitivity);
    // ...
}
```

**問題点**:
- マウスデルタ(Δx, Δy)を直接角度に変換している
- **クリック位置がカメラからの深さ距離に関わらず、常に同じ回転量になる**
  → 空間の奥でクリックしても手前でクリックしても同じ
  → 回転中心が定まらない、ぐらぐらした操作感
- 設定した感度(0.01)に完全に依存
- 2D画面座標の形状（縦長/横長）を無視している

### 改善による効果
1. ✅ **回転中心が明確**: 常に注視点（target）
2. ✅ **直感的な操作感**: マウス移動距離 ∝ 回転角度
3. ✅ **ビューポート対応**: 画面サイズ/アスペクト比に自動適応
4. ✅ **安定した動作**: 感度パラメータが削除（不要に）

## 実装詳細

### 1. 球面マッピング関数

#### `project_on_sphere()`
画面座標をビューポート上の単位球面に投影。

```rust
pub fn project_on_sphere(
    screen_x: f32,
    screen_y: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec3f {
    // ビューポート中心基準の正規化座標に変換
    let radius = (viewport_width.min(viewport_height)) * 0.5;
    let cx = screen_x - viewport_width * 0.5;
    let cy = screen_y - viewport_height * 0.5;
    
    let x = cx / radius;
    let y = cy / radius;
    
    // 単位球への投影（len_sq < 1 なら球内、> 1 なら球面上）
    let len_sq = x * x + y * y;
    let z = if len_sq < 1.0 {
        (1.0 - len_sq).sqrt()    // 球の内側
    } else {
        0.0                        // 球面上
    };
    
    // 正規化して返す
    Vec3f::new(x, -y, z).normalize()
}
```

**座標系**:
- Z軸：画面奥行き（常にプラス）
- X軸：画面右（スクリーン座標そのまま）
- Y軸：スクリーン座標は下が増加するため、負号を付与

**動作**:
```
ビューポート中心クリック   → (0, 0, 1)     ← 球の最奥部
ビューポート左端           → (-1, 0, ...)  ← 球の左側
ビューポート右上1/100      → (0.5, -0.5, ~) ← 球表面上
```

### 2. 球面上の2点から回転を計算

#### `compute_rotation_from_sphere_points()`
**入力**: sphere_from, sphere_to（両方とも単位正規化済み）
**出力**: 回転を表現するクォータニオン

```rust
fn compute_rotation_from_sphere_points(sphere_from: Vec3f, sphere_to: Vec3f) -> Quaternionf {
    // ベクトル間の内積から角度を計算
    let dot = sphere_from.dot(&sphere_to).clamp(-1.0, 1.0);
    let angle = dot.acos();    // 回転角度（ラジアン）
    
    // 回転軸は外積
    let axis = sphere_from.cross(&sphere_to);
    
    // 軸の大きさをチェック（平行なら回転なし）
    let axis_magnitude = axis.dot(&axis).sqrt();
    if axis_magnitude < 1e-6 {
        return Quaternionf::identity();
    }
    
    // クォータニオン生成
    let axis_normalized = axis.normalize()?;
    Quaternionf::from_axis_angle(&axis_normalized, angle)
}
```

**数式**:
```
v1 = sphere_from, v2 = sphere_to（単位ベクトル）
回転軸 = v1 × v2
回転角 = acos(v1 · v2)
q = Quat(axis, angle)
```

**エッジケース**:
- `v1 · v2 ≈ 1.0` (ほぼ同じ方向) → 角度 ≈ 0 → 回転なし ✓
- `v1 · v2 ≈ -1.0` (逆方向) → 角度 ≈ π → 任意の軸で180°回転 (外積=0の時は別)
- `|v1 × v2| < 1e-6` → 平行 → id() を返す ✓

### 3. 新しい回転メソッド

#### `rotate_arcball()`
```rust
pub fn rotate_arcball(
    &mut self,
    prev_x: f32, prev_y: f32,
    curr_x: f32, curr_y: f32,
    viewport_width: f32,
    viewport_height: f32,
) {
    // 画面座標を球面上の点に投影
    let sphere_from = Self::project_on_sphere(prev_x, prev_y, viewport_width, viewport_height);
    let sphere_to = Self::project_on_sphere(curr_x, curr_y, viewport_width, viewport_height);
    
    // 回転を計算
    let q_rotation = Self::compute_rotation_from_sphere_points(sphere_from, sphere_to);
    
    // 現在の回転に合成（右乗算）
    self.rotation = (q_rotation * self.rotation)
        .normalize()
        .unwrap_or(self.rotation);
}
```

**使用方法** (AppState):
```rust
if let (Some((prev_x, prev_y)), Some((curr_x, curr_y))) =
    (self.last_cursor_position, self.cursor_position)
{
    let viewport_width = self.graphic.config.width as f32;
    let viewport_height = self.graphic.config.height as f32;
    self.camera.rotate_arcball(
        prev_x, prev_y,
        curr_x, curr_y,
        viewport_width,
        viewport_height,
    );
}
```

## AppState 統合

### 変更点

1. **フィールド追加**:
   ```rust
   pub struct AppState {
       cursor_position: Option<(f32, f32)>,
       last_cursor_position: Option<(f32, f32)>,  // ← NEW
       // ...
   }
   ```

2. **handle_cursor_moved() 修正**:
   ```rust
   pub fn handle_cursor_moved(&mut self, x: f32, y: f32) {
       self.last_cursor_position = self.cursor_position;  // ← 前フレーム保存
       self.cursor_position = Some((x, y));               // ← 現在フレーム更新
       // ...
   }
   ```

3. **handle_mouse_motion() 修正**:
   ```rust
   MouseOperation::Rotate => {
       if let (Some((prev_x, prev_y)), Some((curr_x, curr_y))) =
           (self.last_cursor_position, self.cursor_position)
       {
           let viewport_width = self.graphic.config.width as f32;
           let viewport_height = self.graphic.config.height as f32;
           self.camera.rotate_arcball(
               prev_x, prev_y,
               curr_x, curr_y,
               viewport_width,
               viewport_height,
           );
       } else {
           // フォールバック（cursor_position が無い場合）
           self.camera.rotate(delta_x, delta_y);
       }
       self.update_camera_uniforms();
   }
   ```

## テスト戦略

### ユーザーテスト項目
- [ ] **回転**: Octreeをドラッグして回転
  - 中心付近のドラッグ → スムーズに回転するか
  - 端付近のドラッグ → 不安定でないか
  - 素早いドラッグ → 加速度的な回転していないか（線形であるか）
  
- [ ] **ビューポート対応**:
  - ウィンドウをリサイズ → 回転感度が一定か
  - アスペクト比の異なるビューポート → 各軸が均等に効くか

### 単体テスト
```rust
#[test]
fn test_project_on_sphere_center() {
    let point = Camera::project_on_sphere(400.0, 300.0, 800.0, 600.0);
    assert!((point[2] - 1.0).abs() < 0.01);  // Z ≈ 1
    assert!(point[0].abs() < 0.01);          // X ≈ 0
    assert!(point[1].abs() < 0.01);          // Y ≈ 0
}

#[test]
fn test_sphere_points_identity_rotation() {
    let p = Vec3f::new(1.0, 0.0, 0.0);
    let q = Camera::compute_rotation_from_sphere_points(p, p);
    assert!(q.is_identity());  // 同じ点 → 回転なし
}
```

## パフォーマンス

- sphere_from, sphere_to 計算: O(1) (正規化のみ)
- 外積・内積計算: O(1)
- **フレーム単位の計算量**: 常に一定（ビューポートサイズに非依存）
- オーバーヘッド: 旧rotate() と同等以下

## マイグレーション

### 旧コードとの互換性
- 旧 `rotate(delta_x, delta_y)` メソッドは**削除されない**
  - ビューポートサイズ不明な箇所からのフォールバック用
  - 部分的な修正での互換性確保

### 段階的導入
1. ✅ **Phase 1** (本変更): Arcball 実装、AppState統合
2. ⏳ **Phase 2** (future): その他の回転呼び出し側を Arcball に置き換え（可能な範囲）
3. ⏳ **Phase 3** (future): デルタベースのAPI廃止検討

## 次のステップ（関連Issue #242）

本Arcball実装に続き、以下の改善が予定されています：

1. **パン操作の方向反転修正** - 符号検証
2. **ズーム操作の反応改善** - 対数スケール/大きさ正規化
3. **ズーム判定が逆の修正** - 対角線方向定義

詳細は [#242](https://github.com/RedRing2020/RedRing/issues/242) 参照。

## 参考文献

- **Arcball**: Ken Shoemake, "Arcball: a user interface for specifying three-dimensional orientation using a mouse", 1992
- **Quaternion Calculus**: analysis crate documentation
- **相互参照**: 
  - [OCTREE_VISUALIZATION_DESIGN.md](OCTREE_VISUALIZATION_DESIGN.md) - Issue #207 (親Issue)
  - Camera implementation: `viewmodel/graphics/src/camera.rs`
  - Integration: `view/app/src/app_state.rs`, `view/app/src/mouse_input.rs`
