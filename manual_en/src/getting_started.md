# Getting Started

Start the RedRing preview either from the Windows package (zip) or by building it from source. The package is attached to releases from v0.1.3.

## Requirements

- OS: tested on Windows (Linux and macOS are expected to work because wgpu, the GPU rendering foundation, supports them, but they are not tested)
- GPU: a GPU that supports Vulkan, DirectX 12, or Metal
- To build from source: Rust (install with [rustup](https://rustup.rs/). The version is set in `rust-toolchain.toml` of the repository, and rustup uses it when you build) and Git

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

- `cargo build --release` builds the job manager (`redring-job-manager`) and the runner (`redring-batch-runner`) with the app. Without them, toolpath generation and the cutting simulation cannot be calculated (loading and viewing shapes still work). `cargo run` starts the app when given `-p redring`
- `--release` makes toolpath generation and the cutting simulation faster (without it, calculations take many times longer)
- The first build takes a few minutes to download the dependent crates
- A window with a menu bar opens. Continue with [Screen and Controls](./screen_and_controls.md) and [Cutting Simulation](./cutting_simulation_demo.md)

## Job Manager

Toolpath generation and the cutting simulation are calculated by the job manager (`redring-job-manager`), separate from the app. The package zip contains the app (`redring.exe`), the job manager, and the runner (`redring-batch-runner.exe`). Keep the three in the same folder.

- The app starts the job manager the first time you press "Calculate" (without a window). The app screen does not stop during calculations
- Starting `redring-job-manager.exe` opens the job list window, where you can check job states, failure reasons, and logs. When a job manager is already running, its window is opened
- While the job manager runs, a red ring icon is shown in the system tray. Left-click the icon to open the window. The right-click menu has these items
  - "Open window"
  - "Start when Windows starts": when checked, the job manager starts without a window when you sign in to Windows
  - "Language": selects the language of the job manager screens (日本語 or English)
  - "Quit": quits the job manager. When a job is being calculated, you are asked to confirm (the calculation stops, and the job is interrupted)
- When a calculation finishes, a notification is shown from the system tray icon. Toolpath generation and the cutting simulation are calculated one after the other, so the notification is shown once, when both have finished (or when one fails). Click the notification to open the window. While the notification is shown, Windows does not pass clicks on the icon to the job manager, so clicking the icon does not open the window (click the notification, or click the icon after the notification disappears). When the job manager stopped during a calculation (for example, it ended abnormally or Windows restarted), a notification shows the interrupted job at the next start
- A job whose calculation stopped because of "Quit" or "Stop" in the job manager, an abnormal end, or a restart of Windows becomes "Interrupted". An interrupted job is not calculated automatically. Put it back to waiting with "Resume" in the job manager or "Resume" in the operation list of the app. An interrupted toolpath generation continues from near the point where it stopped (the last scanline it recorded). A cutting simulation is calculated again from the beginning. The elapsed time of a job calculated again from where it stopped includes the time before the interruption. A cutting simulation that uses the result of an interrupted toolpath generation stays waiting
- "Open job manager" in the operation list of the app also opens the job manager window
- If the job manager of the previous version keeps running after you update the app, operations such as calculating report that the job manager is a different version. Quit the previous job manager with "Quit" in the system tray menu, then try again (the job manager of the new version starts)
- The details of a job whose CL was imported into a space model show "The CL was imported into a space model"
- Select a job in the list and use the buttons at the top of the window. Only the buttons that apply to the job's state can be pressed. `Ctrl`+click adds or removes a job, and `Shift`+click selects a range (when several jobs are selected, only "Delete" can be pressed)
  - "Resume": puts an interrupted job back to waiting. When the toolpath generation whose result the job uses is also interrupted, it is put back together
  - "Stop": stops the calculation of a calculating job and makes it interrupted. You can continue it later with "Resume" (a toolpath generation continues from near the point where it stopped). Other calculating jobs are not stopped
  - "▲ Raise" / "▼ Lower": changes the priority of a waiting job (changing a toolpath generation also changes the cutting simulation that uses its result). Jobs with a higher priority are calculated first
  - "Recalculate": puts a job that is calculation failed, canceled, or interrupted back to waiting, to be calculated from the beginning. The toolpath generation and the cutting simulation that uses its result are put back together when they are calculation failed, canceled, or interrupted
  - "Cancel": cancels a waiting or interrupted job. For a calculating job, stops the calculation
  - "Delete": deletes the selected jobs. Deleting a toolpath generation also deletes the cutting simulation that uses its result (it could be neither calculated nor imported without it). Deleting only a cutting simulation keeps the toolpath generation (you can calculate only the cutting simulation again under it with "Calculate" in the app). Jobs that are waiting or calculating, and toolpath generations whose cutting simulation is waiting or calculating, are kept and their numbers are shown (an interrupted toolpath generation and the waiting cutting simulation that waits for it can be deleted). When several jobs, or jobs deleted together, are involved, the numbers of the jobs to delete are shown for confirmation
  - "Bulk delete…": deletes the items you choose below at once. The number of jobs to delete is shown for confirmation. Deleting a toolpath generation also deletes the cutting simulation that uses its result. Jobs that are waiting or calculating are not deleted
    - Results of previous revisions: jobs for which a later job was submitted and completed for the same machining operation with other conditions (the jobs of the last completed conditions are kept; if you use copies of a model file in turn, results you still use may be included)
    - Sources of imported CLs: toolpath generations whose cutting simulation result was imported in the app, putting the CL into a space model (their cutting simulations are deleted with them; you can still calculate only the cutting simulation again from the CL in the space model with "Submit" in the app. CLs imported into other model files or unsaved changes are included; a toolpath generation is not included if another cutting simulation was calculated under it after the import)
    - Old jobs: finished jobs submitted before the number of days you enter (30 at first; a toolpath generation whose result is used by a cutting simulation submitted later is kept. Interrupted jobs are not included, because their calculation can be continued)
    - Submissions that could not be taken in: submissions the job manager could not take in, for example because of a different app version
- The language of the job manager screens is selected separately from the app ("Language" at the top right of the window, or "Language" in the system tray menu). It changes at once, and the same language is used at the next start. Until you select one, it follows the Windows display language (Japanese for Japanese, English otherwise). The selected language is saved in a per-user settings file (`%APPDATA%\RedRing\job_manager.json` on Windows)
- The job manager keeps calculating after you close the app. When you close the job manager window, it stays in the system tray and keeps calculating. To stop it, use "Quit" in the system tray menu (opening a machining sequence in the app does not start the job manager; it starts when you calculate, import, or press "Open job manager". If you quit the job manager while jobs are waiting, the calculation stops until you do one of those operations)
- Job records and results are kept in the job store. At first, it is `%LOCALAPPDATA%\RedRing\jobs` on Windows. The job manager log is `%LOCALAPPDATA%\RedRing\logs\job_manager.log` on Windows (the same place even if you change the job store). Hover over "Job store: …" at the top of the window to see its location
- You can change the job store with "Change job store…" at the top right of the window (not while jobs are waiting or calculating)
  - After you choose a folder, you are asked to confirm. The job manager then restarts and lists the jobs in the new job store. The jobs and files in the previous job store are kept (they are listed again if you change back)
  - Model files do not remember the job store. The app finds the jobs of each process in the current job store to show their status (it does not matter whether you open the model file or change the job store first). After you change the job store, processes calculated in the previous job store are shown in the app as not calculated. "Calculate" calculates them again from the start in the new job store (if the CL of the current conditions was imported, only the cutting simulation is calculated). To continue an interrupted process, change back to the previous job store and use "Resume"
  - If you choose a folder used as a job store before, its jobs are listed. If the folder contains files that are not part of a job store, the confirmation says so. Choose an empty folder or a folder used as a job store
  - The job store cannot be changed in the following cases, and the reason is shown at the top of the window
    - Jobs are waiting or calculating, or submissions are waiting to be taken in (interrupted jobs and the jobs waiting for them are not counted; the job store also cannot be changed while the current one cannot be read, but it can be changed when the job store folder is not found, such as on a removed drive)
    - The folder is the current job store
    - The folder is inside the current job store or contains it (for example, `%LOCALAPPDATA%\RedRing`, which contains the first job store `%LOCALAPPDATA%\RedRing\jobs`)
    - The folder is a network folder (`\\server\…` or a network drive)
    - The folder cannot be read or written
    - The folder is not a job store but contains items with the same names as those in a job store (an `inbox` folder, a `jobs` folder or a `manager.lock` file), so that other data is not mixed in. A folder with both `inbox` and `jobs` can be used as a job store used before
    - The chosen job store cannot be saved in the settings file
  - Choose a folder that other users cannot write to (the job manager calculates from the files in the job store)
  - The chosen job store is saved in the settings file (`%APPDATA%\RedRing\job_manager.json`) and used at the next start. The app uses the job manager's job store automatically (no app setting is needed)

## Data to Prepare

- Prepare the product shape as an STL file (binary or ASCII). Units are treated as mm
- Machining is 3-axis scanline machining approaching from above (+Z). Orient the product shape so that the surfaces to cut face up

## Log

- The log is written to `logs/redring.log` in the working folder at startup (the extracted folder when the package is started by double-clicking, or the repository root in the steps above when built from source). A new log is started at each start, and the logs of the previous five starts are kept with numbered names (`redring.1.log` is the previous one, `redring.2.log` the one before, and so on; the job manager log works the same way). If the app or the job manager stops, you can still check the log from before it stopped after starting it again
- A build with `--release` does not open a console window. A build without `--release` also writes the same content to the console (the terminal it was started from)
- The location of the log file is also shown in "Help" → "About"
- The results of operations and the reasons why an operation could not be done also appear in the status bar at the bottom of the screen
- When the app ends because of an unexpected error, it shows an error message with the location of the log file
- The list of key bindings opens with the `F1` or `h` key, or "Help" → "Controls"
- When you report a problem, attach the log file

## Quit

- Quit with "File" → "Exit", the `Esc` key, or the window's close button. When there are unsaved changes, the app asks whether to save them
