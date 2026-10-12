# Cargo コマンド・デバッグ

## 最終更新日: 2026年10月12日

RedRing の開発で使う Cargo コマンド・チェックスクリプト・デバッグの手順を定義します。

---

## チェックスクリプト（push・PR の前）

```powershell
# push・PR の前に必ず実行する（fmt / clippy / テスト / 実行時文言の言語ポリシー / GitHub 固有情報 / 呼び出し元のない API）
pwsh -NoProfile -File .\scripts\check_all_before_push.ps1

# アーキテクチャの依存関係（CI と同じ）
.\scripts\check_architecture_dependencies.ps1
```

- コミット時は pre-commit フックがフォーマットを確認する。フック・署名を省略するオプション（`--no-verify` 等）は使わない

---

## Cargo 基本コマンド

ルートの `Cargo.toml` はワークスペースだけを定義する（パッケージを持たない）ため、実行・個別のビルドでは `-p` でパッケージを指定する。

### ビルド・実行

```bash
cargo build --workspace
cargo build -p geo_primitives

# メインアプリケーション（view/app、パッケージ名 redring）
cargo run -p redring
cargo run -p redring --release
```

### テスト

```bash
cargo test --workspace
cargo test -p geo_primitives

# 形状のテスト（{shape}/tests.rs のモジュールのパスで絞り込む）
cargo test -p geo_primitives line_segment_3d::tests

# 特定のテスト関数
cargo test -p geo_primitives test_basic_creation

# 形状をまたぐ結合テスト
cargo test -p geo_primitives --test cross_shape

# 出力を表示・直列に実行
cargo test -p geo_primitives -- --nocapture --test-threads=1
```

### ドキュメント

```bash
cargo doc --no-deps --open

# mdbook（manual/ → docs/）
mdbook build
mdbook serve --open
```

---

## 依存関係

```bash
cargo tree -p geo_primitives
cargo tree -p geo_primitives --depth 1

# 逆依存（どのクレートが使っているか）
cargo tree -i geo_core

# 重複した依存
cargo tree --duplicates

cargo update -p wgpu
```

---

## コード品質

```bash
cargo fmt --all
cargo fmt --all -- --check

# CI と同じ条件（警告をエラーとして扱う）
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 使っていない import 等の機械的な修正
cargo fix --workspace --all-targets --allow-dirty
```

- ワークスペースの lint はルートの `Cargo.toml` の `[workspace.lints]` で設定し、各クレートは `[lints] workspace = true` で使う（`mod.rs` を禁止する `mod_module_files` 等）

---

## デバッグ・トレース

ログは `tracing` を使い、`logging_foundation` が `RUST_LOG` 環境変数でフィルタを設定する。

```rust
use tracing::{debug, error, info, trace, warn};
```

```powershell
$env:RUST_LOG = "debug"
cargo run -p redring

# クレート別
$env:RUST_LOG = "geo_algorithms=debug,cam_sim=trace"
```

```bash
RUST_LOG=debug cargo run -p redring
RUST_BACKTRACE=1 cargo test -p geo_primitives
```

---

## CI（GitHub Actions）で実行されるコマンド

`develop` 向け（`develop_ci.yml`）:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo build --workspace --verbose
cargo test --workspace --verbose
```

アーキテクチャ（`common_architecture_check.yml`）:

```powershell
./scripts/check_architecture_dependencies.ps1
./scripts/check_github_refs_in_source.ps1
```

---

## トラブルシューティング

```bash
# キャッシュを消して再ビルド
cargo clean
cargo build --workspace

# 詳細なビルドログ
cargo build -p geo_primitives -vv
```

- `Cargo.lock` はリポジトリで管理しているため、削除しない

---

## 参照文書

- **Git/PR の運用**: `.github/skills/git-workflow/skill.md`、`dev/GIT_PR_WORKFLOW_OPERATION.md`
- **CI 設定**: `.github/workflows/`
- **Cargo Book**: https://doc.rust-lang.org/cargo/
