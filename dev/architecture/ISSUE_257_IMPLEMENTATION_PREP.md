# Issue #257 実施準備チェックリスト

対象Issue: [#257: [CAM] 5軸ツールパス表現の導入（Gコード互換 + 機械制約保持）](https://github.com/RedRing2020/RedRing/issues/257)

## 1. 目的

- 3軸前提の現行 `ToolPath` を拡張可能な形で整理し、3軸 / 3+2 / 同時5軸を同一モデルで表現できる設計案を固める
- Gコード互換性を評価軸にしつつ、機械制約と姿勢補間方針を保持できる中間モデルを定義する
- #214 / #300 / #411 とは責務境界を分け、5軸表現だけを独立に仕様化する

## 2. 現状整理（2026-03-25）

### 2.1 現行 `cam_core::ToolPath` の構造

現行実装は `model/cam_core/src/toolpath.rs` にあり、以下の特徴を持つ。

- `ToolPath`
  - `tool_id`
  - `cutting_direction`
  - `approach_segments`
  - `contour_levels`
  - `retract_segments`
  - `ext_attributes`
- `PathSegment`
  - `start`
  - `geometry`（Line / Arc）
  - `segment_type`
  - `ext_attributes`
- 幾何は位置中心で、姿勢（工具軸方向・回転軸角）は保持しない
- `TOOLPATH_SCHEMA_VERSION_V0_1 = (0, 1)` は 3軸相当の wire schema を表す

### 2.2 現行モデルで不足しているもの

- 工具姿勢（少なくとも start/end pose）
- 姿勢補間ポリシー
- 0/360 同値・巻き戻し（rewind）情報
- 回転軸制約（min/max、速度、加速度）
- 特異点近傍や軸反転の扱い方針
- 3+2 と同時5軸を区別する表現

## 3. 責務境界

- `cam_core`
  - ToolPath / Segment / ToolPose / 機械制約などの中間モデルを保持する候補
- `cam_sim`
  - 5軸 ToolPath の「消費側」になり得るが、初回は依存させない
- `artifact_binary`
  - v0.1 は 3軸前提。5軸 wire format 追加は別フェーズで扱う
- `cam_algorithms`（将来）
  - 5軸経路生成・最適化・姿勢補間のアルゴリズム責務候補

## 4. 設計選択肢

### 選択肢A: `PathSegment` に start/end pose を直接追加

例:
- `start_pose: ToolPose<T>`
- `end_pose: ToolPose<T>`

利点:
- セグメント単位で完結しやすい
- 3+2 / 同時5軸を同一構造で表しやすい

懸念:
- 3軸ケースでも pose 情報を常に持つことになる
- 既存 3軸 API への影響が大きい

### 選択肢B: 位置幾何は現行維持し、姿勢を別レイヤーで付与

例:
- `PathSegment` は位置中心のまま維持
- `PoseAnnotatedSegment` / `ToolPoseSpan` を別型で追加

利点:
- 既存 3軸 API への破壊を抑えやすい
- 5軸機能を段階導入しやすい

懸念:
- 位置と姿勢の整合を複数型で管理する必要がある
- API が二層になりやすい

### 選択肢C: `ToolPath` を世代分離する

例:
- `ToolPath3Axis`
- `ToolPathMultiAxis`

利点:
- 3軸と5軸の責務差を明示できる
- 既存 API 互換を維持しやすい

懸念:
- 二重モデル化で利用側が複雑化しやすい
- #300 v0.1 契約との対応整理が増える

## 5. 初回推奨方針（案）

初回は **選択肢B** を採択案の検討基準とする。

理由:
- 現行 `ToolPath` / `PathSegment` を即座に破壊せず、3軸利用を温存できる
- 5軸用の姿勢モデルだけを独立に設計しやすい
- #214 / `cam_sim` / artifact v0.1 へ影響を波及させずに設計を先行できる

検討の進め方:

- まず選択肢Bで API 草案と代表ケースを具体化する
- その上で、A/C にしか解決できない問題が出るかを比較して判断する

## 6. API 草案（たたき台）

### 6.1 ToolPose

候補フィールド:

- `position: Point3D<T>`
- `tool_axis: Vector3D<T>` または `Direction3D<T>` 相当
- `rotary_axes: Option<RotaryAxisState<T>>`
- `machine_frame: Option<String>`

### 6.2 RotaryAxisState

候補フィールド:

- `a_axis_deg: Option<T>`
- `b_axis_deg: Option<T>`
- `c_axis_deg: Option<T>`
- `rewind_required: bool`

### 6.3 PoseInterpolationPolicy

候補:

- `FixedOrientation`（3+2）
- `ShortestAngularPath`
- `ContinuousPreferred`
- `MachineConstrained`

### 6.4 MachineConstraint

候補フィールド:

- 軸ごとの `min_deg` / `max_deg`
- `max_velocity_deg_per_sec`
- `max_acceleration_deg_per_sec2`
- 特異点回避フラグ

## 7. 代表ケース（準備段階）

- 3軸
  - 工具姿勢は常に +Z / -Z に固定されるケース
- 3+2
  - セグメント内姿勢固定、セグメント間でのみ姿勢変更
- 同時5軸
  - start/end pose が異なり、補間ポリシーが必要なケース

## 8. 影響ファイル候補

- `model/cam_core/src/toolpath.rs`
- `model/cam_core/src/lib.rs`
- `model/cam_core/src/toolpath_tests.rs`
- 将来フェーズ:
  - `model/cam_core/src/artifact_binary.rs`
  - `model/cam_sim/src/**`

## 9. 実装前に詰める論点

- 姿勢表現を方向ベクトルにするか、回転軸角中心にするか
- 3軸 API と 5軸 API を同一型で持つか、別型へ分離するか
- 3+2 を `FixedOrientation` の特殊ケースとして扱うか
- Gコード互換評価に必要な最小情報をどこまで中間モデルに持つか
- 機械制約を `ToolPath` 側に持つか、post 前の別コンテキストに持つか

## 10. 初回完了条件

- [ ] 現行 `ToolPath` の拡張ポイントが整理される
- [ ] `ToolPose` / 姿勢補間 / 機械制約の API 草案がある
- [ ] 3軸 / 3+2 / 同時5軸の3ケース表現例が揃う
- [ ] 実装フェーズへ移れる候補ファイルと変更順が整理される

## 11. 次アクション

1. `ToolPose` の最小フィールド集合を確定する
2. 選択肢A/B/Cの比較を絞り込み、採択案を決める
3. 3ケースの具体例を文書へ追加する
4. 実装開始前にユーザー承認を得る
