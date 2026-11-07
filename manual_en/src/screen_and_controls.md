# Screen and Controls

## Screen Layout

| Part | Contents |
|---|---|
| Menu bar | "File" → "Load…" loads a shape file (STL for now). You can also drop an STL file onto the window. "View" → "Language" selects the language of the screen text. "Help" → "Controls" shows the list of keys, and "About" shows the version, the location of the log file, and a link to the manual |
| Model view | Shows the product shape, the stock, the toolpath lines, and the tool lines |
| Progress bar at the top left | Appears when a cutting simulation is imported. Drag it with the left button to move to any time (scrubbing) |
| Status bar at the bottom of the screen | Shows the last message on one line: the results of loading a shape, submitting, running, and importing, and the reasons why an operation could not be done. Warnings are yellow and failures are red |
| Window title | When a cutting simulation is imported, shows the current time (`Snapshot 1/N`), the segment number, the remaining volume, and the playback speed |

The screen text can be Japanese or English, selected with "View" → "Language" (Japanese at first). It changes at once, and the same language is used at the next start. The selected language is saved in a per-user settings file (`%APPDATA%\RedRing\settings.json` on Windows). The log text is English regardless of the screen language.

## Windows

| Window | How to open | Contents |
|---|---|---|
| Machining Conditions | `Shift+M` | Input of the tool, machining, and stock conditions, and submitting and importing the cutting simulation ([Cutting Simulation](./cutting_simulation_demo.md)) |
| Controls | `F1`, `h`, or "Help" → "Controls" | The list of key and mouse controls (the same as "Key Bindings" below) |
| Settings | `Shift+S` | Display settings. The tabs are "Display" (frame step distance, cutting animation colors, gouge coloring), "Tool" (tool wireframe divisions, toolpath display), and "Debug" (debug views) |

While editing a field in the settings panel or the Machining Conditions window, typed characters are not used as keys (except `Esc` to quit).

## Camera

| Control | Action |
|---|---|
| `Ctrl+left drag` | Rotate |
| `Ctrl+middle drag` | Pan |
| `Ctrl+right drag` or `Ctrl+wheel` | Zoom |
| `r` | Reset the camera (fit to the shapes) |
| `t` | Standard CAD view |
| `f` | Front view |
| `e` | Emergency escape (bring back a camera inside a shape) |

## Key Bindings

The same as the "Controls" window opened with the `F1` or `h` key.

| Category | Key | Action |
|---|---|---|
| Camera | `r` | Reset the camera |
| | `t` | Standard CAD view |
| | `f` | Front view |
| | `e` | Emergency escape (bring back a camera inside a shape) |
| Cutting simulation | `Shift+M` | Show or hide the Machining Conditions window (submit with "Submit") |
| | `Shift+L` | Import the latest cutting simulation calculated with the current conditions |
| | `k` / `j` | Step forward / back by the frame step distance |
| | `.` / `,` | Move the tool to the next / previous segment endpoint |
| | `Space` | Play / pause |
| | `Shift+J` | Stop playback and go to the start |
| | `+` / `-` | Double / halve the playback speed |
| Display | `w` | Solid / wireframe |
| | `v` | Show or hide the toolpath lines |
| | `Shift+V` | Show or hide the tool lines |
| Application | `Shift+S` | Show or hide the settings panel |
| | `F1` / `h` | Show or hide the Controls window |
| | `Esc` | Quit |

- `w`: switches between the solid view and the wireframe view (triangle edges)
- `v`: the toolpath lines are all visible through the stock surface, so hide them to see how the stock is cut
- The shown or hidden state is kept after stepping, playback, importing, and "Apply Settings"

## Debug Views

The debug views of a line segment, circle, arc, triangle, and the inverse offset shape are shown with the buttons on the Debug tab of the settings panel (`Shift+S`), not with keys. "Back to the normal view" on the Debug tab returns to the product shape and cutting animation shown before the debug view (the same frame and the same camera direction). While a debug view is shown, stepping and playback of the cutting animation are not available. The inverse offset shape view is drawn without changing the camera direction or zoom (a camera moved in that view is kept after going back to the normal view), and the toolpath lines of the imported cutting simulation are drawn over it (so you can check that the tool tip follows the top of the inverse offset shape). Show or hide the lines with the `v` key, as in the normal view. If the tool of the imported result differs from the tool of the current conditions (type or radius), the lines do not match the inverse offset shape, and the status bar tells you so.

"Show" under "Inverse Offset Shape" on the Debug tab builds the same inverse offset shape as toolpath generation from the visible product shape and the tool of the conditions, and shows its elements in colors by role (the product shape is not shown; nothing is shown if the conditions have an error).

| Color | Element |
|---|---|
| Yellow | Faces whose height is used (faces facing up) |
| Blue | Faces whose height is not used (vertical faces and undercut faces, offset outward) |
| Purple | Faces facing down in an open shape, offset upward |
| Red | Faces of a shape whose outside (the side without material) could not be determined |
| Green | Vertices (ball: sphere; flat: a disk only at vertices the tool bottom rests on) |
| Light blue | Edges (ball: cylinder; flat: the part touching the edge of the shape swept by the disk rim along the edge) |

Spheres, cylinders, and disks are split into triangles in 10° steps for display (used only for display, not for calculation).
