# RedRing

**🇯🇵 [日本語版 README はこちら](README.jp.md)**

A research platform for CAD/CAM development built with Rust + wgpu.

Currently focusing on rendering infrastructure and foundational architecture design. CAM processing features are planned for future development.

**Documentation Languages:**

| Document                 | Language                 | Access Method                                                 |
| ------------------------ | ------------------------ | ------------------------------------------------------------- |
| **Online Documentation** | 🌐 Japanese              | **[📖 GitHub Pages](https://redring2020.github.io/RedRing/)** |
| Geometry Abstraction     | 🇺🇸 English (placeholder) | `model/GEOMETRY_README.md`                                    |
| Geometry Abstraction     | 🇯🇵 Japanese (detailed)   | `model/GEOMETRY_README.ja.md`                                 |

---

## 🔍 Overview

RedRing is a research project aiming to create a standalone CAD/CAM development environment using Rust + wgpu.
NURBS and primitive geometric elements are currently under development and will be introduced incrementally.
Machining simulation and CAM path generation features are also planned for future implementation.

### 🌟 Key Features

- **Type Safety with Rust**: Memory safety and performance optimization
- **GPU Rendering**: High-performance 3D graphics using wgpu
- **Modular Design**: Extensible architecture with clear separation of concerns
- **Internationalization**: Multi-language documentation support
- **Research-Oriented**: Open-source exploration of CAD/CAM technologies

---

## 🚧 Current Development Status

### ✅ Implemented Features

- 🎨 **Rendering Infrastructure** (wgpu/winit) - GPU rendering pipeline
- 📐 **Foundation Geometry Library** - Points, lines, planes, vector operations
- 🏗️ **Foundation Pattern** - Unified geometric primitive interface
- 📊 **NURBS System** - Complete NURBS curves and surfaces implementation
- 🔧 **Test Framework** - Comprehensive test suite (23/23 tests passing)
- 📖 **Documentation** - Beautiful technical documentation with mdbook

### 🔄 In Development

- 🎯 Extended geometric algorithms
- 🎮 Interactive user interface
- 💾 CAD file format support (STEP/IGES)

### 📅 Future Plans

- 🔪 CAM path generation engine
- ⚡ Machining simulation
- 🌐 WebAssembly support
- 🖱️ SpaceMouse integration

For implementation progress and design decisions, please check our Issue tracker:
👉 [View Issues](https://github.com/RedRing2020/RedRing/issues)

Development structure and responsibility separation designs are managed via GitHub Projects:
👉 [View Projects](https://github.com/RedRing2020/RedRing/projects)

> **Note:** README is only updated when stable features are implemented. For detailed progress, please check Issues/Projects.

---

## 📚 Documentation Guide

| Document                                                           | Target Audience                | Content                                                      |
| ------------------------------------------------------------------ | ------------------------------ | ------------------------------------------------------------ |
| `README.md`                                                        | General Users & New Developers | Project Overview & Build Instructions                        |
| [`ARCHITECTURE.md`](ARCHITECTURE.md)                               | Developers                     | Workspace Structure & Migration Status & Test Strategy       |
| [`dev/ISSUE_LABEL_OPERATION.md`](dev/ISSUE_LABEL_OPERATION.md)     | Maintainers                    | Issue Label Rules & Release/Phase Operation                  |
| [`manual/philosophy.md`](manual/philosophy.md)                     | Contributors                   | Design Philosophy & Error Handling & Implementation Patterns |
| [`model/GEOMETRY_README.ja.md`](model/GEOMETRY_README.ja.md)       | Geometry Library Developers    | Detailed Geometry Abstraction Specifications                 |
| [`.github/ai_quick_reference.md`](.github/ai_quick_reference.md)   | 🤖 AI Developers               | Session Recovery & Development Continuity Support            |
| [GitHub Issues](https://github.com/RedRing2020/RedRing/issues)     | Developers                     | Feature Requests & Bug Reports & Progress Management         |
| [GitHub Projects](https://github.com/RedRing2020/RedRing/projects) | Developers                     | Development Roadmap & Task Management                        |

---

## 🛠️ Technology Stack

### Core Technologies

- **Rust** (latest stable recommended) - Systems programming language
- **wgpu** - Cross-platform GPU API
- **winit** - Window management and event handling

### Numerical Computing & Geometry

- **nalgebra** - Linear algebra library
- **approx** - Floating-point comparison
- **Custom NURBS** - In-house NURBS engine implementation

### Development & Testing

- **cargo** - Rust package manager
- **mdbook** - Documentation generation
- **GitHub Actions** - CI/CD pipeline

### Future Support

- **WebAssembly** - Browser execution environment
- **STEP/IGES** - CAD file formats
- **OpenCASCADE** - Advanced geometry kernel (under consideration)

---

## 📋 Design Principles

RedRing is built on the following principles:

### 🔒 Type Safety

- Memory safety through Rust's ownership system
- Abstraction via generics and traits
- Compile-time error detection for quality assurance

### 🏗️ Separation of Concerns

- **Foundation**: Basic functionality and numerical analysis
- **Model**: Geometric data layer and algorithms
- **View**: Application and rendering layer
- **ViewModel**: View transformation logic

### 🚀 Future Extensibility

- Modular crate architecture
- Plugin-capable design
- Incremental feature addition support

For detailed design philosophy, error handling guidelines, and trait design patterns:

📖 **[Design Philosophy & Technical Guidelines](manual/philosophy.md)** - Developer Guide

---

## 🚀 Build Instructions

### Prerequisites

#### Basic Requirements

- **Rust** (latest stable recommended) - Install from [official site](https://www.rust-lang.org/)
- **cargo** (included with Rust)
- **git** - For repository cloning

#### Platform-Specific Requirements

**Windows:**

- Visual Studio Build Tools or Visual Studio Community
- Windows 10/11 (DirectX 12 support)

**macOS:**

- Xcode Command Line Tools: `xcode-select --install`
- macOS 10.15+ (Metal support)

**Linux:**

- Required packages: `sudo apt install build-essential pkg-config libx11-dev`
- Vulkan or OpenGL drivers

### Build Steps

#### 1. Clone Repository

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
```

#### 2. Verify Dependencies

```bash
# Check Rust version
rustc --version

# Check build tools
cargo --version
```

#### 3. Build Project

```bash
# Debug build (fast compilation)
cargo build

# Release build (optimized)
cargo build --release
```

#### 4. Run Application

```bash
# Requires GUI environment (X11/Wayland/Windows/macOS)
cargo run

# Run tests
cargo test --workspace
```

### 🎮 Practical Key Bindings (after `cargo run`)

> Note: `cargo run -h` shows Cargo help. The key bindings below are available **after launching the app**.

#### Frequently Used

| Key | Action |
| --- | ------ |
| `h` | Show in-app help in logs |
| `r` | Reset camera |
| `t` | Return to standard CAD view |
| `w` | Toggle display mode (Wire/Solid in CAM view) |

#### CAM Debug Workflow

| Key | Action |
| --- | ------ |
| `p` | Show cutter path only (color-coded lines) |
| `Shift+B` | Show ball end mill CAM simulation visualization |
| `Shift+F` | Show flat end mill CAM simulation visualization |
| `k` | Next snapshot frame |
| `j` | Previous snapshot frame |
| `Space` | Toggle autoplay / pause |
| `Shift+J` | Stop autoplay and return to the first frame |
| `+` / `-` | Adjust playback speed (current multiplier is shown in the title) |
| `Shift+S` | Toggle settings panel |

#### Octree Inspection

| Key | Action |
| --- | ------ |
| `o` | Show Octree / advance depth |
| `Shift+O` | Play Octree depth animation |

#### Mouse Controls

- `Ctrl + Left Drag`: Rotate
- `Ctrl + Middle Drag`: Pan
- `Ctrl + Right Drag`: Zoom

#### 5. Generate Documentation (Optional)

```bash
# Requires mdbook: cargo install mdbook
mdbook build  # Generates manual/ -> docs/
mdbook serve  # Serve locally for preview
```

### Troubleshooting

#### GPU Driver Issues

```bash
# If wgpu cannot detect GPU
export WGPU_BACKEND=vulkan  # Linux
# or
export WGPU_BACKEND=dx12    # Windows
```

#### Build Errors

```bash
# Update dependencies
cargo update

# Clean build
cargo clean && cargo build
```

---

## 🤝 Contributing

We welcome contributions to the RedRing project!

### How to Contribute

1. **Check Issues**: Review existing issues on [GitHub Issues](https://github.com/RedRing2020/RedRing/issues)
2. **Fork**: Fork the repository and create a working branch
3. **Implement**: Add features or fix bugs
4. **Test**: Ensure `cargo test --workspace` passes
5. **Pull Request**: Create a pull request with clear description

### Development Guidelines

- **Code Style**: Use `cargo fmt` for automatic formatting
- **Linting**: Run `cargo clippy` for quality checks
- **Documentation**: Add rustdoc comments for public APIs
- **Testing**: Include tests for new features

### Community

- 🐛 **Bug Reports**: Report via [Issues](https://github.com/RedRing2020/RedRing/issues)
- 💡 **Feature Requests**: Discuss in [Discussions](https://github.com/RedRing2020/RedRing/discussions)
- 📖 **Documentation Improvements**: Submit via pull requests

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).

---

## � License

RedRing is available under a dual license:

- **[MIT License](LICENSE-MIT)** - Simple, permissive license
- **[Apache License 2.0](LICENSE-APACHE)** - Includes patent grant protection

You may choose either license. See [LICENSE](LICENSE) for details.

This dual licensing follows the standard practice of the Rust ecosystem and provides maximum compatibility for both academic and commercial use.

---

## �🙏 Acknowledgments

We thank all contributors who have helped develop RedRing.

This project also benefits from the following open-source projects:

- [Rust Programming Language](https://www.rust-lang.org/)
- [wgpu](https://wgpu.rs/) - WebGPU implementation
- [winit](https://github.com/rust-windowing/winit) - Window handling
- [nalgebra](https://nalgebra.org/) - Linear algebra library

---

## 🔗 Links

- 📧 **Contact**: [Issues](https://github.com/RedRing2020/RedRing/issues) or [Discussions](https://github.com/RedRing2020/RedRing/discussions)
- 🌐 **Website**: [GitHub Pages](https://redring2020.github.io/RedRing/)
- 🐙 **GitHub**: [RedRing2020/RedRing](https://github.com/RedRing2020/RedRing)

---
