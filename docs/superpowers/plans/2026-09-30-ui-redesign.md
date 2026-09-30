# DevTray UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign DevTray's Slint UI to remove generic AI-slop aesthetics and replace it with a refined Raycast/Linear Dark aesthetic featuring graphite surfaces, hairline micro-borders, quiet resting card state, and reveal-on-hover icon action toolbars.

**Architecture:** Update declarative Slint components starting from design tokens in `ui/theme.slint`, cascading into `ui/task_card.slint` (reveal on hover), `ui/main_window.slint` (header & footer), modal dialogs (`ui/task_dialog.slint`, `ui/confirm_dialog.slint`), and `ui/log_viewer.slint`. All changes are fully verified by `cargo test` and interactive UI checks.

**Tech Stack:** Rust, Slint declarative UI toolkit, `cargo test`.

## Global Constraints
- Window Dimensions: min-width 380px, min-height 480px, preferred 440px × 580px.
- Background: `#0d0f12`, Surface: `#101317`, Card: `#14171c`, Card Hover: `#1b2028`.
- Borders: hairline 1px `#20252e`, hover `#2c3442`.
- Action buttons reveal only on hover; resting card state has zero button clutter.
- Running: emerald `#10b981`, Stopped: slate `#475569`.
- Primary Button: graphite `#1c212b`, border `#30394a`, text `#f8fafc`.

---

### Task 1: Design Tokens & Theme Modernization

**Files:**
- Modify: `ui/theme.slint`

**Interfaces:**
- Produces: Updated `Theme` global tokens used by all Slint components (`background`, `surface`, `card-bg`, `card-hover`, `border`, `border-light`, `input-bg`, `text-primary`, `text-secondary`, `text-muted`, `accent`, `accent-hover`, `accent-border`, `accent-text`, `running`, `running-bg`, `running-border`, `stopped`, `danger`, `danger-hover-bg`, `terminal-bg`, `terminal-text`, `modal-backdrop`).

- [ ] **Step 1: Update `Theme` global in `ui/theme.slint`**

Replace `ui/theme.slint` with the new Raycast/Linear graphite token palette:
```slint
export global Theme {
    // Window & Surfaces
    out property <color> background: #0d0f12;
    out property <color> surface: #101317;
    out property <color> card-bg: #14171c;
    out property <color> card-hover: #1b2028;
    out property <color> border: #20252e;
    out property <color> border-light: #2c3442;
    out property <color> input-bg: #0f1217;

    // Typography
    out property <color> text-primary: #f8fafc;
    out property <color> text-secondary: #94a3b8;
    out property <color> text-muted: #64748b;

    // Primary Actions (Graphite sleek button)
    out property <color> accent: #1c212b;
    out property <color> accent-hover: #252c3a;
    out property <color> accent-pressed: #161a22;
    out property <color> accent-border: #30394a;
    out property <color> accent-text: #f8fafc;

    // Status Semantics
    out property <color> running: #10b981;
    out property <color> running-bg: #10b98114;
    out property <color> running-border: #10b98133;
    out property <color> stopped: #475569;
    out property <color> stopped-bg: #14171c;
    out property <color> stopped-border: #20252e;

    // Danger / Destructive
    out property <color> danger: #f87171;
    out property <color> danger-hover: #ef4444;
    out property <color> danger-pressed: #dc2626;
    out property <color> danger-hover-bg: #ef444418;
    out property <color> danger-surface: #311316;
    out property <color> danger-border: #631b21;

    // Terminal
    out property <color> terminal-bg: #0a0c0f;
    out property <color> terminal-text: #cbd5e1;
    out property <color> terminal-border: #1e232d;

    // Modals
    out property <color> modal-backdrop: #000000c0;
}
```

- [ ] **Step 2: Verify Slint syntax with cargo check**

Run: `cargo check`  
Expected: PASS with no Slint compilation errors.

- [ ] **Step 3: Commit**

```bash
git add ui/theme.slint
git commit -m "style(ui): update design tokens with Raycast/Linear graphite palette"
```

---

### Task 2: TaskCard Reveal-on-Hover Restyle

**Files:**
- Modify: `ui/task_card.slint`

**Interfaces:**
- Consumes: `Theme` tokens from Task 1.
- Produces: `TaskCard` component with resting state (no action buttons, quiet row) and hover state (revealing 4 ghost icon buttons: toggle `▶/⏹`, logs `>_`, edit `✎`, delete `✕`).

- [ ] **Step 1: Restructure `TaskCard` in `ui/task_card.slint`**

Update `TaskCard` to:
1. Height 54px, border-radius 6px.
2. Left: Muted chevrons (`#334155`, brightens to `#94a3b8` on hover), 7px status dot (`#10b981` or `#475569`).
3. Center: 2 rows (Task Name 13px weight 600, command 10.5px monospace muted slate).
4. Right:
   - When not hovered (`!cardTouch.has-hover`): empty or subtle status tag.
   - When hovered (`cardTouch.has-hover`): 4 compact ghost icon buttons:
     - Toggle: `root.task.is_running ? "⏹" : "▶"`, colored `#f87171` or `#34d399`.
     - Logs: `>_`, colored `#94a3b8` (hover `#f8fafc`).
     - Edit: `✎`, colored `#94a3b8` (hover `#f8fafc`).
     - Delete: `✕`, colored `#94a3b8` (hover `#f87171`).

- [ ] **Step 2: Verify Slint compilation and test suite**

Run: `cargo test --test slint_bridge_test`  
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add ui/task_card.slint
git commit -m "style(ui): restyle TaskCard with reveal-on-hover icon toolbar"
```

---

### Task 3: MainWindow Header & Footer Restyle

**Files:**
- Modify: `ui/main_window.slint`

**Interfaces:**
- Consumes: `Theme` tokens, `TaskCard`.
- Produces: Restyled `MainWindow` top bar, category headers, and 38px bottom toolbar.

- [ ] **Step 1: Restyle Top Header in `ui/main_window.slint`**

1. Title: "DevTray" in 15px font-weight 700 `#f8fafc`.
2. Active badge: Sleek graphite micro-pill (`#14171d`, border `#232832`, radius 10px, emerald dot + `N active` text in `#34d399`).
3. `+ New Task` button: 28px height, 6px radius, background `#1c212b`, border `#30394a`, hover `#252c3a`, text `#f8fafc`.

- [ ] **Step 2: Restyle Group Headers & Bottom Action Bar in `ui/main_window.slint`**

1. Section headers: 10px uppercase `#64748b`, letter-spacing 0.8px, clean margin.
2. Bottom action bar:
   - Height: 38px, background `Theme.surface` (`#101317`), top border 1px `Theme.border`, no side/bottom borders.
   - Ghost pill buttons for `Start All` (`▶ Start All`), `Stop All` (`⏹ Stop All`), and `Quit` with subtle hover tints instead of loud colored borders.

- [ ] **Step 3: Verify Slint compilation and test suite**

Run: `cargo test`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add ui/main_window.slint
git commit -m "style(ui): restyle MainWindow header, group headers, and bottom toolbar"
```

---

### Task 4: Modal Dialogs Alignment (TaskDialog & ConfirmDialog)

**Files:**
- Modify: `ui/task_dialog.slint`
- Modify: `ui/confirm_dialog.slint`

**Interfaces:**
- Consumes: `Theme` tokens.
- Produces: Cohesive modals for task editing and confirmation.

- [ ] **Step 1: Restyle `TaskDialog` in `ui/task_dialog.slint`**

1. Modal backdrop: `Theme.modal-backdrop` (`#000000c0`).
2. Dialog box: background `Theme.card-bg` (`#14171d`), border `Theme.border` (`#252b36`), radius 10px.
3. Form fields: input background `#0f1217`, border `#232832`, label `#94a3b8`.
4. Buttons: Cancel ghost button + Save graphite primary button (`#1c212b`, border `#30394a`, text `#f8fafc`).

- [ ] **Step 2: Restyle `ConfirmDialog` in `ui/confirm_dialog.slint`**

1. Dialog box: background `Theme.card-bg` (`#14171d`), border `Theme.border` (`#252b36`), radius 10px.
2. Destructive confirm button: `#311316` surface, `#631b21` border, `#fca5a5` text (hover `#3f171b`).
3. Non-destructive confirm: graphite primary button.

- [ ] **Step 3: Verify with cargo test**

Run: `cargo test`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add ui/task_dialog.slint ui/confirm_dialog.slint
git commit -m "style(ui): restyle TaskDialog and ConfirmDialog with graphite surface tokens"
```

---

### Task 5: Log Viewer Modernization

**Files:**
- Modify: `ui/log_viewer.slint`

**Interfaces:**
- Consumes: `Theme` tokens.
- Produces: Polished `LogViewer` modal with deep console and ghost controls.

- [ ] **Step 1: Restyle `LogViewer` in `ui/log_viewer.slint`**

1. Dialog container: background `Theme.card-bg` (`#14171d`), border `Theme.border` (`#252b36`), radius 10px.
2. Console display box: background `Theme.terminal-bg` (`#0a0c0f`), border `Theme.terminal-border` (`#1e232d`), text `Theme.terminal-text` (`#cbd5e1`).
3. Action buttons: ghost buttons for `Copy Logs`, `Clear`, and header close `✕`.

- [ ] **Step 2: Verify with cargo test**

Run: `cargo test`  
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add ui/log_viewer.slint
git commit -m "style(ui): restyle LogViewer with deep console and ghost action buttons"
```

---

### Task 6: Full Verification & Release Build

**Files:**
- Verify: all touched files

- [ ] **Step 1: Run complete test suite**

Run: `cargo test --all-targets`  
Expected: All unit and integration tests pass cleanly.

- [ ] **Step 2: Build release binary**

Run: `cargo build --release`  
Expected: Clean release build with zero warnings.

- [ ] **Step 3: Final commit if any residual adjustments needed**
