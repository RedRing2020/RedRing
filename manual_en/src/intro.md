# Introduction

[日本語のマニュアル](../index.html)

RedRing is a research platform for CAD/CAM built with Rust + wgpu.
With a design that values type safety and separation of responsibilities, it is being developed from the geometry foundation up to CAM toolpath generation and cutting simulation.

## Background and Goals

Traditional CAD kernels have trouble with maintainability and extensibility because of complex dependencies and unclear division of responsibilities.
RedRing uses the Rust type system and aims at a modern architecture that values **type safety, separation of responsibilities, and room for future extension**.

## Design Principles

- ✅ **Type safety**: a robust design that catches errors at compile time
- 🧩 **Separation of responsibilities**: a clean architecture where each module has a single responsibility (split into the screen, ViewModel, Application, and Model layers)
- 🌍 **Internationalization**: the screen text is available in Japanese and English
- ⚡ **Performance**: GPU-accelerated rendering with wgpu
- ✅ **Extensibility**: abstraction with traits to add future features

## Current Structure

RedRing is a workspace with the following layers.

### Foundation

- `analysis`: numerical analysis and tolerances
- `logging_foundation`, `i18n_foundation`: log initialization and text tables

### Geometry (geo in model)

- `geo_contracts`: the source of truth for the geometry trait definitions
- `geo_core`, `geo_commons`: geometry value types and numerical calculations shared by shapes
- `geo_primitives`, `geo_nurbs`: analytic shapes and NURBS curves and surfaces
- `geo_topology`: topology
- `geo_algorithms`: operations across shapes (distance, intersection, tessellation)
- `geo_entity`: the entity foundation (identifiers, attributes, relations)
- `geo_io`: file input and output (STL and others)

### CAM (cam in model)

- `cam_core`: neutral CAM data (toolpaths, tools, artifact formats)
- `cam_algorithms`: toolpath generation (scanline machining with the inverse offset method)
- `cam_entity`: entities for machining plans and machining processes
- `cam_sim`: cutting simulation (tri-dexel)

### Jobs and Use Cases

- `job_runtime`, `job_domain`: job execution control and submission rules
- `application`: the entry for use cases (the only entry from the ViewModel to the Model)

### Screen

- `viewmodel`, `viewmodel-graphics`: conversion for display and control of the camera and others
- `render`, `stage`: the GPU rendering foundation and rendering stages
- `redring`: the main application

For design details, see the repository's [architecture overview](https://github.com/RedRing2020/RedRing/blob/develop/dev/architecture/ARCHITECTURE.md) and the [list of design documents](https://github.com/RedRing2020/RedRing/tree/develop/dev/architecture) (in Japanese).

## Audience

- People who try the RedRing preview (start from [Getting Started](./getting_started.md))
- Rust developers interested in developing a CAD/CAM platform
- Engineers in geometry computation and GPU rendering
- Contributors to open-source CAD projects

## Development Status

**🌍 v0.1.2 preview** (November 2026): Made the screen easier to use. The list of controls opens from the "Help" menu, and the results of operations and the reasons for failures appear in the status bar at the bottom of the screen. The screen text can be switched between Japanese and English with "View" → "Language" ([Screen and Controls](./screen_and_controls.md)).

**🔒 v0.1.1 preview** (November 2026): A fix release of v0.1.0. It includes security fixes (crashes of the app on crafted artifact or STL files, and known vulnerabilities in dependent crates). If you use v0.1.0, use v0.1.1 instead. The machining conditions are now held as a machining plan and a machining process, and only results calculated with the current conditions and the visible product shape can be imported ([Cutting Simulation](./cutting_simulation_demo.md)).

**🔪 v0.1.0 preview** (November 2026): Load an STL, calculate a 3-axis scanline toolpath, and check it with the cutting simulation. There is no NC output yet, so it cannot be used for actual machining.

The next version (v0.2.0) is planned to add machining management (a list of machining processes, a job manager, and tool registration).

For the latest progress, see:

- [GitHub Issues](https://github.com/RedRing2020/RedRing/issues)
- [Change log (CHANGELOG, in Japanese)](https://github.com/RedRing2020/RedRing/blob/develop/CHANGELOG.md)

---

This manual is for people who use the preview. Explanations for developers (design, crate structure, API) are kept in `dev/architecture/` in the repository and are not part of this manual.
