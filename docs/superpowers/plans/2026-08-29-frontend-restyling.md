# DevTray Frontend Restyling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restyle DevTray's Slint frontend to a clean, modern, non-slop Linear/Raycast-inspired developer tool UI.

**Architecture:** Update theme color tokens and typography in `ui/theme.slint`, refine the 2-row layout and action controls in `ui/task_card.slint`, streamline header and action toolbars in `ui/main_window.slint`, and align modal dialogs (`ui/task_dialog.slint`, `ui/confirm_dialog.slint`, `ui/log_viewer.slint`) with unified dark zinc tokens and 6px/8px radii.

**Tech Stack:** Slint UI (1.17.1), Rust 2021, Cargo

## Global Constraints

- Deep zinc palette: background `#09090b`, surface `#121215`, card `#141418`, hover `#1c1c22`, border `#27272a`, focus `#3f3f46`.
- Typography: primary `#f4f4f5`, secondary `#a1a1aa`, muted `#71717a`.
- Semantics: emerald `#10b981` active dot & pill, subdued zinc `#52525b` idle dot, rose `#ef4444` destructive/delete.
- Strict 6px radius for buttons/inputs, 8px radius for cards and modal windows.
- No placeholder or incomplete code blocks.

---

### Task 1: Update Theme Tokens & Color Palette in `ui/theme.slint`

**Files:**
- Modify: `ui/theme.slint:1-41`

**Interfaces:**
- Consumes: None
- Produces: `Theme` global singleton with updated properties (`background`, `surface`, `card-bg`, `card-hover`, `border`, `border-light`, `input-bg`, `text-primary`, `text-secondary`, `text-muted`, `accent`, `accent-hover`, `accent-pressed`, `running`, `running-bg`, `running-border`, `stopped`, `stopped-bg`, `stopped-border`, `danger`, `danger-hover`, `danger-pressed`, `terminal-bg`, `terminal-text`, `modal-backdrop`).

- [ ] **Step 1: Replace `ui/theme.slint` with the new design tokens**

Update `ui/theme.slint`:
```slint
export global Theme {
    // Window & Surfaces
    out property <color> background: #09090b;
    out property <color> surface: #121215;
    out property <color> card-bg: #141418;
    out property <color> card-hover: #1c1c22;
    out property <color> border: #27272a;
    out property <color> border-light: #3f3f46;
    out property <color> input-bg: #0d0d10;

    // Typography
    out property <color> text-primary: #f4f4f5;
    out property <color> text-secondary: #a1a1aa;
    out property <color> text-muted: #71717a;

    // Accents & Actions
    out property <color> accent: #3b82f6;
    out property <color> accent-hover: #60a5fa;
    out property <color> accent-pressed: #2563eb;

    // Status
    out property <color> running: #10b981;
    out property <color> running-bg: #064e3b;
    out property <color> running-border: #059669;
    out property <color> stopped: #52525b;
    out property <color> stopped-bg: #18181b;
    out property <color> stopped-border: #27272a;

    // Danger / Destructive
    out property <color> danger: #ef4444;
    out property <color> danger-hover: #dc2626;
    out property <color> danger-pressed: #b91c1c;

    // Terminal
    out property <color> terminal-bg: #09090b;
    out property <color> terminal-text: #d4d4d8;

    // Modals
    out property <color> modal-backdrop: #000000a0;
}
```

- [ ] **Step 2: Run `cargo check` to verify theme syntax**

Run: `cargo check`  
Expected: PASS with no compilation errors.

- [ ] **Step 3: Commit theme updates**

```bash
git add ui/theme.slint
git commit -m "style(ui): update design tokens to Linear zinc palette"
```

---

### Task 2: Restyle `TaskCard` Component (`ui/task_card.slint`)

**Files:**
- Modify: `ui/task_card.slint:1-228`

**Interfaces:**
- Consumes: `Theme` tokens
- Produces: `TaskCard` component and `TaskItem` struct with streamlined 2-row layout, command preview, subtle reorder controls, and ghost action buttons.

- [ ] **Step 1: Rewrite `ui/task_card.slint` with clean 2-row layout and ghost buttons**

Update `ui/task_card.slint`:
```slint
import { Theme } from "theme.slint";

export struct TaskItem {
    id: string,
    name: string,
    command: string,
    working_directory: string,
    group: string,
    is_running: bool,
    can_move_up: bool,
    can_move_down: bool,
}

export component TaskCard inherits Rectangle {
    in property <TaskItem> task;
    callback toggle_task();
    callback show_logs();
    callback edit_task();
    callback delete_task();
    callback move_up();
    callback move_down();

    height: 56px;
    background: cardTouch.has-hover ? Theme.card-hover : Theme.card-bg;
    border-color: cardTouch.has-hover ? Theme.border-light : Theme.border;
    border-width: 1px;
    border-radius: 8px;

    cardTouch := TouchArea {}

    HorizontalLayout {
        padding-left: 10px;
        padding-right: 10px;
        spacing: 10px;

        // 1. Move Up / Down Buttons (subtle chevrons)
        VerticalLayout {
            alignment: center;
            width: 14px;
            spacing: 2px;

            Rectangle {
                width: 14px;
                height: 12px;
                background: (btnUp.has-hover && root.task.can_move_up) ? #27272a : transparent;
                border-radius: 3px;

                btnUp := TouchArea {
                    enabled: root.task.can_move_up;
                    mouse-cursor: root.task.can_move_up ? pointer : default;
                    clicked => { root.move_up(); }
                }

                Text {
                    text: "▲";
                    font-size: 7px;
                    color: root.task.can_move_up ? (btnUp.has-hover ? Theme.text-primary : Theme.text-muted) : #27272a;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }

            Rectangle {
                width: 14px;
                height: 12px;
                background: (btnDown.has-hover && root.task.can_move_down) ? #27272a : transparent;
                border-radius: 3px;

                btnDown := TouchArea {
                    enabled: root.task.can_move_down;
                    mouse-cursor: root.task.can_move_down ? pointer : default;
                    clicked => { root.move_down(); }
                }

                Text {
                    text: "▼";
                    font-size: 7px;
                    color: root.task.can_move_down ? (btnDown.has-hover ? Theme.text-primary : Theme.text-muted) : #27272a;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
        }

        // 2. Status Dot
        VerticalLayout {
            alignment: center;
            width: 10px;

            Rectangle {
                width: 8px;
                height: 8px;
                border-radius: 4px;
                background: root.task.is_running ? Theme.running : Theme.stopped;
            }
        }

        // 3. Task Name & Command Subtitle (2-row stack)
        VerticalLayout {
            alignment: center;
            spacing: 2px;
            horizontal-stretch: 1;

            Text {
                text: root.task.name;
                font-size: 13px;
                font-weight: 700;
                color: Theme.text-primary;
                overflow: elide;
            }

            Text {
                text: root.task.command != "" ? "$ " + root.task.command : (root.task.working_directory != "" && root.task.working_directory != "." ? root.task.working_directory : "default");
                font-size: 10px;
                font-family: "DejaVu Sans Mono, monospace";
                color: Theme.text-muted;
                overflow: elide;
            }
        }

        // 4. Action Buttons
        VerticalLayout {
            alignment: center;

            HorizontalLayout {
                spacing: 5px;
                alignment: center;

                // Start/Stop Toggle
                Rectangle {
                    width: 28px;
                    height: 26px;
                    background: root.task.is_running ?
                        (btnToggle.pressed ? #271414 : (btnToggle.has-hover ? #381a1a : #1f1212)) :
                        (btnToggle.pressed ? #0e2417 : (btnToggle.has-hover ? #153622 : #0f2418));
                    border-color: root.task.is_running ? #7f1d1d : #065f46;
                    border-width: 1px;
                    border-radius: 5px;

                    btnToggle := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.toggle_task(); }
                    }

                    Text {
                        text: root.task.is_running ? "⏹" : "▶";
                        font-size: 10px;
                        color: root.task.is_running ? #f87171 : Theme.running;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }

                // Logs
                Rectangle {
                    width: 38px;
                    height: 26px;
                    background: btnLogs.pressed ? #18181b : (btnLogs.has-hover ? #27272a : transparent);
                    border-color: btnLogs.has-hover ? #3f3f46 : #27272a;
                    border-width: 1px;
                    border-radius: 5px;

                    btnLogs := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.show_logs(); }
                    }

                    Text {
                        text: "Logs";
                        font-size: 11px;
                        font-weight: 500;
                        color: btnLogs.has-hover ? Theme.text-primary : Theme.text-secondary;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }

                // Edit
                Rectangle {
                    width: 36px;
                    height: 26px;
                    background: btnEdit.pressed ? #18181b : (btnEdit.has-hover ? #27272a : transparent);
                    border-color: btnEdit.has-hover ? #3f3f46 : #27272a;
                    border-width: 1px;
                    border-radius: 5px;

                    btnEdit := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.edit_task(); }
                    }

                    Text {
                        text: "Edit";
                        font-size: 11px;
                        font-weight: 500;
                        color: btnEdit.has-hover ? Theme.text-primary : Theme.text-secondary;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }

                // Delete
                Rectangle {
                    width: 34px;
                    height: 26px;
                    background: btnDelete.pressed ? #3b1111 : (btnDelete.has-hover ? #2b1212 : transparent);
                    border-color: btnDelete.has-hover ? #7f1d1d : #27272a;
                    border-width: 1px;
                    border-radius: 5px;

                    btnDelete := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.delete_task(); }
                    }

                    Text {
                        text: "Del";
                        font-size: 11px;
                        font-weight: 500;
                        color: btnDelete.has-hover ? #f87171 : Theme.text-secondary;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Run `cargo check` to verify Slint compilation**

Run: `cargo check`  
Expected: PASS

- [ ] **Step 3: Commit TaskCard updates**

```bash
git add ui/task_card.slint
git commit -m "style(ui): streamline TaskCard with 2-row command preview and ghost buttons"
```

---

### Task 3: Restyle `MainWindow` Component (`ui/main_window.slint`)

**Files:**
- Modify: `ui/main_window.slint:1-390`

**Interfaces:**
- Consumes: `Theme`, `TaskCard`, `LogViewer`, `TaskDialog`, `ConfirmDialog`
- Produces: `MainWindow` component with clean header, group tags, active status pill, and bottom toolbar.

- [ ] **Step 1: Update `ui/main_window.slint` layout and styles**

Update `ui/main_window.slint`:
```slint
import { Theme } from "theme.slint";
import { TaskItem, TaskCard } from "task_card.slint";
import { LogViewer } from "log_viewer.slint";
import { TaskDialog } from "task_dialog.slint";
import { ConfirmDialog } from "confirm_dialog.slint";
import { ScrollView } from "std-widgets.slint";

export { TaskItem, Theme }

export component MainWindow inherits Window {
    title: "DevTray";
    min-width: 380px;
    min-height: 480px;
    preferred-width: 440px;
    preferred-height: 580px;
    background: Theme.background;

    // Model properties
    in property <[TaskItem]> tasks: [];
    in property <int> running_count: 0;

    // Log viewer properties
    in-out property <string> log_viewer_text: "";
    in-out property <string> log_viewer_task_name: "";
    in-out property <bool> log_viewer_open: false;

    // Task dialog properties
    in-out property <bool> task_dialog_open: false;
    in-out property <bool> task_dialog_is_edit: false;
    in-out property <string> task_dialog_id: "";
    in-out property <string> task_dialog_name: "";
    in-out property <string> task_dialog_command: "";
    in-out property <string> task_dialog_working_dir: ".";
    in-out property <string> task_dialog_group: "";
    in-out property <string> task_dialog_error: "";

    // Delete confirmation dialog properties
    in-out property <bool> delete_dialog_open: false;
    in-out property <string> delete_task_id: "";
    in-out property <string> delete_task_name: "";

    // Quit confirmation dialog properties
    in-out property <bool> quit_dialog_open: false;

    // Callbacks exposed to Rust
    callback toggle_task(string /* task_id */);
    callback show_logs(string /* task_id */, string /* task_name */);
    callback edit_task(string /* task_id */);
    callback delete_task(string /* task_id */);
    callback move_task(string /* task_id */, int /* direction */);
    callback save_task(string /* id */, string /* name */, string /* command */, string /* working_dir */, string /* group */);
    callback confirm_delete_task(string /* id */);
    callback start_all();
    callback stop_all();
    callback quit_app();
    callback clear_logs();

    // Main layout
    VerticalLayout {
        padding: 12px;
        spacing: 10px;

        // Top Header
        HorizontalLayout {
            height: 28px;
            alignment: space-between;

            // App Title & Status Badge
            HorizontalLayout {
                spacing: 8px;
                alignment: start;

                Text {
                    text: "DevTray";
                    font-size: 16px;
                    font-weight: 800;
                    color: Theme.text-primary;
                    vertical-alignment: center;
                }

                if root.running_count > 0 : Rectangle {
                    height: 20px;
                    background: Theme.running-bg;
                    border-color: Theme.running-border;
                    border-width: 1px;
                    border-radius: 10px;
                    y: 4px;

                    HorizontalLayout {
                        padding-left: 8px;
                        padding-right: 8px;
                        spacing: 5px;
                        alignment: center;

                        Rectangle {
                            width: 6px;
                            height: 6px;
                            border-radius: 3px;
                            background: Theme.running;
                            y: 7px;
                        }

                        Text {
                            text: root.running_count + " active";
                            font-size: 10px;
                            font-weight: 700;
                            color: Theme.running;
                            vertical-alignment: center;
                        }
                    }
                }
            }

            // + Add Task Button
            Rectangle {
                width: 90px;
                height: 28px;
                background: btnAddTouch.pressed ? Theme.accent-pressed : (btnAddTouch.has-hover ? Theme.accent-hover : Theme.accent);
                border-radius: 6px;

                btnAddTouch := TouchArea {
                    mouse-cursor: pointer;
                    clicked => {
                        root.task_dialog_id = "";
                        root.task_dialog_name = "";
                        root.task_dialog_command = "";
                        root.task_dialog_working_dir = ".";
                        root.task_dialog_group = "";
                        root.task_dialog_error = "";
                        root.task_dialog_is_edit = false;
                        root.task_dialog_open = true;
                    }
                }

                Text {
                    text: "+ Add Task";
                    color: #ffffff;
                    font-size: 11px;
                    font-weight: 700;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
        }

        // Task List Container
        Rectangle {
            vertical-stretch: 1;
            clip: true;

            // Empty State
            if root.tasks.length == 0 : VerticalLayout {
                alignment: center;
                spacing: 6px;

                Text {
                    text: "No tasks configured";
                    color: Theme.text-secondary;
                    font-size: 13px;
                    font-weight: 600;
                    horizontal-alignment: center;
                }

                Text {
                    text: "Click '+ Add Task' above to start managing background services.";
                    color: Theme.text-muted;
                    font-size: 11px;
                    horizontal-alignment: center;
                }
            }

            // Scrollable Task List
            if root.tasks.length > 0 : ScrollView {
                VerticalLayout {
                    spacing: 6px;
                    padding-right: 4px;

                    for task[idx] in root.tasks : VerticalLayout {
                        spacing: 6px;

                        // Section Header
                        if idx == 0 || root.tasks[idx - 1].group != task.group : Rectangle {
                            height: 24px;
                            background: transparent;

                            HorizontalLayout {
                                alignment: start;
                                spacing: 8px;

                                Text {
                                    text: task.group != "" ? task.group : "UNCATEGORIZED";
                                    color: Theme.text-muted;
                                    font-size: 10px;
                                    font-weight: 700;
                                    letter-spacing: 0.5px;
                                    vertical-alignment: bottom;
                                }
                            }
                        }

                        TaskCard {
                            task: task;

                            toggle_task => {
                                root.toggle_task(task.id);
                            }

                            show_logs => {
                                root.log_viewer_task_name = task.name;
                                root.log_viewer_open = true;
                                root.show_logs(task.id, task.name);
                            }

                            edit_task => {
                                root.task_dialog_id = task.id;
                                root.task_dialog_name = task.name;
                                root.task_dialog_command = task.command;
                                root.task_dialog_working_dir = task.working_directory;
                                root.task_dialog_group = task.group;
                                root.task_dialog_error = "";
                                root.task_dialog_is_edit = true;
                                root.task_dialog_open = true;
                                root.edit_task(task.id);
                            }

                            delete_task => {
                                root.delete_task_id = task.id;
                                root.delete_task_name = task.name;
                                root.delete_dialog_open = true;
                            }

                            move_up => {
                                root.move_task(task.id, -1);
                            }

                            move_down => {
                                root.move_task(task.id, 1);
                            }
                        }
                    }
                }
            }
        }

        // Bottom Action Bar
        Rectangle {
            height: 42px;
            background: Theme.surface;
            border-color: Theme.border;
            border-width: 1px;
            border-radius: 6px;

            HorizontalLayout {
                padding-left: 10px;
                padding-right: 10px;
                spacing: 8px;
                alignment: center;

                // Start All Button
                Rectangle {
                    width: 86px;
                    height: 28px;
                    background: btnStartAll.pressed ? #0e2417 : (btnStartAll.has-hover ? #153622 : #0f2418);
                    border-color: #065f46;
                    border-width: 1px;
                    border-radius: 5px;

                    btnStartAll := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.start_all(); }
                    }

                    Text {
                        text: "▶ Start All";
                        color: Theme.running;
                        font-size: 11px;
                        font-weight: 600;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }

                // Stop All Button
                Rectangle {
                    width: 86px;
                    height: 28px;
                    background: btnStopAll.pressed ? #271414 : (btnStopAll.has-hover ? #381a1a : #1f1212);
                    border-color: #7f1d1d;
                    border-width: 1px;
                    border-radius: 5px;

                    btnStopAll := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.stop_all(); }
                    }

                    Text {
                        text: "⏹ Stop All";
                        color: #f87171;
                        font-size: 11px;
                        font-weight: 600;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }

                Rectangle {
                    horizontal-stretch: 1;
                }

                // Quit Button
                Rectangle {
                    width: 52px;
                    height: 28px;
                    background: btnQuit.pressed ? #18181b : (btnQuit.has-hover ? #27272a : transparent);
                    border-color: btnQuit.has-hover ? #3f3f46 : #27272a;
                    border-width: 1px;
                    border-radius: 5px;

                    btnQuit := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.quit_dialog_open = true; }
                    }

                    Text {
                        text: "Quit";
                        color: Theme.text-secondary;
                        font-size: 11px;
                        font-weight: 500;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }
            }
        }
    }

    // Modal Dialogs
    TaskDialog {
        is_open <=> root.task_dialog_open;
        is_edit: root.task_dialog_is_edit;
        task_id: root.task_dialog_id;
        name_text <=> root.task_dialog_name;
        command_text <=> root.task_dialog_command;
        working_dir_text <=> root.task_dialog_working_dir;
        group_text <=> root.task_dialog_group;
        error_message <=> root.task_dialog_error;

        save(id, name, cmd, dir, group) => {
            root.save_task(id, name, cmd, dir, group);
        }
    }

    LogViewer {
        is_open <=> root.log_viewer_open;
        task_name: root.log_viewer_task_name;
        log_text: root.log_viewer_text;

        clear_view => {
            root.clear_logs();
        }
    }

    ConfirmDialog {
        is_open <=> root.delete_dialog_open;
        title: "Delete Task";
        message: "Delete task '" + root.delete_task_name + "'?";
        sub_message: "This will stop the task if running and remove it permanently.";
        confirm_text: "Delete";
        is_destructive: true;
        context_id: root.delete_task_id;

        confirmed(id) => {
            root.confirm_delete_task(id);
        }
    }

    ConfirmDialog {
        is_open <=> root.quit_dialog_open;
        title: "Quit DevTray";
        message: "Are you sure you want to quit?";
        sub_message: "This will stop all running tasks.";
        confirm_text: "Quit";
        is_destructive: true;

        confirmed(_) => {
            root.quit_app();
        }
    }
}
```

- [ ] **Step 2: Run `cargo check` to verify Slint compilation**

Run: `cargo check`  
Expected: PASS

- [ ] **Step 3: Commit MainWindow updates**

```bash
git add ui/main_window.slint
git commit -m "style(ui): restyle MainWindow header and toolbar with unified theme tokens"
```

---

### Task 4: Restyle `TaskDialog` & `ConfirmDialog` Modal Dialogs (`ui/task_dialog.slint`, `ui/confirm_dialog.slint`)

**Files:**
- Modify: `ui/task_dialog.slint:1-252`
- Modify: `ui/confirm_dialog.slint:1-168`

**Interfaces:**
- Consumes: `Theme` tokens
- Produces: `TaskDialog` and `ConfirmDialog` components with dark zinc styling, 8px radius, clean inputs, and ghost/accent buttons.

- [ ] **Step 1: Update `ui/task_dialog.slint`**

Update `ui/task_dialog.slint`:
```slint
import { Theme } from "theme.slint";
import { LineEdit } from "std-widgets.slint";

export component TaskDialog inherits Rectangle {
    in-out property <bool> is_open: false;
    in-out property <string> task_id: "";
    in-out property <string> name_text: "";
    in-out property <string> command_text: "";
    in-out property <string> working_dir_text: ".";
    in-out property <string> group_text: "";
    in-out property <string> error_message: "";
    in property <bool> is_edit: false;

    callback save(string /* id */, string /* name */, string /* command */, string /* working_dir */, string /* group */);
    callback cancel();

    visible: root.is_open;
    z: 100;
    x: 0;
    y: 0;
    width: 100%;
    height: 100%;
    background: Theme.modal-backdrop;

    // Intercept clicks on backdrop
    TouchArea {}

    // Centering Container
    VerticalLayout {
        alignment: center;
        padding: 16px;

        HorizontalLayout {
            alignment: center;

            dialog_box := Rectangle {
                width: min(root.width - 32px, 380px);
                background: Theme.card-bg;
                border-color: Theme.border;
                border-width: 1px;
                border-radius: 8px;
                clip: true;

                VerticalLayout {
                    padding: 16px;
                    spacing: 12px;

                    // Header Row
                    HorizontalLayout {
                        height: 24px;
                        alignment: space-between;

                        Text {
                            text: root.is_edit ? "Edit Task" : "Add Task";
                            color: Theme.text-primary;
                            font-size: 14px;
                            font-weight: 700;
                            vertical-alignment: center;
                        }

                        Rectangle {
                            width: 22px;
                            height: 22px;
                            border-radius: 11px;
                            background: closeBtnTouch.has-hover ? #27272a : transparent;

                            closeBtnTouch := TouchArea {
                                mouse-cursor: pointer;
                                clicked => {
                                    root.cancel();
                                    root.is_open = false;
                                    root.error_message = "";
                                }
                            }

                            Text {
                                text: "✕";
                                color: Theme.text-muted;
                                font-size: 11px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }

                    // Field 1: Name
                    VerticalLayout {
                        spacing: 4px;

                        Text {
                            text: "Name";
                            color: Theme.text-secondary;
                            font-size: 11px;
                            font-weight: 600;
                        }

                        nameInput := LineEdit {
                            height: 32px;
                            text <=> root.name_text;
                            placeholder-text: "e.g. Frontend Server";
                            edited => {
                                if root.error_message != "" && root.name_text != "" {
                                    root.error_message = "";
                                }
                            }
                        }
                    }

                    // Field 2: Command
                    VerticalLayout {
                        spacing: 4px;

                        Text {
                            text: "Command";
                            color: Theme.text-secondary;
                            font-size: 11px;
                            font-weight: 600;
                        }

                        cmdInput := LineEdit {
                            height: 32px;
                            text <=> root.command_text;
                            placeholder-text: "e.g. npm run dev";
                            edited => {
                                if root.error_message != "" && root.command_text != "" {
                                    root.error_message = "";
                                }
                            }
                        }
                    }

                    // Field 3: Working Directory
                    VerticalLayout {
                        spacing: 4px;

                        Text {
                            text: "Working Directory";
                            color: Theme.text-secondary;
                            font-size: 11px;
                            font-weight: 600;
                        }

                        dirInput := LineEdit {
                            height: 32px;
                            text <=> root.working_dir_text;
                            placeholder-text: ".";
                        }
                    }

                    // Field 4: Group
                    VerticalLayout {
                        spacing: 4px;

                        Text {
                            text: "Group (Optional)";
                            color: Theme.text-secondary;
                            font-size: 11px;
                            font-weight: 600;
                        }

                        groupInput := LineEdit {
                            height: 32px;
                            text <=> root.group_text;
                            placeholder-text: "e.g. Web";
                        }
                    }

                    // Error Message
                    if root.error_message != "" : Text {
                        text: root.error_message;
                        color: #f87171;
                        font-size: 11px;
                    }

                    // Action Buttons
                    HorizontalLayout {
                        height: 30px;
                        spacing: 8px;
                        alignment: end;

                        // Cancel Button
                        Rectangle {
                            width: 72px;
                            height: 30px;
                            background: cancelBtnTouch.pressed ? #18181b : (cancelBtnTouch.has-hover ? #27272a : transparent);
                            border-color: #27272a;
                            border-width: 1px;
                            border-radius: 6px;

                            cancelBtnTouch := TouchArea {
                                mouse-cursor: pointer;
                                clicked => {
                                    root.cancel();
                                    root.is_open = false;
                                    root.error_message = "";
                                }
                            }

                            Text {
                                text: "Cancel";
                                color: Theme.text-secondary;
                                font-size: 11px;
                                font-weight: 500;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }

                        // Save Button
                        Rectangle {
                            width: 76px;
                            height: 30px;
                            background: saveBtnTouch.pressed ? Theme.accent-pressed : (saveBtnTouch.has-hover ? Theme.accent-hover : Theme.accent);
                            border-radius: 6px;

                            saveBtnTouch := TouchArea {
                                mouse-cursor: pointer;
                                clicked => {
                                    if root.name_text == "" {
                                        root.error_message = "Task name cannot be empty";
                                    } else if root.command_text == "" {
                                        root.error_message = "Task command cannot be empty";
                                    } else {
                                        root.save(
                                            root.task_id,
                                            root.name_text,
                                            root.command_text,
                                            root.working_dir_text != "" ? root.working_dir_text : ".",
                                            root.group_text
                                        );
                                        root.is_open = false;
                                        root.error_message = "";
                                    }
                                }
                            }

                            Text {
                                text: "Save";
                                color: #ffffff;
                                font-size: 11px;
                                font-weight: 700;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Update `ui/confirm_dialog.slint`**

Update `ui/confirm_dialog.slint`:
```slint
import { Theme } from "theme.slint";

export component ConfirmDialog inherits Rectangle {
    in-out property <bool> is_open: false;
    in property <string> title: "Confirm";
    in property <string> message: "";
    in property <string> sub_message: "";
    in property <string> confirm_text: "OK";
    in property <bool> is_destructive: false;
    in-out property <string> context_id: "";

    callback confirmed(string /* context_id */);
    callback cancelled();

    visible: root.is_open;
    z: 100;
    x: 0;
    y: 0;
    width: 100%;
    height: 100%;
    background: Theme.modal-backdrop;

    // Intercept clicks on backdrop
    TouchArea {}

    // Centering Container
    VerticalLayout {
        alignment: center;
        padding: 16px;

        HorizontalLayout {
            alignment: center;

            dialog_box := Rectangle {
                width: min(root.width - 32px, 360px);
                background: Theme.card-bg;
                border-color: Theme.border;
                border-width: 1px;
                border-radius: 8px;
                clip: true;

                VerticalLayout {
                    padding: 16px;
                    spacing: 12px;

                    // Header
                    HorizontalLayout {
                        height: 24px;
                        alignment: space-between;

                        Text {
                            text: root.title;
                            color: Theme.text-primary;
                            font-size: 14px;
                            font-weight: 700;
                            vertical-alignment: center;
                        }

                        Rectangle {
                            width: 22px;
                            height: 22px;
                            border-radius: 11px;
                            background: closeBtnTouch.has-hover ? #27272a : transparent;

                            closeBtnTouch := TouchArea {
                                mouse-cursor: pointer;
                                clicked => {
                                    root.cancelled();
                                    root.is_open = false;
                                }
                            }

                            Text {
                                text: "✕";
                                color: Theme.text-muted;
                                font-size: 11px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }

                    // Body Message
                    VerticalLayout {
                        spacing: 4px;

                        if root.message != "" : Text {
                            text: root.message;
                            color: Theme.text-primary;
                            font-size: 13px;
                            font-weight: 600;
                            wrap: word-wrap;
                        }

                        if root.sub_message != "" : Text {
                            text: root.sub_message;
                            color: Theme.text-muted;
                            font-size: 11px;
                            wrap: word-wrap;
                        }
                    }

                    // Buttons
                    HorizontalLayout {
                        height: 30px;
                        spacing: 8px;
                        alignment: end;

                        // Cancel Button
                        Rectangle {
                            width: 68px;
                            height: 30px;
                            background: cancelBtnTouch.pressed ? #18181b : (cancelBtnTouch.has-hover ? #27272a : transparent);
                            border-color: #27272a;
                            border-width: 1px;
                            border-radius: 6px;

                            cancelBtnTouch := TouchArea {
                                mouse-cursor: pointer;
                                clicked => {
                                    root.cancelled();
                                    root.is_open = false;
                                }
                            }

                            Text {
                                text: "Cancel";
                                color: Theme.text-secondary;
                                font-size: 11px;
                                font-weight: 500;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }

                        // Confirm Button
                        Rectangle {
                            width: 76px;
                            height: 30px;
                            background: root.is_destructive ?
                                (confirmBtnTouch.pressed ? Theme.danger-pressed : (confirmBtnTouch.has-hover ? Theme.danger-hover : Theme.danger)) :
                                (confirmBtnTouch.pressed ? Theme.accent-pressed : (confirmBtnTouch.has-hover ? Theme.accent-hover : Theme.accent));
                            border-radius: 6px;

                            confirmBtnTouch := TouchArea {
                                mouse-cursor: pointer;
                                clicked => {
                                    root.confirmed(root.context_id);
                                    root.is_open = false;
                                }
                            }

                            Text {
                                text: root.confirm_text;
                                color: #ffffff;
                                font-size: 11px;
                                font-weight: 700;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 3: Run `cargo check` to verify Slint compilation**

Run: `cargo check`  
Expected: PASS

- [ ] **Step 4: Commit Modal Dialog updates**

```bash
git add ui/task_dialog.slint ui/confirm_dialog.slint
git commit -m "style(ui): restyle TaskDialog and ConfirmDialog with zinc surfaces and unified radii"
```

---

### Task 5: Restyle `LogViewer` Modal Component (`ui/log_viewer.slint`)

**Files:**
- Modify: `ui/log_viewer.slint:1-176`

**Interfaces:**
- Consumes: `Theme` tokens
- Produces: `LogViewer` component with `#09090b` terminal box, crisp monospaced output, and clean action toolbar.

- [ ] **Step 1: Update `ui/log_viewer.slint`**

Update `ui/log_viewer.slint`:
```slint
import { Theme } from "theme.slint";
import { ScrollView } from "std-widgets.slint";

export component LogViewer inherits Rectangle {
    in-out property <bool> is_open: false;
    in property <string> task_name: "";
    in property <string> log_text: "";
    callback clear_view();
    callback close();

    visible: root.is_open;
    z: 100;
    x: 0;
    y: 0;
    width: 100%;
    height: 100%;
    background: Theme.modal-backdrop;

    // Intercept clicks on backdrop
    TouchArea {}

    // Modal Dialog Box
    dialog_box := Rectangle {
        width: min(root.width - 24px, 560px);
        height: min(root.height - 30px, 440px);
        x: (root.width - self.width) / 2;
        y: (root.height - self.height) / 2;
        background: Theme.card-bg;
        border-color: Theme.border;
        border-width: 1px;
        border-radius: 8px;
        clip: true;

        VerticalLayout {
            padding: 14px;
            spacing: 10px;

            // Header
            HorizontalLayout {
                height: 24px;
                alignment: space-between;

                Text {
                    text: "Logs: " + (root.task_name != "" ? root.task_name : "Task");
                    color: Theme.text-primary;
                    font-size: 14px;
                    font-weight: 700;
                    vertical-alignment: center;
                    overflow: elide;
                    horizontal-stretch: 1;
                }

                Rectangle {
                    width: 22px;
                    height: 22px;
                    border-radius: 11px;
                    background: closeBtnTouch.has-hover ? #27272a : transparent;

                    closeBtnTouch := TouchArea {
                        mouse-cursor: pointer;
                        clicked => {
                            root.close();
                            root.is_open = false;
                        }
                    }

                    Text {
                        text: "✕";
                        color: Theme.text-muted;
                        font-size: 11px;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }
            }

            // Terminal Log Box
            Rectangle {
                vertical-stretch: 1;
                background: Theme.terminal-bg;
                border-color: Theme.border;
                border-width: 1px;
                border-radius: 6px;
                clip: true;

                ScrollView {
                    x: 8px;
                    y: 8px;
                    width: parent.width - 16px;
                    height: parent.height - 16px;

                    TextInput {
                        read-only: true;
                        single-line: false;
                        text: root.log_text;
                        color: Theme.terminal-text;
                        font-family: "DejaVu Sans Mono, monospace";
                        font-size: 11px;
                        wrap: word-wrap;
                    }
                }

                if root.log_text == "" : Text {
                    text: "(No logs recorded yet...)";
                    color: Theme.text-muted;
                    font-size: 11px;
                    font-italic: true;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }

            // Action Toolbar
            HorizontalLayout {
                height: 28px;
                spacing: 8px;

                // Clear View Button
                Rectangle {
                    width: 80px;
                    height: 28px;
                    background: clearBtnTouch.pressed ? #18181b : (clearBtnTouch.has-hover ? #27272a : transparent);
                    border-color: #27272a;
                    border-width: 1px;
                    border-radius: 5px;

                    clearBtnTouch := TouchArea {
                        mouse-cursor: pointer;
                        clicked => { root.clear_view(); }
                    }

                    Text {
                        text: "Clear Logs";
                        color: Theme.text-secondary;
                        font-size: 11px;
                        font-weight: 500;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }

                Rectangle {
                    horizontal-stretch: 1;
                }

                // Close Button
                Rectangle {
                    width: 64px;
                    height: 28px;
                    background: closeActionTouch.pressed ? #18181b : (closeActionTouch.has-hover ? #27272a : transparent);
                    border-color: #27272a;
                    border-width: 1px;
                    border-radius: 5px;

                    closeActionTouch := TouchArea {
                        mouse-cursor: pointer;
                        clicked => {
                            root.close();
                            root.is_open = false;
                        }
                    }

                    Text {
                        text: "Close";
                        color: Theme.text-secondary;
                        font-size: 11px;
                        font-weight: 500;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Run `cargo check` to verify Slint compilation**

Run: `cargo check`  
Expected: PASS

- [ ] **Step 3: Commit LogViewer updates**

```bash
git add ui/log_viewer.slint
git commit -m "style(ui): restyle LogViewer with crisp dark terminal and ghost controls"
```

---

### Task 6: Full Verification & Build Validation

**Files:**
- Test all components end-to-end via cargo test and binary build.

- [ ] **Step 1: Run full test suite**

Run: `cargo test`  
Expected: PASS (all tests pass)

- [ ] **Step 2: Run cargo build**

Run: `cargo build --bin devtray`  
Expected: Build succeeds with 0 errors.

- [ ] **Step 3: Commit full changes and verify git clean state**

```bash
git status
```
