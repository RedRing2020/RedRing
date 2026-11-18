# Tool Library

Register tools (tool sets) in a tool library and choose the tool of each operation from the library. You do not need to enter the same tool for every operation, and when you change a tool in the library, you can find and update the operations that use it.

## Tool library files

- A tool library is a ledger file of tool sets (extension `.rrtl`). One library can be used for machining on several machine tools. You can have any number of libraries
- A tool set has a name and a tool (ball end mill or flat end mill, diameter, tool length). Names and tool values may be the same as other tool sets (tool sets are told apart by a number decided when they are created)
- The list of libraries in use is saved in the app settings of each PC and is used the next time the app starts
- Library files are not stored in model files. An operation keeps a reference to the chosen tool set and a copy of the tool values used for calculation. On a PC without the library file, the operations in a model file can still be calculated as they are

### Where to put the files

- On one PC (standalone): put the file in a folder on that PC
- On several PCs (multiple clients): put the file in a network folder (shared folder) and open the same file on each PC. All PCs use the same tool sets
- To let only certain people change a shared library, restrict who can write with the folder permissions. On PCs without write permission, the library is "read-only": tool sets can be chosen but not changed

## Creating and opening libraries

1. Open the window with Machining > Tool library… on the menu bar (it can be opened without opening a machining sequence)
2. To create a library, press "New…" and choose the folder and the file name. The file name becomes the library name. A library is not created over an existing file
3. To use a library made by someone else (such as a file in a shared folder), press "Open…" and choose the file. It is only added to the list; the file is not copied
4. To stop using a library, select it and press "Remove from list". It is removed from the list (the file is not deleted)
5. To make another library from the current one, press "Duplicate as another library…" and choose the folder and the file name. All tool sets are copied, and the copy is a different library from the original (operations keep referring to the original library)

The following marks may appear next to a library name. Select the library to see the reason in the window.

| Mark | Meaning |
|---|---|
| read-only | No permission to write to the file. Tool sets can be chosen but not changed |
| unreadable | The file cannot be read because it is not found, broken, or saved by a newer version of the app. The contents read before, if any, are used read-only |
| same library | The same library as an earlier file in the list (for example, a copied file). This file is not used |

## Registering and editing tool sets

- Press "Add" to add a tool set, enter the name, type, diameter, and tool length in the fields below the table, and press "Apply". If an entry is invalid, the reason is shown below the field and "Apply" cannot be pressed
- "Duplicate" adds a copy of the selected tool set, "Delete" removes it, and "Up" and "Down" reorder the tool sets
- Each press writes to the library file. Changes written to the same library on other PCs are read again before writing and kept
- When you select a tool set, the number of operations in the open model that use it is shown, so you can check which operations are affected before deleting or changing it

### Finding and sorting

- Use the fields above the table to filter by part of the name, tool type, and diameter. The diameter filter shows only tool sets with the entered diameter
- Click a column header to sort by that column (each click switches between ascending, descending, and the library order). Filtering and sorting only change the view, not the order in the library. "Up" and "Down" cannot be used while filtering or sorting. "Clear filter and sort" restores the view
- When another tool set has the same tool values (type, diameter, corner R, tool length), a mark appears in the "Same values" column. "Same values only" shows only those, which helps to find tool sets registered twice

### When another PC is writing

- While another PC is writing to the same library, you cannot write to it. The PC and user writing are shown at the bottom of the window and on the status bar. Wait a moment and try again
- If the PC that was writing stopped and its write mark (the lock file `<file name>.lock`) remains for more than 5 minutes, the app shows the PC and user and asks whether to remove the mark and write. Press "Yes" only after confirming that no other PC is actually writing

## Choosing the tool of an operation

1. Select an operation in the operation list and open the operation settings dialog
2. Under "Tool" on the "Machining conditions" page, choose a library and then a tool set from the list. Enter part of a name in "Filter" to narrow the list
3. Press "Apply" to make it the tool of the operation. Choosing a tool set with the same tool values as now does not require recalculation

There are no fields to enter tool values directly. To add or change tools, open the library window with "Tool library…". A new operation takes over the tool of the selected operation.

## Tool states

The "Tool" section of the operation settings and the tool column of the operation list show the state of the operation's tool compared with the library. Operations can be calculated in any state (calculation uses the copy of the tool values kept by the operation).

| State (mark in the operation list) | Meaning | What you can do |
|---|---|---|
| Same as the library (no mark) | The tool values are the same as the chosen tool set | ― |
| Differs from the library (differs) | The tool set was changed in the library | "Match the library" takes the library values (recalculation is needed) |
| Not in the library (not in library) | The chosen tool set was removed from the library | "Add to the library" registers the operation's tool in the library again |
| Library not found (library not found) | The library of the chosen tool set is not in the list of libraries in use or cannot be read | Open the library file with "Tool library…", or register the tool in the selected library with "Add to the library" |
| Not from a library (not registered) | An operation from a model file of an earlier version, or one created before choosing from a library | "Add to the library" registers the tool in the selected library |

## When a library is changed

- Changing a tool set in a library does not change the operations of the open model automatically, so that opening a saved model does not discard its calculation results. Operations that use the changed tool set become "Differs from the library"
- To update all operations of the sequence, press "Match tools to the libraries" in the operation list. Operations whose tool values change need recalculation, and their number is shown on the status bar

## Opening a model file on another PC

- If the same library file is open (the same file in a shared folder, or a copied file), the operations are "Same as the library" as they are. Libraries are told apart by a number inside the library, not by the file location, so the folder may differ between PCs
- On a PC without the library file, the operations become "Library not found", but they can still be calculated as they are
