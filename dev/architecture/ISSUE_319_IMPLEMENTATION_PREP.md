# Issue #319 実施準備チェックリスト

対象Issue: [#319 Transform再編: geo_core共通核 + geo_primitives/geo_nurbs形状実装の2層化](https://github.com/RedRing2020/RedRing/issues/319)

## 1. 目的

- Transformの共通責務を `geo_core` に統一する
- 形状固有のTransform実装を `geo_primitives` / `geo_nurbs` に明確化する
- `geo_foundation` 経由のTransform参照を段階的に除去し、#318の前提を揃える

## 2. 現状調査（Transform実装の依存）

`*_transform.rs` の棚卸し結果:

- `model/geo_primitives/src/*_transform.rs`: 32ファイル
- `model/geo_nurbs/src/*_transform.rs`: 3ファイル
- 総35ファイルすべてで `geo_foundation` 参照あり。
- **細江**: 2026-03-20、geo_foundation 廃止完了によりを上記の参照を `geo_contracts` / `geo_core` へ切替。

代表的な参照パターン:

- `AnalysisTransform2D` / `AnalysisTransform3D`
- `TransformError`
- `AnalysisTransformSupport`

## 3. 実施方針

### 3.1 2層ルール

- 共通ルール（trait定義/エラー/共通変換核）: `geo_core`
- 形状適用（各形状の変換実装）: `geo_primitives` / `geo_nurbs`
- クロス形状演算: `geo_algorithms`（本Issueでは対象外）

### 3.2 変更方針

- 先に import 参照を `geo_core` 基準に切り替える
- 変換アルゴリズム本体は極力維持して、責務境界の整理を優先する
- 破壊的変更は許容するが、1PRあたりの差分は小さく分割する

## 4. 段階タスク（推奨）

### Phase A: NURBS側のTransform切替（小PR）

- [ ] `curve_2d_transform.rs` / `curve_3d_transform.rs` / `surface_3d_transform.rs` の `AnalysisTransform*` と `TransformError` を `geo_core` 参照へ切替
- [ ] `cargo check -p geo_nurbs`
- [ ] `cargo test -p geo_nurbs`

### Phase B: Primitives側のTransform切替（中PR）

- [ ] `model/geo_primitives/src/*_transform.rs`（32ファイル）の `AnalysisTransform*` / `AnalysisTransformSupport` / `TransformError` を `geo_core` 基準へ切替
- [ ] 既存テストの import 置換
- [ ] `cargo check -p geo_primitives`
- [ ] `cargo test -p geo_primitives`

### Phase C: 呼び出し側と公開APIの整理（小PR）

- [ ] `geo_primitives` / `geo_nurbs` の `lib.rs` 再エクスポートを `geo_core` 基準へ更新
- [ ] `README` とサンプルのTransform importを更新
- [ ] 互換コメント/旧経路注記の整理

### Phase D: ワークスペース検証（小PR）

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies.ps1 -ExitOnError`

## 5. スコープ外（本Issueではやらない）

- ✅ `geo_foundation` 全廃（#318で実施、厳枠担観妨）- `geo_commons` 分解移管（#320で実施）
- `geo_algorithms` の責務再編そのもの

## 6. リスクと対策

- リスク: import切替で trait 境界が崩れ、大量コンパイルエラーが発生
  - 対策: Phase Aで3ファイル先行し、切替パターンを固定してからPhase Bへ展開

- リスク: `AnalysisTransformSupport` 実装漏れでAPI互換が壊れる
  - 対策: `impl AnalysisTransformSupport` の存在確認を機械検索で実施

- リスク: READMEやサンプルだけ古い import のまま残る
  - 対策: Phase Cで `rg "geo_foundation::AnalysisTransform|geo_foundation::TransformError"` を最終チェック

## 7. 完了条件（#319）

- [ ] `geo_primitives` / `geo_nurbs` のTransform実装が `geo_core` のTransform trait/エラー参照で統一
- [ ] Transform共通責務が `geo_core` に集約されている
- [ ] ワークスペースの `fmt/check/test` と依存チェックスクリプトが通る

## 8. 調査追記（2026-03-16）

### 8.1 実装現況の確認結果

- `geo_nurbs` / `geo_primitives` の Transform 実装は `geo_core` の `AnalysisTransform*` / `TransformError` 参照へ移行済み
- 検証結果:
  - `cargo check -p geo_nurbs`: pass
  - `cargo test -p geo_nurbs`: pass
  - `cargo check -p geo_primitives`: pass
  - `cargo test -p geo_primitives`: pass
  - `scripts/check_architecture_dependencies.ps1 -ExitOnError`: pass

### 8.2 将来リファクタに向けた共通化候補（`geo_core`）

- 2D/3D の行列生成ヘルパー（translation/rotation/scale）
- 変換前バリデーション（ゼロスケール/ゼロ軸のガード）
- 複合変換の行列組み立て（`apply_composite_transform` の共通核）
- 同次座標変換の安全適用（`w == 0` ガードを含む）
- `TransformError` のメッセージ規約と生成パターン

### 8.3 共通化しない方針（各形状に残す）

- 形状意味に依存する制約（例: 非一様スケール可否）
- 形状固有の再構築ロジック（半径/法線/ノット/次数などの更新）
- 形状固有の妥当性判定と補正

### 8.4 判断ルール（明文化）

- 3形状以上で同一意味・同一挙動・同一エラーモデルが成立するものだけ共通化する
- 上記条件を満たさないものは重複を許容し、形状実装側に残す
- 本Issueでは「全面抽象化」は行わず、「必要最小限の共通化」に限定する
