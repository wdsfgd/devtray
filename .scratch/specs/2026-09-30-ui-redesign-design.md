# DevTray UI Redesign (Raycast / Linear Dark Aesthetic)

**Date:** 2026-09-30  
**Status:** Approved  
**Topic:** UI Visual Overhaul & AI-Slop Elimination  

---

## 1. Context & Motivation

DevTray was recently ported to Slint declarative UI. While functional and lightweight, the interface exhibited classic "AI slop" template aesthetics:
* Harsh, repetitive box-in-box card layouts with thick borders.
* Saturated primary colors (generic Tailwind blue `#2563eb` for Add Task, loud red/green square buttons).
* Cluttered row of text pill buttons (`[Logs] [Edit] [Del]`) displayed unconditionally on every task row.
* Clunky unicode arrows and jarring contrast in the bottom action bar.

The goal of this redesign is to bring a refined, modern desktop developer utility aesthetic modeled after **Raycast and Linear Dark**, featuring sleek graphite surfaces, hairline micro-borders, elegant typography, and **reveal-on-hover** interaction for task actions.

---

## 2. Design Tokens (`ui/theme.slint`)

### 2.1 Surfaces & Geometry
* `background`: `#0d0f12` — Deep graphite desktop canvas (richer and less harsh than pure black).
* `surface`: `#101317` — Elevated surface for header/footer toolbars and modal backdrops.
* `card-bg`: `#14171c` — Resting task card surface, subtle and cohesive.
* `card-hover`: `#1b2028` — Responsive, crisp hover elevation.
* `border`: `#20252e` — Hairline 1px subtle divider and boundary.
* `border-light`: `#2c3340` — Focused/hovered boundary line.
* `input-bg`: `#0f1217` — Inset form field surface.
* `modal-backdrop`: `#000000c0` — Focused semi-transparent backdrop overlay.

### 2.2 Typography
* `text-primary`: `#f8fafc` — Crisp bright slate for headings and active service names.
* `text-secondary`: `#94a3b8` — Neutral slate for secondary descriptions and labels.
* `text-muted`: `#64748b` — Muted slate for commands, paths, and metadata.

### 2.3 Status & Semantics
* `running`: `#10b981` — Emerald dot and accent icon.
* `running-bg`: `#10b98114` — Soft 8% opacity emerald tint.
* `running-border`: `#10b98133` — Soft translucent emerald border.
* `stopped`: `#475569` — Subdued neutral slate dot.
* `stopped-bg`: `#14171c` — Resting neutral card background.
* `stopped-border`: `#20252e` — Standard subtle border.

### 2.4 Actions & Accents
* `accent`: `#1c212b` — Sleek graphite primary button surface.
* `accent-hover`: `#252c3a` — Subtle interactive hover state.
* `accent-border`: `#30394a` — Crisp boundary for primary button.
* `accent-text`: `#f8fafc` — Crisp white primary action text.
* `danger`: `#f87171` — Soft rose/crimson for destructive actions and stop indicators.
* `danger-hover-bg`: `#ef444418` — Subtle red tint hover background.
* `terminal-bg`: `#0a0c0f` — Deep console background for log viewing.
* `terminal-text`: `#cbd5e1` — High-readability monospace log output.

---

## 3. Component Architecture & Specifications

### 3.1 Main Window (`ui/main_window.slint`)
* **Window Dimensions**:
  * Min-width: 380px, Min-height: 480px, Preferred: 440px × 580px.
  * Window background: `Theme.background` (`#0d0f12`).
* **Header Bar**:
  * Title: "DevTray" in 15px semi-bold (`#f8fafc`).
  * Active Badge: Sleek graphite pill container (`#14171d`, border `#232832`, radius 10px) with 6px emerald dot (`#10b981`) and `N active` label (`#34d399`, 11px, weight 600).
  * `+ New Task` Button: 28px height, 6px radius, graphite background (`#1c212b`, border `#30394a`, hover `#252c3a`), white text (`#f8fafc`).
* **Group Headers**:
  * Uppercase section titles (10px, weight 700, color `#64748b`, letter-spacing 0.8px) with clean spacing between groups.
* **Bottom Footer Bar**:
  * 38px height footer container (`#101317`, top border 1px `#1e2229`, no side borders).
  * Ghost action controls:
    * `▶ Start All` (ghost pill, text `#94a3b8`, hover text `#34d399`, hover bg `#10b98114`).
    * `⏹ Stop All` (ghost pill, text `#94a3b8`, hover text `#f87171`, hover bg `#ef444414`).
    * `Quit` (subtle ghost button, text `#64748b`, hover text `#94a3b8`).

### 3.2 Task Card (`ui/task_card.slint`)
* **Dimensions & Container**:
  * Height: 54px, border-radius: 6px.
  * Idle: background `#14171c`, border 1px `#20252e`.
  * Hover: background `#1b2028`, border 1px `#2c3340`.
* **Left Section**:
  * Reorder controls (`▲ ▼`): Muted chevrons (`#334155`), brightens to `#94a3b8` on card hover.
  * Status indicator: 7px circular dot (`#10b981` emerald when active, `#475569` slate when stopped).
* **Center Section**:
  * Task Name: 13px, weight 600, color `#f8fafc`.
  * Command preview: 10.5px monospace, color `#64748b` (`$ <command>`).
* **Right Section (Reveal-on-Hover Toolbar)**:
  * **Resting state**: Action buttons are hidden (`opacity: 0` or conditional visibility), giving a quiet, distraction-free service overview.
  * **Hover state**: Smoothly reveals 4 compact ghost icon buttons (26px × 24px, 5px radius):
    1. **Toggle (Start / Stop)**:
       * Running: `⏹` icon (`#f87171`, hover bg `#ef444418`).
       * Stopped: `▶` icon (`#34d399`, hover bg `#10b98118`).
    2. **Logs**: Terminal icon `>_` (`#94a3b8`, hover `#f8fafc`, hover bg `#262d3a`).
    3. **Edit**: Pencil icon `✎` (`#94a3b8`, hover `#f8fafc`, hover bg `#262d3a`).
    4. **Delete**: Close icon `✕` (`#94a3b8`, hover `#f87171`, hover bg `#ef444418`).

### 3.3 Modal Dialogs (`ui/task_dialog.slint` & `ui/confirm_dialog.slint`)
* **Backdrop**: `#000000c0` semi-transparent overlay.
* **Dialog Container**: `#14171d` surface, 10px border radius, 1px `#252b36` border.
* **Form Inputs**: `#0f1217` background, 1px `#232832` border (focus: `#3b82f6`), 6px radius.
* **Action Buttons**:
  * Cancel: Ghost button (`#94a3b8`, hover `#f8fafc`).
  * Save / Primary: Graphite solid button (`#1c212b`, border `#30394a`, hover `#252c3a`, text `#f8fafc`).
  * Destructive Confirm: Subtle crimson button (`#311316`, border `#631b21`, text `#fca5a5`, hover `#3f171b`).

### 3.4 Log Viewer (`ui/log_viewer.slint`)
* **Dialog Container**: `#14171d` surface, 10px radius, 1px `#252b36` border.
* **Terminal Box**: `#0a0c0f` deep dark console background, 1px `#1e232d` border.
* **Log Output**: Monospace font (`#cbd5e1`), sharp and clean.
* **Toolbar Actions**: Ghost `Copy Logs`, `Clear`, and header close `✕`.

---

## 4. Verification & Testing Strategy

1. **Compilation Check**: Run `cargo check` and `cargo test` to ensure all Slint templates compile without syntax or type errors.
2. **Visual & Interaction Verification**:
   * Verify all tasks render with clean resting state (no action clutter).
   * Verify mouse hover on any card reveals all 4 icon buttons and illuminates border.
   * Verify click callbacks (Start/Stop, Logs, Edit, Delete, Move Up/Down) fire properly.
   * Verify header button (+ New Task) and bottom actions (Start All, Stop All, Quit) remain fully functional.
   * Verify dialogs and log viewer open with aligned palette and proper backdrop contrast.
