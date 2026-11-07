# Getting Started

Start the RedRing preview either from the Windows package (zip) or by building it from source. The package is attached to releases from v0.1.3.

## Requirements

- OS: tested on Windows (Linux and macOS are expected to work because wgpu, the GPU rendering foundation, supports them, but they are not tested)
- GPU: a GPU that supports Vulkan, DirectX 12, or Metal
- To build from source: the latest stable Rust (install with [rustup](https://rustup.rs/)) and Git

A finer stock precision (for example 0.01 mm) needs more memory and GPU load. Try the default precision (0.05 mm) first.

## Start from the Package (Windows)

1. Download `redring-<version>-windows-x86_64.zip` and `SHA256SUMS.txt` from the [GitHub releases](https://github.com/RedRing2020/RedRing/releases)
2. Check the zip (see "Check the Package" below)
3. Extract the zip into a writable folder (for example under Documents) and double-click `redring.exe`. The log is written to `logs` in the working folder at startup (the extracted folder when started by double-clicking), so do not place it under `Program Files`. When you start it from a shortcut, set the shortcut's "Start in" folder to the extracted folder

The executable does not have a code signature yet, so Windows SmartScreen shows "Windows protected your PC" the first time you start it. After checking the package, start it with "More info" → "Run anyway".

### Check the Package

To check that the zip was not damaged while downloading, compare its SHA-256 with the value in `SHA256SUMS.txt`. Run the following in PowerShell; `True` means they are the same (replace `<value>` with the value on the zip line of `SHA256SUMS.txt`).

```powershell
(Get-FileHash .\redring-v0.1.3-windows-x86_64.zip -Algorithm SHA256).Hash -eq '<value>'
```

To check that the zip was not replaced (that it was built by the CI of the RedRing repository from the release tag), use the [GitHub CLI](https://cli.github.com/). If the release assets were replaced together, the checksums would be replaced too, so this is the reliable check.

```powershell
gh attestation verify .\redring-v0.1.3-windows-x86_64.zip --repo RedRing2020/RedRing
```

The release also includes the list of dependent crates (SBOM, a `.cdx.json` file in the CycloneDX format).

## Build from Source and Start

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
git checkout v0.1.3
cargo build --release
cargo run --release -p redring
```

- `cargo build --release` builds the job manager (`redring-job-manager`) and the runner (`redring-batch-runner`) with the app. Without them, toolpath generation and the cutting simulation cannot be submitted (loading and viewing shapes still work). `cargo run` starts the app when given `-p redring`
- `--release` makes toolpath generation and the cutting simulation faster (without it, calculations take many times longer)
- The first build takes a few minutes to download the dependent crates
- A window with a menu bar opens. Continue with [Screen and Controls](./screen_and_controls.md) and [Cutting Simulation](./cutting_simulation_demo.md)

## Job Manager

Toolpath generation and the cutting simulation are calculated by the job manager (`redring-job-manager`), separate from the app. The package zip contains the app (`redring.exe`), the job manager, and the runner (`redring-batch-runner.exe`). Keep the three in the same folder.

- The app starts the job manager the first time you press "Submit" (without a window). The app screen does not stop during calculations
- Starting `redring-job-manager.exe` opens the job list window, where you can check job states, failure reasons, and logs (for now, the window cannot be opened while a job manager is already running; end the running job manager first)
- The job manager keeps calculating after you close the app, and also after you close the job manager window. To stop it, end `redring-job-manager` in Task Manager (quitting from the system tray will be added in a later version)
- Job records and results are kept in `%LOCALAPPDATA%\RedRing\jobs` on Windows. The job manager log is `manager.log` there

## Data to Prepare

- Prepare the product shape as an STL file (binary or ASCII). Units are treated as mm
- Machining is 3-axis scanline machining approaching from above (+Z). Orient the product shape so that the surfaces to cut face up

## Log

- The log is written to `logs/redring.log` in the working folder at startup (the extracted folder when the package is started by double-clicking, or the repository root in the steps above when built from source). It is rewritten at each start
- A build with `--release` does not open a console window. A build without `--release` also writes the same content to the console (the terminal it was started from)
- The location of the log file is also shown in "Help" → "About"
- The results of operations and the reasons why an operation could not be done also appear in the status bar at the bottom of the screen
- When the app ends because of an unexpected error, it shows an error message with the location of the log file
- The list of key bindings opens with the `F1` or `h` key, or "Help" → "Controls"
- When you report a problem, attach the log file

## Quit

- Quit with the `Esc` key or the window's close button
