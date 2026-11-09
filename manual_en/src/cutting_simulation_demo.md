# Cutting Simulation

Load a product shape (STL), submit and run a cutting simulation with the visible product shape as the machining target, and check the result over time on top of the product shape. For key and mouse controls, see [Screen and Controls](./screen_and_controls.md). For known issues, see [Known Issues and Troubleshooting](./known_issues.md).

## Current Scope

- Tools: flat end mill and ball end mill (the default is a ball end mill D10). The tool length is not specified by default, and stock touched above the tool is also removed. When a tool length is specified, stock above the tool length remains
- Machining: 3-axis scanline machining (the defaults are climb cutting, stepover 2 mm, and feed rate 1200 mm/min). CL points are calculated along each scanline at intervals no longer than the tool radius, and the ends where the tool touches the shape are found to the toolpath tolerance. Points are added where needed so that the tool does not cut into the product shape beyond the toolpath tolerance between CL points, and at a step in the CL the tool keeps the height of the higher side before going down. The cut direction is selected from climb, conventional, and zigzag (starting with climb and alternating the direction on each line). The spindle is assumed to turn clockwise. The scanline direction is set by an angle counterclockwise from the X axis (the default is 0°, scanlines along X). Where the scanlines run along a steep wall, the spacing between passes widens on the wall and leaves more uncut stock, so choose an angle that crosses the main walls
- Stock: a box that adds margins outside each face (-X, +X, -Y, +Y, top, bottom) of the box around the product shape (the default margin is 0, the box around the product shape itself)
- Stock precision: the default is 0.05 mm (it sets how fine the stock surface is; with a D10 tool, the grid spacing is about 1.4 mm)
- Snapshot controls: step, rewind, scrub, and playback
- Display modes: solid and wireframe

The tool and machining conditions are entered in the operation settings window, and the stock in the sequence settings window. They are submitted with "Submit" in the operation list at the bottom of the screen. The operation list appears when you open a machining sequence with "Machining" → "Open machining sequence" in the menu bar. These screens are separate from the display settings (the settings panel, Shift+S).

## Steps

1. Start RedRing ([Getting Started](./getting_started.md)).
2. Choose an STL file with "File" → "Load…" in the menu bar, or drop an STL file onto the window, to load the shape. The loaded shape becomes a visible product shape and the machining target.
3. Open the machining sequence with "Machining" → "Open machining sequence" in the menu bar. The operation list appears at the bottom of the screen. Double-click an operation row to open the operation settings window, and enter the tool and machining conditions. Enter the stock in the window opened with "Sequence settings" in the operation list.
4. Press "Submit" in the operation list. A cutting simulation of the visible product shape is submitted, and the job manager calculates it ("Job Manager" in [Getting Started](./getting_started.md)). The screen does not stop during the calculation, and the operation list shows the state of the calculation for the current conditions (waiting, calculating, completed, failed).
5. When the calculation has finished, press "Import cutting simulation" to import the latest cutting simulation that finished successfully with the current conditions. After changing the conditions or the visible product shape, nothing can be imported until you submit again.

`Shift+L` is the same as "Import cutting simulation" (only while a machining sequence is open).

After a successful import, the following appear.

- The product shape and the stock surface, the toolpath lines, and the tool lines
- The progress bar at the top left
- The current frame information in the window title

Importing does not change the camera (direction and zoom), so you can compare the same place after importing again with other conditions. To fit the camera to the shapes again, press `r` (only when there is no visible product shape, the camera is fitted to the stock on import).

## Controls

### Opening and Closing the Machining Sequence ("Machining")

| "Machining" item | Key | Contents |
|---|---|---|
| Open machining sequence | | Opens the machining sequence and shows the operation list at the bottom of the screen. While it is open, the CL of the selected operation is shown over the product shape, and while the operation list is shown, the calculation state is read |
| Close machining sequence | | Closes the machining sequence. The operation list, the operation settings and sequence settings windows, the CL display, and the cutting animation are closed, and the memory they used is released. Jobs being calculated are continued by the job manager |
| Operation list | `Shift+M` | While a machining sequence is open, shows or hides the operation list. Hiding the operation list keeps the machining sequence open (the CL, the cutting animation, and the settings windows stay as they are) |
| Operation settings | | While a machining sequence is open, opens or closes the settings window of the selected operation |
| Sequence settings | | While a machining sequence is open, opens or closes the sequence settings window |

- For now, the first machining sequence of the model file is opened (choosing a machining sequence to open comes in a later version)
- After opening a model file or creating a new one, no machining sequence is open
- While no machining sequence is open, `Shift+M` and `Shift+L` do nothing

### Operation List (Bottom of the Screen)

- The toolbar at the top shows the sequence name, "Sequence settings" and "Operation settings" (open or close the windows), and the list operations ("Add", "Duplicate", "Remove", "▲", "▼")
- Below it are the calculation buttons. Except for "Submit the whole sequence", they act on the selected operation
  - "Submit": submits the selected operation. When the space model has the CL of the current conditions and no calculation of the current conditions remains (no job, failed, or canceled), only the cutting simulation is calculated from that CL, without toolpath generation (for example after the job manager's results have been deleted, or after opening the model file on another PC). If the toolpath generation result for the current conditions is still in the job manager, the cutting simulation is put under it. Otherwise, it is submitted from toolpath generation. The result line shows which was submitted
  - "Redo": deletes the CL of the selected operation from the space model and submits again from toolpath generation. If the cutting animation of that operation is shown, it is closed (if it cannot be submitted, the CL is not deleted)
  - "Submit the whole sequence": submits all operations of the sequence in machining order. The conditions of all operations are checked first; if any operation has an error, nothing is submitted, and the name of that operation and the reason are shown. Operations whose calculation for the current conditions is waiting, calculating, or calculated are not submitted again (if the calculation status has not been read yet, nothing is submitted and a message says so). For each operation submitted, whether only the cutting simulation is calculated is decided in the same way as "Submit"
  - "Import CL": imports only the CL of the toolpath generation that finished successfully with the current conditions of the selected operation into the space model. The cutting animation is not opened. It can be imported even before the cutting simulation has finished
  - "Import cutting simulation": imports and shows the cutting simulation that finished successfully with the current conditions of the selected operation, and imports its CL into the space model
  - "Open job manager": opens the job manager window
  - Below the buttons, the result of the last operation (submit or import), the state of the calculation for the current conditions of the selected operation, and whether the CL in the space model is the result of the current conditions (not imported yet, result of the current conditions, result of earlier conditions) are shown. The state is read again every second
  - If the conditions of the selected operation or the sequence have errors, the errors are shown in red, and "Submit" and "Redo" cannot be pressed (they are also shown below the fields to fix in the settings windows)
  - For now, the cutting simulation of every operation starts from the sequence stock (starting from the stock left by the previous operation comes in a later version)
- The operation table lists the operations of the machining sequence in machining order. The columns are No., Name, Stage, Machining type, Tool, and Status. The status is the calculation state (not submitted, waiting, calculating, calculated, imported, failed, canceled). "Imported" means the CL in the space model is the result of the current conditions
  - Click a row to select it, and double-click it to open the operation settings window. Right-click shows a menu with operation settings, Submit, Redo, Import CL, Import cutting simulation, Add, Duplicate, Remove, and moving earlier or later (the right-clicked row is selected first)
  - When you select an operation, its CL in the space model is shown over the product shape (the job manager is not asked). A CL that is not the result of the current conditions is shown in gray. No CL is shown for an operation whose CL has not been imported
  - The CL is shown only while the machining sequence is open. It is not shown just by opening a model file, and it disappears when the machining sequence is closed
  - The cutting animation is shown only while the imported operation is selected. Selecting another operation closes the cutting animation and shows that operation's CL (press "Import cutting simulation" to see it again)
  - "Add": adds a new operation (default conditions) right after the selected one
  - "Duplicate": adds an operation with the same conditions right after the selected one (calculation results and the imported CL are not carried over)
  - "Remove": removes the selected operation and its imported CL. It cannot be pressed when there is only one operation
  - "▲" / "▼": moves the selected operation earlier or later in the machining order
- Drag the top edge of the operation list to change its height

### Operation Settings Window

- Edits the conditions of the selected operation. If you select another operation in the operation list while it is open, it switches to that operation's conditions. The window title shows the operation name
- "Machining conditions" page: name (applied when you click outside the field or press `Enter`), stage (rough, semi-finish, finish), machining type (only scanline for now), tool (type, diameter, length), and scanline conditions (cut direction, stepover, feed rate, scan angle, tool range). The toolpath tolerance (0.001 mm) is shown only and cannot be changed
- The scanline conditions section shows the upper limit of the cusp height, the height of the ridges left between neighboring passes on a flat surface (the spacing between passes is at most the stepover, because the machining range is divided equally). For a ball end mill it is set by the diameter and the stepover (for example 0.1010 mm with diameter 10 mm and stepover 2 mm); for a flat end mill it is 0. It changes on inclined surfaces, so it is a guide value. It is not shown when the stepover exceeds the tool diameter
- "Region and target" page: the target (the faces of the visible product shape; the same for all operations of the sequence) and the machining region (for now, the XY extent of the target; specifying a region comes in a later version)
- If a condition has an error, it is shown in red below the field to fix, and "Submit" and "Redo" in the operation list cannot be pressed. Examples: the precision is finer than the toolpath tolerance; the precision is too fine for the stock size (the finest precision that can be submitted is shown); the stepover exceeds the tool diameter (the tool diameter is shown). Error messages are shown in the display language (Japanese or English). Failures that conditions cannot fix, such as running a job or importing a result, are also shown with the reason in the display language. Detailed reasons are kept in the log (in English)
- Importing (Import CL or Import cutting simulation) copies the calculated CL (toolpath) into the space model, so that it can be saved in the model file (this is an unsaved change). After the conditions change, the CL in the space model becomes "result of earlier conditions", and importing the current result replaces it
- While the cutting animation of the selected operation is shown, "Conditions of the displayed result" at the bottom of the window shows the conditions used for that calculation (tool, machining, tolerance, stock size, precision)

### Sequence Settings Window

- Edits the sequence name (shown only) and the stock (margins, precision). The stock is the same for all operations of the sequence. Changing it makes all operations of the sequence need recalculation
- Stock errors are shown in red below the fields

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
