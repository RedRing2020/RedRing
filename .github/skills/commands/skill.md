# Cargo コマンド・デバッグ

## 最終更新日: 2026年2月13日

RedRing開発で頻繁に使用するCargoコマンドとデバッグ手法を定義します。

---

## Cargo 基本コマンド

### ビルド

```bash
# 全体ビルド（デバッグモード）
cargo build

# リリースビルド（最適化あり）
cargo build --release

# 個別クレートのビルド
cargo build -p geo_primitives
cargo build -p render

# クリーンビルド（依存関係も再ビルド）
cargo clean
cargo build
```

### テスト

```bash
# 全体テスト実行
cargo test --workspace

# 個別クレートのテスト
cargo test -p geo_primitives
cargo test -p geo_algorithms

# 特定のテストモジュール実行
cargo test -p geo_primitives line_segment_3d_tests

# 特定のテスト関数実行
cargo test -p geo_primitives test_new_valid

# テスト出力を全て表示
cargo test -- --nocapture

# 並列実行を無効化（デバッグ時）
cargo test -- --test-threads=1
```

### 実行

```bash
# メインアプリケーション実行
cargo run

# リリースモードで実行
cargo run --release

# 個別クレートの実行（bin ターゲットがある場合）
cargo run -p app
```

### ドキュメント

```bash
# Rustdoc ドキュメント生成
cargo doc --no-deps --open

# mdbook ドキュメント生成
mdbook build  # manual/ -> docs/

# mdbook サーバー起動（ライブリロード）
mdbook serve --open
```

---

## 依存関係管理

### 依存関係の確認

```bash
# クレートの依存関係ツリー
cargo tree

# 深さ1で表示（直接依存のみ）
cargo tree --depth 1

# 特定クレートの依存関係
cargo tree -p geo_primitives

# 逆依存（どのクレートが使用しているか）
cargo tree -i geo_foundation

# 重複依存の検出
cargo tree --duplicates
```

### 依存関係の更新

```bash
# Cargo.lock の更新
cargo update

# 特定クレートのみ更新
cargo update -p wgpu
```

---

## コード品質チェック

### フォーマット

```bash
# コードフォーマット実行
cargo fmt --all

# フォーマットチェック（CI用）
cargo fmt --all -- --check

# 特定クレートのみ
cargo fmt -p geo_primitives
```

### Clippy（リント）

```bash
# 警告レベルでチェック
cargo clippy

# エラーとして扱う（CI用）
cargo clippy -- -D warnings

# 全ワークスペースをチェック
cargo clippy --workspace

# 修正案を自動適用
cargo clippy --fix
```

### アーキテクチャチェック

```powershell
# RedRing独自のアーキテクチャルール検証
.\scripts\check_architecture_dependencies_simple.ps1

# MVVM アーキテクチャ検証
.\scripts\check_mvvm_architecture.ps1
```

---

## デバッグ・トレース

### tracing の使用

```rust
use tracing::{debug, error, info, trace, warn};

fn example_function() {
    trace!("詳細なトレース情報");
    debug!("デバッグ情報");
    info!("一般情報");
    warn!("警告");
    error!("エラー");
}
```

### ログレベルの制御

```bash
# 環境変数で制御
export RUST_LOG=debug
cargo run

# Windows PowerShell
$env:RUST_LOG="debug"
cargo run

# クレート別レベル設定
export RUST_LOG=geo_primitives=trace,geo_algorithms=debug
cargo run

# モジュール別レベル設定
export RUST_LOG=geo_primitives::line_segment_3d=trace
cargo run
```

### ログレベル一覧

- `trace`: 最も詳細（全ての実行経路）
- `debug`: デバッグ情報
- `info`: 一般的な情報（デフォルト）
- `warn`: 警告
- `error`: エラーのみ

---

## パフォーマンス分析

### ベンチマーク

```bash
# ベンチマーク実行
cargo bench

# 特定ベンチマークのみ
cargo bench --bench line_segment_benchmark
```

### プロファイリング

```bash
# release ビルドでプロファイリング
cargo build --release
cargo run --release

# flamegraph 生成（要cargo-flamegraph）
cargo flamegraph
```

---

## トラブルシューティング

### ビルドエラー

```bash
# キャッシュクリア
cargo clean

# 依存関係を再取得
rm -rf Cargo.lock
cargo build

# verbose モードでビルド
cargo build -vv
```

### テストエラー

```bash
# テスト失敗時の詳細表示
cargo test -- --nocapture --test-threads=1

# バックトレース有効化
RUST_BACKTRACE=1 cargo test

# 完全なバックトレース
RUST_BACKTRACE=full cargo test
```

### 型エラー

```bash
# 詳細な型情報を表示
cargo check --verbose

# マクロ展開結果の確認
cargo expand -p geo_primitives
```

---

## CI/CD コマンド

### GitHub Actions で実行されるコマンド

```bash
# フォーマットチェック
cargo fmt --all -- --check

# Clippy チェック
cargo clippy --workspace -- -D warnings

# 全テスト実行
cargo test --workspace

# ビルド確認
cargo build --release

# アーキテクチャチェック
.\scripts\check_architecture_dependencies_simple.ps1
```

---

## 便利なエイリアス設定

### .bashrc / .zshrc に追加

```bash
# よく使うコマンドのエイリアス
alias cb='cargo build'
alias ct='cargo test --workspace'
alias cr='cargo run'
alias cc='cargo clippy -- -D warnings'
alias cf='cargo fmt --all'
alias ccheck='cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace'
```

### PowerShell プロファイルに追加

```powershell
# $PROFILE に追加
function cb { cargo build }
function ct { cargo test --workspace }
function cr { cargo run }
function cc { cargo clippy -- -D warnings }
function cf { cargo fmt --all }
function ccheck {
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings
    cargo test --workspace
    .\scripts\check_architecture_dependencies_simple.ps1
}
```

---

## 参照文書

- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **tracing ドキュメント**: https://docs.rs/tracing/
- **CI設定**: `.github/workflows/`
