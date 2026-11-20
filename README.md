# RedRing

**🇯🇵 [日本語版 README はこちら](README.jp.md)**

![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/RedRing2020/RedRing?utm_source=oss&utm_medium=github&utm_campaign=RedRing2020%2FRedRing&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

RedRing is a standalone CAD/CAM application built with Rust, wgpu and egui, developed as a research project.

## Status

The latest release, v0.1.3, is a preview: import an STL, compute a 3-axis scanline toolpath with a ball or flat end mill, and check it with a cutting simulation. NC output is not available yet, so the toolpaths cannot be used for real machining.

Planned releases:

- v0.2.0 (in development): machining management (operations, jobs, tool library) and machining STEP shapes
- v0.3.0: long operations in the background, with progress and cancel
- v0.4.0: connecting moves and NC output
- v0.5.0: picking faces to set the machining area and coordinates

See [CHANGELOG.md](CHANGELOG.md) for the changes and the [known issues](manual_en/src/known_issues.md) for the current limits.

## Quick Start

On Windows, download the package from [GitHub Releases](https://github.com/RedRing2020/RedRing/releases) and start `redring.exe`.

To build from source, install [rustup](https://rustup.rs/) and Git, then:

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
cargo build --release
cargo run --release -p redring
```

`cargo build --release` also builds the job manager and the runner, which compute toolpaths and cutting simulations. See [Getting Started](manual_en/src/getting_started.md) for requirements, checking the package and the job manager.

## Documentation

- [User manual](https://redring2020.github.io/RedRing/en/) ([Japanese](https://redring2020.github.io/RedRing/)): getting started, [screen and controls](manual_en/src/screen_and_controls.md), [cutting simulation](manual_en/src/cutting_simulation_demo.md)
- [Contributing](CONTRIBUTING.md): development setup, checks before a push, pull requests
- [Design documents](dev/architecture/) (Japanese): [architecture](dev/architecture/ARCHITECTURE.md) and the design of each part
- [Issues](https://github.com/RedRing2020/RedRing/issues): bug reports, feature requests and progress

## License

RedRing is dual-licensed under the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE), at your option. See [LICENSE](LICENSE).
