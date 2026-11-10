# Known Issues and Troubleshooting

## Known Issues (v0.1.3 preview)

Machining:

- There is no NC output. The toolpath is for checking with the cutting simulation and cannot be used for actual machining (NC output is planned for v0.3.0)
- There are no air cuts or linking moves (approach and retract) between passes. In the cutting simulation, the tool jumps from the end of a pass to the start of the next pass and does not cut in between
- Gaps and holes narrower than the tool radius are bridged by one pass across both sides, which passes over the gap at the cutting feed (the walls of the gap are not gouged). Small features that the tool touches along a scanline for less than the tool radius, in places where the tool does not touch the shape, may be missed and left uncut
- For NURBS shapes, the sum of the triangulation error and the toolpath error can be up to twice the toolpath tolerance (STL stays within the tolerance)
- The toolpath tolerance (0.001 mm) cannot be changed
- Only ball end mills and flat end mills are available. Even when a tool length is specified, a path that touches the shape above the tool length is not reported as an error (the stock above the tool length is just left)

Management:

- There is one machining sequence. It can have several machining operations, but the cutting simulation of every operation starts from the sequence stock (starting from the stock left by the previous operation comes in a later version). There is no tool registration (tool library) yet (planned for v0.2.0)
- The job manager calculates one job at a time. After reopening the app, the results of jobs submitted before cannot be imported from the process (saving processes will be added in a later version)
- The machining target is all visible product shapes (selecting faces is a later feature)
- There is no screen yet to choose which job to import, and only the latest result of the current conditions can be imported

Distribution:

- The package is for Windows only. The executable does not have a code signature yet, so Windows SmartScreen shows a warning the first time you start it (see [Getting Started](./getting_started.md) for how to check the package)

Display:

- The log text is English regardless of the screen language. Attach it as is when you report a problem
- If the settings file (`%APPDATA%\RedRing\settings.json` on Windows) cannot be read, the app starts in Japanese and shows a message in the status bar at the bottom of the screen. The unreadable file is not overwritten, so a language selected meanwhile is not kept for the next start. Fix or delete the file and start again
- The stock color (with opacity) and the tool line color are selected in "Cutting Animation Colors" on the Display tab of the settings panel (`Shift+S`) and applied to the display with "Apply Settings". Only the stock color changes; the surface is not rebuilt. Until applied, the previous colors are used even during playback
- Where a cut surface passes exactly through a grid line of the stock, small gaps can appear in the stock surface
- With a translucent stock, the product shape inside shows through. Translucent surfaces are not sorted from back to front, so the view can look wrong where stock surfaces overlap
- Settings changed on the Tool tab of the settings panel (`Shift+S`) after importing (tool wireframe divisions, toolpath display) are applied with "Apply Settings", redrawing with the current frame, camera, and display mode

## Troubleshooting

| Symptom | What to check |
|---|---|
| "Import CL" or "Show cutting simulation" does not import | After changing the conditions or the visible product shape, nothing can be imported until you calculate again. Check the list opened with 📋 in the status bar and the log (`logs/redring.log`) |
| "Calculate" cannot be pressed | A condition error is shown in red below the field to fix. Fix it to enable the button |
| "Calculate" says that the job manager is not installed | Check that `redring-job-manager` and `redring-batch-runner` are in the same folder as the app. When built from source, build with `cargo build --release` |
| The calculation stays "waiting" | Check the job manager log (`%LOCALAPPDATA%\RedRing\jobs\manager.log`) |
| The calculation "failed" | The reason is in the job log (`%LOCALAPPDATA%\RedRing\jobs\<job number>\logs\runner.log`) |
| Drawing stops while the open-file dialog is open | It resumes when the dialog is closed |
| Rotating the stock stops drawing or the screen disappears | With a fine stock precision (for example 0.01 mm), the GPU load becomes high and the GPU device can be lost. The app then shows an error message and ends. Calculate again with a coarser precision |
| The camera is inside a shape | Press the `e` key (emergency escape) or the `r` key (reset) |
| Keys do not work | While editing a field in the settings panel or a settings dialog, typed characters are not used as keys. While the operation settings or sequence settings dialog is open, `Shift+M`, `Shift+L`, and the file keys are not accepted. Click outside the field |

When you report a problem, open [GitHub Issues](https://github.com/RedRing2020/RedRing/issues) and attach the steps and the log file (`logs/redring.log`).
