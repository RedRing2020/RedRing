# Contributing to RedRing

**🇯🇵 [日本語版はこちら](CONTRIBUTING.jp.md)**

Bug reports, feature requests and pull requests are welcome. Please check the [Issues](https://github.com/RedRing2020/RedRing/issues) first.

## Development Setup

- Rust: install [rustup](https://rustup.rs/). rustup uses the version set in `rust-toolchain.toml` when you build
- Git
- Windows: Visual Studio Build Tools (or Visual Studio Community) with the "Desktop development with C++" workload, and [PowerShell 7](https://learn.microsoft.com/powershell/scripting/install/installing-powershell-on-windows) (`pwsh`) for the scripts
- macOS: Xcode Command Line Tools (`xcode-select --install`)
- Linux: `sudo apt install build-essential pkg-config libx11-dev`, and a Vulkan driver

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
cargo build              # also builds the job manager and the runner
cargo run -p redring     # needs a GUI environment
cargo test --workspace
```

Build with `--release` when you try toolpaths and cutting simulations; the debug build computes them many times slower.

If no GPU adapter is found, choose the backend with the environment variable `WGPU_BACKEND` (for example `vulkan` or `dx12`).

## Pre-commit Hook

The hook checks formatting on every `git commit` and rejects the commit if `cargo fmt` would change a file. Set it up once per clone:

```bash
# Windows (PowerShell)
pwsh scripts/setup_hooks.ps1

# macOS/Linux
cp scripts/hooks/pre-commit.template .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

If the commit is rejected, run `cargo fmt --all`, stage the changes and commit again.

## Checks Before a Push

Run the full check before every push (Windows, PowerShell):

```powershell
pwsh scripts/check_all_before_push.ps1
```

It runs formatting, Clippy (`-D warnings`), a release-profile check, the tests and the repository's own checks (message language, GitHub references in source, unused public API). The CI also checks formatting, Clippy and the tests on every pull request, so running the script first saves a failed run.

## Pull Requests

- Branch from `develop` as `feature/<topic>` (also for fixes and documents) and open the pull request against `develop`
- Add tests for new features and rustdoc comments for public APIs
- Keep a pull request small and focused on one change

The detailed rules (in Japanese) are in [dev/GIT_PR_WORKFLOW_OPERATION.md](dev/GIT_PR_WORKFLOW_OPERATION.md) and [dev/ISSUE_LABEL_OPERATION.md](dev/ISSUE_LABEL_OPERATION.md). Design decisions are in [dev/architecture/](dev/architecture/).

## Building the Manual

The manual is built with [mdBook](https://rust-lang.github.io/mdBook/) (the CI uses 0.5.4):

```bash
cargo install mdbook --version 0.5.4
mdbook build            # Japanese manual: manual/ -> target/book/
mdbook build manual_en  # English manual: manual_en/src/ -> target/book_en/
mdbook serve manual_en  # preview the English manual locally
```
