# RedRing

**📖🇯🇵 [日本語版 README はこちら](README.jp.md)**

[![Version](https://img.shields.io/github/v/release/RedRing2020/RedRing?include_prereleases&label=release&color=blue)](https://github.com/RedRing2020/RedRing/releases)
[![Rust](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FRedRing2020%2FRedRing%2Fdevelop%2Frust-toolchain.toml&query=%24.toolchain.channel&label=rust&logo=rust&color=orange)](rust-toolchain.toml)
[![Last Commit](https://img.shields.io/github/last-commit/RedRing2020/RedRing/develop)](https://github.com/RedRing2020/RedRing/commits/develop)
[![Commit Activity](https://img.shields.io/github/commit-activity/m/RedRing2020/RedRing)](https://github.com/RedRing2020/RedRing/graphs/commit-activity)
![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-green)

RedRing is a standalone CAD/CAM application built with Rust, wgpu and egui, developed as a research project.

## Status

The latest release, v0.2.0, is a preview: import an STL or STEP shape, set up machining operations with tools from a tool library, compute 3-axis scanline toolpaths with a ball or flat end mill in the job manager, and check them with a cutting simulation that cuts from the stock left by the previous operation. Save everything in a model file. NC output is not available yet, so the toolpaths cannot be used for real machining.

Planned releases:

- v0.3.0 (in development): long operations in the background, with progress and cancel
- v0.4.0: connecting moves and NC output
- v0.5.0: picking faces to set the machining area and coordinates

See [CHANGELOG.md](CHANGELOG.md) for the changes and the [known issues](https://redring2020.github.io/RedRing/en/known_issues.html) for the current limits.

## Quick Start

On Windows, download the package from [GitHub Releases](https://github.com/RedRing2020/RedRing/releases) and start `redring.exe`.

To build from source, install [rustup](https://rustup.rs/) and Git, then:

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
git checkout v0.2.0
cargo build --release
cargo run --release -p redring
```

`cargo build --release` also builds the job manager and the runner, which compute toolpaths and cutting simulations. See [Getting Started](https://redring2020.github.io/RedRing/en/getting_started.html) for requirements, checking the package and the job manager.

## Documentation

- [User manual](https://redring2020.github.io/RedRing/en/) ([Japanese](https://redring2020.github.io/RedRing/)) for the latest release: getting started, [screen and controls](https://redring2020.github.io/RedRing/en/screen_and_controls.html), [cutting simulation](https://redring2020.github.io/RedRing/en/cutting_simulation_demo.html). The manual for the version in development is in [manual_en/src/](manual_en/src/)
- [Contributing](CONTRIBUTING.md): development setup, checks before a push, pull requests
- [Design documents](dev/architecture/) (Japanese): [architecture](dev/architecture/ARCHITECTURE.md) and the design of each part
- [Issues](https://github.com/RedRing2020/RedRing/issues): bug reports, feature requests and progress

## License

RedRing is dual-licensed under the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE), at your option. See [LICENSE](LICENSE).
