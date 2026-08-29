# DevTray Frontend Restyling Design Specification

**Date:** 2026-08-29  
**Status:** Approved  
**Topic:** Frontend Restyling (Modern, Simple, Non-Slop Developer UI)

---

## 1. Overview & Objective

Redesign DevTray's Slint frontend UI away from generic, high-saturation, template-like UI ("AI slop") toward a restrained, precision-engineered **Linear / Raycast-inspired developer tool aesthetic**.

### Core Goals:
1. **Visual Clarity & Restraint**: Use a deep dark zinc palette (`#09090b` / `#121215`), crisp 1px borders (`#27272a`), and low visual noise.
2. **Improved Information Density**: 2-row task card structure showing task name and monospaced command preview with compact ghost action controls.
3. **Calm Status Semantics**: Emerald green (`#10b981`) for active tasks and neutral zinc (`#52525b`) for idle/stopped tasks to avoid screaming bright red alert states for ordinary stopped services.
4. **Consistency**: Unified 6px/8px border radii and consistent padding across all components and dialogs.

---

## 2. Design System Tokens (`ui/theme.slint`)

### 2.1 Surfaces & Geometry
* `background`: `#09090b` (Deep Zinc background)
* `surface`: `#121215` (Elevated surface for toolbars / dialogs)
* `card-bg`: `#141418` (Default card surface)
* `card-hover`: `#1c1c22` (Card surface on hover)
* `border`: `#27272a` (Default 1px divider and outline)
* `border-light`: `#3f3f46` (Active/focused border)
* `input-bg`: `#0d0d10` (Dark inset input background)
* `modal-backdrop`: `#000000a0` (Semi-transparent backdrop)

### 2.2 Typography Hierarchy
* `text-primary`: `#f4f4f5` (Crisp off-white for headers and active names)
* `text-secondary`: `#a1a1aa` (Zinc-400 for regular labels & secondary buttons)
* `text-muted`: `#71717a` (Zinc-500 for command snippets, directory paths, and hints)

### 2.3 Status & Action Colors
* `accent`: `#3b82f6` (Primary action blue)
* `accent-hover`: `#60a5fa`
* `accent-pressed`: `#2563eb`
* `running`: `#10b981` (Emerald green dot / indicator)
* `running-bg`: `#064e3b` (Dark emerald tint for active pill)
* `running-border`: `#059669` (Active pill outline)
* `stopped`: `#52525b` (Subdued zinc dot when inactive)
* `stopped-bg`: `#1f1f23` (Subdued background)
* `stopped-border`: `#3f3f46`
* `danger`: `#ef4444` (Destructive action red)
* `danger-hover`: `#dc2626`
* `danger-pressed`: `#b91c1c`
* `terminal-bg`: `#09090b` (Terminal console background)
* `terminal-text`: `#d4d4d8` (Neutral monospaced terminal text)

---

## 3. Component Specifications

### 3.1 Main Window (`ui/main_window.slint`)
* **Header Area**:
  * DevTray title in 16px bold (`#f4f4f5`).
  * Running count pill: 20px height, 10px radius, `#064e3b` background, `#059669` border, with 6px `#10b981` dot and "N active" text.
  * "+ Add Task" button: 28px height, 6px radius, `#3b82f6` background, 12px bold white text.
* **Task List & Group Headers**:
  * Group headers: 10px uppercase bold text (`#71717a`) with clean spacing.
  * Empty state: Centered clean text with muted hint.
* **Bottom Toolbar**:
  * 42px height container (`#121215`, 1px border `#27272a`, 6px radius).
  * "▶ Start All" button: Outline button with subtle emerald accent.
  * "⏹ Stop All" button: Outline button with subtle rose accent.
  * "Quit" button: Discrete ghost button with `#a1a1aa` text.

### 3.2 Task Card (`ui/task_card.slint`)
* **Container**: 56px height, `#141418` background, 8px radius, 1px `#27272a` border (hover: `#1c1c22` background, `#3f3f46` border).
* **Left Section**:
  * Reorder arrows (▲ / ▼): Subtle 14px buttons, `#52525b` muted text, hoverable.
  * Status indicator: 8px circular dot (`#10b981` if running, `#52525b` if stopped).
* **Center Section (2 Rows)**:
  * Top: Task Name (13px, font-weight 700, `#f4f4f5`).
  * Bottom: Command preview snippet (11px monospaced, `#71717a`, fallback to working directory).
* **Right Section (Action Cluster)**:
  * Toggle Button (`▶` / `⏹`): 28px × 26px, 5px radius, tinted outline & icon.
  * Logs Button: 38px × 26px ghost button with subtle hover background (`#27272a`).
  * Edit Button: 36px × 26px ghost button.
  * Delete Button: 34px × 26px ghost button with subtle red hover (`#3f1d1d`).

### 3.3 Modal Dialogs (`ui/task_dialog.slint` & `ui/confirm_dialog.slint`)
* **Backdrop**: `#000000a0` semi-transparent overlay.
* **Card Container**: `#141418` background, 8px radius, 1px `#27272a` border, 16px padding.
* **Header**: Bold title + minimalist `✕` close button.
* **Form Inputs**: `#0d0d10` inset background, 34px height, `#27272a` border, 5px radius.
* **Buttons**: 32px height, 6px radius. Ghost `Cancel` + primary `Save` / `Delete`.

### 3.4 Log Viewer (`ui/log_viewer.slint`)
* **Container**: `#141418` dialog box, 8px radius, 1px `#27272a` border.
* **Terminal Box**: `#09090b` background, 1px `#27272a` border, 6px radius.
* **Log Text**: Monospace font (`#d4d4d8`), readable and sharp.
* **Toolbar**: Compact `Clear View` and `Close` buttons.

---

## 4. Verification & Testing Plan
* **Build Verification**: Run `cargo check` and `cargo test` to ensure Slint compilation succeeds.
* **GUI Verification**: Run `cargo build --bin devtray` to confirm all assets and templates render properly.
