# Issue #320 実施準備チェックリスト（改訂: 2026-03-19）

対象Issue: [#320 geo_contracts正規化: geo_foundation::commons重複解消とgeo_commons責務明確化](https://github.com/RedRing2020/RedRing/issues/320)

## 1. 改訂目的

- 主目的を「`geo_contracts` を trait定義の正規参照先に統一する」へ再定義する
- `geo_commons` の扱いを「廃止前提」ではなく「純粋数値実装の集約先として維持可能」へ更新する
- helper関数の配置ルールを明文化し、将来の `common` 肥大化を防止する

## 2. 事実ベース現状（2026-03-19確認）

- `model/geo_foundation/Cargo.toml` は `geo_commons` に依存していない
- `geo_commons` はワークスペース上で直接依存されていない
- `geo_foundation::commons` と `geo_commons` に重複実装が存在する
- `geo_primitives` に `geo_foundation::commons` 経由の呼び出しが残存する

## 3. アーキテクチャ方針（改訂）

### 3.1 レイヤ責務

- `analysis`
  - 形状意味を持たない汎用数値計算のみ
- `geo_contracts`
  - 幾何形状に関する trait定義のみ
  - 実装本体は置かない
- `geo_commons`
  - 幾何に関する純粋数値計算の実装（shape型非依存）
  - 例: 近似式、面積/体積公式、距離公式
- `geo_core`
  - 幾何基本型と最小限の基盤ユーティリティ
  - shape依存ヘルパーは置かない
- `geo_primitives` / `geo_nurbs`
  - 具象型ごとの trait実装
  - 必要に応じて `geo_commons` を呼び出して実装を構成
- `geo_algorithms`
  - 複数形状を横断するアルゴリズム（衝突判定・交差判定・空間分割）

### 3.2 依存方向

```text
analysis <- geo_contracts
analysis <- geo_commons
geo_contracts + geo_commons <- geo_primitives
geo_contracts + geo_commons <- geo_nurbs
geo_primitives + geo_nurbs + geo_contracts <- geo_algorithms
```

`geo_commons` は依存方向が特殊なクレートとして許容する。  
ただし責務を「shape型非依存の純粋関数」に限定する。

## 4. helper関数 配置ルール

### 4.1 配置判定

- 形状意味を持たない数値処理: `analysis`
- 形状意味を持つが shape型に依存しない純粋関数: `geo_commons`
- trait定義: `geo_contracts`
- 単一クレートの内部都合だけで使う補助関数: そのクレート内 private module
- 複数形状横断の処理本体: `geo_algorithms`

### 4.2 禁止事項

- `geo_contracts` に実装本体を追加しない
- `geo_core` に shape特化 helper を追加しない
- `geo_foundation::commons` に新規の重複実装を追加しない
- `common` 名の汎用フォルダを各クレートへ無秩序に増やさない

## 5. 移行タスク（再定義）

### Phase A: trait定義の正規化

- [x] `geo_foundation::commons::ellipse_calculation_traits` の trait定義を `geo_contracts` へ移設
- [x] `geo_foundation` は互換維持のため `geo_contracts` 再エクスポートへ薄層化
- [x] `cargo check -p geo_contracts -p geo_foundation`

### Phase B: 呼び出し側の経路切替

- [x] `geo_primitives` の `geo_foundation::commons` 参照を `geo_contracts` trait + `geo_commons` 実装呼び出しへ切替
- [x] `geo_nurbs` 側に同種参照があれば同様に切替
- [x] `cargo check -p geo_primitives -p geo_nurbs`
- [x] `cargo test -p geo_primitives -p geo_nurbs`

### Phase C: 重複実装の解消

- [x] `geo_foundation::commons` 内の重複実装を削減（再エクスポート or 廃止）
- [x] `geo_commons` を唯一の実装ソースに統一
- [x] `cargo check --workspace`

### Phase D: #320スコープ確定

- [x] Issue #320 を「geo_commons廃止」から「geo_contracts正規化と重複解消」へ更新するか判断
- [ ] 廃止を継続する場合の代替実装先を明示したうえで別Issue化

## 6. 完了条件

- [ ] 幾何 trait定義の正規参照先が `geo_contracts` に統一される
- [ ] `geo_foundation::commons` の重複実装が解消される
- [ ] `geo_commons` の責務が「shape型非依存の純粋関数」に限定される
- [ ] helper関数の配置ルール違反がない
- [ ] ワークスペース全体の `fmt/check/clippy/test` と依存チェックが通る

## 7. 検証コマンド

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1 -ExitOnError
```
