# RedRing への貢献の手引き

**🇺🇸 [English version is here](CONTRIBUTING.md)**

不具合の報告、機能の提案、プルリクエストを歓迎します。先に [Issues](https://github.com/RedRing2020/RedRing/issues) を確かめてください。

## 開発の準備

- Rust: [rustup](https://rustup.rs/) を入れます。ビルドするときに、rustup が `rust-toolchain.toml` で決めた版を使います
- Git
- Windows: Visual Studio Build Tools（または Visual Studio Community）
- macOS: Xcode Command Line Tools（`xcode-select --install`）
- Linux: `sudo apt install build-essential pkg-config libx11-dev` と Vulkan のドライバー

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
cargo build              # ジョブマネージャーとランナーも一緒にビルドする
cargo run -p redring     # GUI の環境が要る
cargo test --workspace
```

ツールパスと切削シミュレーションを試すときは `--release` を付けてビルドします。付けないと、計算に何倍も時間がかかります。

GPU が見つからないときは、環境変数 `WGPU_BACKEND`（例えば `vulkan`・`dx12`）で描画の仕組みを選びます。

## コミットの前のフック

フックは `git commit` のたびにフォーマットを確かめ、`cargo fmt` で変わるファイルがあればコミットを止めます。クローンごとに 1 回入れます。

```bash
# Windows (PowerShell)
pwsh scripts/setup_hooks.ps1

# macOS/Linux
cp scripts/hooks/pre-commit.template .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

コミットが止まったら、`cargo fmt --all` を実行し、変更をステージしてからコミットし直します。

## push の前の検査

push の前に、毎回すべての検査を実行します（Windows、PowerShell）。

```powershell
pwsh scripts/check_all_before_push.ps1
```

フォーマット、Clippy（`-D warnings`）、release の設定での確認、テスト、このリポジトリの検査（メッセージの言語、ソースの中の GitHub の情報、使われていない公開 API）を実行します。CI もプルリクエストごとにフォーマット・Clippy・テストを確かめるので、先にスクリプトで確かめておくと CI の失敗を避けられます。

## プルリクエスト

- `develop` から `feature/<題>` のブランチを切り（修正・文書も同じ）、`develop` に向けてプルリクエストを作ります
- 新しい機能にはテストを、公開 API には rustdoc のコメントを付けます
- プルリクエストは小さく、1 つの変更にします

詳しい決まりは [dev/GIT_PR_WORKFLOW_OPERATION.md](dev/GIT_PR_WORKFLOW_OPERATION.md) と [dev/ISSUE_LABEL_OPERATION.md](dev/ISSUE_LABEL_OPERATION.md) に、設計の判断は [dev/architecture/](dev/architecture/) にあります。

## マニュアルのビルド

マニュアルは [mdBook](https://rust-lang.github.io/mdBook/) でビルドします（CI は 0.5.4 を使います）。

```bash
cargo install mdbook --version 0.5.4
mdbook build            # 日本語のマニュアル: manual/ -> target/book/
mdbook build manual_en  # 英語のマニュアル: manual_en/src/ -> target/book_en/
mdbook serve            # 日本語のマニュアルを手元で確かめる
```
