# Cutting Simulation

Load a product shape (STL), submit and run a cutting simulation with the visible product shape as the machining target, and check the result over time on top of the product shape. For key and mouse controls, see [Screen and Controls](./screen_and_controls.md). For known issues, see [Known Issues and Troubleshooting](./known_issues.md).

## Current Scope

- Tools: flat end mill and ball end mill (the default is a ball end mill D10). The tool length is not specified by default, and stock touched above the tool is also removed. When a tool length is specified, stock above the tool length remains
- Machining: 3-axis scanline machining (the defaults are climb cutting, stepover 2 mm, and feed rate 1200 mm/min). CL points are calculated along each scanline at intervals no longer than the tool radius, and the ends where the tool touches the shape are found to the toolpath tolerance. Points are added where needed so that the tool does not cut into the product shape beyond the toolpath tolerance between CL points, and at a step in the CL the tool keeps the height of the higher side before going down. The cut direction is selected from climb, conventional, and zigzag (starting with climb and alternating the direction on each line). The spindle is assumed to turn clockwise. The scanline direction is set by an angle counterclockwise from the X axis (the default is 0°, scanlines along X). Where the scanlines run along a steep wall, the spacing between passes widens on the wall and leaves more uncut stock, so choose an angle that crosses the main walls
- Stock: a box that adds margins outside each face (-X, +X, -Y, +Y, top, bottom) of the box around the product shape (the default margin is 0, the box around the product shape itself)
- Stock precision: the default is 0.05 mm (it sets how fine the stock surface is; with a D10 tool, the grid spacing is about 1.4 mm)
- Snapshot controls: step, rewind, scrub, and playback
- Display modes: solid and wireframe

The tool, machining, and stock conditions are entered in the Machining Conditions window (Shift+M) and submitted with "Submit". This window is separate from the display settings (the settings panel, Shift+S).

## Steps

1. Start RedRing ([Getting Started](./getting_started.md)).
2. Choose an STL file with "File" → "Load…" in the menu bar, or drop an STL file onto the window, to load the shape. The loaded shape becomes a visible product shape and the machining target.
3. Press `Shift+M` to open the Machining Conditions window, enter the tool, machining, and stock conditions, and press "Submit". A cutting simulation of the visible product shape is submitted, and the job manager calculates it ("Job Manager" in [Getting Started](./getting_started.md)). The screen does not stop during the calculation, and the Machining Conditions window shows the state of the calculation for the current conditions (waiting, calculating, completed, failed).
4. When the calculation has finished, press "Import" to import the latest cutting simulation that finished successfully with the current conditions. After changing the conditions or the visible product shape, nothing can be imported until you submit again.

`Shift+L` is the same as "Import".

After a successful import, the following appear.

- The product shape and the stock surface, the toolpath lines, and the tool lines
- The progress bar at the top left
- The current frame information in the window title

Importing does not change the camera (direction and zoom), so you can compare the same place after importing again with other conditions. To fit the camera to the shapes again, press `r` (only when there is no visible product shape, the camera is fitted to the stock on import).

## Controls

### Machining Conditions Window (`Shift+M`)

- Enter the tool (type, diameter, length), the machining (cut direction, stepover, feed rate, scan angle, tool range), and the stock (margins, precision). The toolpath tolerance (0.001 mm) is shown only and cannot be changed
- The machining section shows the upper limit of the cusp height, the height of the ridges left between neighboring passes on a flat surface (the spacing between passes is at most the stepover, because the machining range is divided equally). For a ball end mill it is set by the diameter and the stepover (for example 0.1010 mm with diameter 10 mm and stepover 2 mm); for a flat end mill it is 0. It changes on inclined surfaces, so it is a guide value. It is not shown when the stepover exceeds the tool diameter
- If a condition has an error, it is shown in red below the field to fix, and "Submit" cannot be pressed. Examples: the precision is finer than the toolpath tolerance; the precision is too fine for the stock size (the finest precision that can be submitted is shown); the stepover exceeds the tool diameter (the tool diameter is shown). Error messages are shown in the display language (Japanese or English). Failures that conditions cannot fix, such as running a job or importing a result, are also shown with the reason in the display language. Detailed reasons are kept in the log (in English)
- Below "Submit" and "Import", the result of the last operation (submit or import) and the state of the calculation for the current conditions are shown. The state is read again every second
- Importing also copies the calculated CL (toolpath) into the space model, so that it can be saved in the model file (this is an unsaved change). The next line shows whether the CL in the space model is the result of the current conditions (not imported yet, result of the current conditions, result of earlier conditions). After the conditions change, it becomes "result of earlier conditions", and importing the current result replaces it
- When the CL in the space model is the result of the current conditions, "Simulate cutting only" recalculates only the cutting simulation, without toolpath generation. Use it after the job manager's results have been deleted, or after opening the model file on another PC. If the toolpath generation result for the current conditions is still in the job manager (when only the cutting simulation was deleted), the cutting simulation is put under it; otherwise it is calculated from the CL in the space model. To avoid repeating the same calculation, it cannot be pressed while the calculation for the current conditions is waiting, calculating, or completed. When the calculation has finished, press "Import" to show it
- "Conditions of the displayed result" shows the conditions used to calculate the imported cutting simulation (tool, machining, tolerance, stock size, precision)

### Moving in Time and Playback

- `k`: steps forward to the frame the frame step distance ahead along the path (from the last frame, to the first)
- `j`: steps back to the frame the frame step distance behind along the path (from the first frame, to the last)
- The frame step distance is entered in mm in "Frame Step" on the Display tab of the settings panel (Shift+S). It is reset to 1/100 of the total toolpath length (one step is about 1% of the progress bar) on each import. The "Reset to the initial value" button (with the initial value in mm) returns it to the value right after the import. If the distance is shorter than the interval of one frame (including 0), it moves one frame at a time
- `.` / `,`: moves the tool to the next / previous segment endpoint (the start or end of a straight segment of the toolpath). The stock shows the state of the closest frame before that endpoint (frames are recorded at intervals set by the tool and the grid, so the stock shape exactly at the endpoint is not available). While the tool is at an endpoint, the window title shows the endpoint number of the tool and the frame of the stock shown. When the frame changes with `k`, `j`, scrubbing, or playback, the tool returns to the frame position
- Left-drag the progress bar at the top left: scrubs to any frame
- `Space`: switches between playback and pause. At 1x, the tool moves 1 m per second along the path (with the same speed setting, the tool speed is the same for every workpiece)
- `Shift+J`: stops playback and returns to the first frame
- `+` / `-`: doubles or halves the playback speed (from 0.25x to 32x). The current speed is shown in the window title

Pressing `k`, `j`, or `Space` before importing writes to the log that nothing is imported.

When moving forward or back, only the stock surface of the changed parts is replaced. When going back far toward the start (such as pressing `Space` at the end to play from the start), the surface is rebuilt from the initial stock.

During playback, where drawing cannot keep up, playback slows down instead of skipping frames. When the speed is raised so that the tool moves more than its size in one draw (for example 4x or more with a D10 tool), redrawing cannot follow the tool, and the afterimage makes the tool look like several tools. The tool lines are always drawn.

### Gouge and Uncut Coloring

At the last frame, the stock surface is colored by comparison with the product shape (on by default).

| Color (default) | Meaning |
|---|---|
| Red | Gouge (cut into the product shape deeper than the toolpath tolerance) |
| Green | Within tolerance (the distance from the product shape is within the tolerance) |
| Blue | Uncut (farther than the tolerance outside the product shape) |
| Stock color | Not compared (such as the uncut outer surface of the stock) |

- The tolerance is 0.001 mm, so the ridges between ball end mill passes (the cusp height) are colored as uncut. Green appears where the product shape was reached, such as near the center of a pass
- Colors are decided for each grid interval of the stock, so color borders can shift by about a grid interval (about 1.4 mm with precision 0.05 mm), and gouges narrower than that can be missed
- Coloring on or off and the three colors are selected in "Cutting Animation Colors" on the Display tab of the settings panel (Shift+S) and applied with "Apply Settings". Frames other than the last are drawn in the stock color

## What to Check Right after Importing

- The window title changes to the `Snapshot 1/N` form
- The progress bar appears at the top left
- With `k` / `j`, the frame number and the remaining volume change, and the stock surface is cut
- `w` switches between the solid view and the wireframe view
- `v` hides the toolpath lines and `Shift+V` hides the tool lines; pressing again shows them
