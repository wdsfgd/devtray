# DevTray Specification

## Feature: Task Grouping & Drag-and-Drop

### 1. Data Model
- **Schema**: Use Flat Tagging. The `Task` struct gains an optional `Group string` field.
- **Persistence**: JSON parsing should natively support this without breaking older config files (which will default to an empty string `""` for Group, meaning "Uncategorized").

### 2. System Tray (AppIndicator) UI
- **Hierarchy**: The tray menu will display Group names as primary menu items.
- **Submenus**: Hovering over a Group name opens a submenu containing:
  - `▶️ Start All` (Starts all tasks belonging to this group in parallel)
  - `🛑 Stop All` (Stops all tasks belonging to this group in parallel)
  - `---` (Separator)
  - Individual tasks belonging to the group (using `CheckMenuItem` to indicate running status).
- **Uncategorized Tasks**: Tasks with no group (`Group == ""`) will be listed directly in the main tray menu beneath the group folders.

### 3. Main Window (GTK3) UI
- **List Box**: The custom Box layout will be migrated to a `GtkListBox`.
- **Headers**: Grouping will be displayed using list box header functions to render visual separators / bold text for Group names.
- **Drag and Drop**: The `GtkListBox` will support drag-and-drop reordering. When a task is dropped, its position in the list is updated, and if it is dropped under a different Group header, its `Group` field is updated and saved.
- **Functionality**: Existing functional buttons (Play/Stop with native icons, Edit, Delete) will remain intact on each task row within the List Box.
