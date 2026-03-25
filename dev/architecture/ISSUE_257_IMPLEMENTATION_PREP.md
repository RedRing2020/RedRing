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
- 機械軸制約（回転軸/直動軸の min/max、速度、加速度）
- 特異点近傍や軸反転の扱い方針
- 3+2 と同時5軸を区別する表現
- 3軸 / 4軸 / 5軸以上を明示する構成分類メタ
- indexed / continuous を明示する運動モードメタ
- レーザー加工のような変則軸構成（例: A/C, U/W）を吸収する機械依存情報

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
- 3軸ケースでは pose レイヤーを必須化せず、将来の binary 追加時も軽量表現を維持しやすい

検討の進め方:

- まず選択肢Bで API 草案と代表ケースを具体化する
- その上で、A/C にしか解決できない問題が出るかを比較して判断する

## 6. API 草案（たたき台）

### 6.1 ToolPose

候補フィールド:

- `position: Point3D<T>`
- `process_axis: Vector3D<T>` または `Direction3D<T>` 相当
- `machine_axes: Option<Vec<MachineAxisValue<T>>>`
- `machine_frame: Option<String>`

### 6.1.1 最小フィールド集合（B案の基準）

選択肢Bを前提にした初回の最小集合は以下とする。

- `position: Point3D<T>`
  - 現行 `PathSegment` の位置幾何と突き合わせる基準点
- `process_axis: Vector3D<T>` 相当
  - 工具軸方向またはレーザーヘッド方向を表す。3軸では固定方向、5軸では姿勢差分を表す
- `machine_axes: Option<Vec<MachineAxisValue<T>>>`
  - A/B/C のような回転軸だけでなく、U/V/W のような補助軸も軸名付きで保持する

初回は以下を `ToolPose` へ持ち込まない。

- `machine_frame`
  - 加工機依存の文脈が強く、pose 本体より上位コンテキストで保持した方が整理しやすい
- 速度・加速度制約
  - `MachineConstraint` 側へ分離する

### 6.1.2 B案の具体API草案

```rust
ToolPose<T> {
    position: Point3D<T>,
  process_axis: Vector3D<T>,
  machine_axes: Option<Vec<MachineAxisValue<T>>>,
}

ToolPathKinematicMeta {
  configuration_class: MachineConfigurationClass,
  kinematic_mode: KinematicMode,
  pose_data_policy: PoseDataPolicy,
}

ToolPoseSpan<T> {
    start_pose: ToolPose<T>,
    end_pose: ToolPose<T>,
    interpolation_policy: PoseInterpolationPolicy,
}

PoseAnnotatedSegment<T> {
    segment: PathSegment<T>,
    pose_span: ToolPoseSpan<T>,
}

MachineAxisValue<T> {
  axis_name: String,
  axis_kind: MachineAxisKind,
  value: T,
}
```

意図:

- `PathSegment` 自体は 3軸互換のまま維持する
- 5軸が必要な箇所だけ `PoseAnnotatedSegment` を使う
- 3+2 と同時5軸の差分は `pose_span` の start/end と補間ポリシーで吸収する
- 回転軸名を固定せず、レーザー加工の変則軸構成も `machine_axes` で吸収する
- 3軸 / 4軸 / 5軸以上の区分は `ToolPose` ではなく `ToolPathKinematicMeta` 側で明示する
- 3軸の軽量維持は `pose_data_policy` で「位置中心データのみで成立する」ことを宣言する

### 6.2 ToolPathKinematicMeta

候補フィールド:

- `configuration_class: MachineConfigurationClass`
- `kinematic_mode: KinematicMode`
- `pose_data_policy: PoseDataPolicy`

`MachineConfigurationClass` 候補:

- `TwoAxis`（PR-5 追加）
- `ThreeAxis`
- `FourAxis`
- `FiveAxisOrMore`

`KinematicMode` 候補:

- `PureTwoAxis`（PR-5 追加） — 純2軸（1平面内の輪郭加工）
- `TwoPointFiveAxis`（PR-5 追加） — 2.5軸（Z段付き2軸）
- `PureThreeAxis`
- `IndexedMultiAxis`
- `ContinuousFourAxis`
- `ContinuousFiveAxis`

`PoseDataPolicy` 候補:

- `PositionOnlyCompatible`
- `PoseLayerOptional`
- `PoseLayerRequired`

注記:

- `configuration_class` は機械構成の大分類を示す
- `kinematic_mode` は同じ5軸以上でも 3+2 と同時多軸を区別する
- `pose_data_policy` はデータ量要件の表明であり、3軸では `PositionOnlyCompatible` を基本にする
- このメタは `ToolPath` 全体、または `ToolPath` に付随する上位コンテキストへ持たせる想定とし、各 pose に重複保持しない
- `TwoAxis` は XY 平面など1平面内の加工機構成を表し、Z軸を持つ `ThreeAxis` と明示的に区別する
- `PureTwoAxis` と `TwoPointFiveAxis` の差は「Z軸が固定か段階変化か」であり、両方とも `PositionOnlyCompatible` で運用できる

### 6.3 MachineAxisValue / MachineAxisKind

候補フィールド:

- `axis_name: String`
- `axis_kind: MachineAxisKind`
- `value: T`

`MachineAxisKind` 候補:

- `Linear`
- `Rotary`

注記:

- `A/B/C` 固定ではなく、`U/V/W` を含む任意軸名を扱えるようにする
- 巻き戻し（rewind）は軸値自体ではなく、補間/機械制約評価の結果として扱う方針を優先する

### 6.4 PoseInterpolationPolicy

候補:

- `FixedOrientation`（3+2）
- `ShortestAngularPath`
- `ContinuousPreferred`
- `MachineConstrained`

### 6.5 MachineConstraint

候補フィールド:

- 軸ごとの `min_value` / `max_value`
- `max_velocity_per_sec`
- `max_acceleration_per_sec2`
- 特異点回避フラグ

注記:

- 回転軸では度、直動軸では長さ単位を使うため、単位は軸定義側で持つ想定とする

## 7. 代表ケース（準備段階）

- 3軸
  - 工具姿勢は常に +Z / -Z に固定されるケース
- 3+2
  - セグメント内姿勢固定、セグメント間でのみ姿勢変更
- 同時5軸
  - start/end pose が異なり、補間ポリシーが必要なケース

### 7.1 3軸ケース

```rust
PoseAnnotatedSegment {
  segment: PathSegment::new_line(...),
  pose_span: ToolPoseSpan {
    start_pose: ToolPose { position: P0, process_axis: (0, 0, -1), machine_axes: None },
    end_pose: ToolPose { position: P1, process_axis: (0, 0, -1), machine_axes: None },
    interpolation_policy: FixedOrientation,
  },
}
```

解釈:

- 位置は移動するが姿勢は固定
- 現行3軸 `ToolPath` とほぼ同義の表現になる
- `ToolPathKinematicMeta { configuration_class: ThreeAxis, kinematic_mode: PureThreeAxis, pose_data_policy: PositionOnlyCompatible }` を付けることで、binary では pose レイヤー省略可能と判断できる

### 7.2 4軸ケース

```rust
PoseAnnotatedSegment {
  segment: PathSegment::new_line(...),
  pose_span: ToolPoseSpan {
    start_pose: ToolPose { position: P0, process_axis: Axis0, machine_axes: Some([C=10]) },
    end_pose: ToolPose { position: P1, process_axis: Axis1, machine_axes: Some([C=40]) },
    interpolation_policy: ContinuousPreferred,
  },
}
```

解釈:

- 回転軸が1本だけ連続変化する 4軸ケースを表す
- `ToolPathKinematicMeta { configuration_class: FourAxis, kinematic_mode: ContinuousFourAxis, pose_data_policy: PoseLayerOptional }` により、3軸との差分を明示できる

### 7.3 3+2 ケース

```rust
PoseAnnotatedSegment {
  segment: PathSegment::new_line(...),
  pose_span: ToolPoseSpan {
    start_pose: ToolPose { position: P0, process_axis: TiltA, machine_axes: Some([A=30, C=90]) },
    end_pose: ToolPose { position: P1, process_axis: TiltA, machine_axes: Some([A=30, C=90]) },
    interpolation_policy: FixedOrientation,
  },
}
```

解釈:

- セグメント内は姿勢固定
- 次セグメントへ移る前後でのみ角度変更が起きる
- `ToolPathKinematicMeta { configuration_class: FiveAxisOrMore, kinematic_mode: IndexedMultiAxis, pose_data_policy: PoseLayerOptional }` を付けると、同時5軸と混同しない

### 7.4 同時5軸ケース

```rust
PoseAnnotatedSegment {
  segment: PathSegment::new_arc(...),
  pose_span: ToolPoseSpan {
    start_pose: ToolPose { position: P0, process_axis: Axis0, machine_axes: Some([B=10, C=20]) },
    end_pose: ToolPose { position: P1, process_axis: Axis1, machine_axes: Some([B=35, C=70]) },
    interpolation_policy: MachineConstrained,
  },
}
```

解釈:

- セグメント内で位置と姿勢が同時に変化する
- 最短角だけでなく機械制約を考慮した補間が必要になる
- `ToolPathKinematicMeta { configuration_class: FiveAxisOrMore, kinematic_mode: ContinuousFiveAxis, pose_data_policy: PoseLayerRequired }` により、姿勢レイヤー必須のケースだと表明できる

### 7.5 レーザー加工の変則軸ケース

```rust
PoseAnnotatedSegment {
  segment: PathSegment::new_line(...),
  pose_span: ToolPoseSpan {
    start_pose: ToolPose {
      position: P0,
      process_axis: Beam0,
      machine_axes: Some([
        A=15,
        C=120,
        U=25,
        W=5,
      ]),
    },
    end_pose: ToolPose {
      position: P1,
      process_axis: Beam1,
      machine_axes: Some([
        A=20,
        C=135,
        U=40,
        W=8,
      ]),
    },
    interpolation_policy: MachineConstrained,
  },
}
```

解釈:

- `process_axis` は工具軸ではなくビーム方向として読める
- `machine_axes` は A/C の回転軸と U/W の補助軸を同一枠で保持する
- これにより、フライス前提の固定軸名に縛られない中間表現を維持できる
- `ToolPathKinematicMeta` を併記すれば、レーザーでも 4軸相当か 5軸以上かを分類できる

## 8. 影響ファイル候補

- `model/cam_core/src/toolpath.rs`
- `model/cam_core/src/lib.rs`
- `model/cam_core/src/toolpath_tests.rs`
- 将来フェーズ:
  - `model/cam_core/src/artifact_binary.rs`
  - `model/cam_sim/src/**`

## 9. 実装前に詰める論点

- 姿勢表現を `process_axis` 中心にするか、機械軸角中心にするか
- 3軸 API と 5軸 API を同一型で持つか、別型へ分離するか
- 3+2 を `FixedOrientation` の特殊ケースとして扱うか
- Gコード互換評価に必要な最小情報をどこまで中間モデルに持つか
- 機械制約を `ToolPath` 側に持つか、post 前の別コンテキストに持つか
- レーザー加工のような非フライス系プロセスでも `process_axis` で十分抽象化できるか
- `ToolPathKinematicMeta` を `ToolPath` 本体へ持つか、artifact/ジョブ文脈へ持つか

## 10. 初回完了条件

- [x] 現行 `ToolPath` の拡張ポイントが整理される
- [x] `ToolPose` / 姿勢補間 / 機械制約の API 草案がある
- [x] 3軸 / 4軸 / 3+2 / 同時5軸のケース表現例が揃う
- [x] 3軸軽量維持と軸分類メタの方針が整理される
- [x] 実装フェーズへ移れる候補ファイルと変更順が整理される

## 11. 次アクション

1. 選択肢Bを採択案として維持できるか、A/C にしか解けない論点が残るか確認する
2. `ToolPathKinematicMeta` と `MachineConstraint` を `ToolPath` 本体に持つか、別コンテキストで持つか切り分ける
3. 実装開始前にユーザー承認を得る

## 12. 実装フェーズ分割（PR分離案）

### 12.1 PR-1: `cam_core` 型追加（最小）

目的:

- Option B の中核型を `cam_core` に最小追加し、既存3軸 API を壊さないことを確認する

対象ファイル:

- `model/cam_core/src/toolpath.rs`
- `model/cam_core/src/lib.rs`
- `model/cam_core/src/toolpath_tests.rs`

実装範囲:

- `ToolPose<T>` / `ToolPoseSpan<T>` / `PoseAnnotatedSegment<T>` 追加
- `MachineAxisValue<T>` / `MachineAxisKind` 追加
- `ToolPathKinematicMeta` / `MachineConfigurationClass` / `KinematicMode` / `PoseDataPolicy` 追加
- 3軸既存 API への破壊的変更を行わない

受け入れ条件:

- 既存 `ToolPath` 利用コードが修正なしでビルドできる
- 新規型の基本生成と比較をテストで確認できる

### 12.2 PR-2: 3軸軽量方針のテスト固定

目的:

- 3軸ケースで pose レイヤーが必須でないことをテストで固定し、将来拡張時の肥大化を防ぐ

対象ファイル:

- `model/cam_core/src/toolpath_tests.rs`

実装範囲:

- `PoseDataPolicy::PositionOnlyCompatible` の期待動作テスト追加
- 3軸 / 4軸 / 3+2 / 同時5軸のメタ分類テーブルテスト追加

受け入れ条件:

- 分類メタの誤設定をテストで検出できる
- 3軸ケースの軽量運用（pose optional）を明示的に検証できる

### 12.3 PR-3: artifact 連携方針（別フェーズ設計）

目的:

- #300 v0.1 を維持したまま、将来の multi-axis wire format 拡張方針を整理する

対象ファイル:

- `model/cam_core/src/artifact_binary.rs`
- `dev/architecture/ISSUE_257_IMPLEMENTATION_PREP.md`

実装範囲:

- 初回は実装せず、互換ポリシーと version 戦略のみ文書化する
- `v0.1` 読み書き互換を壊さない前提を固定する

受け入れ条件:

- #300/#411 の既存契約に反しない移行案になっている

PR-3 で確定した方針:

- 現行 artifact は `v0.1`（`version_major=0`, `version_minor=1`）を唯一の安定 reader/writer 対象として維持する
- multi-axis 向け拡張は **新しい minor/major の追加** で扱い、`v0.1` の wire layout は改変しない
- 3軸データは引き続き `PositionOnlyCompatible` な payload を前提にし、pose レイヤーの必須化を行わない
- 3軸以外の姿勢情報は将来の拡張 payload に閉じ込め、`v0.1` reader が誤読しないよう `version` で分岐させる
- `read_toolpath_artifact_v1` / `write_toolpath_payload_v1` の意味は「API世代名」であり、wire format 実値はヘッダの version で判定する

非目標（PR-3時点）:

- `artifact_binary` への新payload実装追加
- `v0.2` / `v1.0` の wire layout 確定
- `cam_sim` 側の multi-axis 読み取り対応

将来移行の最小手順（方針）:

1. 新版ヘッダ version（例: `0.2` もしくは `1.0`）を導入
2. 新payload reader/writer を追加し、`v0.1` との共存を保証
3. `compatibility_decision` に新版判定を追加
4. 3軸運用の既存ジョブが `v0.1` のまま通ることを回帰テストで固定

### 12.4 推奨実施順

1. PR-1（型追加）
2. PR-2（テスト固定）
3. PR-3（artifact 拡張設計）
4. PR-4（`ToolPath` 本体への運動学メタ搭載）

注記:

- 実装開始時はこの順序で小さく分割し、各PRは `Refs #257` を使用する
- 最終的に #257 を閉じるPRのみ `Closes #257` を使用する

### 12.5 進捗ステータス（2026-03-25）

- [x] PR-1: `cam_core` 型追加（最小）
- [x] PR-2: 3軸軽量方針のテスト固定
- [x] PR-3: artifact 連携方針の文書化（実装変更なし）
- [x] PR-4: `ToolPath` 本体へ `kinematic_meta` を追加（既定は3軸互換）
- [x] PR-5: `MachineConfigurationClass`/`KinematicMode` に2軸・2.5軸バリアントを追加

### 12.6 PR-4 実施内容

目的:

- 未解決論点だった「`ToolPathKinematicMeta` の保持場所」を `ToolPath` 本体に確定する

対象ファイル:

- `model/cam_core/src/toolpath.rs`
- `model/cam_core/src/toolpath_tests.rs`
- `model/cam_core/src/artifact_binary.rs`

実装範囲:

- `ToolPath` に `kinematic_meta: ToolPathKinematicMeta` を追加
- 既存 `ToolPath::new` は `three_axis_position_only()` を既定設定し、既存呼び出し互換を維持
- `ToolPath::new_with_kinematic_meta` を追加し、multi-axis メタの明示指定を可能にする
- `read_toolpath_payload_v1` で `v0.1` 読み取り時に3軸既定メタを設定する

受け入れ条件:

- 既存 `ToolPath::new` 呼び出しは修正なしで利用できる
- 明示メタ指定の `ToolPath` をテストで検証できる
- `v0.1` 読み取りが3軸互換メタで初期化される

### 12.7 PR-5 実施内容

目的:

- 2軸（XY平面加工）および 2.5軸（Z段付き2軸）を `ToolPathKinematicMeta` で明示区別できるようにする
- 既存の3軸以上のバリアントとの整合を保つ

対象ファイル:

- `model/cam_core/src/toolpath.rs`
- `model/cam_core/src/toolpath_tests.rs`

実装範囲:

- `MachineConfigurationClass` に `TwoAxis` バリアントを追加
- `KinematicMode` に `PureTwoAxis` / `TwoPointFiveAxis` バリアントを追加
- `ToolPathKinematicMeta::two_axis_position_only()` convenience fn を追加
- `ToolPathKinematicMeta::two_point_five_axis_position_only()` convenience fn を追加
- `test_two_axis_convenience_fns` テストを追加
- `test_toolpath_kinematic_meta_matrix` に 2軸 / 2.5軸ケースを追加

受け入れ条件:

- 既存バリアント（`ThreeAxis`, `PureThreeAxis` 等）の挙動に変更なし
- 2軸・2.5軸どちらも `is_position_only_compatible()` が `true` を返す
- `cargo test -p cam_core --lib`: 64 passed（PR-4 の 63 から +1）
